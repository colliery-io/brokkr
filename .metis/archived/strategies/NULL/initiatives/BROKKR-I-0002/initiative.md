---
id: stack-templating-system
level: initiative
title: "Stack Templating System"
short_code: "BROKKR-I-0002"
created_at: 2025-10-16T02:38:42.474134+00:00
updated_at: 2025-12-09T13:33:32.624490+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: stack-templating-system
---

# Stack Templating System Initiative

## Context **\[REQUIRED\]**

Brokkr's strength lies in enabling dynamic, runtime-driven service management across multiple clusters. However, generators currently must create complete YAML content for each deployment object, making it difficult to rapidly deploy variations of similar services. This limitation becomes apparent in scenarios where:

- A control plane needs to launch multiple database instances with different configurations
- Platform teams want to provide self-service deployment of standardized application stacks
- Generators need to create multiple environments (dev/staging/prod) of the same service with different parameters

The current workflow requires significant custom code in each generator to handle parameter substitution and YAML generation, duplicating templating logic across different generator implementations.

**Important Architectural Clarification:**
Templates in Brokkr create **single deployment objects**, not entire stacks. Each template contains YAML that may include multiple Kubernetes resources (similar to a Helm chart), but the template instantiation produces one versioned deployment object. Multiple templates can be instantiated into the same stack to build complex multi-tier applications.

## Goals & Non-Goals **\[REQUIRED\]**

**Goals:**

- Enable generators to define reusable deployment object templates with parameter placeholders
- Provide RESTful API for instantiating templates into existing stacks
- Support full Tera templating features (variables, conditionals, loops, filters)
- Implement label/annotation-based targeting validation (matching agent targeting patterns)
- Maintain complete provenance tracking (template_id + version + parameters) for reconstruction
- Validate parameters against JSON Schema before rendering
- Support both generator-owned and system templates (admin-managed)

**Non-Goals:**

- Replace existing direct YAML creation workflows (templates are additive, not replacements)
- Support templating languages other than Tera
- Provide runtime template modification (templates are immutable, versioning handles changes)
- Implement template composition or inheritance (single template = single deployment object)

## Requirements **\[REQUIRED\]**

### User Requirements

- **User Characteristics**: Platform engineers and generator developers familiar with Kubernetes YAML, JSON Schema, and basic templating concepts
- **System Functionality**: Create reusable deployment object templates with runtime parameter substitution, validate compatibility before deployment
- **User Interfaces**: RESTful API for programmatic access, Admin UI for manual template management and instantiation

### System Requirements

- **Functional Requirements**:
  - REQ-001: Templates must support full Tera templating syntax (variables, conditionals, loops, filters)
  - REQ-002: Parameters must be validated against JSON Schema before template rendering
  - REQ-003: Templates must support label/annotation targeting matching stack targeting logic
  - REQ-004: Template instantiation must create deployment object with provenance tracking
  - REQ-005: Templates must be versioned immutably (each update creates new version)
  - REQ-006: System templates (generator_id=NULL) must be admin-only, generator templates modifiable by owner + admin

- **Non-Functional Requirements**:
  - NFR-001: Template syntax must be validated at creation time (fail fast)
  - NFR-002: Template rendering must not timeout (trust users, no artificial limits)
  - NFR-003: Template/stack label mismatch must return 422 with detailed error
  - NFR-004: Template content size is unlimited (TEXT field, no artificial constraints)

## Use Cases **\[CONDITIONAL: User-Facing Initiative\]**

### Use Case 1: Single PostgreSQL Database from Template

- **Actor**: Control plane generator managing database services
- **Scenario**:
  1. Generator creates stack for new database deployment
  2. Generator calls `POST /api/v1/stacks/{stack_id}/deployment-objects/from-template` with postgres-template and parameters: database_name, storage_size, cpu_limit, memory_limit
  3. Broker validates template labels match stack labels, validates parameters against JSON Schema
  4. Broker renders template with Tera, creates deployment object containing Deployment + Service + PVC resources
  5. Broker creates provenance record in rendered_deployment_objects table
  6. Agent deploys rendered YAML to target cluster
- **Expected Outcome**: PostgreSQL database deployed with requested configuration, full audit trail maintained

