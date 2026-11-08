from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

T = TypeVar("T", bound="AgentHealthReport")


@_attrs_define
class AgentHealthReport:
    """One agent's latest report on a deployment object.

    Attributes:
        agent_id (UUID): The agent that reported.
        checked_at (datetime.datetime): When the agent checked.
        status (str): healthy, degraded, failing or unknown.
    """

    agent_id: UUID
    checked_at: datetime.datetime
    status: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        checked_at = self.checked_at.isoformat()

        status = self.status

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "checked_at": checked_at,
                "status": status,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        checked_at = isoparse(d.pop("checked_at"))

        status = d.pop("status")

        agent_health_report = cls(
            agent_id=agent_id,
            checked_at=checked_at,
            status=status,
        )

        agent_health_report.additional_properties = d
        return agent_health_report

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
