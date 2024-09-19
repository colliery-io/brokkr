# Data Design Description

```mermaid
classDiagram
    class stacks {
        +UUID id
        +TIMESTAMP created_at
        +TIMESTAMP updated_at
        +TIMESTAMP deleted_at
        +VARCHAR(255) name
        +TEXT description
        +UUID generator_id
    }

    class agents {
        +UUID id
        +TIMESTAMP created_at
        +TIMESTAMP updated_at
        +TIMESTAMP deleted_at
        +VARCHAR(255) name
        +VARCHAR(255) cluster_name
        +TIMESTAMP last_heartbeat
        +VARCHAR(50) status
        +TEXT pak_hash
    }

    class deployment_objects {
        +UUID id
        +TIMESTAMP created_at
        +TIMESTAMP updated_at
        +TIMESTAMP deleted_at
        +BIGSERIAL sequence_id
        +UUID stack_id
        +TEXT yaml_content
        +TEXT yaml_checksum
        +TIMESTAMP submitted_at
        +BOOLEAN is_deletion_marker
        +UUID generator_id
    }

    class agent_events {
        +UUID id
        +TIMESTAMP created_at
        +TIMESTAMP updated_at
        +TIMESTAMP deleted_at
        +UUID agent_id
        +UUID deployment_object_id
        +VARCHAR(50) event_type
        +VARCHAR(10) status
        +TEXT message
    }

    class agent_targets {
        +UUID id
        +UUID agent_id
        +UUID stack_id
    }

    class stack_labels {
        +UUID id
        +UUID stack_id
        +VARCHAR(64) label
    }

    class stack_annotations {
        +UUID id
        +UUID stack_id
        +VARCHAR(64) key
        +VARCHAR(64) value
    }

    class agent_labels {
        +UUID id
        +UUID agent_id
        +VARCHAR(64) label
    }

    class agent_annotations {
        +UUID id
        +UUID agent_id
        +VARCHAR(64) key
        +VARCHAR(64) value
    }

    class generators {
        +UUID id
        +TIMESTAMP created_at
        +TIMESTAMP updated_at
        +VARCHAR(255) name
        +TEXT description
        +TEXT api_key_hash
        +TIMESTAMP last_active_at
        +BOOLEAN is_active
    }

    generators "1" -- "1..*" stacks
    generators "1" -- "1..*" deployment_objects
    stacks "1" -- "0..*" deployment_objects
    stacks "1" -- "0..*" agent_targets
    agents "1" -- "0..*" agent_events
    agents "1" -- "0..*" agent_targets
    deployment_objects "1" -- "0..*" agent_events
    stacks "1" -- "0..*" stack_labels
    stacks "1" -- "0..*" stack_annotations
    agents "1" -- "0..*" agent_labels
    agents "1" -- "0..*" agent_annotations
```

## Individual Table Descriptions

1. Stacks Table:
   - Primary key: `id` (UUID)
   - Unique constraint: `name`
   - Contains basic information about stacks including name and description
   - Has a soft delete mechanism (`deleted_at`)
   - Associated with deployment objects and agent targets
   - Now includes a required `generator_id` to link to the generator that created it

2. Agents Table:
   - Primary key: `id` (UUID)
   - Unique constraint: combination of `name` and `cluster_name`
   - Stores information about agents including status, last heartbeat, and PAK hash
   - Has a soft delete mechanism (`deleted_at`)
   - Associated with agent events and agent targets

3. Deployment Objects Table:
   - Primary key: `id` (UUID)
   - Contains YAML content for deployments and its checksum
   - Has a `sequence_id` for ordering
   - Linked to a stack via `stack_id`
   - Includes `is_deletion_marker` flag for marking deletions
   - Has a soft delete mechanism (`deleted_at`)
   - Now includes a required `generator_id` to link to the generator that created it

4. Agent Events Table:
   - Primary key: `id` (UUID)
   - Records events related to agents and deployment objects
   - Linked to both an agent and a deployment object
   - Includes event type, status, and a message
   - Has a soft delete mechanism (`deleted_at`)

5. Agent Targets Table:
   - Primary key: `id` (UUID)
   - Links agents with stacks
   - Unique constraint: combination of `agent_id` and `stack_id`

6. Stack Labels Table:
   - Primary key: `id` (UUID)
   - Stores labels for stacks
   - Unique constraint: combination of `stack_id` and `label`

7. Stack Annotations Table:
   - Primary key: `id` (UUID)
   - Stores key-value annotations for stacks
   - Unique constraint: combination of `stack_id` and `key`

8. Agent Labels Table:
   - Primary key: `id` (UUID)
   - Stores labels for agents
   - Unique constraint: combination of `agent_id` and `label`

9. Agent Annotations Table:
   - Primary key: `id` (UUID)
   - Stores key-value annotations for agents
   - Unique constraint: combination of `agent_id` and `key`

10. Generators Table:
    - Primary key: `id` (UUID)
    - Stores information about generators including name, description, and API key hash
    - Tracks the last active time and whether the generator is currently active
    - Associated with stacks and deployment objects that it creates
    - Must be associated with at least one stack or deployment object

Key Features:
1. Soft Delete: All main tables support soft delete via the `deleted_at` column.
2. Timestamps: All main tables have `created_at` and `updated_at` columns, automatically managed by triggers.
3. Immutability: Deployment objects are designed to be immutable after creation, with exceptions for soft deletion and updating deletion markers.
4. Cascading Deletes: The system implements cascading soft deletes and hard deletes through triggers and functions.
5. Indexing: Appropriate indexes are created for efficient querying, especially on foreign keys and frequently used columns.

The data model supports a system where:
- Generators can create multiple stacks and deployment objects.
- Stacks can have multiple deployment objects and be targeted by multiple agents.
- Agents can generate multiple events related to deployment objects and target multiple stacks.
- Both stacks and agents can have multiple labels and annotations.
- The system maintains a history of deployments and agent activities through the deployment_objects and agent_events tables.

[Rest of the content remains unchanged]

