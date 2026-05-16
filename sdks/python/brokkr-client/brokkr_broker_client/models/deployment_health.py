from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="DeploymentHealth")


@_attrs_define
class DeploymentHealth:
    """Represents a deployment health record in the database.

    Example:
        {'agent_id': '123e4567-e89b-12d3-a456-426614174001', 'checked_at': '2023-01-01T00:00:00Z', 'created_at':
            '2023-01-01T00:00:00Z', 'deployment_object_id': '123e4567-e89b-12d3-a456-426614174002', 'id':
            '123e4567-e89b-12d3-a456-426614174000', 'status': 'healthy', 'summary': '{"pods_ready": 3, "pods_total": 3,
            "conditions": []}', 'updated_at': '2023-01-01T00:00:00Z'}

    Attributes:
        agent_id (UUID): ID of the agent that reported this health status. Example:
            123e4567-e89b-12d3-a456-426614174001.
        checked_at (datetime.datetime): Timestamp when the agent last checked health. Example: 2023-01-01T00:00:00Z.
        created_at (datetime.datetime): Timestamp when the record was created. Example: 2023-01-01T00:00:00Z.
        deployment_object_id (UUID): ID of the deployment object this health status applies to. Example:
            123e4567-e89b-12d3-a456-426614174002.
        id (UUID): Unique identifier for the health record. Example: 123e4567-e89b-12d3-a456-426614174000.
        status (str): Health status: healthy, degraded, failing, or unknown. Example: healthy.
        updated_at (datetime.datetime): Timestamp when the record was last updated. Example: 2023-01-01T00:00:00Z.
        summary (None | str | Unset): JSON-encoded summary with pod counts, conditions, and resource details. Example:
            {"pods_ready": 3, "pods_total": 3, "conditions": []}.
    """

    agent_id: UUID
    checked_at: datetime.datetime
    created_at: datetime.datetime
    deployment_object_id: UUID
    id: UUID
    status: str
    updated_at: datetime.datetime
    summary: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        checked_at = self.checked_at.isoformat()

        created_at = self.created_at.isoformat()

        deployment_object_id = str(self.deployment_object_id)

        id = str(self.id)

        status = self.status

        updated_at = self.updated_at.isoformat()

        summary: None | str | Unset
        if isinstance(self.summary, Unset):
            summary = UNSET
        else:
            summary = self.summary

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "checked_at": checked_at,
                "created_at": created_at,
                "deployment_object_id": deployment_object_id,
                "id": id,
                "status": status,
                "updated_at": updated_at,
            }
        )
        if summary is not UNSET:
            field_dict["summary"] = summary

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        checked_at = isoparse(d.pop("checked_at"))

        created_at = isoparse(d.pop("created_at"))

        deployment_object_id = UUID(d.pop("deployment_object_id"))

        id = UUID(d.pop("id"))

        status = d.pop("status")

        updated_at = isoparse(d.pop("updated_at"))

        def _parse_summary(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        summary = _parse_summary(d.pop("summary", UNSET))

        deployment_health = cls(
            agent_id=agent_id,
            checked_at=checked_at,
            created_at=created_at,
            deployment_object_id=deployment_object_id,
            id=id,
            status=status,
            updated_at=updated_at,
            summary=summary,
        )

        deployment_health.additional_properties = d
        return deployment_health

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
