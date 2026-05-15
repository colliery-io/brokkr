from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.webhook_filters import WebhookFilters


T = TypeVar("T", bound="CreateWebhookRequest")


@_attrs_define
class CreateWebhookRequest:
    """
    Attributes:
        event_types (list[str]):
        name (str):
        url (str):
        auth_header (None | str | Unset):
        filters (WebhookFilters | Unset): Filters for webhook subscriptions.
        max_retries (int | None | Unset):
        target_labels (list[str] | None | Unset):
        timeout_seconds (int | None | Unset):
        validate (bool | Unset):
    """

    event_types: list[str]
    name: str
    url: str
    auth_header: None | str | Unset = UNSET
    filters: WebhookFilters | Unset = UNSET
    max_retries: int | None | Unset = UNSET
    target_labels: list[str] | None | Unset = UNSET
    timeout_seconds: int | None | Unset = UNSET
    validate: bool | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        event_types = self.event_types

        name = self.name

        url = self.url

        auth_header: None | str | Unset
        if isinstance(self.auth_header, Unset):
            auth_header = UNSET
        else:
            auth_header = self.auth_header

        filters: dict[str, Any] | Unset = UNSET
        if not isinstance(self.filters, Unset):
            filters = self.filters.to_dict()

        max_retries: int | None | Unset
        if isinstance(self.max_retries, Unset):
            max_retries = UNSET
        else:
            max_retries = self.max_retries

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

        validate = self.validate

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "event_types": event_types,
                "name": name,
                "url": url,
            }
        )
        if auth_header is not UNSET:
            field_dict["auth_header"] = auth_header
        if filters is not UNSET:
            field_dict["filters"] = filters
        if max_retries is not UNSET:
            field_dict["max_retries"] = max_retries
        if target_labels is not UNSET:
            field_dict["target_labels"] = target_labels
        if timeout_seconds is not UNSET:
            field_dict["timeout_seconds"] = timeout_seconds
        if validate is not UNSET:
            field_dict["validate"] = validate

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.webhook_filters import WebhookFilters

        d = dict(src_dict)
        event_types = cast(list[str], d.pop("event_types"))

        name = d.pop("name")

        url = d.pop("url")

        def _parse_auth_header(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        auth_header = _parse_auth_header(d.pop("auth_header", UNSET))

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

        validate = d.pop("validate", UNSET)

        create_webhook_request = cls(
            event_types=event_types,
            name=name,
            url=url,
            auth_header=auth_header,
            filters=filters,
            max_retries=max_retries,
            target_labels=target_labels,
            timeout_seconds=timeout_seconds,
            validate=validate,
        )

        create_webhook_request.additional_properties = d
        return create_webhook_request

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
