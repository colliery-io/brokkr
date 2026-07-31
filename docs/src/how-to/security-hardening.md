# Security Hardening

This guide collects the operational security tasks for running Brokkr in production: what to verify before go-live, what to monitor, and what to do when you suspect a compromise. For the underlying trust model and authentication design, see the [Security Model](../explanation/security-model.md).

## Production Deployment Checklist

Before deploying Brokkr to production, verify these security configurations:

- [ ] **Replace the Default Admin PAK (do this first)**: The broker's built-in configuration ships an admin PAK hash whose matching PAK is publicly known; any broker started without an explicit override accepts it. Generate a fresh credential and set the hash before first startup, then verify the default is rejected (see [Replace the Default Admin PAK](#replace-the-default-admin-pak))
- [ ] **TLS at the Edge**: The broker serves plain HTTP and has no TLS support of its own — terminate TLS for all external connections at an ingress controller or service mesh in front of it. The broker chart's `tls.*` values mount certificates that nothing in the broker reads and do **not** enable broker-side TLS; if you exposed a broker directly with `tls.enabled: true`, that traffic (including PAKs in Authorization headers) was plaintext
- [ ] **Strong Secrets**: Use cryptographically secure random values for all PAKs and encryption keys
- [ ] **External Database**: Use managed PostgreSQL with encryption at rest
- [ ] **Secret Management**: Source every credential the charts accept — database URL, admin PAK hash, webhook encryption key, agent PAK — from a pre-created Kubernetes Secret rather than a plaintext Helm value, so none of them is rendered into a ConfigMap (see [Keep Credentials Out of ConfigMaps](#keep-credentials-out-of-configmaps))
- [ ] **NetworkPolicy**: Enable and configure network policies to restrict traffic (see [Network Configuration](./network-configuration.md))
- [ ] **RBAC**: Use minimal required permissions for service accounts (namespace-scoped agents watch and report health in-namespace via `BROKKR__AGENT__WATCH_NAMESPACE`; cluster-wide RBAC is required only for cluster-scoped resources and cross-namespace pruning; see [Security Model](../explanation/security-model.md))
- [ ] **Agent Registration and Selector Hygiene**: Register each agent with the generators whose stacks it should target — at startup via the agent's `--generator-ids` flag or `BROKKR__AGENT__GENERATOR_IDS`, or after deployment with `brokkr register`. Registration gates the creation of explicit targets (attempts for an unregistered generator's stacks are rejected with `agent_not_registered`), but do not rely on registration alone to bound what reaches an agent: stacks are also delivered when their labels or annotations match the agent's, so keep label and annotation selectors distinct per tenant or application and audit each agent's labels/annotations as part of scope hardening (see [Configuring Agent Scopes](#configuring-agent-scopes) and [Agent Registration](./agent-registration.md))
- [ ] **Pod Security**: Enable pod security standards (restricted profile)
- [ ] **Audit Logging**: Enable and monitor audit logs
- [ ] **Resource Limits**: Set CPU/memory limits to prevent resource exhaustion
- [ ] **Image Scanning**: Scan container images for vulnerabilities before deployment
- [ ] **Restrict `/metrics`**: The broker's `/metrics` endpoint is unauthenticated; limit access to your monitoring infrastructure via NetworkPolicy (`networkPolicy.allowMetricsFrom`) or firewall rules
- [ ] **Restrict the Console Surface**: In the published broker image, any client that can GET the broker URL receives the operator console page with an embedded read-only admin credential — network reachability of port 3000 is the console's entire auth boundary, and the credential grants read visibility across all tenants. Limit who can reach the broker port via NetworkPolicy or firewall rules, add authentication at the ingress if the console must be exposed more widely, and enable sticky sessions when running more than one broker replica (each replica mints its own console token). See the [Security Model](../explanation/security-model.md#read-only-console-authentication-the-ui-pak)

## Replace the Default Admin PAK

The broker's embedded default configuration sets the admin PAK hash to a development credential whose raw PAK is published in the Brokkr source tree — it exists so local development works out of the box, and it must never survive into production. A broker started without an explicit hash override silently accepts that publicly-known PAK as its admin credential.

1. **Generate a fresh credential.** `brokkr-broker generate-pak` mints a PAK and its SHA-256 hash offline — no database or running broker required. Store the PAK itself in your secret manager; it is shown only once.

2. **Configure the hash before first startup.** Set `BROKKR__BROKER__PAK_HASH` to the generated hash. For Helm installs, put the hash in a Kubernetes Secret and reference it with `broker.pakHashExistingSecret` — the chart then leaves the hash out of the ConfigMap and injects it into the pod from the Secret. The plaintext `broker.pakHash` value does the same job on a dev cluster, at the cost of writing the hash into your values files and the broker's ConfigMap.

3. **Verify the default credential is dead.** The publicly-known development PAK must be rejected:

   ```bash
   curl -s -o /dev/null -w "%{http_code}\n" \
     -X POST "https://<broker>/api/v1/auth/pak" \
     -H "Authorization: Bearer brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"
   ```

   Expect `401`. Any other status means the broker is still running with the default admin credential — stop and fix the configuration before proceeding.

### The Broker's Startup Warning

The broker checks its admin credential against the shipped default at every startup — both the configured `broker.pak_hash` and the hash actually stored in `admin_role`. When either is still the default it logs a banner at `WARN` and repeats it hourly for as long as the process runs:

```text
==============================================================================
!! SECURITY: THIS BROKER ACCEPTS A PUBLICLY-KNOWN ADMIN CREDENTIAL !!
==============================================================================
WHAT: broker.pak_hash is set to the value shipped in Brokkr's default.toml.
WHY IT MATTERS: the PAK matching that hash is published in Brokkr's public
source tree. Anyone who can reach this broker's API can authenticate as admin:
full read/write access to every agent, generator, stack and secret it serves.
FIX: run `brokkr-broker generate-pak`; set BROKKR__BROKER__PAK_HASH to the minted
hash (Helm: broker.pakHash, or broker.pakHashExistingSecret to source it from an
existing Secret); restart. On an install that has already started once, also run
`brokkr-broker rotate admin` so the stored hash is replaced.
==============================================================================
```

The same state is exported on `/metrics` as `brokkr_default_admin_pak_hash_in_use` — `1` while the default is in use, `0` once it has been replaced. The gauge is always present, so alert on `brokkr_default_admin_pak_hash_in_use == 1` rather than on the log line.

Two things this warning is **not**:

- **It is not a control.** The broker warns and keeps serving; it does not refuse to start. Refusing would require a development opt-out flag, and such flags end up in production manifests — a worse failure mode than the one being prevented. The steps above are the control; this is the backstop for an install that skipped them. (Contrast the webhook encryption key, where the broker *does* refuse to start; see [Webhooks](./webhooks.md).)
- **It is not a check you can silence.** There is no configuration to disable it. Seeing it in local development or in the Brokkr test harnesses is expected — those deliberately run on the default credential.

If the warning names the *stored* hash rather than the configured one, correcting the configuration is not enough: `admin_role` is written only on first startup, so an install that first booted with the default keeps accepting the public PAK until `brokkr-broker rotate admin` replaces the stored hash.

## Keep Credentials Out of ConfigMaps

Replacing the default admin PAK closes the largest hole; where you *put* the replacement decides whether you have opened a smaller one. Every credential the Brokkr charts accept as a plaintext Helm value is rendered into a ConfigMap, which is not a secret store: its contents are readable by any principal with `get configmap` in the namespace, are returned by `helm get values`, and persist in whatever values file or git history produced them.

Both charts can read each credential from a Kubernetes Secret you create yourself. Setting the `existingSecret` value **removes the credential from the ConfigMap** and injects it into the pod from the Secret instead — the ConfigMap no longer contains it at all. Treat this as the default for any production install:

| Credential | Chart value | Default key in the Secret |
|------------|-------------|---------------------------|
| Database connection URL | `postgresql.existingSecret` / `postgresql.existingSecretKey` (broker) | `database-url` |
| Admin PAK hash | `broker.pakHashExistingSecret` / `broker.pakHashExistingSecretKey` (broker) | `BROKKR__BROKER__PAK_HASH` |
| Webhook encryption key | `broker.webhookEncryptionKeyExistingSecret` / `broker.webhookEncryptionKeyExistingSecretKey` (broker) | `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` |
| Agent PAK | `broker.existingSecret` / `broker.existingSecretKey` (agent) | `BROKKR__AGENT__PAK` |

The `existingSecret` form takes precedence over the plaintext value it replaces, so switching an existing release over is a single `helm upgrade`. The install commands are in [Production Install: Credentials from Kubernetes Secrets](../getting-started/installation.md#production-install-credentials-from-kubernetes-secrets).

Verify the change rather than assuming it: the credential should be absent from the ConfigMap and present in the pod's environment.

```bash
kubectl get configmap brokkr-broker -o yaml | grep BROKKR__BROKER__PAK_HASH   # expect no output
kubectl exec deploy/brokkr-broker -- printenv BROKKR__BROKER__PAK_HASH        # expect the hash
```

Three qualifications worth understanding:

- **Kubernetes Secrets are base64-encoded, not encrypted, by default.** Sourcing credentials from Secrets narrows exposure from "anyone who can read ConfigMaps or your values files" to "anyone who can read Secrets", which is the boundary most RBAC policies actually draw on. Combine it with encryption at rest for etcd, and with an external secret manager where you have one — the `existingSecret` values pair directly with tools that vend credentials into a Secret at deploy time.
- **The admin PAK hash is the least sensitive of the four.** It is a plain SHA-256 digest; the PAK cannot be recovered from it. Keeping it in a Secret is about keeping it out of values files and git, and denying a namespace reader the ability to check your broker's hash against the published default.
- **Sourcing the webhook encryption key from a Secret is also an availability measure.** With no key configured, the broker generates a random one per process, so anything encrypted by a previous process — webhook URLs and auth headers — is unreadable after a restart. Once at least one webhook subscription exists, a broker that finds the key unset refuses to start rather than come up in that state. A key held in a Secret survives restarts, upgrades, and reschedules, which is what keeps that failure from happening. See [Configuring Webhooks](./webhooks.md).

Credentials injected this way are read once, when the container starts; updating a Secret does not change what a running pod holds. For the ordering that rotation requires — particularly for the admin PAK, where the Secret and the database must both be updated and the order matters — see [Rotating a Credential Held in a Secret](./install-operations.md#rotating-a-credential-held-in-a-secret).

## Configuring Agent Scopes

An agent can only have *explicit targets* created for a generator's stacks once it is registered with that generator — Brokkr's application-level access control (see [Generator Registration and Application Scopes](../explanation/security-model.md#generator-registration-and-application-scopes)). Configure the scopes an agent serves at deployment time:

1. **Set the generator IDs on the agent.** Provide a comma-separated list of generator UUIDs (for example, `BROKKR__AGENT__GENERATOR_IDS=<gen-id-1>,<gen-id-2>`) through one of the following sources, highest precedence first:
   - the `--generator-ids` CLI flag,
   - the `BROKKR__AGENT__GENERATOR_IDS` environment variable (config key `agent.generator_ids`),
   - the legacy `BROKKR_GENERATOR_IDS` variable (**deprecated** — still honored, but logs a warning).

   For Helm deployments, set `broker.generatorIds` (a YAML list or comma-separated string) in the `brokkr-agent` chart; it renders to `BROKKR__AGENT__GENERATOR_IDS` in the agent ConfigMap.

2. **Confirm the resulting scope.** An empty or unset value means the agent joins no application scopes beyond the automatic system-generator registration, so explicit targets cannot be created for any other generator's stacks. Every agent is auto-registered with the system generator at creation, so fleet-wide stacks always reach it regardless of this setting.

3. **Audit the agent's labels and annotations.** Registration bounds explicit targeting, but stacks are also delivered when their labels or annotations match the agent's — a delivery path that operates independently of registration. Keep selectors distinct per tenant or application (for example, prefix them with the tenant name) so a generic label like `env=prod` cannot pull another tenant's stacks onto the agent.

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
| `brokkr_default_admin_pak_hash_in_use` | `== 1` | The broker still accepts the publicly-known default admin PAK (see [Replace the Default Admin PAK](#replace-the-default-admin-pak)) |

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
- [Installing, Upgrading & Uninstalling](./install-operations.md) — moving a release onto Secrets and rotating credentials held in them
- [CLI Reference](../reference/cli.md) — `brokkr register` / `deregister` / `registrations` and `brokkr-broker generate-pak`
- [Network Configuration](./network-configuration.md) — ingress TLS termination, NetworkPolicy, and firewall setup
