from http import HTTPStatus
from typing import Any

import httpx

from ... import errors
from ...client import AuthenticatedClient, Client
from ...models.create_webhook_request import CreateWebhookRequest
from ...models.error_response import ErrorResponse
from ...models.webhook_response import WebhookResponse
from ...types import Response


def _get_kwargs(
    *,
    body: CreateWebhookRequest,
) -> dict[str, Any]:
    headers: dict[str, Any] = {}

    _kwargs: dict[str, Any] = {
        "method": "post",
        "url": "/webhooks",
    }

    _kwargs["json"] = body.to_dict()

    headers["Content-Type"] = "application/json"

    _kwargs["headers"] = headers
    return _kwargs


def _parse_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> ErrorResponse | WebhookResponse | None:
    if response.status_code == 201:
        response_201 = WebhookResponse.from_dict(response.json())

        return response_201

    if response.status_code == 400:
        response_400 = ErrorResponse.from_dict(response.json())

        return response_400

    if response.status_code == 403:
        response_403 = ErrorResponse.from_dict(response.json())

        return response_403

    if response.status_code == 422:
        response_422 = ErrorResponse.from_dict(response.json())

        return response_422

    if response.status_code == 500:
        response_500 = ErrorResponse.from_dict(response.json())

        return response_500

    if client.raise_on_unexpected_status:
        raise errors.UnexpectedStatus(response.status_code, response.content)
    else:
        return None


def _build_response(
    *, client: AuthenticatedClient | Client, response: httpx.Response
) -> Response[ErrorResponse | WebhookResponse]:
    return Response(
        status_code=HTTPStatus(response.status_code),
        content=response.content,
        headers=response.headers,
        parsed=_parse_response(client=client, response=response),
    )


def sync_detailed(
    *,
    client: AuthenticatedClient,
    body: CreateWebhookRequest,
) -> Response[ErrorResponse | WebhookResponse]:
    """
    Args:
        body (CreateWebhookRequest): Body of `POST /webhooks`.

            Unknown keys are ignored rather than rejected, with two deliberate
            exceptions that this endpoint used to accept and silently ignore
            (BROKKR-T-0288). Both are now **rejected with 422** by
            [`reject_removed_write_fields`] before this struct is deserialized:

            * `validate` ("send test request on creation") was documented and parsed but
              never read by `create_webhook` — it did nothing, ever. The real mechanism
              is `POST /webhooks/{id}/test`.
            * `filters.labels` was stored and echoed but never evaluated; label-based
              routing is `target_labels`, which is real.

            Rejecting is the lesser evil. A caller who sends `filters.labels` believes
            their deliveries are scoped; accepting the request and dropping the key
            leaves them with a subscription that fires on everything and a response they
            have no reason to re-read. A 422 naming the field and its replacement is
            noisy exactly once, at the moment the operator can still fix it.

            This is a **write-path** rule only. Subscription rows already stored with a
            `labels` key keep loading and keep delivering — see [`WebhookFilters`],
            whose deserialization stays tolerant of unknown keys.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | WebhookResponse]
    """

    kwargs = _get_kwargs(
        body=body,
    )

    response = client.get_httpx_client().request(
        **kwargs,
    )

    return _build_response(client=client, response=response)


