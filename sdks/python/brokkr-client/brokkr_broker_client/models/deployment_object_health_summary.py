from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="DeploymentObjectHealthSummary")


@_attrs_define
class DeploymentObjectHealthSummary:
    """Summary of health for a deployment object within a stack.

    Attributes:
        degraded_agents (int): Number of agents reporting degraded.
        failing_agents (int): Number of agents reporting failing.
        healthy_agents (int): Number of agents reporting healthy.
        id (UUID): The deployment object ID.
        status (str): Overall status for this deployment object.
    """

    degraded_agents: int
    failing_agents: int
    healthy_agents: int
    id: UUID
    status: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        degraded_agents = self.degraded_agents

        failing_agents = self.failing_agents

        healthy_agents = self.healthy_agents

        id = str(self.id)

        status = self.status

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "degraded_agents": degraded_agents,
                "failing_agents": failing_agents,
                "healthy_agents": healthy_agents,
                "id": id,
                "status": status,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        degraded_agents = d.pop("degraded_agents")

        failing_agents = d.pop("failing_agents")

        healthy_agents = d.pop("healthy_agents")

        id = UUID(d.pop("id"))

        status = d.pop("status")

        deployment_object_health_summary = cls(
            degraded_agents=degraded_agents,
            failing_agents=failing_agents,
            healthy_agents=healthy_agents,
            id=id,
            status=status,
        )

        deployment_object_health_summary.additional_properties = d
        return deployment_object_health_summary

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
