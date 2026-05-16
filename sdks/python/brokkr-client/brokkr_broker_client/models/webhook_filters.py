from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.webhook_filters_labels_type_0 import WebhookFiltersLabelsType0


T = TypeVar("T", bound="WebhookFilters")


@_attrs_define
class WebhookFilters:
    """Filters for webhook subscriptions.

    Attributes:
        agent_id (None | Unset | UUID): Filter by specific agent ID.
        labels (None | Unset | WebhookFiltersLabelsType0): Filter by labels (all must match).
        stack_id (None | Unset | UUID): Filter by specific stack ID.
    """

    agent_id: None | Unset | UUID = UNSET
    labels: None | Unset | WebhookFiltersLabelsType0 = UNSET
    stack_id: None | Unset | UUID = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        from ..models.webhook_filters_labels_type_0 import WebhookFiltersLabelsType0

        agent_id: None | str | Unset
        if isinstance(self.agent_id, Unset):
            agent_id = UNSET
        elif isinstance(self.agent_id, UUID):
            agent_id = str(self.agent_id)
        else:
            agent_id = self.agent_id

        labels: dict[str, Any] | None | Unset
        if isinstance(self.labels, Unset):
            labels = UNSET
        elif isinstance(self.labels, WebhookFiltersLabelsType0):
            labels = self.labels.to_dict()
        else:
            labels = self.labels

        stack_id: None | str | Unset
        if isinstance(self.stack_id, Unset):
            stack_id = UNSET
        elif isinstance(self.stack_id, UUID):
            stack_id = str(self.stack_id)
        else:
            stack_id = self.stack_id

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update({})
        if agent_id is not UNSET:
            field_dict["agent_id"] = agent_id
        if labels is not UNSET:
            field_dict["labels"] = labels
        if stack_id is not UNSET:
            field_dict["stack_id"] = stack_id

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.webhook_filters_labels_type_0 import WebhookFiltersLabelsType0

        d = dict(src_dict)

        def _parse_agent_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                agent_id_type_0 = UUID(data)

                return agent_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        agent_id = _parse_agent_id(d.pop("agent_id", UNSET))

        def _parse_labels(data: object) -> None | Unset | WebhookFiltersLabelsType0:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, dict):
                    raise TypeError()
                labels_type_0 = WebhookFiltersLabelsType0.from_dict(data)

                return labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | WebhookFiltersLabelsType0, data)

        labels = _parse_labels(d.pop("labels", UNSET))

        def _parse_stack_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                stack_id_type_0 = UUID(data)

                return stack_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        stack_id = _parse_stack_id(d.pop("stack_id", UNSET))

        webhook_filters = cls(
            agent_id=agent_id,
            labels=labels,
            stack_id=stack_id,
        )

        webhook_filters.additional_properties = d
        return webhook_filters

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
