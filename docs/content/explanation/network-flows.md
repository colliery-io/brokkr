---
title: "Network Flows"
weight: 5
---

# Network Flows

This document describes the network traffic flows between Brokkr components, useful for operators configuring firewalls, network policies, and understanding system connectivity requirements.

## Network Topology Overview

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

**Key Points:**
- Agents initiate all connections to the broker (pull model)
- No inbound connections required to agents
- Webhook delivery is outbound from broker to external endpoints

## Connection Details

### Complete Connection Matrix

| Source | Destination | Port | Protocol | Direction | Required | Purpose |
|--------|-------------|------|----------|-----------|----------|---------|
| Admin/UI | Broker | 3000 | HTTPS | Inbound | Yes | API access, management UI |
| Generator | Broker | 3000 | HTTPS | Inbound | Yes | Stack/deployment creation |
| Agent | Broker | 3000 | HTTPS | Outbound | Yes | Fetch deployments, report events |
| Broker | PostgreSQL | 5432 | TCP | Internal | Yes | Database operations |
| Agent | K8s API | 6443 | HTTPS | Local | Yes | Resource management |
| Broker | Webhook endpoints | 443 | HTTPS | Outbound | Optional | Event notifications |
| Prometheus | Broker | 3000 | HTTP | Inbound | Optional | Metrics scraping (/metrics) |
| Prometheus | Agent | 8080 | HTTP | Inbound | Optional | Metrics scraping (/metrics) |
| Broker | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |
| Agent | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |

## Broker Network Requirements

### Inbound Connections

The broker accepts inbound connections on a single port:

| Port | Protocol | Source | Purpose |
|------|----------|--------|---------|
| 3000 | HTTP/HTTPS | Agents, Admins, Generators | All API traffic |

The broker service can be exposed via:
- **ClusterIP** (default): Internal cluster access only
- **LoadBalancer**: Direct external access
- **Ingress**: Recommended for production with TLS termination

### Outbound Connections

| Destination | Port | Protocol | Purpose | Required |
|-------------|------|----------|---------|----------|
| PostgreSQL | 5432 | TCP/TLS | Database | Yes |
| Webhook URLs | 443 | HTTPS | Event delivery | If webhooks configured |
| OTLP Collector | 4317 | gRPC | Telemetry | If tracing enabled |

### Database Connectivity

The broker requires persistent connectivity to PostgreSQL:

```
# Connection string format
postgres://user:password@host:5432/database

# With SSL
postgres://user:password@host:5432/database?sslmode=require
```

**Bundled PostgreSQL**: When `postgresql.enabled=true`, the database runs as a sidecar and uses cluster-internal DNS (`brokkr-broker-postgresql:5432`).

**External PostgreSQL**: Configure via `postgresql.external.*` values or provide connection string via `postgresql.existingSecret`.

## Agent Network Requirements

### Outbound Connections Only

Agents only make outbound connections - no inbound ports are required:

| Destination | Port | Protocol | Purpose |
|-------------|------|----------|---------|
| Broker API | 3000 | HTTPS | Fetch deployments, report events |
| Kubernetes API | 6443 | HTTPS | Manage cluster resources |
| OTLP Collector | 4317 | gRPC | Telemetry (optional) |

### Kubernetes API Access

The agent communicates with the local Kubernetes API server:
- Uses in-cluster configuration automatically
- Requires appropriate RBAC permissions (created by Helm chart)
- Connection is always local to the cluster

### Broker Connectivity

Agents poll the broker at configurable intervals (default: 30 seconds):

```yaml
broker:
  url: https://broker.example.com:3000  # Or internal: http://brokkr-broker:3000
  pak: "brokkr_BR..."                    # Pre-Authentication Key
```

## TLS Configuration

### Broker TLS Options

1. **Ingress TLS Termination** (Recommended)
   - TLS terminates at ingress controller
   - Internal traffic uses HTTP
   - Simplest certificate management

2. **Direct TLS on Broker**
   - Enable via `tls.enabled=true`
   - Provide certificates via secret or cert-manager
   - Use for non-ingress deployments

3. **Cert-Manager Integration**
   ```yaml
   tls:
     enabled: true
     certManager:
       enabled: true
       issuer: letsencrypt-prod
       issuerKind: ClusterIssuer
   ```

### Agent-to-Broker TLS

Agents should always use HTTPS when communicating with the broker:

```yaml
broker:
  url: https://broker.example.com:3000
```

The agent validates the broker's TLS certificate. For self-signed certificates, you may need to:
- Add the CA to the agent's trust store
- Or use `--insecure` flag (not recommended for production)

