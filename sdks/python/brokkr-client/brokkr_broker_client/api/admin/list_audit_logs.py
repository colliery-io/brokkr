import datetime
from http import HTTPStatus
from typing import Any
from uuid import UUID

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.audit_log_list_response import AuditLogListResponse
from ...models.error_response import ErrorResponse
from ...types import UNSET, Response, Unset


def _get_kwargs(
    *,
    actor_type: None | str | Unset = UNSET,
    actor_id: None | Unset | UUID = UNSET,
    action: None | str | Unset = UNSET,
    resource_type: None | str | Unset = UNSET,
    resource_id: None | Unset | UUID = UNSET,
    from_: datetime.datetime | None | Unset = UNSET,
    to: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
    offset: int | None | Unset = UNSET,
) -> dict[str, Any]:

    params: dict[str, Any] = {}

    json_actor_type: None | str | Unset
    if isinstance(actor_type, Unset):
        json_actor_type = UNSET
    else:
        json_actor_type = actor_type
    params["actor_type"] = json_actor_type

    json_actor_id: None | str | Unset
    if isinstance(actor_id, Unset):
        json_actor_id = UNSET
    elif isinstance(actor_id, UUID):
        json_actor_id = str(actor_id)
    else:
        json_actor_id = actor_id
    params["actor_id"] = json_actor_id

    json_action: None | str | Unset
    if isinstance(action, Unset):
        json_action = UNSET
    else:
        json_action = action
    params["action"] = json_action

    json_resource_type: None | str | Unset
    if isinstance(resource_type, Unset):
        json_resource_type = UNSET
    else:
        json_resource_type = resource_type
    params["resource_type"] = json_resource_type

    json_resource_id: None | str | Unset
    if isinstance(resource_id, Unset):
        json_resource_id = UNSET
    elif isinstance(resource_id, UUID):
        json_resource_id = str(resource_id)
    else:
        json_resource_id = resource_id
    params["resource_id"] = json_resource_id

    json_from_: None | str | Unset
    if isinstance(from_, Unset):
        json_from_ = UNSET
    elif isinstance(from_, datetime.datetime):
        json_from_ = from_.isoformat()
    else:
        json_from_ = from_
    params["from"] = json_from_

    json_to: None | str | Unset
    if isinstance(to, Unset):
        json_to = UNSET
    elif isinstance(to, datetime.datetime):
        json_to = to.isoformat()
    else:
        json_to = to
    params["to"] = json_to

    json_limit: int | None | Unset
    if isinstance(limit, Unset):
        json_limit = UNSET
    else:
        json_limit = limit
    params["limit"] = json_limit

    json_offset: int | None | Unset
    if isinstance(offset, Unset):
        json_offset = UNSET
    else:
        json_offset = offset
    params["offset"] = json_offset

    params = {k: v for k, v in params.items() if v is not UNSET and v is not None}

    _kwargs: dict[str, Any] = {
        "method": "get",
        "url": "/admin/audit-logs",
        "params": params,
    }

    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> AuditLogListResponse | ErrorResponse | None:
    if response.status_code == 200:
        response_200 = AuditLogListResponse.from_dict(response.json())

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
) -> Response[AuditLogListResponse | ErrorResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    *,
    client: AuthenticatedClient,
    actor_type: None | str | Unset = UNSET,
    actor_id: None | Unset | UUID = UNSET,
    action: None | str | Unset = UNSET,
    resource_type: None | str | Unset = UNSET,
    resource_id: None | Unset | UUID = UNSET,
    from_: datetime.datetime | None | Unset = UNSET,
    to: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
    offset: int | None | Unset = UNSET,
) -> Response[AuditLogListResponse | ErrorResponse]:
    """Lists audit logs with optional filtering and pagination.

     Returns audit log entries matching the specified filters, ordered by
    timestamp descending (most recent first).

    # Authentication

    Requires admin PAK authentication.

    # Query Parameters

    - `actor_type`: Filter by actor type (admin, agent, generator, system).
    - `actor_id`: Filter by actor UUID.
    - `action`: Filter by action (exact match or prefix with *).
    - `resource_type`: Filter by resource type.
    - `resource_id`: Filter by resource UUID.
    - `from`: Filter by start time (inclusive).
    - `to`: Filter by end time (exclusive).
    - `limit`: Maximum results (default 100, max 1000).
    - `offset`: Number of results to skip.

    # Returns

    - `200 OK`: List of audit logs with pagination info.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Database error.

    Args:
        actor_type (None | str | Unset):
        actor_id (None | Unset | UUID):
        action (None | str | Unset):
        resource_type (None | str | Unset):
        resource_id (None | Unset | UUID):
        from_ (datetime.datetime | None | Unset):
        to (datetime.datetime | None | Unset):
        limit (int | None | Unset):
        offset (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[AuditLogListResponse | ErrorResponse]
    """

    kwargs = _get_kwargs(
        actor_type=actor_type,
        actor_id=actor_id,
        action=action,
        resource_type=resource_type,
        resource_id=resource_id,
        from_=from_,
        to=to,
        limit=limit,
        offset=offset,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    *,
    client: AuthenticatedClient,
    actor_type: None | str | Unset = UNSET,
    actor_id: None | Unset | UUID = UNSET,
    action: None | str | Unset = UNSET,
    resource_type: None | str | Unset = UNSET,
    resource_id: None | Unset | UUID = UNSET,
    from_: datetime.datetime | None | Unset = UNSET,
    to: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
    offset: int | None | Unset = UNSET,
) -> AuditLogListResponse | ErrorResponse | None:
    """Lists audit logs with optional filtering and pagination.

     Returns audit log entries matching the specified filters, ordered by
    timestamp descending (most recent first).

    # Authentication

    Requires admin PAK authentication.

    # Query Parameters

    - `actor_type`: Filter by actor type (admin, agent, generator, system).
    - `actor_id`: Filter by actor UUID.
    - `action`: Filter by action (exact match or prefix with *).
    - `resource_type`: Filter by resource type.
    - `resource_id`: Filter by resource UUID.
    - `from`: Filter by start time (inclusive).
    - `to`: Filter by end time (exclusive).
    - `limit`: Maximum results (default 100, max 1000).
    - `offset`: Number of results to skip.

    # Returns

    - `200 OK`: List of audit logs with pagination info.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Database error.

    Args:
        actor_type (None | str | Unset):
        actor_id (None | Unset | UUID):
        action (None | str | Unset):
        resource_type (None | str | Unset):
        resource_id (None | Unset | UUID):
        from_ (datetime.datetime | None | Unset):
        to (datetime.datetime | None | Unset):
        limit (int | None | Unset):
        offset (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        AuditLogListResponse | ErrorResponse
    """

    return sync_detailed(
        client=client,
        actor_type=actor_type,
        actor_id=actor_id,
        action=action,
        resource_type=resource_type,
        resource_id=resource_id,
        from_=from_,
        to=to,
        limit=limit,
        offset=offset,
    ).parsed


