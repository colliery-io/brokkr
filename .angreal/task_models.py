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
-- Start transaction
BEGIN;

-- Test 1: Create a stack
-- Intended behavior: Successfully insert a new stack with name, description, and verify constraints
INSERT INTO stacks (name, description) VALUES ('test-stack', 'A test stack');
-- Verify insertion
SELECT * FROM stacks WHERE name = 'test-stack';
-- Test unique constraint
INSERT INTO stacks (name, description) VALUES ('test-stack', 'Duplicate name')
ON CONFLICT (name) DO NOTHING;
-- Verify no duplicate was inserted
SELECT COUNT(*) FROM stacks WHERE name = 'test-stack';

-- Test 2: Create an agent
-- Intended behavior: Successfully insert a new agent and verify constraints
INSERT INTO agents (name, cluster_name, status) VALUES ('test-agent', 'test-cluster', 'ACTIVE');
-- Verify insertion
SELECT * FROM agents WHERE name = 'test-agent';
-- Test unique constraint
INSERT INTO agents (name, cluster_name, status) VALUES ('test-agent', 'test-cluster', 'INACTIVE')
ON CONFLICT (name, cluster_name) DO NOTHING;
-- Verify no duplicate was inserted
SELECT COUNT(*) FROM agents WHERE name = 'test-agent' AND cluster_name = 'test-cluster';

-- Test 3: Create a deployment object
-- Intended behavior: Successfully insert a new deployment object and verify constraints
INSERT INTO deployment_objects (stack_id, yaml_content, yaml_checksum)
VALUES ((SELECT id FROM stacks WHERE name = 'test-stack'), 'test: content', md5('test: content'));
-- Verify insertion and auto-generated fields
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'test-stack');

-- Test 4: Create an agent target
-- Intended behavior: Successfully link an agent to a stack
INSERT INTO agent_targets (stack_id, agent_id)
VALUES (
    (SELECT id FROM stacks WHERE name = 'test-stack'),
    (SELECT id FROM agents WHERE name = 'test-agent')
);
-- Verify insertion
SELECT * FROM agent_targets;

-- Test 5: Create an agent event
-- Intended behavior: Successfully insert a new agent event
INSERT INTO agent_events (agent_id, deployment_object_id, event_type, status, message)
VALUES (
    (SELECT id FROM agents WHERE name = 'test-agent'),
    (SELECT id FROM deployment_objects LIMIT 1),
    'APPLIED',
    'SUCCESS',
    'Test event'
);
-- Verify insertion
SELECT * FROM agent_events;

-- Test 6: Test stack soft delete trigger
-- Intended behavior: Soft delete stack, create deletion marker, and soft delete related deployment objects
UPDATE stacks SET deleted_at = CURRENT_TIMESTAMP WHERE name = 'test-stack';
-- Verify soft deletion of stack
SELECT * FROM stacks WHERE name = 'test-stack';
-- Verify soft deletion of related deployment objects
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'test-stack');
-- Verify creation of deletion marker
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'test-stack') AND is_deletion_marker = TRUE;

-- Test 7: Test prevention of deployment object modifications
-- Intended behavior: Prevent updates to non-deletion marker deployment objects
DO $$
DECLARE
    test_id UUID;
BEGIN
    SELECT id INTO test_id FROM deployment_objects WHERE is_deletion_marker = FALSE LIMIT 1;
    BEGIN
        UPDATE deployment_objects SET yaml_content = 'modified content' WHERE id = test_id;
        RAISE EXCEPTION 'Expected update to fail, but it succeeded';
    EXCEPTION WHEN others THEN
        RAISE NOTICE 'Update failed as expected: %', SQLERRM;
    END;
END $$;

-- Test 8: Test cascade soft delete of agents
-- Intended behavior: Soft delete agent and cascade to related agent events
UPDATE agents SET deleted_at = CURRENT_TIMESTAMP WHERE name = 'test-agent';
-- Verify soft deletion of agent
SELECT * FROM agents WHERE name = 'test-agent';
-- Verify soft deletion of related agent events
SELECT * FROM agent_events WHERE agent_id = (SELECT id FROM agents WHERE name = 'test-agent');

-- Test 9: Test hard delete of stack
-- Intended behavior: Hard delete stack and cascade to all related objects
DELETE FROM stacks WHERE name = 'test-stack';
-- Verify deletion of stack
SELECT COUNT(*) FROM stacks WHERE name = 'test-stack';
-- Verify deletion of related deployment objects
SELECT COUNT(*) FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'test-stack');
-- Verify deletion of related agent targets
SELECT COUNT(*) FROM agent_targets WHERE stack_id = (SELECT id FROM stacks WHERE name = 'test-stack');

-- Rollback transaction to clean up test data
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
