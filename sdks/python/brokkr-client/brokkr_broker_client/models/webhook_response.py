from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.webhook_filters import WebhookFilters


T = TypeVar("T", bound="WebhookResponse")


@_attrs_define
class WebhookResponse:
    """
    Attributes:
        created_at (datetime.datetime):
        enabled (bool):
        event_types (list[str]):
        has_auth_header (bool):
        has_url (bool):
        id (UUID):
        max_retries (int):
        name (str):
        timeout_seconds (int):
        updated_at (datetime.datetime):
        created_by (None | str | Unset):
        filters (WebhookFilters | Unset): Filters for webhook subscriptions.
        target_labels (list[str] | None | Unset):
    """

    created_at: datetime.datetime
    enabled: bool
    event_types: list[str]
    has_auth_header: bool
    has_url: bool
    id: UUID
    max_retries: int
    name: str
    timeout_seconds: int
    updated_at: datetime.datetime
    created_by: None | str | Unset = UNSET
    filters: WebhookFilters | Unset = UNSET
    target_labels: list[str] | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        enabled = self.enabled

        event_types = self.event_types

        has_auth_header = self.has_auth_header

        has_url = self.has_url

        id = str(self.id)

        max_retries = self.max_retries

        name = self.name

        timeout_seconds = self.timeout_seconds

        updated_at = self.updated_at.isoformat()

        created_by: None | str | Unset
        if isinstance(self.created_by, Unset):
            created_by = UNSET
        else:
            created_by = self.created_by

        filters: dict[str, Any] | Unset = UNSET
        if not isinstance(self.filters, Unset):
            filters = self.filters.to_dict()

        target_labels: list[str] | None | Unset
        if isinstance(self.target_labels, Unset):
            target_labels = UNSET
        elif isinstance(self.target_labels, list):
            target_labels = self.target_labels

        else:
            target_labels = self.target_labels

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "enabled": enabled,
                "event_types": event_types,
                "has_auth_header": has_auth_header,
                "has_url": has_url,
                "id": id,
                "max_retries": max_retries,
                "name": name,
                "timeout_seconds": timeout_seconds,
                "updated_at": updated_at,
            }
        )
        if created_by is not UNSET:
            field_dict["created_by"] = created_by
        if filters is not UNSET:
            field_dict["filters"] = filters
        if target_labels is not UNSET:
            field_dict["target_labels"] = target_labels

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.webhook_filters import WebhookFilters

        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        enabled = d.pop("enabled")

        event_types = cast(list[str], d.pop("event_types"))

        has_auth_header = d.pop("has_auth_header")

        has_url = d.pop("has_url")

        id = UUID(d.pop("id"))

        max_retries = d.pop("max_retries")

        name = d.pop("name")

        timeout_seconds = d.pop("timeout_seconds")

        updated_at = isoparse(d.pop("updated_at"))

        def _parse_created_by(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        created_by = _parse_created_by(d.pop("created_by", UNSET))

        _filters = d.pop("filters", UNSET)
        filters: WebhookFilters | Unset
        if isinstance(_filters, Unset):
            filters = UNSET
        else:
            filters = WebhookFilters.from_dict(_filters)

        def _parse_target_labels(data: object) -> list[str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                target_labels_type_0 = cast(list[str], data)

                return target_labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[str] | None | Unset, data)

        target_labels = _parse_target_labels(d.pop("target_labels", UNSET))

        webhook_response = cls(
            created_at=created_at,
            enabled=enabled,
            event_types=event_types,
            has_auth_header=has_auth_header,
            has_url=has_url,
            id=id,
            max_retries=max_retries,
            name=name,
            timeout_seconds=timeout_seconds,
            updated_at=updated_at,
            created_by=created_by,
            filters=filters,
            target_labels=target_labels,
        )

        webhook_response.additional_properties = d
        return webhook_response

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
