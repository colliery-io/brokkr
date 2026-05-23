from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.agent_k8s_event import AgentK8SEvent
    from ..models.retention_info import RetentionInfo


T = TypeVar("T", bound="K8SEventHistoryResponse")


@_attrs_define
class K8SEventHistoryResponse:
    """
    Attributes:
        events (list[AgentK8SEvent]):
        retention (RetentionInfo):
    """

    events: list[AgentK8SEvent]
    retention: RetentionInfo
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        events = []
        for events_item_data in self.events:
            events_item = events_item_data.to_dict()
            events.append(events_item)

        retention = self.retention.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "events": events,
                "retention": retention,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.agent_k8s_event import AgentK8SEvent
        from ..models.retention_info import RetentionInfo

        d = dict(src_dict)
        events = []
        _events = d.pop("events")
        for events_item_data in _events:
            events_item = AgentK8SEvent.from_dict(events_item_data)

            events.append(events_item)

        retention = RetentionInfo.from_dict(d.pop("retention"))

        k8s_event_history_response = cls(
            events=events,
            retention=retention,
        )

        k8s_event_history_response.additional_properties = d
        return k8s_event_history_response

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
