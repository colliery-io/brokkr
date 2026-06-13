from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.agent_event import AgentEvent
    from ..models.fleet_agent_record import FleetAgentRecord


T = TypeVar("T", bound="AgentFleetStatusResponse")


@_attrs_define
class AgentFleetStatusResponse:
    """Response body for the per-agent fleet-status detail view: the agent's fleet
    record plus its most recent events (newest first).

        Attributes:
            recent_events (list[AgentEvent]): The agent's most recent events, newest first (up to 20).
            record (FleetAgentRecord): A per-agent fleet record: measured signals only, no health verdicts.

                All time-relative fields (`heartbeat_age_seconds`, `seconds_since_last_event`)
                are computed on read as `now - timestamp`, clamped to be non-negative.
    """

    recent_events: list[AgentEvent]
    record: FleetAgentRecord
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        recent_events = []
        for recent_events_item_data in self.recent_events:
            recent_events_item = recent_events_item_data.to_dict()
            recent_events.append(recent_events_item)

        record = self.record.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "recent_events": recent_events,
                "record": record,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.agent_event import AgentEvent
        from ..models.fleet_agent_record import FleetAgentRecord

        d = dict(src_dict)
        recent_events = []
        _recent_events = d.pop("recent_events")
        for recent_events_item_data in _recent_events:
            recent_events_item = AgentEvent.from_dict(recent_events_item_data)

            recent_events.append(recent_events_item)

        record = FleetAgentRecord.from_dict(d.pop("record"))

        agent_fleet_status_response = cls(
            recent_events=recent_events,
            record=record,
        )

        agent_fleet_status_response.additional_properties = d
        return agent_fleet_status_response

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
