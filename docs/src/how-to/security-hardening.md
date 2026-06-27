# Security Hardening

This guide collects the operational security tasks for running Brokkr in production: what to verify before go-live, what to monitor, and what to do when you suspect a compromise. For the underlying trust model and authentication design, see the [Security Model](../explanation/security-model.md).

## Production Deployment Checklist

Before deploying Brokkr to production, verify these security configurations:

- [ ] **TLS Everywhere**: Enable TLS for all external connections via ingress or direct TLS
- [ ] **Strong Secrets**: Use cryptographically secure random values for all PAKs and encryption keys
- [ ] **External Database**: Use managed PostgreSQL with encryption at rest
- [ ] **Secret Management**: Store credentials in Kubernetes Secrets or external vault
- [ ] **NetworkPolicy**: Enable and configure network policies to restrict traffic (see [Network Configuration](./network-configuration.md))
- [ ] **RBAC**: Use minimal required permissions for service accounts (namespace-scoped agents watch and report health in-namespace via `BROKKR__AGENT__WATCH_NAMESPACE`; cluster-wide RBAC is required only for cluster-scoped resources and cross-namespace pruning; see [Security Model](../explanation/security-model.md))
- [ ] **Agent Registration**: Register each agent with the generators whose stacks it should target — at startup via the agent's `--generator-ids` flag or `BROKKR__AGENT__GENERATOR_IDS`, or after deployment with `brokkr register`. An unregistered agent can only serve system/fleet-scoped stacks; explicit targets for any other generator's stacks are rejected with `agent_not_registered` (see [Configuring Agent Scopes](#configuring-agent-scopes) and [Agent Registration](./agent-registration.md))
- [ ] **Pod Security**: Enable pod security standards (restricted profile)
- [ ] **Audit Logging**: Enable and monitor audit logs
- [ ] **Resource Limits**: Set CPU/memory limits to prevent resource exhaustion
- [ ] **Image Scanning**: Scan container images for vulnerabilities before deployment
- [ ] **Restrict `/metrics`**: The broker's `/metrics` endpoint is unauthenticated; limit access to your monitoring infrastructure via NetworkPolicy (`networkPolicy.allowMetricsFrom`) or firewall rules

## Configuring Agent Scopes

An agent can only have a generator's stacks targeted at it once it is registered with that generator — Brokkr's application-level access control (see [Generator Registration and Application Scopes](../explanation/security-model.md#generator-registration-and-application-scopes)). Configure the scopes an agent serves at deployment time:

1. **Set the generator IDs on the agent.** Provide a comma-separated list of generator UUIDs (for example, `BROKKR__AGENT__GENERATOR_IDS=<gen-id-1>,<gen-id-2>`) through one of the following sources, highest precedence first:
   - the `--generator-ids` CLI flag,
   - the `BROKKR__AGENT__GENERATOR_IDS` environment variable (config key `agent.generator_ids`),
   - the legacy `BROKKR_GENERATOR_IDS` variable (**deprecated** — still honored, but logs a warning).

   For Helm deployments, set `broker.generatorIds` (a YAML list or comma-separated string) in the `brokkr-agent` chart; it renders to `BROKKR__AGENT__GENERATOR_IDS` in the agent ConfigMap.

2. **Confirm the resulting scope.** An empty or unset value means the agent serves system/fleet scope only. Every agent is auto-registered with the system generator at creation, so fleet-wide stacks always reach it regardless of this setting.

To add or remove scopes after deployment, register or deregister agents with the `brokkr` CLI; see [Agent Registration](./agent-registration.md) for the operational steps. Configuration keys are documented in [Environment Variables](../reference/environment-variables.md).

## Monitoring for Security Events

Monitor these metrics and events for security-relevant activity:

| Indicator | Alert Threshold | Potential Issue |
|-----------|-----------------|-----------------|
| Failed authentication rate | > 10/minute | Brute force attack |
| Unexpected agent disconnections | Any | Possible compromise or network attack |
| Webhook delivery failure rate | > 50% | Network issues or endpoint compromise |
| Audit log volume spike | 10x normal | Unusual activity, possible attack |
| Admin action from unknown IP | Any | Credential theft |

See the [Monitoring reference](../reference/monitoring.md) for the available metrics and the [Audit Logs how-to](./audit-logs.md) for querying audit data.

## Incident Response

### Suspected Agent Compromise

If you suspect an agent's credentials have been compromised:

1. **Revoke immediately**: Delete or disable the agent via the admin API
2. **Review audit logs**: Search for unusual actions by the agent's actor_id
3. **Inspect cluster**: Review resources the agent may have created or modified
4. **Rotate secrets**: Generate new PAK if re-enabling the agent
5. **Investigate**: Determine how the compromise occurred

### Suspected Broker Compromise

If you suspect the broker itself has been compromised:

1. **Isolate**: Remove external network access to the broker
2. **Preserve evidence**: Capture logs, database state, and container images
3. **Rotate all credentials**: Generate new PAKs for all agents, generators, and admins. For the admin PAK, the offline `brokkr-broker generate-pak` command mints a PAK and its SHA-256 hash without a database or keyfile; set `BROKKR__BROKER__PAK_HASH` to that hash before the broker's next startup (see the [CLI reference](../reference/cli.md))
4. **Review webhooks**: Check for unauthorized webhook subscriptions
5. **Audit database**: Look for unauthorized modifications to stacks or agents
6. **Rebuild**: Consider deploying fresh broker instances rather than cleaning compromised ones

## Related Documentation

- [Security Model](../explanation/security-model.md) — trust boundaries, authentication, and authorization design
- [Agent Registration](./agent-registration.md) — registering agents with generators and managing scopes after deployment
- [Managing PAKs](./pak-management.md) — credential creation and rotation
- [CLI Reference](../reference/cli.md) — `brokkr register` / `deregister` / `registrations` and `brokkr-broker generate-pak`
- [Network Configuration](./network-configuration.md) — TLS, NetworkPolicy, and firewall setup
