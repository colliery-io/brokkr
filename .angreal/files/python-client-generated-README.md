# brokkr-client-generated

`brokkr-client-generated` is the low-level Python client for the Brokkr broker
API. `openapi-python-client` generates it from the broker's OpenAPI
specification (`openapi/brokkr-v1.json`). The import name is
`brokkr_broker_client`.

Most users want the `brokkr-client` package instead (`from brokkr import
BrokkrClient`). That wrapper adds one-step authentication, typed errors, a
retry helper, and `apply` for a folder of manifests. It installs this package
for you. See the
[Python SDK guide](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/sdks/python.md).

## Version

The package version matches the broker version. Use the release that has the
same version as your broker. See the
[releases page](https://github.com/colliery-io/brokkr/releases).

## Use the raw client

This client does not change the base URL. Give it the broker URL **with** the
`/api/v1` prefix. Each operation is a module under
`brokkr_broker_client.api.<tag>`. Each module has the functions `sync`,
`sync_detailed`, `asyncio` and `asyncio_detailed`.

```python
from brokkr_broker_client import AuthenticatedClient
from brokkr_broker_client.api.auth import verify_pak
from brokkr_broker_client.api.stacks import list_stacks

client = AuthenticatedClient(
    base_url="http://localhost:3000/api/v1",  # this client needs the prefix
    token="<generator PAK>",
)

with client as c:
    identity = verify_pak.sync(client=c)
    print("generator:", identity.generator)

    response = list_stacks.sync_detailed(client=c)
    print("status:", response.status_code)
    for stack in response.parsed:
        print(stack.name, stack.id)
```

A documented error status (for example 401 or 404) does not raise an
exception. The call returns an `ErrorResponse` in place of the model. Use the
`*_detailed` functions to read the HTTP status.

## Regeneration

Do not edit the generated code by hand. Run `angreal openapi gen-python` from
the repository root. That task also copies this README from
`.angreal/files/python-client-generated-README.md`. Edit the README there.
