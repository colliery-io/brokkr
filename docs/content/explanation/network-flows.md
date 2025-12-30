---
title: "Network Flows"
weight: 5
---

# Network Flows

Understanding the network architecture of a distributed system is essential for proper deployment, security hardening, and troubleshooting. This document provides a comprehensive analysis of the network traffic patterns between Brokkr components, including detailed port and protocol specifications, firewall requirements, and Kubernetes NetworkPolicy configurations.

## Network Topology

Brokkr implements a hub-and-spoke network topology where the broker acts as the central coordination point. All agents initiate outbound connections to the broker—there are no inbound connections required to agents. This pull-based model simplifies firewall configuration and enables agents to operate behind NAT without special accommodations.

{{< mermaid >}}
flowchart TB
    subgraph External["External Traffic"]
        Admin[Admin Users]
        Generator[Generators/CI]
        Webhook[Webhook Endpoints]
    end

    subgraph BrokerCluster["Broker Cluster"]
        Ingress[Ingress Controller]
        Broker[Broker Service :3000]
        DB[(PostgreSQL :5432)]
    end

    subgraph TargetClusters["Target Cluster(s)"]
        Agent[Agent]
        K8sAPI[K8s API :6443]
        Workloads[Deployed Workloads]
    end

    Admin -->|HTTPS| Ingress
    Generator -->|HTTPS| Ingress
    Ingress -->|HTTP| Broker
    Broker -->|TCP| DB
    Broker -->|HTTPS| Webhook

    Agent -->|HTTPS| Broker
    Agent -->|HTTPS| K8sAPI
    Agent -.->|Manages| Workloads
{{< /mermaid >}}

The diagram above illustrates the three primary network zones in a typical Brokkr deployment. External traffic from administrators and generators enters through an ingress controller, which terminates TLS and forwards requests to the broker service. The broker maintains persistent connectivity to its PostgreSQL database and sends outbound webhook deliveries to configured external endpoints. Meanwhile, agents in target clusters poll the broker for deployment instructions and interact with their local Kubernetes API servers to apply resources.

## Connection Specifications

### Complete Connection Matrix

The following table enumerates every network connection in the Brokkr system, including the source and destination components, ports, protocols, and whether each connection is required for basic operation.

| Source | Destination | Port | Protocol | Direction | Required | Purpose |
|--------|-------------|------|----------|-----------|----------|---------|
| Admin/UI | Broker | 3000 | HTTPS | Inbound | Yes | API access, management operations |
| Generator | Broker | 3000 | HTTPS | Inbound | Yes | Stack and deployment object creation |
| Agent | Broker | 3000 | HTTPS | Outbound | Yes | Fetch deployments, report events |
| Broker | PostgreSQL | 5432 | TCP | Internal | Yes | Database operations |
| Agent | K8s API | 6443 | HTTPS | Local | Yes | Resource management |
| Broker | Webhook endpoints | 443 | HTTPS | Outbound | Optional | Event notifications |
| Prometheus | Broker | 3000 | HTTP | Inbound | Optional | Metrics scraping at /metrics |
| Prometheus | Agent | 8080 | HTTP | Inbound | Optional | Metrics scraping at /metrics |
| Broker | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |
| Agent | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |

### Port Assignments

Brokkr uses a small number of well-defined ports. The broker service listens on port 3000 for all API traffic, including agent communication, administrator operations, and generator requests. This single-port design simplifies ingress configuration and firewall rules. Agents expose a health and metrics server on port 8080, which serves the `/healthz`, `/ready`, `/health`, and `/metrics` endpoints used by Kubernetes liveness probes and Prometheus scraping.

The PostgreSQL database uses the standard port 5432. When deploying the bundled PostgreSQL instance via the Helm chart, this connection remains internal to the broker cluster. External PostgreSQL deployments may use different ports, which can be configured via the `postgresql.external.port` value.

OpenTelemetry tracing, when enabled, uses gRPC on port 4317 to communicate with OTLP collectors. This optional integration provides distributed tracing capabilities for debugging and performance analysis.

