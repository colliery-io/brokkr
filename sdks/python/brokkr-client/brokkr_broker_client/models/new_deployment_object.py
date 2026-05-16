from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="NewDeploymentObject")


@_attrs_define
class NewDeploymentObject:
    """Represents a new deployment object to be inserted into the database.

    Attributes:
        is_deletion_marker (bool): Indicates if this object marks a deletion.
        stack_id (UUID): ID of the stack this deployment object belongs to.
        yaml_checksum (str): SHA-256 checksum of the YAML content.
        yaml_content (str): YAML content of the deployment.
    """

    is_deletion_marker: bool
    stack_id: UUID
    yaml_checksum: str
    yaml_content: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        is_deletion_marker = self.is_deletion_marker

        stack_id = str(self.stack_id)

        yaml_checksum = self.yaml_checksum

        yaml_content = self.yaml_content

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "is_deletion_marker": is_deletion_marker,
                "stack_id": stack_id,
                "yaml_checksum": yaml_checksum,
                "yaml_content": yaml_content,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        is_deletion_marker = d.pop("is_deletion_marker")

        stack_id = UUID(d.pop("stack_id"))

        yaml_checksum = d.pop("yaml_checksum")

        yaml_content = d.pop("yaml_content")

        new_deployment_object = cls(
            is_deletion_marker=is_deletion_marker,
            stack_id=stack_id,
            yaml_checksum=yaml_checksum,
            yaml_content=yaml_content,
        )

        new_deployment_object.additional_properties = d
        return new_deployment_object

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
