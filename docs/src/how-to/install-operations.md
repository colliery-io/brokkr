# Installing, Upgrading, and Uninstalling with Helm

This guide covers the lifecycle operations for a Helm-based Brokkr installation: pinning chart versions, trying development builds, upgrading, and uninstalling. For a first installation, start with the [Installation Guide](../getting-started/installation.md).

Brokkr Helm charts are published to GitHub Container Registry (GHCR).

## Install a Specific Chart Version

To pin an installation to a specific release version:

```bash
# Install a specific release version
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 1.0.0 \
  --set postgresql.enabled=true

# List available versions
# Visit: https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker
```

## Install a Development Build

Development builds use semver pre-release versions with timestamps (for example, `0.0.0-develop.20251021150606`). Find the latest one in the [package listing](https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker), then install it by version:

```bash
# Install development build (replace timestamp with actual version)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.0.0-develop.20251021150606 \
  --set postgresql.enabled=true
```

## Upgrade Brokkr

To upgrade an existing installation to a newer version while keeping your current values:

```bash
# Upgrade broker
helm upgrade brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 1.1.0 \
  --reuse-values

# Upgrade agent
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --version 1.1.0 \
  --reuse-values
```

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