def sync(
    *,
    client: AuthenticatedClient,
    body: CreateWebhookRequest,
) -> ErrorResponse | WebhookResponse | None:
    """
    Args:
        body (CreateWebhookRequest): Body of `POST /webhooks`.

            Unknown keys are ignored rather than rejected, with two deliberate
            exceptions that this endpoint used to accept and silently ignore
            (BROKKR-T-0288). Both are now **rejected with 422** by
            [`reject_removed_write_fields`] before this struct is deserialized:

            * `validate` ("send test request on creation") was documented and parsed but
              never read by `create_webhook` — it did nothing, ever. The real mechanism
              is `POST /webhooks/{id}/test`.
            * `filters.labels` was stored and echoed but never evaluated; label-based
              routing is `target_labels`, which is real.

            Rejecting is the lesser evil. A caller who sends `filters.labels` believes
            their deliveries are scoped; accepting the request and dropping the key
            leaves them with a subscription that fires on everything and a response they
            have no reason to re-read. A 422 naming the field and its replacement is
            noisy exactly once, at the moment the operator can still fix it.

            This is a **write-path** rule only. Subscription rows already stored with a
            `labels` key keep loading and keep delivering — see [`WebhookFilters`],
            whose deserialization stays tolerant of unknown keys.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | WebhookResponse
    """

    return sync_detailed(
        client=client,
        body=body,
    ).parsed


async def asyncio_detailed(
    *,
    client: AuthenticatedClient,
    body: CreateWebhookRequest,
) -> Response[ErrorResponse | WebhookResponse]:
    """
    Args:
        body (CreateWebhookRequest): Body of `POST /webhooks`.

            Unknown keys are ignored rather than rejected, with two deliberate
            exceptions that this endpoint used to accept and silently ignore
            (BROKKR-T-0288). Both are now **rejected with 422** by
            [`reject_removed_write_fields`] before this struct is deserialized:

            * `validate` ("send test request on creation") was documented and parsed but
              never read by `create_webhook` — it did nothing, ever. The real mechanism
              is `POST /webhooks/{id}/test`.
            * `filters.labels` was stored and echoed but never evaluated; label-based
              routing is `target_labels`, which is real.

            Rejecting is the lesser evil. A caller who sends `filters.labels` believes
            their deliveries are scoped; accepting the request and dropping the key
            leaves them with a subscription that fires on everything and a response they
            have no reason to re-read. A 422 naming the field and its replacement is
            noisy exactly once, at the moment the operator can still fix it.

            This is a **write-path** rule only. Subscription rows already stored with a
            `labels` key keep loading and keep delivering — see [`WebhookFilters`],
            whose deserialization stays tolerant of unknown keys.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        Response[ErrorResponse | WebhookResponse]
    """

    kwargs = _get_kwargs(
        body=body,
    )

    response = await client.get_async_httpx_client().request(**kwargs)

    return _build_response(client=client, response=response)


async def asyncio(
    *,
    client: AuthenticatedClient,
    body: CreateWebhookRequest,
) -> ErrorResponse | WebhookResponse | None:
    """
    Args:
        body (CreateWebhookRequest): Body of `POST /webhooks`.

            Unknown keys are ignored rather than rejected, with two deliberate
            exceptions that this endpoint used to accept and silently ignore
            (BROKKR-T-0288). Both are now **rejected with 422** by
            [`reject_removed_write_fields`] before this struct is deserialized:

            * `validate` ("send test request on creation") was documented and parsed but
              never read by `create_webhook` — it did nothing, ever. The real mechanism
              is `POST /webhooks/{id}/test`.
            * `filters.labels` was stored and echoed but never evaluated; label-based
              routing is `target_labels`, which is real.

            Rejecting is the lesser evil. A caller who sends `filters.labels` believes
            their deliveries are scoped; accepting the request and dropping the key
            leaves them with a subscription that fires on everything and a response they
            have no reason to re-read. A 422 naming the field and its replacement is
            noisy exactly once, at the moment the operator can still fix it.

            This is a **write-path** rule only. Subscription rows already stored with a
            `labels` key keep loading and keep delivering — see [`WebhookFilters`],
            whose deserialization stays tolerant of unknown keys.

    Raises:
        errors.UnexpectedStatus: If the server returns an undocumented status code and Client.raise_on_unexpected_status is True.
        httpx.TimeoutException: If the request takes longer than Client.timeout.

    Returns:
        ErrorResponse | WebhookResponse
    """

    return (
        await asyncio_detailed(
            client=client,
            body=body,
        )
    ).parsed
