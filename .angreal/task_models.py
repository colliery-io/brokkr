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
-- Data Model Test Script

-- Stage 1: Insert sample data into the stacks table
INSERT INTO stacks (name, description)
VALUES 
('Stack1', 'First test stack'),
('Stack2', 'Second test stack');

-- Stage 2: Insert sample data into the agents table
INSERT INTO agents (name, cluster_name, status, pak_hash)
VALUES 
('Agent1', 'Cluster1', 'ACTIVE', 'hash1'),
('Agent2', 'Cluster2', 'INACTIVE', 'hash2');

-- Stage 3: Create deployment objects for the stacks
INSERT INTO deployment_objects (stack_id, yaml_content, yaml_checksum)
VALUES 
((SELECT id FROM stacks WHERE name = 'Stack1'), 'yaml: content1', 'checksum1'),
((SELECT id FROM stacks WHERE name = 'Stack2'), 'yaml: content2', 'checksum2');

-- Stage 4: Create agent_targets to associate agents with stacks
INSERT INTO agent_targets (agent_id, stack_id)
VALUES 
((SELECT id FROM agents WHERE name = 'Agent1'), (SELECT id FROM stacks WHERE name = 'Stack1')),
((SELECT id FROM agents WHERE name = 'Agent2'), (SELECT id FROM stacks WHERE name = 'Stack2'));

-- Stage 5: Add labels and annotations to stacks
INSERT INTO stack_labels (stack_id, label)
VALUES 
((SELECT id FROM stacks WHERE name = 'Stack1'), 'label1'),
((SELECT id FROM stacks WHERE name = 'Stack2'), 'label2');

INSERT INTO stack_annotations (stack_id, key, value)
VALUES 
((SELECT id FROM stacks WHERE name = 'Stack1'), 'key1', 'value1'),
((SELECT id FROM stacks WHERE name = 'Stack2'), 'key2', 'value2');

-- Stage 6: Add labels and annotations to agents
INSERT INTO agent_labels (agent_id, label)
VALUES 
((SELECT id FROM agents WHERE name = 'Agent1'), 'agent_label1'),
((SELECT id FROM agents WHERE name = 'Agent2'), 'agent_label2');

INSERT INTO agent_annotations (agent_id, key, value)
VALUES 
((SELECT id FROM agents WHERE name = 'Agent1'), 'agent_key1', 'agent_value1'),
((SELECT id FROM agents WHERE name = 'Agent2'), 'agent_key2', 'agent_value2');

-- Stage 7: Create agent events
INSERT INTO agent_events (agent_id, deployment_object_id, event_type, status, message)
VALUES 
((SELECT id FROM agents WHERE name = 'Agent1'), 
 (SELECT id FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'Stack1')),
 'DEPLOYMENT', 'SUCCESS', 'Deployment successful'),
((SELECT id FROM agents WHERE name = 'Agent2'), 
 (SELECT id FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'Stack2')),
 'DEPLOYMENT', 'FAILURE', 'Deployment failed');

-- Stage 8: Test soft deletion of a stack
UPDATE stacks SET deleted_at = CURRENT_TIMESTAMP WHERE name = 'Stack1';

-- Stage 9: Test hard deletion of an agent
DELETE FROM agents WHERE name = 'Agent2';

-- Stage 10: Verify data integrity and cascading operations
-- Check if deployment objects are soft-deleted when stack is soft-deleted
SELECT * FROM deployment_objects WHERE stack_id = (SELECT id FROM stacks WHERE name = 'Stack1');

-- Check if agent events are deleted when an agent is hard-deleted
SELECT * FROM agent_events WHERE agent_id = (SELECT id FROM agents WHERE name = 'Agent2');

-- Check if agent_targets are deleted when an agent is hard-deleted
SELECT * FROM agent_targets WHERE agent_id = (SELECT id FROM agents WHERE name = 'Agent2');

-- Check if agent labels and annotations are deleted when an agent is hard-deleted
SELECT * FROM agent_labels WHERE agent_id = (SELECT id FROM agents WHERE name = 'Agent2');
SELECT * FROM agent_annotations WHERE agent_id = (SELECT id FROM agents WHERE name = 'Agent2');

-- Stage 11: Test prevention of deployment object modifications
DO $$
DECLARE
    error_message TEXT;
BEGIN
    UPDATE deployment_objects 
    SET yaml_content = 'modified content' 
    WHERE stack_id = (SELECT id FROM stacks WHERE name = 'Stack2');

    RAISE EXCEPTION 'Test failed: Deployment object modification was allowed';
EXCEPTION
    WHEN others THEN
        GET STACKED DIAGNOSTICS error_message = MESSAGE_TEXT;
        IF error_message LIKE 'Deployment objects cannot be modified%' THEN
            RAISE NOTICE 'Test passed: Deployment object modification prevented as expected';
        ELSE
            RAISE EXCEPTION 'Test failed: Unexpected error: %', error_message;
        END IF;
END $$;

-- Stage 12: Verify unique constraints
-- Test unique stack name constraint
DO $$
DECLARE
    error_message TEXT;
BEGIN
    INSERT INTO stacks (name, description) VALUES ('Stack2', 'Duplicate stack name');

    RAISE EXCEPTION 'Test failed: Duplicate stack name was allowed';
EXCEPTION
    WHEN unique_violation THEN
        GET STACKED DIAGNOSTICS error_message = MESSAGE_TEXT;
        IF error_message LIKE '%unique constraint "unique_stack_name"%' THEN
            RAISE NOTICE 'Test passed: Duplicate stack name prevented as expected';
        ELSE
            RAISE EXCEPTION 'Test failed: Unexpected error: %', error_message;
        END IF;
END $$;

-- Test unique agent-cluster constraint
DO $$
DECLARE
    error_message TEXT;
BEGIN
    INSERT INTO agents (name, cluster_name, status, pak_hash)
    VALUES ('Agent1', 'Cluster1', 'ACTIVE', 'hash3');

    RAISE EXCEPTION 'Test failed: Duplicate agent-cluster combination was allowed';
EXCEPTION
    WHEN unique_violation THEN
        GET STACKED DIAGNOSTICS error_message = MESSAGE_TEXT;
        IF error_message LIKE '%unique constraint "unique_agent_cluster"%' THEN
            RAISE NOTICE 'Test passed: Duplicate agent-cluster combination prevented as expected';
        ELSE
            RAISE EXCEPTION 'Test failed: Unexpected error: %', error_message;
        END IF;
END $$;

-- Stage 13: Basic queries to test indexes
SELECT * FROM stacks WHERE name = 'Stack1';
SELECT * FROM agents WHERE cluster_name = 'Cluster1';
SELECT * FROM deployment_objects WHERE yaml_checksum = 'checksum1';
SELECT * FROM agent_events WHERE event_type = 'DEPLOYMENT';
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
    run_sql_in_docker(TEST_SQL_SCRIPT)
