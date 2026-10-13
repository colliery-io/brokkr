---
title: Rust API Reference
---

This section contains the complete Rust API documentation for all Brokkr crates, automatically generated from source code doc comments using [plissken](https://github.com/colliery-io/plissken).

These docs are regenerated on every build, so they always reflect the current state of the codebase.

[All modules](../api/) lists every page with its summary.

## Crates

- [**brokkr-broker**](../api/rust/brokkr-broker/) — The main control plane and API server
- [**brokkr-agent**](../api/rust/brokkr-agent/) — The agent that applies resources to Kubernetes clusters
- [**brokkr-models**](../api/rust/brokkr-models/) — Shared data models and database schema
- [**brokkr-utils**](../api/rust/brokkr-utils/) — Utilities, configuration, and helpers

## Python SDK

plissken also writes the API pages of the Python SDK (`sdks/python/brokkr`):

- [**brokkr**](../api/brokkr/) — The package
- [**client**](../api/brokkr/client/) — `BrokkrClient`, the client
- [**errors**](../api/brokkr/errors/) — `BrokkrError`, the typed error

## Client SDKs

The pages above document Brokkr's **internal** Rust crates. If you're *consuming*
the broker API from your own code, use one of the generated client SDKs instead
— all three are produced from the same OpenAPI spec and kept in lockstep with the
broker version:

- [**Client SDKs overview**](../../how-to/sdks/) — shared shape, auth, versioning, error handling
- [**Rust SDK**](../../how-to/sdks/rust/) — `brokkr-client` crate (wraps the progenitor-generated client)
- [**Python SDK**](../../how-to/sdks/python/) — `brokkr-client` distribution
- [**TypeScript SDK**](../../how-to/sdks/typescript/) — `@colliery-io/brokkr-client` package
- [Regenerating SDKs](../../how-to/sdks/regeneration/) — how the SDKs are produced from the spec

## Interactive API Documentation

For REST API endpoint documentation (request/response schemas, authentication, try-it-out), see the [API Reference](../rest-api/) or access Swagger UI directly at `http://<broker-url>/swagger-ui`.

Looking for higher-level guides? [Return to the main documentation](../../).
