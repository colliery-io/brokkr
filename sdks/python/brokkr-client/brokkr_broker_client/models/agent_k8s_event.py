from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="AgentK8SEvent")


@_attrs_define
class AgentK8SEvent:
    """
    Attributes:
        agent_id (UUID):
        created_at (datetime.datetime):
        event_type (str):
        id (UUID):
        involved_object (Any):
        message (str):
        observed_at (datetime.datetime):
        reason (str):
        stack_id (UUID):
        source (None | str | Unset):
    """

    agent_id: UUID
    created_at: datetime.datetime
    event_type: str
    id: UUID
    involved_object: Any
    message: str
    observed_at: datetime.datetime
    reason: str
    stack_id: UUID
    source: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        created_at = self.created_at.isoformat()

        event_type = self.event_type

        id = str(self.id)

        involved_object = self.involved_object

        message = self.message

        observed_at = self.observed_at.isoformat()

        reason = self.reason

        stack_id = str(self.stack_id)

        source: None | str | Unset
        if isinstance(self.source, Unset):
            source = UNSET
        else:
            source = self.source

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "created_at": created_at,
                "event_type": event_type,
                "id": id,
                "involved_object": involved_object,
                "message": message,
                "observed_at": observed_at,
                "reason": reason,
                "stack_id": stack_id,
            }
        )
        if source is not UNSET:
            field_dict["source"] = source

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        created_at = isoparse(d.pop("created_at"))

        event_type = d.pop("event_type")

        id = UUID(d.pop("id"))

        involved_object = d.pop("involved_object")

        message = d.pop("message")

        observed_at = isoparse(d.pop("observed_at"))

        reason = d.pop("reason")

        stack_id = UUID(d.pop("stack_id"))

        def _parse_source(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        source = _parse_source(d.pop("source", UNSET))

        agent_k8s_event = cls(
            agent_id=agent_id,
            created_at=created_at,
            event_type=event_type,
            id=id,
            involved_object=involved_object,
            message=message,
            observed_at=observed_at,
            reason=reason,
            stack_id=stack_id,
            source=source,
        )

        agent_k8s_event.additional_properties = d
        return agent_k8s_event

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