### Use Case 2: Multi-Tier Application from Single Template

- **Actor**: Platform engineer deploying 3-tier application
- **Scenario**:
  1. Engineer creates stack with labels: app=myapp, env=prod
  2. Engineer has "3-tier-app-template" containing multi-document YAML with PostgreSQL (Deployment + Service + PVC), Backend (Deployment + Service), Frontend (Deployment + Service + Ingress)
  3. Engineer calls `POST /api/v1/stacks/{stack_id}/deployment-objects/from-template` with template and parameters: db_size, backend_replicas, frontend_replicas, image_tags
  4. Broker renders template with Tera, creates one deployment object containing all 9 K8s resources
  5. Agent deploys single deployment object (multi-document YAML) to cluster
- **Expected Outcome**: Complete 3-tier application deployed from single template instantiation, all resources in one deployment object

### Use Case 3: Environment Promotion with Template Versioning

- **Actor**: CI/CD system managing environment promotions
- **Scenario**:
  1. CI/CD creates dev stack, instantiates app-template v1 with parameters: replicas=1, memory=512Mi
  2. After testing, CI/CD creates staging stack, instantiates app-template v1 with parameters: replicas=2, memory=1Gi
  3. Template is updated (bug fix), becomes v2
  4. CI/CD creates prod stack, instantiates app-template v2 with parameters: replicas=5, memory=2Gi
  5. Each deployment object stores provenance: which template version + which parameters used
- **Expected Outcome**: Consistent deployments across environments with version control, ability to reconstruct any deployment

## Architecture **\[CONDITIONAL: Technically Complex Initiative\]**

### Overview

The templating system extends Brokkr's existing generator/stack/deployment object model while following established architectural patterns:

**Core Concept:** Templates are reusable definitions for deployment objects. Each template contains multi-document YAML (potentially many K8s resources) with parameter placeholders. Template instantiation renders this YAML and creates a single versioned deployment object.

**Key Components:**
- **Stack Templates**: Immutable versioned templates with parameter schemas (similar to deployment object versioning)
- **Template Targeting**: Labels/annotations/targets system mirroring agent targeting for validation
- **Provenance Tracking**: `rendered_deployment_objects` table tracks template_id + version + parameters for reconstruction
- **Tera Engine**: Full-featured templating with syntax validation at creation time
- **RESTful Integration**: Templates instantiate via `POST /stacks/{id}/deployment-objects/from-template`

### Architectural Principles

1. **Template = Single Deployment Object**: One template renders to one deployment object (containing potentially many K8s resources as multi-document YAML)
2. **Immutable Versioning**: Templates never mutate; updates create new version rows (v1, v2, v3...)
3. **Targeting Validation**: Template labels/annotations must match stack using same logic as agent targeting
4. **Complete Provenance**: Every template-generated deployment object stores full reconstruction data
5. **Open Internal Access**: Any generator can instantiate any template (defensive validation via label matching)

### Data Model Extensions

**New Tables:**
- `stack_templates`: Template definitions with Tera content and JSON Schema parameters
- `template_labels`: Template label selectors (mirrors stack_labels/agent_labels pattern)
- `template_annotations`: Template annotations (mirrors stack_annotations/agent_annotations pattern)
- `template_targets`: Pre-computed template→stack compatibility (mirrors agent_targets pattern)
- `rendered_deployment_objects`: Provenance tracking (one-to-one with deployment_objects)

**Template Ownership Model:**
- `generator_id` nullable: NULL = system template (admin-only modify), non-NULL = generator-owned
- Modification permissions: admins modify any, generators modify only their own
- Instantiation permissions: any generator can instantiate any template (open internal access)

### API Extensions

**Template Management:**
- `POST /api/v1/templates` - Create new template (auto-increments version if name+generator_id exists)
- `GET /api/v1/templates/{id}` - Retrieve template by ID
- `GET /api/v1/templates?generator_id={id}&name={name}` - List versions of a template
- `PUT /api/v1/templates/{id}` - Update template (creates new version, soft-deletes old)
- `DELETE /api/v1/templates/{id}` - Soft-delete template
- `POST /api/v1/templates/{id}/labels` - Add label to template
- `POST /api/v1/templates/{id}/annotations` - Add annotation to template

