# Regenerating SDKs

> **For contributors to Brokkr itself.** If you are *using* an SDK, nothing on this page applies — the published packages already contain generated code. Start from [Using the SDKs](./README.md) instead.

The broker's OpenAPI spec and the three SDKs are checked into the repo and verified by CI. When you change the API surface, regenerate all four artifacts in the same PR.

## Workflow

```bash
# 1. Re-emit the spec from the broker's utoipa annotations.
angreal openapi export

# 2. Regenerate the Python SDK.
angreal openapi gen-python

# 3. Regenerate the TypeScript types.
angreal openapi gen-typescript
```

The Rust SDK needs no separate generator step: `progenitor::generate_api!` regenerates it inline on every `cargo build`. It does **not** read the workspace spec, though — it reads the crate-local mirror `crates/brokkr-client/spec/brokkr-v1.json`, which lives inside the crate so the spec survives `cargo package`. `angreal openapi export` writes both copies and keeps them byte-identical.

Commit all four paths alongside your broker changes:

- `openapi/brokkr-v1.json`
- `crates/brokkr-client/spec/brokkr-v1.json` — forgetting this one fails CI even though nothing else looks stale
- `sdks/python/brokkr-client/**`
- `sdks/typescript/brokkr-client/src/schema.d.ts`

## CI drift check

The OpenAPI workflow runs six gates on every PR:

| Gate                              | Fails if…                                                       |
|-----------------------------------|-----------------------------------------------------------------|
| `angreal openapi check`           | `openapi/brokkr-v1.json` is stale relative to the broker schema, or the crate-local mirror has drifted from it (or is missing). |
| `redocly lint`                    | The spec has structural problems the Rust build cannot catch — missing summaries, malformed examples, and similar. |
| `cargo build -p brokkr-client --tests` | The Rust SDK fails to regenerate against the committed spec. |
| `angreal openapi check-python`    | `sdks/python/brokkr-client` is stale relative to the spec.      |
| `angreal openapi check-typescript`| `sdks/typescript/brokkr-client/src/schema.d.ts` is stale.       |
| `npm run typecheck` + `npm test`  | The TypeScript SDK no longer typechecks or its surface tests fail. |

If a drift check fails, run the matching `export`/`gen-*` task locally and commit the result.

## Adding a new endpoint

1. Add the handler with a `#[utoipa::path(...)]` annotation. Include every status code the handler can return (notably `409` for create paths that hit unique constraints — the drift check will catch you).
2. Wire the handler into the `OpenApi` derive in `crates/brokkr-broker/src/api/v1/openapi.rs`.
3. Run the three commands above.
4. If you introduced a new error `code`, document it in [stable error codes](../../reference/error-codes.md).
