from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.webhook_filters import WebhookFilters


T = TypeVar("T", bound="UpdateWebhookRequest")


@_attrs_define
class UpdateWebhookRequest:
    """
    Attributes:
        auth_header (None | str | Unset):
        enabled (bool | None | Unset):
        event_types (list[str] | None | Unset):
        filters (WebhookFilters | Unset): Filters for webhook subscriptions.
        max_retries (int | None | Unset):
        name (None | str | Unset):
        target_labels (list[str] | None | Unset):
        timeout_seconds (int | None | Unset):
        url (None | str | Unset):
    """

    auth_header: None | str | Unset = UNSET
    enabled: bool | None | Unset = UNSET
    event_types: list[str] | None | Unset = UNSET
    filters: WebhookFilters | Unset = UNSET
    max_retries: int | None | Unset = UNSET
    name: None | str | Unset = UNSET
    target_labels: list[str] | None | Unset = UNSET
    timeout_seconds: int | None | Unset = UNSET
    url: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        auth_header: None | str | Unset
        if isinstance(self.auth_header, Unset):
            auth_header = UNSET
        else:
            auth_header = self.auth_header

        enabled: bool | None | Unset
        if isinstance(self.enabled, Unset):
            enabled = UNSET
        else:
            enabled = self.enabled

        event_types: list[str] | None | Unset
        if isinstance(self.event_types, Unset):
            event_types = UNSET
        elif isinstance(self.event_types, list):
            event_types = self.event_types

        else:
            event_types = self.event_types

        filters: dict[str, Any] | Unset = UNSET
        if not isinstance(self.filters, Unset):
            filters = self.filters.to_dict()

        max_retries: int | None | Unset
        if isinstance(self.max_retries, Unset):
            max_retries = UNSET
        else:
            max_retries = self.max_retries

        name: None | str | Unset
        if isinstance(self.name, Unset):
            name = UNSET
        else:
            name = self.name

        target_labels: list[str] | None | Unset
        if isinstance(self.target_labels, Unset):
            target_labels = UNSET
        elif isinstance(self.target_labels, list):
            target_labels = self.target_labels

        else:
            target_labels = self.target_labels

        timeout_seconds: int | None | Unset
        if isinstance(self.timeout_seconds, Unset):
            timeout_seconds = UNSET
        else:
            timeout_seconds = self.timeout_seconds

        url: None | str | Unset
        if isinstance(self.url, Unset):
            url = UNSET
        else:
            url = self.url

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update({})
        if auth_header is not UNSET:
            field_dict["auth_header"] = auth_header
        if enabled is not UNSET:
            field_dict["enabled"] = enabled
        if event_types is not UNSET:
            field_dict["event_types"] = event_types
        if filters is not UNSET:
            field_dict["filters"] = filters
        if max_retries is not UNSET:
            field_dict["max_retries"] = max_retries
        if name is not UNSET:
            field_dict["name"] = name
        if target_labels is not UNSET:
            field_dict["target_labels"] = target_labels
        if timeout_seconds is not UNSET:
            field_dict["timeout_seconds"] = timeout_seconds
        if url is not UNSET:
            field_dict["url"] = url

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.webhook_filters import WebhookFilters

        d = dict(src_dict)

        def _parse_auth_header(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        auth_header = _parse_auth_header(d.pop("auth_header", UNSET))

        def _parse_enabled(data: object) -> bool | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(bool | None | Unset, data)

        enabled = _parse_enabled(d.pop("enabled", UNSET))

        def _parse_event_types(data: object) -> list[str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                event_types_type_0 = cast(list[str], data)

                return event_types_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[str] | None | Unset, data)

        event_types = _parse_event_types(d.pop("event_types", UNSET))

        _filters = d.pop("filters", UNSET)
        filters: WebhookFilters | Unset
        if isinstance(_filters, Unset):
            filters = UNSET
        else:
            filters = WebhookFilters.from_dict(_filters)

        def _parse_max_retries(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        max_retries = _parse_max_retries(d.pop("max_retries", UNSET))

        def _parse_name(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        name = _parse_name(d.pop("name", UNSET))

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

        def _parse_timeout_seconds(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        timeout_seconds = _parse_timeout_seconds(d.pop("timeout_seconds", UNSET))

        def _parse_url(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        url = _parse_url(d.pop("url", UNSET))

        update_webhook_request = cls(
            auth_header=auth_header,
            enabled=enabled,
            event_types=event_types,
            filters=filters,
            max_retries=max_retries,
            name=name,
            target_labels=target_labels,
            timeout_seconds=timeout_seconds,
            url=url,
        )

        update_webhook_request.additional_properties = d
        return update_webhook_request

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
