# Python SDK

The `brokkr` package is an ergonomic wrapper around `brokkr-broker-client`, which is generated from `openapi/brokkr-v1.json` by `openapi-python-client`. The wrapper adds a single-credential constructor, a typed `BrokkrError`, and an opt-in retry helper.

## Install

Both packages live in this workspace; install from a local checkout via `uv`:

```bash
uv pip install -e sdks/python/brokkr
```

This pulls in `brokkr-broker-client` automatically (declared via `tool.uv.sources`). Neither package is published to PyPI yet.

## Construct a client

```python
from brokkr import BrokkrClient

client = BrokkrClient(
    base_url="https://broker.example.com",
    token="brokkr_BA...",  # agent PAK
)
```

The constructor takes a base URL and one PAK. The wrapper builds the underlying `AuthenticatedClient` and attaches `Authorization: Bearer <pak>` on every request — you do not need to know which of the three `*_pak` security schemes your role maps to. Omit `token` for a client that can only hit unauthenticated endpoints.

## Call one endpoint

The generated API is reachable via `client.api`. Each endpoint is a module under `brokkr_broker_client.api.<tag>` with `.asyncio(...)` / `.sync(...)` entry points:

```python
from brokkr_broker_client.api.agents import list_agents

agents = await list_agents.asyncio(client=client.api)
print(f"{len(agents)} agents")
```

## Handle one error

The wrapper raises `BrokkrError` for documented 4xx/5xx responses. Match on `.code` for the stable wire code:

```python
from brokkr import BrokkrError
from brokkr_broker_client.api.agents import get_agent

try:
    agent = await get_agent.asyncio(client=client.api, id=agent_id)
except BrokkrError as err:
    if err.code == "agent_not_found":
        print("no such agent")
    elif err.code == "unauthorized":
        print("PAK rejected")
    else:
        raise
```

See [stable error codes](./errors.md) for the full list.

For endpoints the generator folds `ErrorResponse` into a return union (rather than raising), check the return type and convert with `BrokkrError.from_response(body, status=...)` if you want a single exception path.

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
