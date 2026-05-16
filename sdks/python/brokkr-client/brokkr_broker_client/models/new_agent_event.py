from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="NewAgentEvent")


@_attrs_define
class NewAgentEvent:
    """Represents a new agent event to be inserted into the database.

    Attributes:
        agent_id (UUID): ID of the agent associated with this event.
        deployment_object_id (UUID): ID of the deployment object associated with this event.
        event_type (str): Type of the event.
        status (str): Status of the event (e.g., "SUCCESS", "FAILURE").
        message (None | str | Unset): Optional message providing additional details about the event.
    """

    agent_id: UUID
    deployment_object_id: UUID
    event_type: str
    status: str
    message: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        deployment_object_id = str(self.deployment_object_id)

        event_type = self.event_type

        status = self.status

        message: None | str | Unset
        if isinstance(self.message, Unset):
            message = UNSET
        else:
            message = self.message

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "deployment_object_id": deployment_object_id,
                "event_type": event_type,
                "status": status,
            }
        )
        if message is not UNSET:
            field_dict["message"] = message

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        deployment_object_id = UUID(d.pop("deployment_object_id"))

        event_type = d.pop("event_type")

        status = d.pop("status")

        def _parse_message(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        message = _parse_message(d.pop("message", UNSET))

        new_agent_event = cls(
            agent_id=agent_id,
            deployment_object_id=deployment_object_id,
            event_type=event_type,
            status=status,
            message=message,
        )

        new_agent_event.additional_properties = d
        return new_agent_event

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
