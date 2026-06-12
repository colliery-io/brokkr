# Rust API Reference

This section contains the complete Rust API documentation for all Brokkr crates, automatically generated from source code doc comments using [plissken](https://github.com/colliery-io/plissken).

These docs are regenerated on every build, so they always reflect the current state of the codebase.

## Crates

- [**brokkr-broker**](rust/brokkr-broker.md) — The main control plane and API server
- [**brokkr-agent**](rust/brokkr-agent.md) — The agent that applies resources to Kubernetes clusters
- [**brokkr-models**](rust/brokkr-models.md) — Shared data models and database schema
- [**brokkr-utils**](rust/brokkr-utils.md) — Utilities, configuration, and helpers

## Client SDKs

The pages above document Brokkr's **internal** Rust crates. If you're *consuming*
the broker API from your own code, use one of the generated client SDKs instead
— all three are produced from the same OpenAPI spec and kept in lockstep with the
broker version:

- [**Client SDKs overview**](../how-to/sdks/README.md) — shared shape, auth, versioning, error handling
- [**Rust SDK**](../how-to/sdks/rust.md) — `brokkr-client` crate (wraps the progenitor-generated client)
- [**Python SDK**](../how-to/sdks/python.md) — `brokkr-client` distribution
- [**TypeScript SDK**](../how-to/sdks/typescript.md) — `@colliery-io/brokkr-client` package
- [Regenerating SDKs](../how-to/sdks/regeneration.md) — how the SDKs are produced from the spec

## Interactive API Documentation

For REST API endpoint documentation (request/response schemas, authentication, try-it-out), see the [API Reference](../reference/api/README.md) or access Swagger UI directly at `http://<broker-url>/swagger-ui`.

Looking for higher-level guides? [Return to the main documentation](../README.md).
