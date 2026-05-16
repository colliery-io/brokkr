from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="AgentEvent")


@_attrs_define
class AgentEvent:
    """Represents an agent event in the database.

    Example:
        {'agent_id': '123e4567-e89b-12d3-a456-426614174001', 'created_at': '2023-01-01T00:00:00Z', 'deleted_at': None,
            'deployment_object_id': '123e4567-e89b-12d3-a456-426614174002', 'event_type': 'DEPLOYMENT', 'id':
            '123e4567-e89b-12d3-a456-426614174000', 'message': 'Deployment completed successfully', 'status': 'SUCCESS',
            'updated_at': '2023-01-01T00:00:00Z'}

    Attributes:
        agent_id (UUID): ID of the agent associated with this event. Example: 123e4567-e89b-12d3-a456-426614174001.
        created_at (datetime.datetime): Timestamp when the event was created. Example: 2023-01-01T00:00:00Z.
        deployment_object_id (UUID): ID of the deployment object associated with this event. Example:
            123e4567-e89b-12d3-a456-426614174002.
        event_type (str): Type of the event. Example: DEPLOYMENT.
        id (UUID): Unique identifier for the event. Example: 123e4567-e89b-12d3-a456-426614174000.
        status (str): Status of the event (e.g., "SUCCESS", "FAILURE", "IN_PROGRESS", "PENDING"). Example: SUCCESS.
        updated_at (datetime.datetime): Timestamp when the event was last updated. Example: 2023-01-01T00:00:00Z.
        deleted_at (datetime.datetime | None | Unset): Timestamp for soft deletion, if applicable. Example: null.
        message (None | str | Unset): Optional message providing additional details about the event. Example: Deployment
            completed successfully.
    """

    agent_id: UUID
    created_at: datetime.datetime
    deployment_object_id: UUID
    event_type: str
    id: UUID
    status: str
    updated_at: datetime.datetime
    deleted_at: datetime.datetime | None | Unset = UNSET
    message: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        created_at = self.created_at.isoformat()

        deployment_object_id = str(self.deployment_object_id)

        event_type = self.event_type

        id = str(self.id)

        status = self.status

        updated_at = self.updated_at.isoformat()

        deleted_at: None | str | Unset
        if isinstance(self.deleted_at, Unset):
            deleted_at = UNSET
        elif isinstance(self.deleted_at, datetime.datetime):
            deleted_at = self.deleted_at.isoformat()
        else:
            deleted_at = self.deleted_at

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
                "created_at": created_at,
                "deployment_object_id": deployment_object_id,
                "event_type": event_type,
                "id": id,
                "status": status,
                "updated_at": updated_at,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at
        if message is not UNSET:
            field_dict["message"] = message

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        created_at = isoparse(d.pop("created_at"))

        deployment_object_id = UUID(d.pop("deployment_object_id"))

        event_type = d.pop("event_type")

        id = UUID(d.pop("id"))

        status = d.pop("status")

        updated_at = isoparse(d.pop("updated_at"))

        def _parse_deleted_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                deleted_at_type_0 = isoparse(data)

                return deleted_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        deleted_at = _parse_deleted_at(d.pop("deleted_at", UNSET))

        def _parse_message(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        message = _parse_message(d.pop("message", UNSET))

        agent_event = cls(
            agent_id=agent_id,
            created_at=created_at,
            deployment_object_id=deployment_object_id,
            event_type=event_type,
            id=id,
            status=status,
            updated_at=updated_at,
            deleted_at=deleted_at,
            message=message,
        )

        agent_event.additional_properties = d
        return agent_event

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
