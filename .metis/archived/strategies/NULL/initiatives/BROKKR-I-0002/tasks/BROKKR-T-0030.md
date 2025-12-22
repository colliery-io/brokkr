---
id: create-database-migration-for
level: task
title: "Create database migration for stack templates"
short_code: "BROKKR-T-0030"
created_at: 2025-12-07T17:57:55.242902+00:00
updated_at: 2025-12-07T20:42:30.143406+00:00
parent: BROKKR-I-0002
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0002
---

# Create database migration for stack templates

## Parent Initiative

[[BROKKR-I-0002]] - Stack Templating System

## Objective

Create the database migration for the stack templating system, including all 5 required tables and associated triggers/indexes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Migration folder `07_stack_templates` created
- [ ] `stack_templates` table created with all columns per design
- [ ] `template_labels` table created following stack_labels pattern
- [ ] `template_annotations` table created following stack_annotations pattern
- [ ] `template_targets` table created following agent_targets pattern
- [ ] `rendered_deployment_objects` table created for provenance tracking
- [ ] All indexes created for performance
- [ ] `update_timestamp` trigger added to stack_templates
- [ ] Cascade soft-delete trigger for generator deletion
- [ ] Migration runs successfully up and down

## Implementation Notes

### Technical Approach

Create migration at `crates/brokkr-models/migrations/07_stack_templates/up.sql`:

**Tables to create:**

1. **stack_templates** - Core template table
   - id, created_at, updated_at, deleted_at (standard)
   - generator_id (nullable FK - NULL = system template)
   - name, description, version
   - template_content (TEXT - Tera syntax)
   - parameters_schema (TEXT - JSON Schema)
   - checksum (SHA-256)
   - UNIQUE(generator_id, name, version)

2. **template_labels** - Mirror stack_labels pattern
   - id, template_id (FK), label (VARCHAR 64), created_at
   - UNIQUE(template_id, label)

3. **template_annotations** - Mirror stack_annotations pattern
   - id, template_id (FK), key, value, created_at
   - UNIQUE(template_id, key)

4. **template_targets** - Mirror agent_targets pattern
   - id, template_id (FK), stack_id (FK), created_at
   - UNIQUE(template_id, stack_id)

5. **rendered_deployment_objects** - Provenance tracking
   - id, deployment_object_id (FK UNIQUE), template_id (FK)
   - template_version, template_parameters (TEXT), created_at

**Triggers:**
- update_stack_templates_timestamp
- cascade_soft_delete_generator_templates (when generator deleted)

### Dependencies

None - foundational migration.

### Reference Files

- `migrations/03_stacks/up.sql` - stack_labels/annotations pattern
- `migrations/01_agents/up.sql` - agent_targets pattern
- `migrations/04_deployment_objects/up.sql` - immutability pattern

## Status Updates

*To be added during implementation*