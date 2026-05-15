from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

T = TypeVar("T", bound="TemplateAnnotation")


@_attrs_define
class TemplateAnnotation:
    """Represents a template annotation in the database.

    Attributes:
        created_at (datetime.datetime): Timestamp when the annotation was created.
        id (UUID): Unique identifier for the template annotation.
        key (str): The annotation key (max 64 characters, no whitespace).
        template_id (UUID): ID of the template this annotation is associated with.
        value (str): The annotation value (max 64 characters, no whitespace).
    """

    created_at: datetime.datetime
    id: UUID
    key: str
    template_id: UUID
    value: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        id = str(self.id)

        key = self.key

        template_id = str(self.template_id)

        value = self.value

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "id": id,
                "key": key,
                "template_id": template_id,
                "value": value,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        key = d.pop("key")

        template_id = UUID(d.pop("template_id"))

        value = d.pop("value")

        template_annotation = cls(
            created_at=created_at,
            id=id,
            key=key,
            template_id=template_id,
            value=value,
        )

        template_annotation.additional_properties = d
        return template_annotation

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
