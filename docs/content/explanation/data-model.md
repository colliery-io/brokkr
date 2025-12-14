---
title: "Data Model Design"
weight: 4
---

# Data Model Design

This document explains the design decisions and architectural philosophy behind Brokkr's data model.

## Entity Relationship Overview

{{< mermaid >}}
classDiagram
    class stacks
    class agents
    class deployment_objects
    class agent_events
    class agent_targets
    class stack_labels
    class stack_annotations
    class agent_labels
    class agent_annotations
    class generators

    generators "1" -- "1..*" stacks : creates
    generators "1" -- "1..*" deployment_objects : creates
    stacks "1" -- "0..*" deployment_objects : contains
    stacks "1" -- "0..*" agent_targets : targeted by
    agents "1" -- "0..*" agent_events : reports
    agents "1" -- "0..*" agent_targets : targets
    deployment_objects "1" -- "0..*" agent_events : triggers
    stacks "1" -- "0..*" stack_labels : has
    stacks "1" -- "0..*" stack_annotations : has
    agents "1" -- "0..*" agent_labels : has
    agents "1" -- "0..*" agent_annotations : has
{{< /mermaid >}}

## Design Philosophy

### Immutability of Deployment Objects

Deployment objects are immutable after creation (except for soft deletion). This design decision ensures:
- **Audit Trail**: Every deployment can be traced back to its exact configuration
- **Rollback Capability**: Previous configurations are always available
- **Consistency**: No accidental modifications to deployed resources

### Soft Deletion Strategy

All primary entities support soft deletion via `deleted_at` timestamps. This approach provides:
- **Recovery**: Accidentally deleted items can be restored
- **Referential Integrity**: Related data remains intact
- **Historical Analysis**: Past configurations and relationships are preserved
- **Compliance**: Audit requirements are met without data loss

### Cascading Operations

The system implements intelligent cascading for both soft and hard deletes:

#### Soft Delete Cascades
- Generator → Stacks and Deployment Objects
- Stack → Deployment Objects (with deletion marker)
- Agent → Agent Events

#### Hard Delete Cascades
- Stack → Agent Targets, Agent Events, Deployment Objects
- Agent → Agent Targets, Agent Events
- Generator → (handled by foreign key constraints)

## Key Architectural Decisions

### Why Generators?

Generators represent external systems that create stacks and deployment objects. This abstraction:
- Provides authentication boundaries for automated systems
- Tracks which system created which resources
- Enables rate limiting and access control per generator
- Maintains audit trail of automated deployments

### Why Agent Targets?

The many-to-many relationship between agents and stacks enables:
- Flexible deployment topologies
- Multi-cluster deployments
- Gradual rollouts
- Environment-specific targeting

### Labels vs Annotations

**Labels** (single values):
- Used for selection and filtering
- Simple categorization
- Fast queries

**Annotations** (key-value pairs):
- Rich metadata
- Configuration hints
- Integration with external systems

## Trigger Behavior

### Stack Deletion Flow

{{< mermaid >}}
sequenceDiagram
    participant User
    participant DB

    Note over User,DB: Soft Delete
    User->>DB: UPDATE stacks SET deleted_at = NOW()
    DB->>DB: Soft delete all deployment objects
    DB->>DB: Insert deletion marker object

    Note over User,DB: Hard Delete
    User->>DB: DELETE FROM stacks
    DB->>DB: Delete agent_targets
    DB->>DB: Delete agent_events
    DB->>DB: Delete deployment_objects
{{< /mermaid >}}

### Deployment Object Protection

Deployment objects cannot be modified except for:
- Setting `deleted_at` (soft delete)
- Updating deletion markers

This is enforced by database triggers to ensure immutability.

## Performance Considerations

### Indexing Strategy

Key indexes are created on:
- Foreign keys for join performance
- `deleted_at` for filtering active records
- Unique constraints for data integrity
- Frequently queried fields (name, status)

### Sequence IDs

Deployment objects use `BIGSERIAL sequence_id` for:
- Guaranteed ordering
- Efficient pagination
- Conflict-free concurrent inserts

## Migration Strategy

The data model is managed through versioned SQL migrations in `crates/brokkr-models/migrations/`. Each migration:
- Is idempotent
- Includes both up and down scripts
- Is tested in CI/CD pipeline

For detailed field definitions and constraints, refer to the [API documentation](/reference/api/) or the source code in `crates/brokkr-models/`.