## Broker Network Requirements

### Inbound Traffic

The broker service accepts all inbound traffic on a single port, simplifying both service exposure and network policy configuration. The default configuration exposes port 3000, though this is rarely accessed directly in production. Instead, an ingress controller typically terminates TLS and forwards traffic to the broker.

The broker service supports three exposure methods through its Helm chart:

**ClusterIP** is the default service type, restricting access to within the Kubernetes cluster. This configuration is appropriate when agents run in the same cluster as the broker or when an ingress controller handles external access.

**LoadBalancer** creates a cloud provider load balancer that exposes the service directly to external traffic. While simpler to configure than ingress, this approach requires managing TLS termination separately and may incur additional cloud provider costs.

**Ingress** (recommended for production) delegates external access and TLS termination to a Kubernetes ingress controller. This approach integrates with cert-manager for automatic certificate management and provides flexible routing options.

### Outbound Traffic

The broker initiates three types of outbound connections. Database connectivity to PostgreSQL is essential—the broker cannot operate without it. The Helm chart supports both bundled PostgreSQL (deployed as a subchart) and external PostgreSQL instances. For bundled deployments, the connection uses internal cluster DNS (`brokkr-broker-postgresql:5432`). External databases are configured via the `postgresql.external` values or by providing a complete connection URL through `postgresql.existingSecret`.

Webhook delivery represents the second outbound connection type. When webhooks are configured, the broker dispatches event notifications to external HTTP/HTTPS endpoints. The webhook delivery worker processes deliveries in batches, with the batch size and interval configurable via `broker.webhookDeliveryBatchSize` (default: 50) and `broker.webhookDeliveryIntervalSeconds` (default: 5). Failed deliveries are retried with exponential backoff.

OpenTelemetry tracing, when enabled, establishes gRPC connections to an OTLP collector. The collector endpoint is configured via `telemetry.otlpEndpoint`, and the sampling rate via `telemetry.samplingRate`. The Helm chart optionally deploys an OTel collector sidecar for environments where the main collector is not directly accessible.

### Database Connectivity

The broker uses Diesel ORM with an r2d2 connection pool for PostgreSQL connectivity. Connection strings follow the standard PostgreSQL URI format:

```
postgres://user:password@host:5432/database
```

For production deployments, TLS should be enabled by appending `?sslmode=require` or stronger modes to the connection string. The broker supports multi-tenant deployments through PostgreSQL schema isolation—the `postgresql.external.schema` value specifies which schema to use for data storage.

## Agent Network Requirements

### Outbound-Only Architecture

Agents operate with an outbound-only network model. They initiate all connections and require no inbound ports for their primary function. This design enables agents to operate behind restrictive firewalls and NAT gateways without special configuration—a critical feature for edge deployments and air-gapped environments.

The agent's network requirements are minimal: connectivity to the broker API and the local Kubernetes API server. When metrics scraping is enabled, the agent also accepts inbound connections from Prometheus on port 8080.

| Destination | Port | Protocol | Purpose |
|-------------|------|----------|---------|
| Broker API | 3000 | HTTPS | Fetch deployments, report events |
| Kubernetes API | 6443 | HTTPS | Manage cluster resources |
| OTLP Collector | 4317 | gRPC | Telemetry (optional) |

### Kubernetes API Access

Agents communicate with their local Kubernetes API server to apply and manage resources. When deployed via the Helm chart, agents use in-cluster configuration automatically—the Kubernetes client discovers the API server address from the cluster's DNS and service account credentials.

The Helm chart creates RBAC resources that grant agents permission to manage resources across the cluster (when `rbac.clusterWide: true`) or within specific namespaces. The agent requires broad resource access for deployment management but can be restricted from sensitive resources like Secrets through the `rbac.secretAccess` configuration.

### Broker Connectivity

Agents poll the broker at a configurable interval (default: 30 seconds, set via `agent.pollingInterval`). Each polling cycle fetches pending deployment objects and reports events for completed operations. The agent also sends deployment health status updates at a separate interval (default: 60 seconds, set via `agent.deploymentHealth.intervalSeconds`).

