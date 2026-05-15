from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.deployment_object_health_update import DeploymentObjectHealthUpdate


T = TypeVar("T", bound="HealthStatusUpdate")


@_attrs_define
class HealthStatusUpdate:
    """Request body for updating health status from an agent.

    Attributes:
        deployment_objects (list[DeploymentObjectHealthUpdate]): List of deployment object health updates.
    """

    deployment_objects: list[DeploymentObjectHealthUpdate]
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        deployment_objects = []
        for deployment_objects_item_data in self.deployment_objects:
            deployment_objects_item = deployment_objects_item_data.to_dict()
            deployment_objects.append(deployment_objects_item)

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "deployment_objects": deployment_objects,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.deployment_object_health_update import DeploymentObjectHealthUpdate

        d = dict(src_dict)
        deployment_objects = []
        _deployment_objects = d.pop("deployment_objects")
        for deployment_objects_item_data in _deployment_objects:
            deployment_objects_item = DeploymentObjectHealthUpdate.from_dict(deployment_objects_item_data)

            deployment_objects.append(deployment_objects_item)

        health_status_update = cls(
            deployment_objects=deployment_objects,
        )

        health_status_update.additional_properties = d
        return health_status_update

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
