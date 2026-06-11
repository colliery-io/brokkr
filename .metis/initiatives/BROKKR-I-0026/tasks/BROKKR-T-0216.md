---
id: docs-modernize-submission-examples
level: task
title: "Docs: modernize submission examples onto the yaml/CLI on-ramp"
short_code: "BROKKR-T-0216"
created_at: 2026-06-11T11:02:08.324875+00:00
updated_at: 2026-06-11T11:02:08.324875+00:00
parent: docs-and-ci-hygiene-staleness
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: BROKKR-I-0026
---

# Docs: modernize submission examples onto the yaml/CLI on-ramp

## Parent Initiative

[[BROKKR-I-0026]]

## Objective

The tutorials and quick start still teach the pre-I-0021 submission path — hand-escaped `yaml_content` JSON — while `how-to/managing-stacks.md` now labels raw `application/yaml` "recommended" and the CLI exists for exactly the CI use case. Misleads by omission; modernize without breaking each page's Diátaxis lane.

## Acceptance Criteria

- [ ] `tutorials/cicd-generators.md`: Steps 5-7 (escaped one-liners at lines 104, 119; `jq -Rs` GitHub Actions step at 182-188) rewritten around `brokkr apply -f ./k8s --stack …` (or raw-YAML curl at minimum); cli-apply.md added to Next Steps. This tutorial is the on-ramp's headline use case.
- [ ] `getting-started/quick-start.md`: the two jq-wrapped curls (lines ~138-145, 239-243) → raw `--data-binary @file -H "Content-Type: application/yaml"`; Next Steps (287-296) links cli-apply.md.
- [ ] `tutorials/first-deployment.md` (101-107, 160-166): keep the tutorial single-path, but note the raw application/yaml alternative once and link the on-ramp in Next Steps (221-226).
- [ ] `tutorials/multi-cluster-targeting.md` (~140-172): cross-link `brokkr apply --target-label` for the fan-out it demonstrates.
- [ ] `getting-started/installation.md` smoke test (~305-330): optional one-line raw-YAML simplification.
- [ ] Each page still respects its quadrant (tutorials stay single-path; no option catalogs added).

## Status Updates

*To be added during implementation*
