# Python SDK

The `brokkr-client` package is an ergonomic wrapper around `brokkr-client-generated`, which is generated from `openapi/brokkr-v1.json` by `openapi-python-client`. The wrapper adds a single-credential constructor, a typed `BrokkrError`, and an opt-in retry helper.

## Install

```bash
pip install brokkr-client
# or
uv pip install brokkr-client
```

This pulls in `brokkr-client-generated` automatically as a transitive dependency. End users don't need to install it separately.

For in-tree workspace development, install editable from a local checkout:

```bash
uv pip install -e sdks/python/brokkr
```

## Construct a client

```python
from brokkr import BrokkrClient

client = BrokkrClient(
    base_url="https://broker.example.com/api/v1",
    token="brokkr_BRabcd1234_AgentLongTokenExample0001",  # agent PAK
)
```

The constructor takes a base URL and one PAK. **The base URL must include the `/api/v1` prefix** — the OpenAPI spec declares its server as `/api/v1`, and the generated endpoint modules append unprefixed paths like `/agents` to whatever base you provide, so omitting the prefix makes every call 404. The wrapper builds the underlying `AuthenticatedClient` and attaches `Authorization: Bearer <pak>` on every request — you do not need to know which of the three `*_pak` security schemes your role maps to. Omit `token` for a client that can only hit unauthenticated endpoints.

The constructor also accepts keyword-only tuning knobs:

| Kwarg | Default | Purpose |
|-------|---------|---------|
| `request_timeout` | `30.0` | Overall per-request timeout in seconds |
| `connect_timeout` | `10.0` | Connection-establishment timeout in seconds |
| `max_retries` | `3` | Maximum retries used by `client.retry()` |
| `initial_backoff` | `0.2` | First backoff in seconds; doubles per attempt, capped at 10 s |

The `brokkr` package exports four names: `BrokkrClient`, `BrokkrError`, `ErrorResponse` (the typed error body, re-exported from the generated package), and `TemplateGenerator` (the generated `Generator` model, re-exported under a clearer name to avoid clashing with `typing.Generator`).

## Call one endpoint

The generated API is reachable via `client.api`. Each endpoint is a module under `brokkr_broker_client.api.<tag>` with `.asyncio(...)` / `.sync(...)` entry points:

```python
from brokkr_broker_client.api.agents import list_agents

agents = await list_agents.asyncio(client=client.api)
print(f"{len(agents)} agents")
```

## Handle one error

Direct calls on the generated modules do **not** raise `BrokkrError`. The generator folds documented 4xx/5xx bodies into the return union, so a failed call returns an `ErrorResponse` (and transport failures surface as `httpx` exceptions). `BrokkrError` enters the picture in two ways: route the call through `client.retry()`, which converts both cases into raised `BrokkrError`s, or convert an `ErrorResponse` yourself with `BrokkrError.from_response()`.

Via `retry()` (recommended single exception path):

```python
from brokkr import BrokkrError
from brokkr_broker_client.api.agents import get_agent

try:
    agent = await client.retry(
        lambda api: get_agent.asyncio(client=api, id=agent_id)
    )
except BrokkrError as err:
    if err.code == "agent_not_found":
        print("no such agent")
    elif err.code == "unauthorized":
        print("PAK rejected")
    else:
        raise
```

Or convert manually after a direct call:

```python
from brokkr import BrokkrError, ErrorResponse
from brokkr_broker_client.api.agents import get_agent

result = await get_agent.asyncio(client=client.api, id=agent_id)
if isinstance(result, ErrorResponse):
    raise BrokkrError.from_response(result, status=404)
```

The `*_detailed` endpoint variants return the HTTP status code alongside the body, which lets you pass an accurate `status=` to `from_response`. Match on `.code` for the stable wire code — see [stable error codes](../../reference/error-codes.md) for the full list.

## Retry on transient failures

`BrokkrClient.retry` re-runs an async closure with exponential backoff (200 ms, doubling, capped at 10 s; 3 attempts by default). Transport errors and HTTP `408/429/502/503/504` retry; everything else surfaces immediately.

```python
async def fetch(api):
    return await list_agents.asyncio(client=api)

agents = await client.retry(fetch)
```

Wrap only operations you consider safe to repeat — typically idempotent GETs.

## When you need to drop to the raw client

For anything the wrapper doesn't cover, use `client.api` directly — it is the same `AuthenticatedClient` and still injects the auth header. The generated package documents every endpoint at `sdks/python/brokkr-client/brokkr_broker_client/api/`.
