import angreal # type: ignore
from utils import docker_up,docker_down, docker_clean
import os
import subprocess


models = angreal.command_group(name="models", about="commands for `brokkr-models`")



brokkr_models_dir = os.path.join(
    angreal.get_root(),
    '..',
    "crates",
    "brokkr-models"
)


@models()
@angreal.command(name="schema", about="generate `src/schema.rs` given current available migrations")
def schema():
    docker_down()
    docker_clean()
    docker_up()
    subprocess.run("diesel migration run"
                    , cwd=brokkr_models_dir, shell=True)
    subprocess.run("diesel print-schema > src/schema.rs"
                    , cwd=brokkr_models_dir, shell=True)




TEST_SQL_SCRIPT = """
-- Start a transaction so we can rollback at the end
BEGIN;

-- Create a temporary table to store the stack_id
CREATE TEMPORARY TABLE temp_stack_id (id UUID);

-- 1. Create a test stack
WITH new_stack AS (
    INSERT INTO stacks (name, description, labels, annotations, agent_target) 
    VALUES ('test-stack', 'A test stack', '{"env": "test"}', '{"version": "1.0"}', '["test-agent"]')
    RETURNING id
)
INSERT INTO temp_stack_id SELECT id FROM new_stack;

-- 2. Create some test deployment objects
INSERT INTO deployment_objects (stack_id, yaml_content, yaml_checksum, is_deletion_marker)
VALUES 
((SELECT id FROM temp_stack_id), 'apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: test-config', md5('test-content-1'), FALSE),
((SELECT id FROM temp_stack_id), 'apiVersion: v1\nkind: Secret\nmetadata:\n  name: test-secret', md5('test-content-2'), FALSE);

-- 3. Verify the deployment objects were created with sequence_ids
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM temp_stack_id) ORDER BY sequence_id;

-- 4. Create a test agent
INSERT INTO agents (name, cluster_name, status)
VALUES ('test-agent', 'test-cluster', 'ACTIVE');

-- 5. Create some test agent events
INSERT INTO agent_events (agent_id, deployment_object_id, event_type, status, message)
SELECT 
    (SELECT uuid FROM agents WHERE name = 'test-agent'),
    uuid,
    'APPLIED',
    'success',
    'Test deployment applied'
FROM deployment_objects
WHERE stack_id = (SELECT id FROM temp_stack_id);

-- 6. Verify the agent events were created
SELECT * FROM agent_events;

-- 7. Test soft delete trigger on stack
UPDATE stacks SET deleted_at = NOW() WHERE id = (SELECT id FROM temp_stack_id);

-- 8. Verify the soft delete effects
-- Check if deployment objects are marked as deleted
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM temp_stack_id) ORDER BY sequence_id;

-- Check if a deletion marker was created
SELECT * FROM deployment_objects 
WHERE stack_id = (SELECT id FROM temp_stack_id) AND is_deletion_marker = TRUE;

-- 9. Test updating a deployment object (should fail due to prevent_deployment_object_changes trigger)
DO $$
DECLARE
    test_uuid UUID;
BEGIN
    SELECT uuid INTO test_uuid FROM deployment_objects WHERE stack_id = (SELECT id FROM temp_stack_id) LIMIT 1;
    BEGIN
        UPDATE deployment_objects SET yaml_content = 'updated content' WHERE uuid = test_uuid;
        RAISE EXCEPTION 'Expected update to fail, but it succeeded';
    EXCEPTION WHEN others THEN
        RAISE NOTICE 'Update failed as expected: %', SQLERRM;
    END;
END $$;

-- 10. Test cascade delete of agent events when an agent is deleted
DELETE FROM agents WHERE name = 'test-agent';

-- Verify that related agent events were deleted
SELECT * FROM agent_events;

-- Drop the temporary table
DROP TABLE temp_stack_id;

-- Rollback the transaction to clean up the test data
ROLLBACK;
"""

tables_available= """
-- Start a transaction so we can rollback at the end
BEGIN;

-- Function to print table names
CREATE OR REPLACE FUNCTION print_tables() RETURNS void AS $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN 
        SELECT tablename 
        FROM pg_tables 
        WHERE schemaname = 'public'
        ORDER BY tablename
    LOOP
        RAISE NOTICE 'Table: %', table_name;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Call the function to print table names
SELECT print_tables();

-- Drop the function
DROP FUNCTION print_tables();

-- Rollback the transaction to clean up
ROLLBACK;
"""

@models()
@angreal.command(name="test")
def test():
    docker_down()
    docker_clean()
    docker_up()
    import subprocess
    import tempfile

    # The SQL script to execute
    
    def run_sql_in_docker(sql):
        # Write the SQL to a temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.sql', delete=False) as temp_sql_file:
            temp_sql_file.write(sql)
            temp_sql_file_path = temp_sql_file.name

        # Command to copy the SQL file into the container
        copy_cmd = f"docker cp {temp_sql_file_path} brokkr-dev-postgres-1:/tmp/test_script.sql"
        
        # Command to execute the SQL script in the container
        exec_cmd = "docker exec brokkr-dev-postgres-1 psql -U brokkr -d brokkr -f /tmp/test_script.sql"
        
        try:
            # Copy the SQL file to the container
            subprocess.run(copy_cmd, shell=True, check=True)
            
            # Execute the SQL script
            result = subprocess.run(exec_cmd, shell=True, check=True, capture_output=True, text=True)
            
            # Print the output
            print(result.stdout)
            
            if result.stderr:
                print("Errors or notices:")
                print(result.stderr)
        
        except subprocess.CalledProcessError as e:
            print(f"An error occurred: {e}")
            if e.output:
                print(f"Output: {e.output}")
            if e.stderr:
                print(f"Error: {e.stderr}")

    # Run our migrations 
    migration_files = []
    migrations = os.path.join(brokkr_models_dir,'migrations')
    
    for root,dirs,files in os.walk(migrations):
        for f in files:
            if f.endswith('up.sql'):
                migration_files.append(os.path.join(root,f))
    migration_files.sort()


    
    for f in migration_files:
        run_sql_in_docker(open(f,'r').read())
    # Run the SQL script
    run_sql_in_docker(tables_available)
    run_sql_in_docker(TEST_SQL_SCRIPT)