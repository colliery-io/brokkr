from http import HTTPStatus
from typing import Any
from urllib.parse import quote
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.create_deployment_object_request import CreateDeploymentObjectRequest
from ...models.deployment_object import DeploymentObject
from ...models.error_response import ErrorResponse
from ...types import UNSET, Response, Unset


def _get_kwargs(
    id: UUID,
    *,
    body: CreateDeploymentObjectRequest,
    deletion_marker: bool | None | Unset = UNSET,
) -> dict[str, Any]:
    headers: dict[str, Any] = {}

    params: dict[str, Any] = {}

    json_deletion_marker: bool | None | Unset
    if isinstance(deletion_marker, Unset):
        json_deletion_marker = UNSET
    else:
        json_deletion_marker = deletion_marker
    params["deletion_marker"] = json_deletion_marker

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "post",
        "url": "/stacks/{id}/deployment-objects".format(
            id=quote(str(id), safe=""),
        ),
        "params": params,
    }

    _kwargs["json"] = body.to_dict()

    headers["Content-Type"] = "application/json"

    _kwargs["headers"] = headers
    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> DeploymentObject | ErrorResponse | None:
    if response.status_code == 201:
        response_201 = DeploymentObject.from_dict(response.json())

        return response_201

    if response.status_code == 400:
        response_400 = ErrorResponse.from_dict(response.json())

        return response_400

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 404:
        response_404 = ErrorResponse.from_dict(response.json())

        return response_404

    if response.status_code == 500:
        response_500 = ErrorResponse.from_dict(response.json())

        return response_500

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[DeploymentObject | ErrorResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: CreateDeploymentObjectRequest,
    deletion_marker: bool | None | Unset = UNSET,
) -> Response[DeploymentObject | ErrorResponse]:
    """
    Args:
        id (UUID):
        deletion_marker (bool | None | Unset):
        body (CreateDeploymentObjectRequest): Wire DTO for creating a deployment object via the
            public API.

            Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
            which carries server-derived fields (e.g. `yaml_checksum`).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[DeploymentObject | ErrorResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        body=body,
        deletion_marker=deletion_marker,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: CreateDeploymentObjectRequest,
    deletion_marker: bool | None | Unset = UNSET,
) -> DeploymentObject | ErrorResponse | None:
    """
    Args:
        id (UUID):
        deletion_marker (bool | None | Unset):
        body (CreateDeploymentObjectRequest): Wire DTO for creating a deployment object via the
            public API.

            Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
            which carries server-derived fields (e.g. `yaml_checksum`).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        DeploymentObject | ErrorResponse
    """

    return sync_detailed(
        id=id,
        client=client,
        body=body,
        deletion_marker=deletion_marker,
    ).parsed


async def asyncio_detailed(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: CreateDeploymentObjectRequest,
    deletion_marker: bool | None | Unset = UNSET,
) -> Response[DeploymentObject | ErrorResponse]:
    """
    Args:
        id (UUID):
        deletion_marker (bool | None | Unset):
        body (CreateDeploymentObjectRequest): Wire DTO for creating a deployment object via the
            public API.

            Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
            which carries server-derived fields (e.g. `yaml_checksum`).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[DeploymentObject | ErrorResponse]
    """

    kwargs = _get_kwargs(
        id=id,
        body=body,
        deletion_marker=deletion_marker,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    id: UUID,
    *,
    client: AuthenticatedClient,
    body: CreateDeploymentObjectRequest,
    deletion_marker: bool | None | Unset = UNSET,
) -> DeploymentObject | ErrorResponse | None:
    """
    Args:
        id (UUID):
        deletion_marker (bool | None | Unset):
        body (CreateDeploymentObjectRequest): Wire DTO for creating a deployment object via the
            public API.

            Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
            which carries server-derived fields (e.g. `yaml_checksum`).

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        DeploymentObject | ErrorResponse
    """

    return (
        await asyncio_detailed(
            id=id,
            client=client,
            body=body,
            deletion_marker=deletion_marker,
        )
    ).parsed
