# brokkr-client

Ergonomic Python client for the Brokkr broker API.

```bash
pip install brokkr-client
```

The import name is `brokkr` (`from brokkr import BrokkrClient`); the
PyPI distribution name is `brokkr-client`.

Versions track the broker release in lockstep — `brokkr-client` `0.3.x`
is the canonical client for broker `0.3.x`.

This is a thin wrapper around the auto-generated `brokkr-client-generated`
package (produced by `openapi-python-client` from the broker's OpenAPI
spec). The wrapper adds:

- A single-credential constructor that injects the `Authorization` header
  on every request. The three security schemes the spec declares
  (`admin_pak` / `agent_pak` / `generator_pak`) all map to the same header
  and the broker disambiguates at runtime — the wrapper hides that detail.
- `BrokkrError`, a single exception type that wraps the generated typed
  `ErrorResponse` and exposes `.code` for stable pattern-matching.
- An opt-in `retry(...)` helper with exponential backoff for transient
  transport / 5xx failures. Retry is per-call so callers decide which
  operations (typically idempotent GETs) are safe.

Pagination iterators are intentionally absent: the v1 broker API returns
full collections without cursor tokens. `Stream`-style adapters belong
here when the API adds pagination.

The wrapper is intentionally small. Most of the surface is the generated
client; reach for it via `client.api` when the wrapper doesn't cover what
you need.
