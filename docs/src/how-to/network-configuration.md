# Network Configuration

This guide covers the hands-on network setup for a Brokkr deployment: TLS termination, Kubernetes NetworkPolicies, cloud provider firewalls, ingress timeouts for the WebSocket channel, and network troubleshooting. For the design rationale see [Network Flows](../explanation/network-flows.md); for the port-by-port tables see the [Network Ports reference](../reference/network-ports.md).

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

## WebSocket Timeouts

The agent's internal WebSocket connection to the broker and the operator-facing live-tail subscription are **long-lived** — they only close on agent crash, broker restart, explicit client close, or credential revocation (see [Internal Broker↔Agent WS Channel](../explanation/internal-ws-channel.md)). Ingress controllers and reverse proxies in front of the broker should allow idle WebSocket connections for at least 5 minutes (anything longer is fine; the broker has no idle timeout of its own).

Specific guidance:

- **nginx-ingress**: set `nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"` and `proxy-send-timeout` on the broker service.
- **Traefik**: defaults are usually fine; bump `transport.respondingTimeouts` if you see cuts at 60s.
- **AWS ALB**: increase the idle timeout on the listener (the default 60s is too aggressive).

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

## Cloud Provider Security Groups

Cloud deployments require security group configuration that permits the flows in the [connection matrix](../reference/network-ports.md#complete-connection-matrix). The following examples demonstrate typical configurations for major cloud providers.

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

## Related Documentation

- [Network Flows](../explanation/network-flows.md) — topology and design rationale
- [Network Ports reference](../reference/network-ports.md) — connection matrix and firewall tables
- [Security Hardening](./security-hardening.md) — production checklist and incident response
