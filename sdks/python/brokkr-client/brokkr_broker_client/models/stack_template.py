from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="StackTemplate")


@_attrs_define
class StackTemplate:
    """Represents a stack template in the database.

    Attributes:
        checksum (str): SHA-256 checksum of template_content.
        created_at (datetime.datetime): Timestamp when the template was created.
        id (UUID): Unique identifier for the template.
        name (str): Name of the template.
        parameters_schema (str): JSON Schema for parameter validation.
        template_content (str): Tera template content.
        updated_at (datetime.datetime): Timestamp when the template was last updated.
        version (int): Version number (auto-incremented per name+generator_id).
        deleted_at (datetime.datetime | None | Unset): Timestamp for soft deletion, if applicable.
        description (None | str | Unset): Optional description of the template.
        generator_id (None | Unset | UUID): Generator ID - NULL for system templates (admin-only).
    """

    checksum: str
    created_at: datetime.datetime
    id: UUID
    name: str
    parameters_schema: str
    template_content: str
    updated_at: datetime.datetime
    version: int
    deleted_at: datetime.datetime | None | Unset = UNSET
    description: None | str | Unset = UNSET
    generator_id: None | Unset | UUID = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        checksum = self.checksum

        created_at = self.created_at.isoformat()

        id = str(self.id)

        name = self.name

        parameters_schema = self.parameters_schema

        template_content = self.template_content

        updated_at = self.updated_at.isoformat()

        version = self.version

        deleted_at: None | str | Unset
        if isinstance(self.deleted_at, Unset):
            deleted_at = UNSET
        elif isinstance(self.deleted_at, datetime.datetime):
            deleted_at = self.deleted_at.isoformat()
        else:
            deleted_at = self.deleted_at

        description: None | str | Unset
        if isinstance(self.description, Unset):
            description = UNSET
        else:
            description = self.description

        generator_id: None | str | Unset
        if isinstance(self.generator_id, Unset):
            generator_id = UNSET
        elif isinstance(self.generator_id, UUID):
            generator_id = str(self.generator_id)
        else:
            generator_id = self.generator_id

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "checksum": checksum,
                "created_at": created_at,
                "id": id,
                "name": name,
                "parameters_schema": parameters_schema,
                "template_content": template_content,
                "updated_at": updated_at,
                "version": version,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at
        if description is not UNSET:
            field_dict["description"] = description
        if generator_id is not UNSET:
            field_dict["generator_id"] = generator_id

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        checksum = d.pop("checksum")

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        name = d.pop("name")

        parameters_schema = d.pop("parameters_schema")

        template_content = d.pop("template_content")

        updated_at = isoparse(d.pop("updated_at"))

        version = d.pop("version")

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

        def _parse_description(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        description = _parse_description(d.pop("description", UNSET))

        def _parse_generator_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                generator_id_type_0 = UUID(data)

                return generator_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        generator_id = _parse_generator_id(d.pop("generator_id", UNSET))

        stack_template = cls(
            checksum=checksum,
            created_at=created_at,
            id=id,
            name=name,
            parameters_schema=parameters_schema,
            template_content=template_content,
            updated_at=updated_at,
            version=version,
            deleted_at=deleted_at,
            description=description,
            generator_id=generator_id,
        )

        stack_template.additional_properties = d
        return stack_template

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