**Template Instantiation:**
- `POST /api/v1/stacks/{stack_id}/deployment-objects/from-template`
  - Body: `{ template_id: UUID, parameters: {...} }`
  - Returns: DeploymentObject (same shape as manual creation)
  - Validates: (1) template/stack labels match, (2) parameters against JSON Schema, (3) Tera rendering succeeds
  - Creates: deployment_objects row + rendered_deployment_objects provenance row
  - Error handling: follows existing pattern (400 for validation, 422 for label mismatch, 500 for DB errors)

### Template Engine Design

- **Engine**: Tera (Rust-native, Jinja2-inspired templating)
- **Syntax**: `{{ variable }}`, `{% if condition %}`, `{% for item in items %}`
- **Features**: Full Tera support (filters, macros, whitespace control, template inheritance)
- **Validation**: Tera syntax validated at template creation time (fail fast)
- **Parameters**: JSON Schema validation before rendering (enforced at instantiation)
- **No Artificial Limits**: No rendering timeout, no content size limits (trust users)

## Detailed Design **\[REQUIRED\]**

### Database Schema

```sql
-- Core template table
CREATE TABLE stack_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    -- Ownership: NULL = system template (admin-only), non-NULL = generator-owned
    generator_id UUID REFERENCES generators(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    description TEXT,
    version INTEGER NOT NULL DEFAULT 1,

    -- Template content (Tera syntax) and parameters (JSON Schema as TEXT, not JSONB)
    template_content TEXT NOT NULL,
    parameters_schema TEXT NOT NULL, -- JSON Schema stored as string

    -- Integrity
    checksum VARCHAR(64) NOT NULL, -- SHA-256 of template_content

    -- Version uniqueness: (generator_id, name, version) must be unique
    -- NULL generator_id is treated as distinct value (system templates)
    UNIQUE(generator_id, name, version)
);

-- Template labels (mirrors stack_labels/agent_labels pattern)
CREATE TABLE template_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES stack_templates(id) ON DELETE CASCADE,
    label VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, label)
);

-- Template annotations (mirrors stack_annotations/agent_annotations pattern)
CREATE TABLE template_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES stack_templates(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, key)
);

-- Template targets (mirrors agent_targets pattern)
-- Pre-computed template→stack compatibility for efficient querying
CREATE TABLE template_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES stack_templates(id) ON DELETE CASCADE,
    stack_id UUID NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, stack_id)
);

-- Provenance tracking for template-generated deployment objects
-- One-to-one relationship with deployment_objects
CREATE TABLE rendered_deployment_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_object_id UUID NOT NULL REFERENCES deployment_objects(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES stack_templates(id),
    template_version INTEGER NOT NULL,
    template_parameters TEXT NOT NULL, -- JSON string of parameters used for rendering
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(deployment_object_id) -- One deployment object has at most one template source
);

-- Indexes for performance
CREATE INDEX idx_stack_templates_generator ON stack_templates(generator_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_stack_templates_name ON stack_templates(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_template_labels_template ON template_labels(template_id);
CREATE INDEX idx_template_annotations_template ON template_annotations(template_id);
CREATE INDEX idx_template_targets_template ON template_targets(template_id);
CREATE INDEX idx_template_targets_stack ON template_targets(stack_id);
CREATE INDEX idx_rendered_deployment_objects_template ON rendered_deployment_objects(template_id, template_version);

-- Trigger for updated_at
CREATE TRIGGER update_stack_templates_timestamp
BEFORE UPDATE ON stack_templates
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
```

### API Endpoints

**Template Management (CRUD):**

```bash
# Create new template (or new version if name+generator_id exists)
POST /api/v1/templates
{
  "generator_id": "uuid" | null,  # null = system template
  "name": "postgres-db",
  "description": "PostgreSQL database deployment",
  "template_content": "...",  # Tera YAML with {{ variables }}
  "parameters_schema": "{...}"  # JSON Schema as string
}

# Retrieve specific template
GET /api/v1/templates/{template_id}

# List all versions of a template
GET /api/v1/templates?generator_id={id}&name={name}

# Update template (creates new version, soft-deletes old)
PUT /api/v1/templates/{template_id}
{
  "description": "Updated description",
  "template_content": "...",
  "parameters_schema": "{...}"
}

# Soft-delete template
DELETE /api/v1/templates/{template_id}

# Add label to template
POST /api/v1/templates/{template_id}/labels
{ "label": "env=prod" }

# Add annotation to template
POST /api/v1/templates/{template_id}/annotations
{ "key": "team", "value": "platform" }
```

