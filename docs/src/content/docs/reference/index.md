---
title: Technical Reference
---

Look-up material for Brokkr: endpoints, flags, fields, codes, and defaults. Each page is a catalog rather than a walkthrough — for step-by-step procedures see the [How-To Guides](../how-to/), and for the reasoning behind a design see [Explanation](../explanation/).

## API and CLI

- [API Reference](rest-api/) — the full endpoint catalog: every route, method, and authorization requirement. The broker also serves an interactive Swagger UI at `/swagger-ui` and the OpenAPI spec at `/docs/openapi.json`.
- [CLI Reference](cli/) — every `brokkr-broker` and `brokkr-agent` subcommand and flag
- [Stable Error Codes](error-codes/) — the `code` strings returned by 4xx/5xx responses, and how SDKs surface them
- [WebSocket Protocol](ws-protocol/) — message catalog and channel behavior for the WS surfaces

## Configuration and deployment

- [Environment Variables](environment-variables/) — every `BROKKR__*` setting, its default, and its config-file equivalent
- [Network Ports](network-ports/) — listening ports and the connection matrix between components
- [Container Images](container-images/) — published repositories, tag formats, and image structure

## Resources

- [Generators](generators/) — the generator (tenant) resource, its registration surface, and the system generator
- [Templates](templates/) — stack template fields, parameter schemas, and matching rules
- [Work Orders](work-orders/) — the work-order lifecycle, states, and claim semantics
- [Webhooks](webhooks/) — subscription fields, event payloads, filters, and delivery behavior
- [Agent Annotations & Labels](agent-annotations/) — reserved keys and validation rules
- [Multi-Tenancy](multi-tenancy/) — how tenant scope is expressed and enforced across resources
- [Soft Deletion](soft-deletion/) — which resources soft-delete, and what that implies for names and queries

## Observability

- [Health Endpoints](health-endpoints/) — the broker's liveness and readiness endpoints and their semantics
- [Deployment Health](deployment-health/) — the per-deployment-object health model agents report
- [Fleet Observability](fleet/) — the per-agent fleet record, its REST endpoints, and the live-push stream
- [Monitoring & Observability](monitoring/) — the Prometheus metrics catalog for broker and agent
- [Diagnostics](diagnostics/) — the on-demand diagnostic request/result contract
- [Audit Logs](audit-logs/) — audit entry schema, emitted actions, query API, and retention

## Rust API Documentation

Module, type, and function documentation generated from the Brokkr source is available in the [Rust API Documentation](rust-api/).