The broker URL is configured via the `broker.url` value in the agent's Helm chart. For deployments where the agent and broker share a cluster, an internal URL like `http://brokkr-broker:3000` provides optimal performance. For multi-cluster deployments, agents use the broker's external URL with TLS: `https://broker.example.com`.

Authentication uses Prefixed API Keys (PAKs), which agents include in the `Authorization` header of every request. The PAK is generated when an agent is registered and should be provided via `broker.pak` in the Helm values or through a Kubernetes Secret.

## TLS Configuration

### Broker TLS Options

The broker supports three TLS configuration approaches, each suited to different deployment scenarios.

**Ingress TLS Termination** is the recommended approach for most production deployments. TLS terminates at the ingress controller, and internal traffic between the ingress and broker uses plain HTTP. This approach centralizes certificate management and integrates smoothly with cert-manager:

```yaml
ingress:
  enabled: true
  className: 'nginx'
  tls:
    - secretName: brokkr-tls
      hosts:
        - broker.example.com
```

**Direct TLS on Broker** enables TLS termination at the broker itself, useful for deployments without an ingress controller or when end-to-end encryption is required. Enable via `tls.enabled: true` and provide certificates either through `tls.existingSecret` or inline via `tls.cert` and `tls.key`.

**Cert-Manager Integration** automates certificate provisioning when combined with ingress TLS. The Helm chart can configure cert-manager annotations to automatically request and renew certificates:

```yaml
tls:
  enabled: true
  certManager:
    enabled: true
    issuer: 'letsencrypt-prod'
    issuerKind: 'ClusterIssuer'
```

### Agent-to-Broker TLS

Agents should always communicate with the broker over HTTPS in production. The agent validates the broker's TLS certificate using the system's trusted certificate authorities. For deployments with self-signed or private CA certificates, the CA must be added to the agent's trust store or mounted as a volume.

## Kubernetes NetworkPolicy Configuration

NetworkPolicies provide defense-in-depth by restricting pod-to-pod and pod-to-external communication at the network layer. Both the broker and agent Helm charts include optional NetworkPolicy resources that implement least-privilege network access.

### Broker NetworkPolicy

The broker's NetworkPolicy allows inbound connections from configured sources and outbound connections to the database and webhook destinations. When `networkPolicy.enabled: true`, the generated policy includes:

**Ingress rules** permit connections on port 3000 from pods matching the selectors specified in `networkPolicy.allowIngressFrom`. If no selectors are specified, the policy allows connections from any pod in the same namespace. Metrics scraping can be separately controlled via `networkPolicy.allowMetricsFrom`.

**Egress rules** permit DNS resolution (UDP/TCP port 53), database connectivity (port 5432 to PostgreSQL pods or external IPs), and optionally webhook delivery (HTTPS port 443 to external IPs, excluding private ranges). The `networkPolicy.allowWebhookEgress` value controls whether webhook egress is permitted.

```yaml
networkPolicy:
  enabled: true
  allowIngressFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: ingress-nginx
      podSelector:
        matchLabels:
          app.kubernetes.io/name: ingress-nginx
  allowMetricsFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: monitoring
  allowWebhookEgress: true
```

### Agent NetworkPolicy

The agent's NetworkPolicy restricts traffic to essential destinations only. When enabled, the policy permits:

**Egress to DNS** (UDP/TCP port 53) for name resolution.

**Egress to the Kubernetes API server** (ports 443 and 6443). The `networkPolicy.kubernetesApiCidr` value controls which IP ranges can receive this traffic—in production, restrict this to the actual API server IP for maximum security.

**Egress to the broker** on the configured port (default 3000). The destination can be specified as a pod selector for same-cluster deployments or as an IP block for external brokers.

**Ingress for metrics** (port 8080) when `metrics.enabled: true` and `networkPolicy.allowMetricsFrom` specifies allowed scrapers.

