---
title: Client SDKs
---

Brokkr ships generated client SDKs for **Rust**, **Python**, and **TypeScript** —
the supported way to consume the broker's REST API from your own code. All three
are generated from the same OpenAPI spec as the broker and released in lockstep
with it, so they always match the broker version you're running.

This page is the API-reference entry point for the SDKs; the full guides live
under How-To:

- [Client SDKs overview](../../how-to/sdks/) — shared shape, authentication, versioning, error handling, pagination
- [Rust SDK](../../how-to/sdks/rust/) — `brokkr-client` crate (wraps the progenitor-generated client)
- [Python SDK](../../how-to/sdks/python/) — `brokkr-client` distribution
- [TypeScript SDK](../../how-to/sdks/typescript/) — `@colliery-io/brokkr-client` package
- [Regenerating SDKs](../../how-to/sdks/regeneration/) — how the SDKs are produced from the spec

For the broker's **internal** Rust crate docs, see the [Rust API Reference](../rust-api/).
For REST endpoint schemas and try-it-out, see the [REST API Reference](../rest-api/)
or Swagger UI at `http://<broker-url>/swagger-ui`.
