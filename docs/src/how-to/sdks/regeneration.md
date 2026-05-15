# Regenerating SDKs

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

The Rust SDK is regenerated automatically on every `cargo build` — `progenitor::generate_api!` reads `openapi/brokkr-v1.json` at compile time, so updating the spec is enough.

Commit the regenerated files alongside your broker changes:

- `openapi/brokkr-v1.json`
- `sdks/python/brokkr-client/**`
- `sdks/typescript/brokkr-client/src/schema.d.ts`

## CI drift check

`.github/workflows/openapi.yml` runs four checks on every PR:

| Task                              | Fails if…                                                       |
|-----------------------------------|-----------------------------------------------------------------|
| `angreal openapi check`           | `openapi/brokkr-v1.json` is stale relative to the broker schema.|
| `angreal openapi check-python`    | `sdks/python/brokkr-client` is stale relative to the spec.      |
| `angreal openapi check-typescript`| `sdks/typescript/brokkr-client/src/schema.d.ts` is stale.       |
| `cargo build -p brokkr-client`    | The Rust SDK fails to regenerate against the committed spec.    |

If a check fails, run the matching `gen-*` task locally and commit the result.

## Adding a new endpoint

1. Add the handler with a `#[utoipa::path(...)]` annotation. Include every status code the handler can return (notably `409` for create paths that hit unique constraints — the drift check will catch you).
2. Wire the handler into the `OpenApi` derive in `crates/brokkr-broker/src/api/v1/openapi.rs`.
3. Run the three commands above.
4. If you introduced a new error `code`, document it in [stable error codes](./errors.md).
