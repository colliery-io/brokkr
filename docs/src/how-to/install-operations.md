# Installing, Upgrading, and Uninstalling with Helm

This guide covers the lifecycle operations for a Helm-based Brokkr installation: pinning chart versions, trying development builds, upgrading, and uninstalling. For a first installation, start with the [Installation Guide](../getting-started/installation.md).

Brokkr Helm charts are published to GitHub Container Registry (GHCR).

## Chart Versions and the `existingSecret` Values

Brokkr releases in lockstep: the broker and agent charts, their container images, and the SDKs all share one version number. **The current release is 0.8.4**, and it is the minimum for the credential-from-Secret install described in the [Installation Guide](../getting-started/installation.md#production-install-credentials-from-kubernetes-secrets):

| Value | First released in |
|-------|-------------------|
| `postgresql.existingSecret` / `postgresql.existingSecretKey` (broker) | 0.8.0 and earlier |
| `broker.pakHashExistingSecret` / `broker.pakHashExistingSecretKey` (broker) | **0.8.4** |
| `broker.webhookEncryptionKeyExistingSecret` / `...Key` (broker) | **0.8.4** |
| `broker.existingSecret` / `broker.existingSecretKey` (agent) | **0.8.4** |

This matters because Helm does not reject values a chart has never heard of. Pin an older chart, pass `--set broker.pakHashExistingSecret=...`, and the install reports success while the templates ignore the value entirely — the admin PAK hash is rendered into the ConfigMap in plaintext, or omitted altogether, and you are left believing a credential is in a Secret when it is not. Nothing warns you.

Two ways to stay out of that trap, and the rest of this page uses both:

- **Omit `--version`** and let Helm resolve the newest published chart. That is what the Installation Guide does, and it is the right default for a first install or a routine upgrade.
- **Pin deliberately, at or above 0.8.4**, and confirm afterwards that the credential really is absent from the ConfigMap (`kubectl get configmap brokkr-broker -o yaml`). The ConfigMap check is the one that catches a too-old chart.

## Install a Specific Chart Version

To pin an installation to a specific release version:

```bash
# Install a specific release version
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.8.4 \
  --set postgresql.enabled=true

# List available versions
# Visit: https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker
```

Check the package listing for anything newer than 0.8.4 before pinning; the version above is the current release at the time of writing, not a recommended ceiling. Pin the agent chart to the same version — the two are released together and are tested as a pair.

## Install a Development Build

Development builds use semver pre-release versions with timestamps (branch builds `0.0.0-main.<ts>`, PR builds `0.0.0-pr<N>.<ts>`; for example `0.0.0-main.20251021150606`). Find the latest one in the [package listing](https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker), then install it by version:

```bash
# Install development build (replace timestamp with actual version)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.0.0-main.20251021150606 \
  --set postgresql.enabled=true
```

## Upgrade Brokkr

To upgrade an existing installation to the newest published charts while keeping your current values:

```bash
# Upgrade broker
helm upgrade brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --reuse-values

# Upgrade agent
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --reuse-values
```

Add `--version <version>` to both commands to land on a specific release instead, keeping broker and agent on the same one. Never upgrade *down* past 0.8.4 with the Secret-based credential values in play — Helm will accept it, and the credentials quietly stop being read from their Secrets. See [Chart Versions and the `existingSecret` Values](#chart-versions-and-the-existingsecret-values).

## Move an Existing Release onto Secrets

If a release was installed with plaintext credentials, switch it over with an upgrade.

The upgrade below carries no `--version`, so it also lands the release on the newest chart — which is what makes `broker.pakHashExistingSecret` do anything at all, since that value did not exist before 0.8.4. If you add a `--version` pin here, keep it at 0.8.4 or above, or the migration silently changes nothing.

If you are only relocating the credential you already have, put its existing hash in the Secret and nothing else changes. To mint a fresh pair instead, run `brokkr-broker generate-pak` — from the published image if you have no binary (`docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak`), or inside the running pod (`kubectl exec deploy/brokkr-broker -- brokkr-broker generate-pak`); it works offline and writes nothing either way. A *new* pair is a rotation, not a relocation: the new PAK does not authenticate until its hash also reaches the admin role in the database, so follow [Admin PAK](#admin-pak) below in full.

The `existingSecret` values take precedence over their plaintext counterparts, so you do not have to remove the old values for the change to take effect — but do clear them anyway, because `helm upgrade --reuse-values` carries them forward and Helm keeps the release's values in the cluster:

```bash
kubectl create secret generic brokkr-broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH='<hash-from-generate-pak>'

helm upgrade brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --reuse-values \
  --set broker.pakHash="" \
  --set broker.pakHashExistingSecret=brokkr-broker-admin-pak-hash
```

Confirm afterwards that the value is gone from the ConfigMap and present in the pod's environment:

```bash
kubectl get configmap brokkr-broker -o yaml | grep BROKKR__BROKER__PAK_HASH   # no output
kubectl exec deploy/brokkr-broker -- printenv BROKKR__BROKER__PAK_HASH        # the hash
```

Moving the admin PAK hash into a Secret changes only where the broker *reads* the hash. It does not change the hash stored in the database — see [Rotating a Credential Held in a Secret](#rotating-a-credential-held-in-a-secret) below.

The equivalents for the other credentials are `postgresql.existingSecret` (clearing `postgresql.external.password`), `broker.webhookEncryptionKeyExistingSecret` (clearing `broker.webhookEncryptionKey`), and, on the agent chart, `broker.existingSecret` (clearing `broker.pak`). All four are listed in the [Existing-Secret Values Reference](../getting-started/installation.md#existing-secret-values-reference).

## Rotating a Credential Held in a Secret

Credentials sourced from a Secret reach the container as environment variables, through `secretKeyRef`. Kubernetes resolves those **once, when the container starts**, and neither chart mounts these credentials as files. So:

- Updating the contents of a Secret has no effect on a running pod.
- Neither chart watches Secrets, and neither puts a content hash on the pod template, so `helm upgrade` does not roll the pods when only the Secret's contents changed.
- Every rotation therefore ends with an explicit restart:

  ```bash
  kubectl rollout restart deploy/brokkr-broker
  kubectl rollout status deploy/brokkr-broker
  ```

The same is true of the plaintext forms of these credentials: they arrive from the ConfigMap through `envFrom`, which is also resolved only at container start. The broker's configuration reload covers settings like log level and CORS; it never applies to a credential delivered as an environment variable.

### Admin PAK

Rotating the admin PAK takes two changes that are easy to confuse, and they must happen in this order:

1. **The hash the broker reads** lives in the Secret behind `broker.pakHashExistingSecret`. Updating it changes what the *next* broker process starts with.
2. **The hash the broker authenticates against** lives in the database, on the admin role. Only `brokkr-broker rotate admin` writes it; a restart does not, because the admin bootstrap runs only against a database that has never been initialized.

`rotate admin` reads `BROKKR__BROKER__PAK_HASH` from its own process environment and stores that value. Run inside the broker pod, it sees whatever the pod started with — so running it before the restart re-applies the *old* hash and silently undoes the rotation.

```bash
# 1. Mint the replacement pair; keep the PAK, take the hash.
#    No local binary? Run the same command from the published image:
#    docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
brokkr-broker generate-pak

# 2. Update the Secret in place
kubectl create secret generic brokkr-broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH='<new-hash>' \
  --dry-run=client -o yaml | kubectl apply -f -

# 3. Restart so the pod picks up the new hash
kubectl rollout restart deploy/brokkr-broker
kubectl rollout status deploy/brokkr-broker

# 4. Only now store it on the admin role
kubectl exec deploy/brokkr-broker -- brokkr-broker rotate admin
```

The old admin PAK may keep working briefly after step 4: a CLI rotation cannot reach a running broker's auth cache, so the previous credential can still authenticate for up to `broker.auth_cache_ttl_seconds` (default 60). See [Managing PAKs](./pak-management.md#rotating-the-admin-pak) for the rotation semantics in full, including what happens when the configured hash is empty.

### Agent PAK

Rotating an agent's PAK through the broker invalidates the old one immediately, so the agent is unauthenticated until its pod restarts with the new value. Keep the window short:

```bash
# 1. Rotate on the broker; the response carries the new PAK once
curl -s -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/rotate-pak" \
  -H "Authorization: $ADMIN_PAK" | jq -r '.pak'

# 2. Update the Secret the agent reads
kubectl create secret generic brokkr-agent-credentials \
  --from-literal=BROKKR__AGENT__PAK='<new-pak>' \
  --dry-run=client -o yaml | kubectl apply -f -

# 3. Restart the agent
kubectl rollout restart deploy/brokkr-agent
```

No `helm upgrade` is needed — `broker.existingSecret` still names the same Secret.

### Database URL

Update the Secret behind `postgresql.existingSecret` and restart the broker. There is no in-place reconnect: the broker builds its connection pool at startup.

### Webhook Encryption Key

This one is not rotatable in place. The key decrypts webhook URLs and auth headers already stored in the database, so replacing it makes every existing subscription permanently undeliverable — they continue to list as healthy but every delivery fails. Changing the key means deleting the existing subscriptions, updating the Secret, restarting the broker, and recreating them. Note also that once any subscription exists, a broker that starts with the key unset refuses to start rather than come up with a fresh random key. See [Configuring Webhooks](./webhooks.md).

## Uninstall Brokkr

To remove Brokkr from your cluster:

```bash
# Uninstall agent
helm uninstall brokkr-agent

# Uninstall broker (this will also remove bundled PostgreSQL if enabled)
helm uninstall brokkr-broker

# Note: PersistentVolumes may remain - delete manually if needed
kubectl get pv
kubectl delete pv <pv-name>
```
