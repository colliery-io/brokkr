---
id: enhance-architecture-md-with
level: task
title: "Enhance architecture.md with component interaction diagrams"
short_code: "BROKKR-T-0083"
created_at: 2025-12-30T02:06:08.992606+00:00
updated_at: 2025-12-30T02:20:04.352756+00:00
parent: BROKKR-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0011
---

# Enhance architecture.md with component interaction diagrams

## Parent Initiative

[[BROKKR-I-0011]] - Architecture Reference Documentation

## Objective

Enhance the existing `docs/content/explanation/architecture.md` with detailed Mermaid diagrams showing how internal components interact within the broker and agent services.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add broker internal component diagram (API → DAL → Database, Event Bus, Background Tasks)
- [ ] Add agent internal component diagram (Broker Client, K8s Client, Reconciler, State Cache)
- [ ] Add cross-service interaction diagram (Broker ↔ Agent communication patterns)
- [ ] Diagrams use Mermaid syntax compatible with Hugo/Geekdoc theme
- [ ] Each diagram has explanatory text describing the interactions

## Documentation Sections

### Diagrams to Add

1. **Broker Internal Architecture**
   - API layer (Axum routes, middleware, handlers)
   - DAL layer (database operations, connection pooling)
   - Event Bus (internal pub/sub, webhook dispatch)
   - Background Tasks (cleanup, notifications, config reload)
   - Utils (audit logger, config management)

2. **Agent Internal Architecture**
   - Broker Client (REST API communication, polling)
   - Kubernetes Client (dynamic client, resource operations)
   - Reconciler (state comparison, apply/delete operations)
   - State Cache (deployment object tracking)

3. **Cross-Service Interactions**
   - Agent registration and heartbeat
   - Deployment object fetch and status reporting
   - Event reporting flow

### Location

Update: `docs/content/explanation/architecture.md`

## Implementation Notes

- Review existing diagrams in architecture.md to maintain consistency
- Use `{{< mermaid >}}` Hugo shortcode for diagrams
- Focus on clarity over completeness - show key interactions