## Kubernetes NetworkPolicy Examples

### Broker NetworkPolicy

Allow inbound from agents and ingress, outbound to PostgreSQL and webhooks:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: brokkr-broker
  namespace: brokkr
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: brokkr-broker
  policyTypes:
    - Ingress
    - Egress

  ingress:
    # Allow from ingress controller
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
          podSelector:
            matchLabels:
              app.kubernetes.io/name: ingress-nginx
      ports:
        - protocol: TCP
          port: 3000

    # Allow from agents (if in same cluster)
    - from:
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: brokkr-agent
      ports:
        - protocol: TCP
          port: 3000

    # Allow Prometheus scraping
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: monitoring
      ports:
        - protocol: TCP
          port: 3000

  egress:
    # Allow to PostgreSQL
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: postgresql
      ports:
        - protocol: TCP
          port: 5432

    # Allow DNS
    - to:
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - protocol: UDP
          port: 53

    # Allow webhook delivery to external HTTPS
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 10.0.0.0/8
              - 172.16.0.0/12
              - 192.168.0.0/16
      ports:
        - protocol: TCP
          port: 443
```

### Agent NetworkPolicy

Allow outbound to broker and Kubernetes API:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: brokkr-agent
  namespace: brokkr
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: brokkr-agent
  policyTypes:
    - Egress

  egress:
    # Allow to Kubernetes API server
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0  # Restrict to API server IP in production
      ports:
        - protocol: TCP
          port: 6443

    # Allow to broker (if in same cluster)
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: brokkr-broker
      ports:
        - protocol: TCP
          port: 3000

    # Allow to external broker (if in different cluster)
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
      ports:
        - protocol: TCP
          port: 443
        - protocol: TCP
          port: 3000

    # Allow DNS
    - to:
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - protocol: UDP
          port: 53
```

## Helm Chart NetworkPolicy

Both Helm charts include optional NetworkPolicy resources:

### Broker Chart
```yaml
networkPolicy:
  enabled: true
  allowIngressFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: ingress-nginx
  allowMetricsFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: monitoring
  allowWebhookEgress: true
```

### Agent Chart
```yaml
networkPolicy:
  enabled: true
  kubernetesApiCidr: "10.0.0.1/32"  # Restrict to API server IP
  allowMetricsFrom:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: monitoring
```

## Firewall Configuration

### Minimum Required Ports

**For Broker Host:**
| Direction | Port | Protocol | Source/Dest | Purpose |
|-----------|------|----------|-------------|---------|
| Inbound | 3000 | TCP | Agents, Admins | API access |
| Outbound | 5432 | TCP | PostgreSQL | Database |
| Outbound | 443 | TCP | Webhook endpoints | Event delivery |

**For Agent Host:**
| Direction | Port | Protocol | Source/Dest | Purpose |
|-----------|------|----------|-------------|---------|
| Outbound | 3000/443 | TCP | Broker | API communication |
| Outbound | 6443 | TCP | K8s API | Cluster management |

### Cloud Provider Security Groups

#### AWS Security Groups

**Broker Security Group:**
```hcl
# Inbound
ingress {
  from_port   = 3000
  to_port     = 3000
  protocol    = "tcp"
  cidr_blocks = ["10.0.0.0/8"]  # VPC CIDR
}

# Outbound to RDS
egress {
  from_port   = 5432
  to_port     = 5432
  protocol    = "tcp"
  cidr_blocks = ["10.0.0.0/8"]
}

# Outbound for webhooks
egress {
  from_port   = 443
  to_port     = 443
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}
```

## Troubleshooting Network Issues

### Common Problems

1. **Agent can't reach broker**
   - Verify broker URL is correct
   - Check DNS resolution
   - Verify no firewall blocking port 3000/443
   - Check NetworkPolicy if enabled

2. **Broker can't reach database**
   - Verify database host is resolvable
   - Check security group/firewall rules for port 5432
   - Verify database credentials

3. **Webhooks not being delivered**
   - Check egress rules allow HTTPS to external IPs
   - Verify webhook URL is accessible
   - Check broker logs for delivery errors

### Diagnostic Commands

```bash
# Test broker connectivity from agent pod
kubectl exec -it deploy/brokkr-agent -- wget -qO- http://brokkr-broker:3000/healthz

# Test database connectivity from broker pod
kubectl exec -it deploy/brokkr-broker -- nc -zv postgresql 5432

# Check NetworkPolicy is applied
kubectl get networkpolicy -n brokkr

# View network policy details
kubectl describe networkpolicy brokkr-broker -n brokkr
```
