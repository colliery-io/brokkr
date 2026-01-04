---
id: add-configmap-watcher-for
level: task
title: "Add ConfigMap watcher for Kubernetes hot-reload"
short_code: "BROKKR-T-0075"
created_at: 2025-12-29T19:32:33.563123+00:00
updated_at: 2025-12-29T19:57:06.326619+00:00
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

# Add ConfigMap watcher for Kubernetes hot-reload

## Parent Initiative

[[BROKKR-I-0009]]

## Objective

Implement a Kubernetes ConfigMap watcher that automatically detects changes to the broker's ConfigMap and triggers configuration reload. Uses the `kube` crate's watch API with debouncing to prevent rapid successive reloads.

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

- [ ] ConfigMap watcher spawned as background task on broker startup
- [ ] Watches for changes to broker's ConfigMap by name/namespace
- [ ] Debounces rapid changes (5 second window)
- [ ] Triggers `ReloadableConfig::reload()` on change detection
- [ ] Logs configuration changes at INFO level
- [ ] Graceful handling when not running in Kubernetes (feature disabled)
- [ ] Optional: can be disabled via config flag

## Implementation Notes

### Files to Create/Modify
- `crates/brokkr-broker/src/utils/config_watcher.rs` - New module for ConfigMap watching
- `crates/brokkr-broker/src/cli/commands.rs` - Spawn watcher task in serve command

### Technical Approach
```rust
pub async fn watch_configmap(
    config: Arc<ReloadableConfig>,
    namespace: &str,
    configmap_name: &str,
) -> Result<(), Error> {
    let client = Client::try_default().await?;
    let cms: Api<ConfigMap> = Api::namespaced(client, namespace);
    
    let watcher = watcher(cms, watcher::Config::default()
        .labels(&format!("app.kubernetes.io/name={}", configmap_name)));
    
    // Debounce and reload on changes
}
```

### Environment Detection
- Check for `KUBERNETES_SERVICE_HOST` env var to detect K8s environment
- Read namespace from `/var/run/secrets/kubernetes.io/serviceaccount/namespace`
- ConfigMap name from Helm release or env var

### Dependencies
- Depends on T-0073 (ReloadableConfig wrapper)
- Uses `kube` crate (already a dependency in brokkr-agent)

## Status Updates

*To be added during implementation*