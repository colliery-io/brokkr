# SDKs

Brokkr ships generated client SDKs for **Rust**, **Python**, and **TypeScript**. All three are produced from the same OpenAPI spec (`openapi/brokkr-v1.json`) and share the same shape:

- Single-credential constructor — pass a PAK; the wrapper handles auth headers.
- Access the raw generated API surface via `.api` / `.api()` when the wrapper doesn't cover what you need.
- Opt-in retry helper with exponential backoff for transient failures.
- Typed errors with a stable `code` field for pattern matching.
- Folder-of-manifests helpers — `submit_manifests`/`apply` (`submitManifests`/`apply` in TS) take a directory or file, validate each document, and submit it as a stack's desired state; `apply` is idempotent (re-submits only when the bundle changed).

## Getting started

| Language | Install | Import |
|----------|---------|--------|
| Rust | `cargo add brokkr-client` | `use brokkr_client::BrokkrClient;` |
| Python | `pip install brokkr-client` | `from brokkr import BrokkrClient` |
| TypeScript | `npm install @colliery-io/brokkr-client` | `import { BrokkrClient } from "@colliery-io/brokkr-client";` |

Detailed walkthroughs:

- [Rust](./rust.md) — `brokkr-client` crate, includes a worked agent example.
- [Python](./python.md) — `brokkr-client` distribution (wraps the low-level `brokkr-client-generated`, pulled in transitively).
- [TypeScript](./typescript.md) — `@colliery-io/brokkr-client` package. Types are generated via `openapi-typescript`; the runtime is `openapi-fetch`.

## Versioning and compatibility

SDK versions track the broker version in **lockstep**. The git tag `vX.Y.Z` drives the version stamped into the broker container images, helm charts, and all three SDKs in the same release. An SDK at `0.8.x` is the canonical client for broker `0.8.x`; mixing major versions is not supported.

There is no separate SDK-only release cadence. If the broker API changes, the SDKs are regenerated and republished in the same tag.

## Authentication

Every Brokkr SDK uses a single credential: a **PAK** (Prefixed API Key), attached to the `Authorization` header of every request. The Python and TypeScript wrappers send it as `Authorization: Bearer <pak>`; the Rust wrapper sends the bare token with no `Bearer` prefix. The broker accepts either, so this only matters if you are reading a packet capture or writing your own client.

The OpenAPI spec declares three security schemes — `admin_pak`, `agent_pak`, `generator_pak` — but they all map to the same header. All PAKs share one format (by default `brokkr_BR<short>_<long>`); the role is not encoded in the token. The broker resolves the role at runtime by hashing the PAK and looking it up against the admin role, agents, and generators tables (`POST /api/v1/auth/pak` tells you which identity a PAK resolves to):

| Role          | What it can do                                          |
|---------------|---------------------------------------------------------|
| Admin         | Full API surface; create/rotate other PAKs              |
| Agent         | Heartbeat, fetch target state, report health and events |
| Generator     | Create/update stacks and deployment objects             |

Where PAKs come from:

- **Admin** — generated at first broker startup (when no `pak_hash` is configured) and written to `/tmp/brokkr-keys/key.txt` inside the broker container (see [Managing PAKs](../pak-management.md)).
- **Agent** — returned once when an agent is created (`POST /api/v1/agents`); rotate with `POST /api/v1/agents/{id}/rotate-pak`, which returns the new PAK once.
- **Generator** — returned once when a generator is created (`POST /api/v1/generators`); rotate with `POST /api/v1/generators/{id}/rotate-pak`, which returns the new PAK once.

Both the REST rotation endpoints and the `brokkr-broker rotate agent/generator` CLI commands print the new PAK once; the REST endpoints additionally invalidate the broker's auth cache immediately.

`POST /api/v1/auth/pak` also returns a `readonly` flag alongside `admin`, `agent`, and `generator`. It is `false` for every PAK you provision through the API or CLI — it marks the broker's own ephemeral console credential, a fourth in-memory credential class that the Operator Console mints per process and that may only issue reads. You will not construct an SDK client with one, but do not assume the field is absent when introspecting.

## Error handling

Every documented 4xx/5xx response returns a typed `ErrorResponse`:

```json
{ "code": "agent_not_found", "message": "...", "details": { ... } }
```

Pattern-match on `code` — it is stable across versions. The `message` is human-readable and may change. See the [stable error codes](../../reference/error-codes.md) table for what to expect.

## Pagination

The v1 API returns full collections without cursor tokens, so no SDK exposes pagination iterators. If pagination is added later, the wrappers will grow `Stream` adapters; consumers won't need to change call sites.

## Keeping SDKs in sync with the broker

If you change the broker's API surface, regenerate the spec and SDKs. See [Regenerating SDKs](./regeneration.md).
