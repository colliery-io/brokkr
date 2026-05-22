from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="CreateDeploymentObjectRequest")


@_attrs_define
class CreateDeploymentObjectRequest:
    """Wire DTO for creating a deployment object via the public API.

    Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
    which carries server-derived fields (e.g. `yaml_checksum`).

        Attributes:
            yaml_content (str): YAML content of the deployment.
            is_deletion_marker (bool | Unset): Optional. Defaults to false.
    """

    yaml_content: str
    is_deletion_marker: bool | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        yaml_content = self.yaml_content

        is_deletion_marker = self.is_deletion_marker

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "yaml_content": yaml_content,
            }
        )
        if is_deletion_marker is not UNSET:
            field_dict["is_deletion_marker"] = is_deletion_marker

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        yaml_content = d.pop("yaml_content")

        is_deletion_marker = d.pop("is_deletion_marker", UNSET)

        create_deployment_object_request = cls(
            yaml_content=yaml_content,
            is_deletion_marker=is_deletion_marker,
        )

        create_deployment_object_request.additional_properties = d
        return create_deployment_object_request

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