```yaml
networkPolicy:
  enabled: true
  kubernetesApiCidr: "10.0.0.1/32"  # API server IP
  brokerEndpoint:
    podSelector:
      matchLabels:
        app.kubernetes.io/name: brokkr-broker
    namespaceSelector:
      matchLabels:
        kubernetes.io/metadata.name: brokkr
  allowMetricsFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: monitoring
```

## Firewall Configuration

### Minimum Required Ports

Organizations deploying Brokkr must configure firewalls to permit the following traffic:

**For the broker host:**

| Direction | Port | Protocol | Source/Destination | Purpose |
|-----------|------|----------|---------------------|---------|
| Inbound | 3000 (or 443 via ingress) | TCP | Agents, Admins, Generators | API access |
| Outbound | 5432 | TCP | PostgreSQL database | Database connectivity |
| Outbound | 443 | TCP | Webhook endpoints | Event delivery |

**For the agent host:**

| Direction | Port | Protocol | Source/Destination | Purpose |
|-----------|------|----------|---------------------|---------|
| Outbound | 3000 or 443 | TCP | Broker | API communication |
| Outbound | 6443 | TCP | Kubernetes API server | Cluster management |

### Cloud Provider Security Groups

Cloud deployments require security group configuration that permits the traffic flows described above. The following examples demonstrate typical configurations for major cloud providers.

**AWS Security Groups:**

For the broker, create an inbound rule permitting TCP port 3000 from the VPC CIDR (or specific agent security groups). Create outbound rules for PostgreSQL (port 5432 to the database security group) and webhook delivery (port 443 to 0.0.0.0/0 or specific webhook destinations).

**GCP Firewall Rules:**

Create an ingress rule with a target tag for broker instances, permitting TCP port 3000 from authorized sources. Create egress rules permitting port 5432 to the Cloud SQL instance and port 443 for webhooks.

**Azure Network Security Groups:**

Configure inbound rules for port 3000 from the virtual network address space. Configure outbound rules for database connectivity and webhook delivery similar to the AWS and GCP examples.

## Troubleshooting Network Issues

### Common Problems

**Agent cannot reach broker:** This typically manifests as repeated connection timeouts or DNS resolution failures in agent logs. Begin by verifying the broker URL is correct and resolvable from the agent pod. Use `nslookup` or `dig` to test DNS resolution. Check that no NetworkPolicy or firewall rule blocks egress on the broker port. For TLS connections, verify the agent trusts the broker's certificate.

**Broker cannot reach database:** Database connectivity failures prevent the broker from starting. Verify the database host is resolvable and the credentials are correct. Check security group rules permit traffic on port 5432. For TLS-enabled database connections, verify the `sslmode` parameter is correctly configured.

**Webhooks not being delivered:** Check the broker logs for delivery errors, which indicate whether the issue is connection-related or response-related. Verify egress rules permit HTTPS traffic to external IPs. If NetworkPolicy is enabled, confirm `allowWebhookEgress: true` is set. Test webhook endpoint accessibility using `curl` from a pod in the broker namespace.

**Metrics not being scraped:** If Prometheus cannot reach the metrics endpoints, verify the ServiceMonitor is correctly configured and the Prometheus operator's selector matches. Check that NetworkPolicy allows ingress from the monitoring namespace on the metrics port (3000 for broker, 8080 for agent).

### Diagnostic Commands

The following commands help diagnose network connectivity issues:

```bash
# Test broker connectivity from agent pod
kubectl exec -it deploy/brokkr-agent -- wget -qO- http://brokkr-broker:3000/healthz

# Test database connectivity from broker pod
kubectl exec -it deploy/brokkr-broker -- nc -zv postgresql 5432

# List NetworkPolicies in the namespace
kubectl get networkpolicy -n brokkr

# Examine NetworkPolicy details
kubectl describe networkpolicy brokkr-broker -n brokkr

# Check agent logs for connection errors
kubectl logs deploy/brokkr-agent | grep -i "connection\|error\|timeout"

# Verify DNS resolution from within a pod
kubectl exec -it deploy/brokkr-agent -- nslookup brokkr-broker
```
