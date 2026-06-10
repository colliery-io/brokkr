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
- [ ] **Pod Security**: Enable pod security standards (restricted profile)
- [ ] **Audit Logging**: Enable and monitor audit logs
- [ ] **Resource Limits**: Set CPU/memory limits to prevent resource exhaustion
- [ ] **Image Scanning**: Scan container images for vulnerabilities before deployment
- [ ] **Restrict `/metrics`**: The broker's `/metrics` endpoint is unauthenticated; limit access to your monitoring infrastructure via NetworkPolicy (`networkPolicy.allowMetricsFrom`) or firewall rules

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
3. **Rotate all credentials**: Generate new PAKs for all agents, generators, and admins
4. **Review webhooks**: Check for unauthorized webhook subscriptions
5. **Audit database**: Look for unauthorized modifications to stacks or agents
6. **Rebuild**: Consider deploying fresh broker instances rather than cleaning compromised ones

## Related Documentation

- [Security Model](../explanation/security-model.md) — trust boundaries, authentication, and authorization design
- [Managing PAKs](./pak-management.md) — credential creation and rotation
- [Network Configuration](./network-configuration.md) — TLS, NetworkPolicy, and firewall setup
