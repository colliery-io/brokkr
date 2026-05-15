from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="NewTemplateAnnotation")


@_attrs_define
class NewTemplateAnnotation:
    """Represents a new template annotation to be inserted into the database.

    Attributes:
        key (str): The annotation key (max 64 characters, no whitespace).
        template_id (UUID): ID of the template this annotation is associated with.
        value (str): The annotation value (max 64 characters, no whitespace).
    """

    key: str
    template_id: UUID
    value: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        key = self.key

        template_id = str(self.template_id)

        value = self.value

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "key": key,
                "template_id": template_id,
                "value": value,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        key = d.pop("key")

        template_id = UUID(d.pop("template_id"))

        value = d.pop("value")

        new_template_annotation = cls(
            key=key,
            template_id=template_id,
            value=value,
        )

        new_template_annotation.additional_properties = d
        return new_template_annotation

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
