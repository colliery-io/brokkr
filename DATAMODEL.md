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
        +UUID agent_target
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
        +UUID stack_id
        +TEXT agent_name
        +TEXT cluster_name
    }
    
    class labels {
        +UUID id
        +UUID external_object_id
        +VARCHAR(255) label
    }
    
    class annotations {
        +UUID id
        +UUID external_object_id
        +VARCHAR(255) key
        +TEXT value
    }
    
    stacks "1" -- "0..*" deployment_objects
    stacks "1" -- "0..*" agent_targets
    agents "1" -- "0..*" agent_events
    deployment_objects "1" -- "0..*" agent_events
    agents "1" -- "0..*" labels
    agents "1" -- "0..*" annotations
    stacks "1" -- "0..*" labels
    stacks "1" -- "0..*" annotations

```

## Individual Table Descriptions

1. Stacks Table:
   - Primary key: `id` (UUID)
   - Unique constraint: `name`
   - Contains basic information about stacks including name and description
   - Has a soft delete mechanism (`deleted_at`)
   - Associated with deployment objects and agent targets

2. Agents Table:
   - Primary key: `id` (UUID)
   - Unique constraint: combination of `name` and `cluster_name`
   - Stores information about agents including status and last heartbeat
   - Has a soft delete mechanism (`deleted_at`)
   - Associated with agent events

3. Deployment Objects Table:
   - Primary key: `id` (UUID)
   - Contains YAML content for deployments and its checksum
   - Has a `sequence_id` for ordering
   - Linked to a stack via `stack_id`
   - Includes `is_deletion_marker` flag for marking deletions
   - Has a soft delete mechanism (`deleted_at`)

4. Agent Events Table:
   - Primary key: `id` (UUID)
   - Records events related to agents and deployment objects
   - Linked to both an agent and a deployment object
   - Includes event type, status, and a message
   - Has a soft delete mechanism (`deleted_at`)

5. Agent Targets Table:
   - Primary key: `id` (UUID)
   - Links stacks with specific agent and cluster combinations
   - Unique constraint: combination of `stack_id`, `agent_name`, and `cluster_name`

6. Labels Table:
   - Primary key: `id` (UUID)
   - Stores labels for external objects (like agents or stacks)
   - Unique constraint: combination of `external_object_id` and `label`

7. Annotations Table:
   - Primary key: `id` (UUID)
   - Stores key-value annotations for external objects
   - Unique constraint: combination of `external_object_id` and `key`

Key Features:
1. Soft Delete: All main tables (stacks, agents, deployment_objects, agent_events) support soft delete via the `deleted_at` column.
2. Timestamps: All main tables have `created_at` and `updated_at` columns, automatically managed by triggers.
3. Immutability: Deployment objects are designed to be immutable after creation, with exceptions for soft deletion and updating deletion markers.
4. Cascading Deletes: The system implements cascading soft deletes and hard deletes through triggers and functions.
5. Indexing: Appropriate indexes are created for efficient querying, especially on foreign keys and frequently used columns.

The data model supports a system where:
- Stacks can have multiple deployment objects and agent targets.
- Agents can generate multiple events related to deployment objects.
- Both stacks and agents can have multiple labels and annotations.
- The system maintains a history of deployments and agent activities through the deployment_objects and agent_events tables.

This design allows for efficient tracking of deployments, agent activities, and system states while maintaining data integrity and supporting soft delete mechanisms for data retention policies.