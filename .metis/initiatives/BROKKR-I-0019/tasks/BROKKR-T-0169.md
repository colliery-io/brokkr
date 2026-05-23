---
id: ws-14-docs-c4-update-agent-ops
level: task
title: "WS-14: Docs — C4 update, agent ops docs, retention/Datadog guidance"
short_code: "BROKKR-T-0169"
created_at: 2026-05-23T02:12:49.896063+00:00
updated_at: 2026-05-23T02:12:49.896063+00:00
parent: BROKKR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0019
---

# WS-14: Docs — C4 update, agent ops docs, retention/Datadog guidance

**Parent**: [[BROKKR-I-0019]] · **ADR**: [[BROKKR-A-0008]] · **Stance**: [[project_log_retention_stance]] · **Convention**: [[feedback_c4_architecture]]

## Objective

Documentation pass for the new internal WS channel and log/event streaming. Update C4 diagrams, write agent operations docs, and publish the retention stance with explicit "use Datadog for long-term" guidance.

## Acceptance Criteria

- [ ] C4 container diagram updated to show the internal WS edge between broker and agent (distinct from the external REST surface)
- [ ] C4 component diagram updated to show broker-side ConnectionRegistry, fan-out hub, and eviction worker
- [ ] Agent ops docs cover: WS-on-by-default behavior, `force_rest_only` flag, behavior under disconnect, expected backoff schedule
- [ ] New docs page on event/log retention: 6h ceiling, per-stack override (down only), continuous eviction, recommended long-term sinks (Datadog)
- [ ] Ops docs include guidance on ingress/proxy timeouts required for long-lived WS subscriptions (WS-11/WS-12)
- [ ] Docs build cleanly via `angreal docs build`

## Implementation Notes

- **Approach**: update existing docs in the docs site; add a new page rather than burying retention guidance inside operations.
- **Dependencies**: most functional tasks complete (so docs reflect shipped behavior, not aspirations).
- **Risk**: docs drift if ADR is later amended. Link the retention page back to [[project_log_retention_stance]] so the stance is canonical.