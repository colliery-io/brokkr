from http import HTTPStatus
from typing import Any

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.config_reload_response import ConfigReloadResponse
from ...models.error_response import ErrorResponse
from ...types import Response


def _get_kwargs() -> dict[str, Any]:

    _kwargs: dict[str, Any] = {
        "method": "post",
        "url": "/admin/config/reload",
    }

    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> ConfigReloadResponse | ErrorResponse | None:
    if response.status_code == 200:
        response_200 = ConfigReloadResponse.from_dict(response.json())

        return response_200

    if response.status_code == 401:
        response_401 = ErrorResponse.from_dict(response.json())

        return response_401

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 500:
        response_500 = ErrorResponse.from_dict(response.json())

        return response_500

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[ConfigReloadResponse | ErrorResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    *,
    client: AuthenticatedClient,
) -> Response[ConfigReloadResponse | ErrorResponse]:
    r"""Reloads the broker configuration from disk.

     This endpoint triggers a hot-reload of configurable settings without
    requiring a broker restart. Only settings marked as \"dynamic\" can be
    reloaded; static settings (like database URL) require a restart.

    # Authentication

    Requires admin PAK authentication.

    # Returns

    - `200 OK`: Configuration reloaded successfully with list of changes.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Failed to reload configuration.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ConfigReloadResponse | ErrorResponse]
    """

    kwargs = _get_kwargs()

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    *,
    client: AuthenticatedClient,
) -> ConfigReloadResponse | ErrorResponse | None:
    r"""Reloads the broker configuration from disk.

     This endpoint triggers a hot-reload of configurable settings without
    requiring a broker restart. Only settings marked as \"dynamic\" can be
    reloaded; static settings (like database URL) require a restart.

    # Authentication

    Requires admin PAK authentication.

    # Returns

    - `200 OK`: Configuration reloaded successfully with list of changes.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Failed to reload configuration.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ConfigReloadResponse | ErrorResponse
    """

    return sync_detailed(
        client=client,
    ).parsed


async def asyncio_detailed(
    *,
    client: AuthenticatedClient,
) -> Response[ConfigReloadResponse | ErrorResponse]:
    r"""Reloads the broker configuration from disk.

     This endpoint triggers a hot-reload of configurable settings without
    requiring a broker restart. Only settings marked as \"dynamic\" can be
    reloaded; static settings (like database URL) require a restart.

    # Authentication

    Requires admin PAK authentication.

    # Returns

    - `200 OK`: Configuration reloaded successfully with list of changes.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Failed to reload configuration.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ConfigReloadResponse | ErrorResponse]
    """

    kwargs = _get_kwargs()

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    *,
    client: AuthenticatedClient,
) -> ConfigReloadResponse | ErrorResponse | None:
    r"""Reloads the broker configuration from disk.

     This endpoint triggers a hot-reload of configurable settings without
    requiring a broker restart. Only settings marked as \"dynamic\" can be
    reloaded; static settings (like database URL) require a restart.

    # Authentication

    Requires admin PAK authentication.

    # Returns

    - `200 OK`: Configuration reloaded successfully with list of changes.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Failed to reload configuration.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ConfigReloadResponse | ErrorResponse
    """

    return (
        await asyncio_detailed(
            client=client,
        )
    ).parsed
