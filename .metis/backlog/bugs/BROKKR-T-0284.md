---
id: docs-every-install-tutorial-flow
level: task
title: "Docs: every install/tutorial flow leaves the agent INACTIVE (and often nameless), so documented verifications can never succeed"
short_code: "BROKKR-T-0284"
created_at: 2026-07-27T14:27:42.671054+00:00
updated_at: 2026-07-27T17:50:54.465282+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: every install/tutorial flow leaves the agent INACTIVE (and often nameless), so documented verifications can never succeed

## Objective

Fix the single most damaging correctness failure in the onboarding docs: agents are created with status `INACTIVE`, and the agent binary gates all reconciliation on `agent.status != "ACTIVE"` (five gates, `crates/brokkr-agent/src/cli/commands.rs:427,545,582,620,706` — code-verified 2026-07-27). No getting-started flow ever activates the agent, so every documented end-to-end verification silently fails:

- `getting-started/installation.md` — Test Deployment's `kubectl get namespace brokkr-test` never succeeds; the doc's own step-4 sample response even shows `"status": "INACTIVE"`. Additionally the agent helm install omits `broker.agentName`/`broker.clusterName`, so the agent's startup self-lookup 404s ("Agent not found") and the pod crashloops before status even matters.
- `getting-started/evaluate.md` — same missing agentName/clusterName crashloop after creating `eval-agent`/`evaluation`. Notably evaluate.md Path A DOES `PUT` status `ACTIVE` — proof the step is known and required, yet absent everywhere else.
- `tutorials/first-deployment.md` — never activates the dev agent; Step 5's promised DEPLOY SUCCESS event and `kubectl get all -n tutorial-nginx` output can't appear.
- `tutorials/cicd-generators.md` — inherits the same gap; its verification only inspects `sequence_id`, so the failure is silent.

Full findings with per-reviewer evidence: `docs/REVIEW-2026-07-27.md` (search "INACTIVE").

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P0 - Critical (every new consumer's first deployment fails with no error message)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Every flow that creates an agent includes the activation step (as evaluate.md Path A does) with a short note on why, or the product decision is made to auto-activate on first heartbeat and docs follow the code.
- [x] All agent helm install examples set `broker.agentName` and `broker.clusterName` matching the created agent.
- [x] Each flow adds a "wait one poll cycle (default 30s)" note before verification.
- [ ] The four flows above are executed end-to-end against a fresh install and their verification commands succeed as written.

## Status Updates

**2026-07-27** — Doc edits landed (agent-assisted, claims re-verified against code):
- installation.md: activation PUT added to Test Deployment (evaluate.md Path A pattern); INACTIVE note on the step-4 sample response; `agentName`/`clusterName` added to all three agent helm examples with crashloop explanation; poll-cycle waits before verifications.
- evaluate.md: Path B helm install gains `agentName`/`clusterName`; poll-cycle waits added both paths (10s dev binary default / 30s chart default). Activation already existed.
- first-deployment.md: Step 3 retitled with activation PUT inserted; prerequisites note the dev agent starts INACTIVE; Step 5 troubleshooting callout (empty events → agent likely INACTIVE).
- cicd-generators.md: prerequisite pointer to first-deployment Step 3 activation; Step 4 reminder + silent-failure note; Step 5 poll-cycle note.

Remaining: end-to-end execution of the four flows against a fresh install (needs a live cluster; suggest running alongside the e2e suite at next release since e2e only runs on release/nightly).