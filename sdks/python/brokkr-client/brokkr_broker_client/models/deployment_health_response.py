from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.deployment_health import DeploymentHealth


T = TypeVar("T", bound="DeploymentHealthResponse")


@_attrs_define
class DeploymentHealthResponse:
    """Response for deployment object health query.

    Attributes:
        deployment_object_id (UUID): The deployment object ID.
        health_records (list[DeploymentHealth]): List of health records from different agents.
        overall_status (str): Overall status (worst status across all agents).
    """

    deployment_object_id: UUID
    health_records: list[DeploymentHealth]
    overall_status: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        deployment_object_id = str(self.deployment_object_id)

        health_records = []
        for health_records_item_data in self.health_records:
            health_records_item = health_records_item_data.to_dict()
            health_records.append(health_records_item)

        overall_status = self.overall_status

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "deployment_object_id": deployment_object_id,
                "health_records": health_records,
                "overall_status": overall_status,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.deployment_health import DeploymentHealth

        d = dict(src_dict)
        deployment_object_id = UUID(d.pop("deployment_object_id"))

        health_records = []
        _health_records = d.pop("health_records")
        for health_records_item_data in _health_records:
            health_records_item = DeploymentHealth.from_dict(health_records_item_data)

            health_records.append(health_records_item)

        overall_status = d.pop("overall_status")

        deployment_health_response = cls(
            deployment_object_id=deployment_object_id,
            health_records=health_records,
            overall_status=overall_status,
        )

        deployment_health_response.additional_properties = d
        return deployment_health_response

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
