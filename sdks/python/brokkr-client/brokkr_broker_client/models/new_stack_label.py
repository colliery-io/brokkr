from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="NewStackLabel")


@_attrs_define
class NewStackLabel:
    """Represents a new stack label to be inserted into the database.

    Attributes:
        label (str): The label text (max 64 characters, no whitespace).
        stack_id (UUID): ID of the stack this label is associated with.
    """

    label: str
    stack_id: UUID
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        label = self.label

        stack_id = str(self.stack_id)

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "label": label,
                "stack_id": stack_id,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        label = d.pop("label")

        stack_id = UUID(d.pop("stack_id"))

        new_stack_label = cls(
            label=label,
            stack_id=stack_id,
        )

        new_stack_label.additional_properties = d
        return new_stack_label

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
