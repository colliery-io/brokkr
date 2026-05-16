from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="StackLabel")


@_attrs_define
class StackLabel:
    """Represents a stack label in the database.

    Attributes:
        id (UUID): Unique identifier for the stack label.
        label (str): The label text (max 64 characters, no whitespace).
        stack_id (UUID): ID of the stack this label is associated with.
    """

    id: UUID
    label: str
    stack_id: UUID
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        id = str(self.id)

        label = self.label

        stack_id = str(self.stack_id)

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "id": id,
                "label": label,
                "stack_id": stack_id,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        id = UUID(d.pop("id"))

        label = d.pop("label")

        stack_id = UUID(d.pop("stack_id"))

        stack_label = cls(
            id=id,
            label=label,
            stack_id=stack_id,
        )

        stack_label.additional_properties = d
        return stack_label

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
