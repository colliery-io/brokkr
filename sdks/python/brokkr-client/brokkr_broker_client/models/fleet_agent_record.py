from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="FleetAgentRecord")


@_attrs_define
class FleetAgentRecord:
    """A per-agent fleet record: measured signals only, no health verdicts.

    All time-relative fields (`heartbeat_age_seconds`, `seconds_since_last_event`)
    are computed on read as `now - timestamp`, clamped to be non-negative.

        Attributes:
            agent_id (UUID): The agent's unique identifier.
            claimed_work_orders (int): Number of work orders currently CLAIMED by this agent.
            health_degraded (int): Count of this agent's deployment-health records with status `degraded`.
            health_failing (int): Count of this agent's deployment-health records with status `failing`.
            name (str): The agent's name.
            pending_object_count (int): Number of pending (not-yet-acknowledged) deployment objects targeted at
                this agent.
            pending_work_orders (int): Number of PENDING work orders this agent is eligible to claim.
            status (str): The agent's lifecycle status (e.g. "ACTIVE").
            ws_connected (bool): Whether the agent currently holds a broker↔agent WebSocket connection.
            connected_since (datetime.datetime | None | Unset): When the current WebSocket connection was established, if
                connected.
            heartbeat_age_seconds (int | None | Unset): Seconds since the last heartbeat (`now - last_heartbeat`, clamped >=
                0).
            last_event_at (datetime.datetime | None | Unset): Timestamp of this agent's most recent (non-deleted) event, if
                any.
            last_heartbeat (datetime.datetime | None | Unset): The agent's last recorded heartbeat timestamp.
            seconds_since_last_event (int | None | Unset): Seconds since the last event (`now - last_event_at`, clamped >=
                0).
    """

    agent_id: UUID
    claimed_work_orders: int
    health_degraded: int
    health_failing: int
    name: str
    pending_object_count: int
    pending_work_orders: int
    status: str
    ws_connected: bool
    connected_since: datetime.datetime | None | Unset = UNSET
    heartbeat_age_seconds: int | None | Unset = UNSET
    last_event_at: datetime.datetime | None | Unset = UNSET
    last_heartbeat: datetime.datetime | None | Unset = UNSET
    seconds_since_last_event: int | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        claimed_work_orders = self.claimed_work_orders

        health_degraded = self.health_degraded

        health_failing = self.health_failing

        name = self.name

        pending_object_count = self.pending_object_count

        pending_work_orders = self.pending_work_orders

        status = self.status

        ws_connected = self.ws_connected

        connected_since: None | str | Unset
        if isinstance(self.connected_since, Unset):
            connected_since = UNSET
        elif isinstance(self.connected_since, datetime.datetime):
            connected_since = self.connected_since.isoformat()
        else:
            connected_since = self.connected_since

        heartbeat_age_seconds: int | None | Unset
        if isinstance(self.heartbeat_age_seconds, Unset):
            heartbeat_age_seconds = UNSET
        else:
            heartbeat_age_seconds = self.heartbeat_age_seconds

        last_event_at: None | str | Unset
        if isinstance(self.last_event_at, Unset):
            last_event_at = UNSET
        elif isinstance(self.last_event_at, datetime.datetime):
            last_event_at = self.last_event_at.isoformat()
        else:
            last_event_at = self.last_event_at

        last_heartbeat: None | str | Unset
        if isinstance(self.last_heartbeat, Unset):
            last_heartbeat = UNSET
        elif isinstance(self.last_heartbeat, datetime.datetime):
            last_heartbeat = self.last_heartbeat.isoformat()
        else:
            last_heartbeat = self.last_heartbeat

        seconds_since_last_event: int | None | Unset
        if isinstance(self.seconds_since_last_event, Unset):
            seconds_since_last_event = UNSET
        else:
            seconds_since_last_event = self.seconds_since_last_event

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "claimed_work_orders": claimed_work_orders,
                "health_degraded": health_degraded,
                "health_failing": health_failing,
                "name": name,
                "pending_object_count": pending_object_count,
                "pending_work_orders": pending_work_orders,
                "status": status,
                "ws_connected": ws_connected,
            }
        )
        if connected_since is not UNSET:
            field_dict["connected_since"] = connected_since
        if heartbeat_age_seconds is not UNSET:
            field_dict["heartbeat_age_seconds"] = heartbeat_age_seconds
        if last_event_at is not UNSET:
            field_dict["last_event_at"] = last_event_at
        if last_heartbeat is not UNSET:
            field_dict["last_heartbeat"] = last_heartbeat
        if seconds_since_last_event is not UNSET:
            field_dict["seconds_since_last_event"] = seconds_since_last_event

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        claimed_work_orders = d.pop("claimed_work_orders")

        health_degraded = d.pop("health_degraded")

        health_failing = d.pop("health_failing")

        name = d.pop("name")

        pending_object_count = d.pop("pending_object_count")

        pending_work_orders = d.pop("pending_work_orders")

        status = d.pop("status")

        ws_connected = d.pop("ws_connected")

        def _parse_connected_since(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                connected_since_type_0 = isoparse(data)

                return connected_since_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        connected_since = _parse_connected_since(d.pop("connected_since", UNSET))

        def _parse_heartbeat_age_seconds(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        heartbeat_age_seconds = _parse_heartbeat_age_seconds(d.pop("heartbeat_age_seconds", UNSET))

        def _parse_last_event_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_event_at_type_0 = isoparse(data)

                return last_event_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_event_at = _parse_last_event_at(d.pop("last_event_at", UNSET))

        def _parse_last_heartbeat(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_heartbeat_type_0 = isoparse(data)

                return last_heartbeat_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_heartbeat = _parse_last_heartbeat(d.pop("last_heartbeat", UNSET))

        def _parse_seconds_since_last_event(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        seconds_since_last_event = _parse_seconds_since_last_event(d.pop("seconds_since_last_event", UNSET))

        fleet_agent_record = cls(
            agent_id=agent_id,
            claimed_work_orders=claimed_work_orders,
            health_degraded=health_degraded,
            health_failing=health_failing,
            name=name,
            pending_object_count=pending_object_count,
            pending_work_orders=pending_work_orders,
            status=status,
            ws_connected=ws_connected,
            connected_since=connected_since,
            heartbeat_age_seconds=heartbeat_age_seconds,
            last_event_at=last_event_at,
            last_heartbeat=last_heartbeat,
            seconds_since_last_event=seconds_since_last_event,
        )

        fleet_agent_record.additional_properties = d
        return fleet_agent_record

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
