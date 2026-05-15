from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="PendingWebhookDelivery")


@_attrs_define
class PendingWebhookDelivery:
    """
    Attributes:
        attempts (int):
        event_type (str):
        id (UUID):
        max_retries (int):
        payload (str):
        subscription_id (UUID):
        timeout_seconds (int):
        url (str):
        auth_header (None | str | Unset):
    """

    attempts: int
    event_type: str
    id: UUID
    max_retries: int
    payload: str
    subscription_id: UUID
    timeout_seconds: int
    url: str
    auth_header: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        attempts = self.attempts

        event_type = self.event_type

        id = str(self.id)

        max_retries = self.max_retries

        payload = self.payload

        subscription_id = str(self.subscription_id)

        timeout_seconds = self.timeout_seconds

        url = self.url

        auth_header: None | str | Unset
        if isinstance(self.auth_header, Unset):
            auth_header = UNSET
        else:
            auth_header = self.auth_header

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "attempts": attempts,
                "event_type": event_type,
                "id": id,
                "max_retries": max_retries,
                "payload": payload,
                "subscription_id": subscription_id,
                "timeout_seconds": timeout_seconds,
                "url": url,
            }
        )
        if auth_header is not UNSET:
            field_dict["auth_header"] = auth_header

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        attempts = d.pop("attempts")

        event_type = d.pop("event_type")

        id = UUID(d.pop("id"))

        max_retries = d.pop("max_retries")

        payload = d.pop("payload")

        subscription_id = UUID(d.pop("subscription_id"))

        timeout_seconds = d.pop("timeout_seconds")

        url = d.pop("url")

        def _parse_auth_header(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        auth_header = _parse_auth_header(d.pop("auth_header", UNSET))

        pending_webhook_delivery = cls(
            attempts=attempts,
            event_type=event_type,
            id=id,
            max_retries=max_retries,
            payload=payload,
            subscription_id=subscription_id,
            timeout_seconds=timeout_seconds,
            url=url,
            auth_header=auth_header,
        )

        pending_webhook_delivery.additional_properties = d
        return pending_webhook_delivery

    @property
    def additional_keys(self) -> list[str]:
        return list(self.additional_properties.keys())

    def __getitem__(self, key: str) -> Any:
        return self.additional_properties[key]

    def __setitem__(self, key: str, value: Any) -> None:
        self.additional_properties[key] = value

    def __delitem__(self, key: str) -> None:
        del self.additional_properties[key]

    def __contains__(self, key: str) -> bool:
        return key in self.additional_properties
