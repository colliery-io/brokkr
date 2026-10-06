# brokkr-client

Ergonomic Python client for the Brokkr broker API.

```bash
pip install brokkr-client
```

The import name is `brokkr` (`from brokkr import BrokkrClient`); the
PyPI distribution name is `brokkr-client`.

## Version

The SDK version matches the broker version. Install the release that has the
same version as your broker. See the
[releases page](https://github.com/colliery-io/brokkr/releases).

## First deployment

This example makes a folder of manifests the desired state of a stack, then
prints the result. It uses a **generator PAK**. A generator owns the stacks
that it creates. An admin can create a generator with `brokkr-broker create
generator --name <name>` or `POST /api/v1/generators`. The broker shows the
PAK only once.

Put one manifest in a folder:

```yaml
# manifests/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hello-brokkr
  namespace: default
data:
  greeting: hello
```

Then apply the folder:

```python
import asyncio

from brokkr import BrokkrClient


async def main() -> None:
    client = BrokkrClient(
        "http://localhost:3000",  # broker URL; the client adds /api/v1
        token="<generator PAK>",
    )
    result = await client.apply("hello-brokkr", "./manifests", ["env:dev"])
    print("status:", result.status)  # created | updated | unchanged
    if result.deployment_object is not None:
        obj = result.deployment_object
        print("stack:", obj.stack_id)
        print("sequence id:", obj.sequence_id)


asyncio.run(main())
```

The base URL can be the broker root (`http://localhost:3000`) or end in
`/api/v1`. The client adds `/api/v1` when it is absent and never adds it twice.
The `brokkr` CLI uses the same rule.

`apply` creates the stack if it does not exist, adds the targeting labels
(`env:dev`), and submits a new revision only when the folder changed. Run it
again with no change and the status is `unchanged`. Agents whose labels match
the stack's targeting deploy the revision.

With an admin PAK, name the generator that owns the stack:
`await client.apply("hello-brokkr", "./manifests", generator="<generator name>")`.

## What the wrapper adds

This is a thin wrapper around the auto-generated `brokkr-client-generated`
package (produced by `openapi-python-client` from the broker's OpenAPI
spec). The wrapper adds:

- A single-credential constructor that injects the `Authorization` header
  on every request. The three security schemes the spec declares
  (`admin_pak` / `agent_pak` / `generator_pak`) all map to the same header
  and the broker disambiguates at runtime. The wrapper hides that detail.
- `submit_manifests` and `apply`, which read a folder (or file) of manifests,
  validate each document, and submit it as a stack's desired state.
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

For the full guide, see the
[Python SDK documentation](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/sdks/python.md).
