# Network Ports

Reference tables for every network connection in a Brokkr deployment. For the reasoning behind the topology, see [Network Flows](../explanation/network-flows.md); for configuration steps, see the [Network Configuration how-to](../how-to/network-configuration.md).

## Port Assignments

| Port | Component | Protocol | Purpose |
|------|-----------|----------|---------|
| 3000 | Broker | HTTP/HTTPS | All API traffic (REST, internal WS, `/healthz`, `/readyz`, `/metrics`) |
| 8080 | Agent | HTTP | Health and metrics server (`/healthz`, `/readyz`, `/health`, `/metrics`) |
| 5432 | PostgreSQL | TCP | Database (standard port; in-cluster and most external deployments) |
| 5433 | PostgreSQL | TCP | Host-mapped port used by the local development docker-compose environment (reflected in the default `BROKKR__DATABASE__URL`) |
| 4317 | OTLP Collector | gRPC | Distributed tracing (optional) |
| 6443 | Kubernetes API | HTTPS | Target-cluster API server (agent egress) |
| 443 | Ingress / webhooks | HTTPS | External TLS entry point; webhook delivery egress |

External PostgreSQL deployments may use other ports, configured via `postgresql.external.port`.

## Complete Connection Matrix

| Source | Destination | Port | Protocol | Direction | Required | Purpose |
|--------|-------------|------|----------|-----------|----------|---------|
| Admin/UI | Broker | 3000 | HTTPS | Inbound | Yes | API access, management operations |
| Generator | Broker | 3000 | HTTPS | Inbound | Yes | Stack and deployment object creation |
| Agent | Broker | 3000 | HTTPS | Outbound | Yes | Fetch deployments, report events (REST + internal WS) |
| Broker | PostgreSQL | 5432 | TCP | Internal | Yes | Database operations |
| Agent | K8s API | 6443 | HTTPS | Local | Yes | Resource management |
| Broker | Webhook endpoints | 443 | HTTPS | Outbound | Optional | Event notifications |
| Prometheus | Broker | 3000 | HTTP | Inbound | Optional | Metrics scraping at `/metrics` |
| Prometheus | Agent | 8080 | HTTP | Inbound | Optional | Metrics scraping at `/metrics` |
| Broker | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |
| Agent | OTLP Collector | 4317 | gRPC | Outbound | Optional | Distributed tracing |

## Minimum Firewall Ports

**Broker host:**

| Direction | Port | Protocol | Source/Destination | Purpose |
|-----------|------|----------|---------------------|---------|
| Inbound | 3000 (or 443 via ingress) | TCP | Agents, Admins, Generators | API access |
| Outbound | 5432 | TCP | PostgreSQL database | Database connectivity |
| Outbound | 443 | TCP | Webhook endpoints | Event delivery |

**Agent host:**

| Direction | Port | Protocol | Source/Destination | Purpose |
|-----------|------|----------|---------------------|---------|
| Outbound | 3000 or 443 | TCP | Broker | API communication |
| Outbound | 6443 | TCP | Kubernetes API server | Cluster management |

## Related Documentation

- [Network Flows](../explanation/network-flows.md) — topology and design rationale
- [Network Configuration how-to](../how-to/network-configuration.md) — TLS, NetworkPolicy, and cloud firewall setup
- [Health Endpoints](./health-endpoints.md)
- [Monitoring](./monitoring.md)
