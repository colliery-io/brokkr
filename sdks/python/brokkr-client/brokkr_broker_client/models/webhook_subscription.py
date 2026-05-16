from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="WebhookSubscription")


@_attrs_define
class WebhookSubscription:
    """A webhook subscription record from the database.

    Attributes:
        created_at (datetime.datetime): When the subscription was created.
        enabled (bool): Whether the subscription is active.
        event_types (list[None | str]): Event types to subscribe to (supports wildcards like "deployment.*").
        id (UUID): Unique identifier for the subscription.
        max_retries (int): Maximum delivery retry attempts.
        name (str): Human-readable name for the subscription.
        timeout_seconds (int): HTTP request timeout in seconds.
        updated_at (datetime.datetime): When the subscription was last updated.
        created_by (None | str | Unset): Who created the subscription.
        filters (None | str | Unset): JSON-encoded filters.
        target_labels (list[None | str] | None | Unset): Labels for delivery targeting (NULL = broker delivers).
    """

    created_at: datetime.datetime
    enabled: bool
    event_types: list[None | str]
    id: UUID
    max_retries: int
    name: str
    timeout_seconds: int
    updated_at: datetime.datetime
    created_by: None | str | Unset = UNSET
    filters: None | str | Unset = UNSET
    target_labels: list[None | str] | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        enabled = self.enabled

        event_types = []
        for event_types_item_data in self.event_types:
            event_types_item: None | str
            event_types_item = event_types_item_data
            event_types.append(event_types_item)

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

        filters: None | str | Unset
        if isinstance(self.filters, Unset):
            filters = UNSET
        else:
            filters = self.filters

        target_labels: list[None | str] | None | Unset
        if isinstance(self.target_labels, Unset):
            target_labels = UNSET
        elif isinstance(self.target_labels, list):
            target_labels = []
            for target_labels_type_0_item_data in self.target_labels:
                target_labels_type_0_item: None | str
                target_labels_type_0_item = target_labels_type_0_item_data
                target_labels.append(target_labels_type_0_item)

        else:
            target_labels = self.target_labels

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "enabled": enabled,
                "event_types": event_types,
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
        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        enabled = d.pop("enabled")

        event_types = []
        _event_types = d.pop("event_types")
        for event_types_item_data in _event_types:

            def _parse_event_types_item(data: object) -> None | str:
                if data is None:
                    return data
                return cast(None | str, data)

            event_types_item = _parse_event_types_item(event_types_item_data)

            event_types.append(event_types_item)

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

        def _parse_filters(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        filters = _parse_filters(d.pop("filters", UNSET))

        def _parse_target_labels(data: object) -> list[None | str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                target_labels_type_0 = []
                _target_labels_type_0 = data
                for target_labels_type_0_item_data in _target_labels_type_0:

                    def _parse_target_labels_type_0_item(data: object) -> None | str:
                        if data is None:
                            return data
                        return cast(None | str, data)

                    target_labels_type_0_item = _parse_target_labels_type_0_item(target_labels_type_0_item_data)

                    target_labels_type_0.append(target_labels_type_0_item)

                return target_labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[None | str] | None | Unset, data)

        target_labels = _parse_target_labels(d.pop("target_labels", UNSET))

        webhook_subscription = cls(
            created_at=created_at,
            enabled=enabled,
            event_types=event_types,
            id=id,
            max_retries=max_retries,
            name=name,
            timeout_seconds=timeout_seconds,
            updated_at=updated_at,
            created_by=created_by,
            filters=filters,
            target_labels=target_labels,
        )

        webhook_subscription.additional_properties = d
        return webhook_subscription

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
