# SDKs

Brokkr ships generated client SDKs for **Rust**, **Python**, and **TypeScript**. All three are produced from the same OpenAPI spec (`openapi/brokkr-v1.json`) and share the same shape:

- Single-credential constructor — pass a PAK; the wrapper handles auth headers.
- Access the raw generated API surface via `.api` / `.api()` when the wrapper doesn't cover what you need.
- Opt-in retry helper with exponential backoff for transient failures.
- Typed errors with a stable `code` field for pattern matching.

## Getting started

- [Rust](./rust.md) — `brokkr-client` crate, includes a worked agent example.
- [Python](./python.md) — `brokkr-client` package (wraps the low-level `brokkr-client-generated`).
- TypeScript — see `sdks/typescript/brokkr-client/README.md`. Types are generated via `openapi-typescript`; the runtime is `openapi-fetch`.

## Authentication

Every Brokkr SDK uses a single credential: a **PAK** (Pre-Authentication Key). The wrapper sends it as `Authorization: Bearer <pak>` on every request.

The OpenAPI spec declares three security schemes — `admin_pak`, `agent_pak`, `generator_pak` — but they all map to the same header. The broker disambiguates at runtime based on the PAK's prefix:

| Prefix       | Role          | What it can do                                          |
|--------------|---------------|---------------------------------------------------------|
| `brokkr_BR…` | Admin         | Full API surface; create/rotate other PAKs              |
| `brokkr_BA…` | Agent         | Heartbeat, fetch target state, report health and events |
| `brokkr_BG…` | Generator     | Create/update stacks and deployment objects             |

Where PAKs come from:

- **Admin** — printed by `brokkr-broker rotate admin` (see [Managing PAKs](../pak-management.md)).
- **Agent** — printed when an agent is created (`POST /api/v1/agents`); rotate with `brokkr-broker rotate agent --uuid <id>`.
- **Generator** — printed when a generator is created (`POST /api/v1/generators`); rotate with `brokkr-broker rotate generator --uuid <id>`.

PAKs are shown **once** at creation and stored only as hashes. Rotate, don't recover.

## Error handling

Every documented 4xx/5xx response returns a typed `ErrorResponse`:

```json
{ "code": "agent_not_found", "message": "...", "details": { ... } }
```

Pattern-match on `code` — it is stable across versions. The `message` is human-readable and may change. See the [stable error codes](./errors.md) table for what to expect.

## Pagination

The v1 API returns full collections without cursor tokens, so no SDK exposes pagination iterators. If pagination is added later, the wrappers will grow `Stream` adapters; consumers won't need to change call sites.

## Keeping SDKs in sync with the broker

If you change the broker's API surface, regenerate the spec and SDKs. See [Regenerating SDKs](./regeneration.md).
