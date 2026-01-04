---
id: create-network-flows-md
level: task
title: "Create network-flows.md documentation"
short_code: "BROKKR-T-0084"
created_at: 2025-12-30T02:06:09.056421+00:00
updated_at: 2025-12-30T02:21:36.655965+00:00
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

# Create network-flows.md documentation

## Parent Initiative

[[BROKKR-I-0011]] - Architecture Reference Documentation

## Objective

Create a new documentation page describing network traffic flows between Brokkr components, useful for operators configuring firewalls, network policies, and understanding system connectivity requirements.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Document all network connections between components
- [ ] List ports, protocols, and connection directions
- [ ] Include network topology diagram showing traffic flows
- [ ] Document TLS requirements and certificate flows
- [ ] Provide Kubernetes NetworkPolicy examples
- [ ] File created at `docs/content/explanation/network-flows.md`

## Documentation Sections

### Content Outline

1. **Network Topology Overview**
   - High-level diagram of all components and connections
   - External vs internal traffic distinction

2. **Broker Network Requirements**
   - Inbound: API (HTTP/HTTPS on port 3000)
   - Inbound: Admin UI (if applicable)
   - Outbound: PostgreSQL (port 5432)
   - Outbound: Webhook destinations (configurable)

3. **Agent Network Requirements**
   - Outbound: Broker API (HTTP/HTTPS)
   - Outbound: Kubernetes API server (port 6443)
   - No inbound connections required

4. **Connection Details Table**
   | Source | Destination | Port | Protocol | Direction | Purpose |
   |--------|-------------|------|----------|-----------|---------|
   | Agent | Broker | 3000 | HTTPS | Outbound | API calls |
   | etc... |

5. **TLS Configuration**
   - Certificate requirements
   - mTLS options (if supported)
   - Certificate rotation considerations

6. **Kubernetes NetworkPolicy Examples**
   - Sample policies for broker namespace
   - Sample policies for agent namespace
   - Egress policies for webhook delivery

### Location

Create: `docs/content/explanation/network-flows.md`

## Implementation Notes

- Reference actual ports from Helm chart values
- Ensure accuracy by reviewing broker and agent code for all network calls