**Template Instantiation:**

```bash
# Instantiate template into stack (creates deployment object)
POST /api/v1/stacks/{stack_id}/deployment-objects/from-template
{
  "template_id": "uuid",
  "parameters": {
    "database_name": "myapp",
    "storage_size": "10Gi",
    "replicas": 3
  }
}

# Returns: DeploymentObject (same as manual creation)
# Also creates: rendered_deployment_objects provenance record
```

**Validation Flow:**
1. Verify template exists and is not soft-deleted
2. Check template labels/annotations match stack (using agent targeting matcher logic)
3. Validate parameters against JSON Schema
4. Render template with Tera
5. Create deployment object with rendered YAML
6. Create provenance record in rendered_deployment_objects

**Error Responses:**
- 400 Bad Request: Invalid parameters, Tera rendering failed, JSON Schema validation failed
- 404 Not Found: Template or stack doesn't exist
- 422 Unprocessable Entity: Template/stack label mismatch (includes detailed mismatch info)
- 500 Internal Server Error: Database errors

### Template Engine Implementation

**Tera Integration:**
- Use `tera` crate for template processing (Rust-native, Jinja2-inspired)
- Validate Tera syntax at template creation time (compile template, fail if invalid)
- Use `jsonschema` crate for parameter validation before rendering
- Template compilation and caching for performance optimization
- Full Tera feature support: filters, macros, whitespace control, template inheritance

**Label/Annotation Matching:**
- Reuse existing agent targeting matcher code
- Templates with no labels = match any stack ("go anywhere")
- Templates with labels = must match stack labels (defensive check)
- Template targets table pre-computed by background job (similar to agent_targets)

### Integration Points

- Templates create standard deployment objects (agents see no difference)
- Deployment objects contain final rendered YAML (agents never see templates)
- Existing audit trails and versioning work unchanged
- Provenance tracked separately in rendered_deployment_objects table
- No changes needed to agent polling or deployment logic

## UI/UX Design **\[CONDITIONAL: Frontend Initiative\]**

### Admin UI Extensions

The existing admin UI will be extended with template management capabilities:

**Template List View**

- New "Templates" section in generator detail pages
- Table showing template name, version, description, created date
- Actions: View, Edit, Delete, Instantiate

**Template Creation Form**

- Template name and description fields
- Code editor for template YAML content with syntax highlighting
- JSON Schema editor for parameter definitions
- Preview functionality showing parameter form

**Template Instantiation Interface**

- Dynamic form generation based on parameter schema
- Real-time parameter validation
- Preview of rendered YAML before deployment
- Target stack selection dropdown

### Integration with Existing Design

- Follows existing admin UI patterns and styling
- Uses same authentication and navigation structure
- Consistent with current stack and deployment object management flows

## Testing Strategy **\[CONDITIONAL: Separate Testing Initiative\]**

### Unit Testing

- **Strategy**: Test template engine, parameter validation, and API endpoints in isolation
- **Coverage Target**: 90% code coverage for new templating components
- **Tools**: Rust built-in testing framework, mockall for mocking database calls

### Integration Testing

- **Strategy**: Test full template creation and instantiation workflows with real database
- **Test Environment**: Dedicated test PostgreSQL instance
- **Data Management**: Test fixtures for generators, stacks, and templates with cleanup

### System Testing

- **Strategy**: End-to-end testing via API calls, including template creation through deployment
- **User Acceptance**: Template creation and instantiation scenarios with platform team
- **Performance Testing**: Template instantiation latency under load (100+ concurrent requests)

### Test Selection

- Template CRUD operations and access control
- Parameter validation edge cases (missing, invalid, type mismatches)
- Template engine functionality (variables, conditionals, loops)
- Error handling for malformed templates and parameters
- Integration with existing deployment object workflows

