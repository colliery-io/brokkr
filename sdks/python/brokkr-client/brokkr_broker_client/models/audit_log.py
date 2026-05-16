from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="AuditLog")


@_attrs_define
class AuditLog:
    """An audit log record from the database.

    Attributes:
        action (str): The action performed (e.g., "agent.created", "auth.failed").
        actor_type (str): Type of actor: admin, agent, generator, system.
        created_at (datetime.datetime): When the record was created.
        id (UUID): Unique identifier for the log entry.
        resource_type (str): Type of resource affected.
        timestamp (datetime.datetime): When the event occurred.
        actor_id (None | Unset | UUID): ID of the actor (NULL for system or unauthenticated).
        details (Any | Unset): Additional structured details.
        ip_address (None | str | Unset): Client IP address.
        resource_id (None | Unset | UUID): ID of the affected resource (NULL if not applicable).
        user_agent (None | str | Unset): Client user agent string.
    """

    action: str
    actor_type: str
    created_at: datetime.datetime
    id: UUID
    resource_type: str
    timestamp: datetime.datetime
    actor_id: None | Unset | UUID = UNSET
    details: Any | Unset = UNSET
    ip_address: None | str | Unset = UNSET
    resource_id: None | Unset | UUID = UNSET
    user_agent: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        action = self.action

        actor_type = self.actor_type

        created_at = self.created_at.isoformat()

        id = str(self.id)

        resource_type = self.resource_type

        timestamp = self.timestamp.isoformat()

        actor_id: None | str | Unset
        if isinstance(self.actor_id, Unset):
            actor_id = UNSET
        elif isinstance(self.actor_id, UUID):
            actor_id = str(self.actor_id)
        else:
            actor_id = self.actor_id

        details = self.details

        ip_address: None | str | Unset
        if isinstance(self.ip_address, Unset):
            ip_address = UNSET
        else:
            ip_address = self.ip_address

        resource_id: None | str | Unset
        if isinstance(self.resource_id, Unset):
            resource_id = UNSET
        elif isinstance(self.resource_id, UUID):
            resource_id = str(self.resource_id)
        else:
            resource_id = self.resource_id

        user_agent: None | str | Unset
        if isinstance(self.user_agent, Unset):
            user_agent = UNSET
        else:
            user_agent = self.user_agent

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "action": action,
                "actor_type": actor_type,
                "created_at": created_at,
                "id": id,
                "resource_type": resource_type,
                "timestamp": timestamp,
            }
        )
        if actor_id is not UNSET:
            field_dict["actor_id"] = actor_id
        if details is not UNSET:
            field_dict["details"] = details
        if ip_address is not UNSET:
            field_dict["ip_address"] = ip_address
        if resource_id is not UNSET:
            field_dict["resource_id"] = resource_id
        if user_agent is not UNSET:
            field_dict["user_agent"] = user_agent

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        action = d.pop("action")

        actor_type = d.pop("actor_type")

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        resource_type = d.pop("resource_type")

        timestamp = isoparse(d.pop("timestamp"))

        def _parse_actor_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                actor_id_type_0 = UUID(data)

                return actor_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        actor_id = _parse_actor_id(d.pop("actor_id", UNSET))

        details = d.pop("details", UNSET)

        def _parse_ip_address(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        ip_address = _parse_ip_address(d.pop("ip_address", UNSET))

        def _parse_resource_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                resource_id_type_0 = UUID(data)

                return resource_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        resource_id = _parse_resource_id(d.pop("resource_id", UNSET))

        def _parse_user_agent(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        user_agent = _parse_user_agent(d.pop("user_agent", UNSET))

        audit_log = cls(
            action=action,
            actor_type=actor_type,
            created_at=created_at,
            id=id,
            resource_type=resource_type,
            timestamp=timestamp,
            actor_id=actor_id,
            details=details,
            ip_address=ip_address,
            resource_id=resource_id,
            user_agent=user_agent,
        )

        audit_log.additional_properties = d
        return audit_log

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
