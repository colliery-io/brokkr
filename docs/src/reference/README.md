# Technical Reference

Look-up material for Brokkr: endpoints, flags, fields, codes, and defaults. Each page is a catalog rather than a walkthrough — for step-by-step procedures see the [How-To Guides](../how-to/README.md), and for the reasoning behind a design see [Explanation](../explanation/README.md).

## API and CLI

- [API Reference](./api/README.md) — the full endpoint catalog: every route, method, and authorization requirement. The broker also serves an interactive Swagger UI at `/swagger-ui` and the OpenAPI spec at `/docs/openapi.json`.
- [CLI Reference](./cli.md) — every `brokkr-broker` and `brokkr-agent` subcommand and flag
- [Stable Error Codes](./error-codes.md) — the `code` strings returned by 4xx/5xx responses, and how SDKs surface them
- [WebSocket Protocol](./ws-protocol.md) — message catalog and channel behavior for the WS surfaces

## Configuration and deployment

- [Environment Variables](./environment-variables.md) — every `BROKKR__*` setting, its default, and its config-file equivalent
- [Network Ports](./network-ports.md) — listening ports and the connection matrix between components
- [Container Images](./container-images.md) — published repositories, tag formats, and image structure

## Resources

- [Generators](./generators.md) — the generator (tenant) resource, its registration surface, and the system generator
- [Templates](./templates.md) — stack template fields, parameter schemas, and matching rules
- [Work Orders](./work-orders.md) — the work-order lifecycle, states, and claim semantics
- [Webhooks](./webhooks.md) — subscription fields, event payloads, filters, and delivery behavior
- [Agent Annotations & Labels](./agent-annotations.md) — reserved keys and validation rules
- [Multi-Tenancy](./multi-tenancy.md) — how tenant scope is expressed and enforced across resources
- [Soft Deletion](./soft-deletion.md) — which resources soft-delete, and what that implies for names and queries

## Observability

- [Health Endpoints](./health-endpoints.md) — the broker's liveness and readiness endpoints and their semantics
- [Deployment Health](./deployment-health.md) — the per-deployment-object health model agents report
- [Fleet Observability](./fleet.md) — the per-agent fleet record, its REST endpoints, and the live-push stream
- [Monitoring & Observability](./monitoring.md) — the Prometheus metrics catalog for broker and agent
- [Diagnostics](./diagnostics.md) — the on-demand diagnostic request/result contract
- [Audit Logs](./audit-logs.md) — audit entry schema, emitted actions, query API, and retention

## Rust API Documentation

Module, type, and function documentation generated from the Brokkr source is available in the [Rust API Documentation](../api/README.md).