### Bug Tracking

- Template-related issues tracked in existing project issue system
- Priority based on impact to core templating functionality
- Performance regressions block release

## Alternatives Considered **\[REQUIRED\]**

**1. External Templating Tools (Helm, Kustomize)**

- **Pros**: Mature, well-supported, familiar to users
- **Cons**: Adds external dependencies, doesn't integrate with Brokkr's generator/audit model, requires additional infrastructure
- **Decision**: Rejected due to complexity and desire for native integration

**2. Full Programming Language Templating (Lua, JavaScript)**

- **Pros**: Maximum flexibility, powerful conditional logic
- **Cons**: Security concerns, complexity, harder to validate and debug
- **Decision**: Rejected in favor of simpler, safer approach

**3. YAML Merge/Patch Approach**

- **Pros**: Leverages existing YAML tools, no new syntax
- **Cons**: Limited expressiveness, complex for conditional logic, harder to understand
- **Decision**: Rejected due to usability concerns

**4. Client-Side Templating Only**

- **Pros**: Simpler server implementation, more flexible for generators
- **Cons**: Duplicates templating logic across generators, no centralized template management
- **Decision**: Rejected in favor of centralized approach for reusability

## Implementation Plan **\[REQUIRED\]**

### Phase 1: Database Schema and Models (3 weeks)

- Create migration for 5 new tables: stack_templates, template_labels, template_annotations, template_targets, rendered_deployment_objects
- Implement Diesel models for all new tables
- Create DAL (Data Access Layer) for template operations
- Write unit tests for DAL operations
- Implement template versioning logic (auto-increment version on name+generator_id match)

### Phase 2: Template CRUD API (3 weeks)

- Implement template creation endpoint with Tera syntax validation
- Implement template retrieval, listing, update, delete endpoints
- Add label/annotation management endpoints for templates
- Implement authorization checks (admin-only for system templates, owner+admin for generator templates)
- Write API integration tests
- Add OpenAPI documentation for template endpoints

### Phase 3: Label/Annotation Matching (2 weeks)

- Extract agent targeting matcher code into reusable module
- Implement template/stack label matching using extracted matcher
- Create background job to populate template_targets table
- Implement validation logic for template instantiation (422 on mismatch)
- Write tests for matching logic edge cases (no labels, partial match, exact match)

### Phase 4: Template Instantiation (3 weeks)

- Implement `POST /stacks/{id}/deployment-objects/from-template` endpoint
- Integrate Tera rendering with parameter substitution
- Implement JSON Schema parameter validation
- Create deployment object + rendered_deployment_objects provenance record
- Error handling for all validation failures (400, 422, 500)
- Integration tests for end-to-end instantiation workflow
- Performance optimization: template compilation caching

### Phase 5: Admin UI Integration (3 weeks)

- Template list view in admin UI (filterable by generator, name)
- Template creation/edit form with Tera syntax highlighting
- JSON Schema editor for parameters with validation
- Template instantiation form with dynamic parameter fields (generated from JSON Schema)
- Preview pane showing rendered YAML before instantiation
- UI tests for template management workflows

### Phase 6: Documentation and Examples (2 weeks)

- API documentation for all template endpoints
- Template authoring guide (Tera syntax, best practices, JSON Schema)
- Example templates: PostgreSQL, Redis, 3-tier application, StatefulSet
- Migration guide for generators (when to use templates vs manual YAML)
- Performance benchmarks (template instantiation latency)
- Troubleshooting guide (common Tera errors, parameter validation failures, label mismatches)

**Total Timeline: 16 weeks**

### Success Criteria

- Templates can be created with full Tera syntax and JSON Schema validation
- Template versioning works correctly (immutable versions, auto-increment)
- Label/annotation matching prevents incompatible template→stack instantiations
- Template instantiation creates deployment object with full provenance tracking
- Any template-generated deployment can be reconstructed from template_id + version + parameters
- Template system has zero impact on agent deployment workflows
- Performance impact < 5% on deployment object creation (compared to manual YAML)
- Admin UI provides intuitive template authoring and instantiation experience
- System templates (generator_id=NULL) are admin-only, generator templates follow ownership rules