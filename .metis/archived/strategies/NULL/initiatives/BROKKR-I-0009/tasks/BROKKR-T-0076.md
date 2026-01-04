---
id: document-reloadable-settings-and
level: task
title: "Document reloadable settings and update Helm charts"
short_code: "BROKKR-T-0076"
created_at: 2025-12-29T19:32:33.666580+00:00
updated_at: 2025-12-29T20:00:23.429490+00:00
parent: BROKKR-I-0009
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0009
---

# Document reloadable settings and update Helm charts

## Parent Initiative

[[BROKKR-I-0009]]

## Objective

Document which configuration settings can be hot-reloaded vs which require a restart. Update Helm chart values.yaml with annotations indicating reloadability and add operational documentation.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Helm chart values.yaml updated with comments indicating hot-reload vs restart-required
- [ ] ConfigMap template updated with reload annotations where applicable
- [ ] values.yaml includes `configReload.enabled` option
- [ ] README or docs section explaining hot-reload feature

## Implementation Notes

### Files to Modify
- `charts/brokkr-broker/values.yaml` - Add reload annotations/comments
- `charts/brokkr-broker/templates/configmap.yaml` - Add reload trigger annotation

### Values.yaml Annotations
```yaml
# Hot-reloadable settings (changes apply without restart)
log:
  # @hot-reload: true
  level: info

webhook:
  # @hot-reload: true
  deliveryIntervalSeconds: 5
  # @hot-reload: true  
  deliveryBatchSize: 50

# Static settings (changes require pod restart)
database:
  # @hot-reload: false - requires restart
  url: postgres://...
```

### ConfigMap Reload Annotation
```yaml
metadata:
  annotations:
    # Triggers rolling restart when ConfigMap changes (if using Reloader)
    configmap.reloader.stakater.com/reload: "true"
```

### Documentation Content
- List of hot-reloadable settings
- List of restart-required settings
- How to trigger manual reload via API
- How ConfigMap watcher works in Kubernetes

### Dependencies
- Depends on T-0073, T-0074, T-0075 being complete

## Status Updates

*To be added during implementation*