from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.deployment_object_health_summary import DeploymentObjectHealthSummary


T = TypeVar("T", bound="StackHealthResponse")


@_attrs_define
class StackHealthResponse:
    """Response for stack health query.

    Attributes:
        deployment_objects (list[DeploymentObjectHealthSummary]): Health per deployment object.
        overall_status (str): Overall status for the stack.
        stack_id (UUID): The stack ID.
    """

    deployment_objects: list[DeploymentObjectHealthSummary]
    overall_status: str
    stack_id: UUID
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        deployment_objects = []
        for deployment_objects_item_data in self.deployment_objects:
            deployment_objects_item = deployment_objects_item_data.to_dict()
            deployment_objects.append(deployment_objects_item)

        overall_status = self.overall_status

        stack_id = str(self.stack_id)

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "deployment_objects": deployment_objects,
                "overall_status": overall_status,
                "stack_id": stack_id,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.deployment_object_health_summary import DeploymentObjectHealthSummary

        d = dict(src_dict)
        deployment_objects = []
        _deployment_objects = d.pop("deployment_objects")
        for deployment_objects_item_data in _deployment_objects:
            deployment_objects_item = DeploymentObjectHealthSummary.from_dict(deployment_objects_item_data)

            deployment_objects.append(deployment_objects_item)

        overall_status = d.pop("overall_status")

        stack_id = UUID(d.pop("stack_id"))

        stack_health_response = cls(
            deployment_objects=deployment_objects,
            overall_status=overall_status,
            stack_id=stack_id,
        )

        stack_health_response.additional_properties = d
        return stack_health_response

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
