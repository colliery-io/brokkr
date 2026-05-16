from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="DeploymentObject")


@_attrs_define
class DeploymentObject:
    """Represents a deployment object in the database.

    Attributes:
        created_at (datetime.datetime): Timestamp when the deployment object was created.
        id (UUID): Unique identifier for the deployment object.
        is_deletion_marker (bool): Indicates if this object marks a deletion.
        sequence_id (int): Auto-incrementing sequence number for ordering.
        stack_id (UUID): ID of the stack this deployment object belongs to.
        submitted_at (datetime.datetime): Timestamp when the deployment was submitted.
        updated_at (datetime.datetime): Timestamp when the deployment object was last updated.
        yaml_checksum (str): SHA-256 checksum of the YAML content.
        yaml_content (str): YAML content of the deployment.
        deleted_at (datetime.datetime | None | Unset): Timestamp for soft deletion, if applicable.
    """

    created_at: datetime.datetime
    id: UUID
    is_deletion_marker: bool
    sequence_id: int
    stack_id: UUID
    submitted_at: datetime.datetime
    updated_at: datetime.datetime
    yaml_checksum: str
    yaml_content: str
    deleted_at: datetime.datetime | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        id = str(self.id)

        is_deletion_marker = self.is_deletion_marker

        sequence_id = self.sequence_id

        stack_id = str(self.stack_id)

        submitted_at = self.submitted_at.isoformat()

        updated_at = self.updated_at.isoformat()

        yaml_checksum = self.yaml_checksum

        yaml_content = self.yaml_content

        deleted_at: None | str | Unset
        if isinstance(self.deleted_at, Unset):
            deleted_at = UNSET
        elif isinstance(self.deleted_at, datetime.datetime):
            deleted_at = self.deleted_at.isoformat()
        else:
            deleted_at = self.deleted_at

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "id": id,
                "is_deletion_marker": is_deletion_marker,
                "sequence_id": sequence_id,
                "stack_id": stack_id,
                "submitted_at": submitted_at,
                "updated_at": updated_at,
                "yaml_checksum": yaml_checksum,
                "yaml_content": yaml_content,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        is_deletion_marker = d.pop("is_deletion_marker")

        sequence_id = d.pop("sequence_id")

        stack_id = UUID(d.pop("stack_id"))

        submitted_at = isoparse(d.pop("submitted_at"))

        updated_at = isoparse(d.pop("updated_at"))

        yaml_checksum = d.pop("yaml_checksum")

        yaml_content = d.pop("yaml_content")

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

        deployment_object = cls(
            created_at=created_at,
            id=id,
            is_deletion_marker=is_deletion_marker,
            sequence_id=sequence_id,
            stack_id=stack_id,
            submitted_at=submitted_at,
            updated_at=updated_at,
            yaml_checksum=yaml_checksum,
            yaml_content=yaml_content,
            deleted_at=deleted_at,
        )

        deployment_object.additional_properties = d
        return deployment_object

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