async def asyncio_detailed(
    *,
    client: AuthenticatedClient,
    actor_type: None | str | Unset = UNSET,
    actor_id: None | Unset | UUID = UNSET,
    action: None | str | Unset = UNSET,
    resource_type: None | str | Unset = UNSET,
    resource_id: None | Unset | UUID = UNSET,
    from_: datetime.datetime | None | Unset = UNSET,
    to: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
    offset: int | None | Unset = UNSET,
) -> Response[AuditLogListResponse | ErrorResponse]:
    """Lists audit logs with optional filtering and pagination.

     Returns audit log entries matching the specified filters, ordered by
    timestamp descending (most recent first).

    # Authentication

    Requires admin PAK authentication.

    # Query Parameters

    - `actor_type`: Filter by actor type (admin, agent, generator, system).
    - `actor_id`: Filter by actor UUID.
    - `action`: Filter by action (exact match or prefix with *).
    - `resource_type`: Filter by resource type.
    - `resource_id`: Filter by resource UUID.
    - `from`: Filter by start time (inclusive).
    - `to`: Filter by end time (exclusive).
    - `limit`: Maximum results (default 100, max 1000).
    - `offset`: Number of results to skip.

    # Returns

    - `200 OK`: List of audit logs with pagination info.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Database error.

    Args:
        actor_type (None | str | Unset):
        actor_id (None | Unset | UUID):
        action (None | str | Unset):
        resource_type (None | str | Unset):
        resource_id (None | Unset | UUID):
        from_ (datetime.datetime | None | Unset):
        to (datetime.datetime | None | Unset):
        limit (int | None | Unset):
        offset (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[AuditLogListResponse | ErrorResponse]
    """

    kwargs = _get_kwargs(
        actor_type=actor_type,
        actor_id=actor_id,
        action=action,
        resource_type=resource_type,
        resource_id=resource_id,
        from_=from_,
        to=to,
        limit=limit,
        offset=offset,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    *,
    client: AuthenticatedClient,
    actor_type: None | str | Unset = UNSET,
    actor_id: None | Unset | UUID = UNSET,
    action: None | str | Unset = UNSET,
    resource_type: None | str | Unset = UNSET,
    resource_id: None | Unset | UUID = UNSET,
    from_: datetime.datetime | None | Unset = UNSET,
    to: datetime.datetime | None | Unset = UNSET,
    limit: int | None | Unset = UNSET,
    offset: int | None | Unset = UNSET,
) -> AuditLogListResponse | ErrorResponse | None:
    """Lists audit logs with optional filtering and pagination.

     Returns audit log entries matching the specified filters, ordered by
    timestamp descending (most recent first).

    # Authentication

    Requires admin PAK authentication.

    # Query Parameters

    - `actor_type`: Filter by actor type (admin, agent, generator, system).
    - `actor_id`: Filter by actor UUID.
    - `action`: Filter by action (exact match or prefix with *).
    - `resource_type`: Filter by resource type.
    - `resource_id`: Filter by resource UUID.
    - `from`: Filter by start time (inclusive).
    - `to`: Filter by end time (exclusive).
    - `limit`: Maximum results (default 100, max 1000).
    - `offset`: Number of results to skip.

    # Returns

    - `200 OK`: List of audit logs with pagination info.
    - `401 UNAUTHORIZED`: Missing or invalid authentication.
    - `403 FORBIDDEN`: Authenticated but not an admin.
    - `500 INTERNAL_SERVER_ERROR`: Database error.

    Args:
        actor_type (None | str | Unset):
        actor_id (None | Unset | UUID):
        action (None | str | Unset):
        resource_type (None | str | Unset):
        resource_id (None | Unset | UUID):
        from_ (datetime.datetime | None | Unset):
        to (datetime.datetime | None | Unset):
        limit (int | None | Unset):
        offset (int | None | Unset):

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        AuditLogListResponse | ErrorResponse
    """

    return (
        await asyncio_detailed(
            client=client,
            actor_type=actor_type,
            actor_id=actor_id,
            action=action,
            resource_type=resource_type,
            resource_id=resource_id,
            from_=from_,
            to=to,
            limit=limit,
            offset=offset,
        )
    ).parsed
