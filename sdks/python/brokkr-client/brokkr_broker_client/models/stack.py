from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="Stack")


@_attrs_define
class Stack:
    """Represents a stack in the database.

    Attributes:
        created_at (datetime.datetime): Timestamp when the stack was created.
        generator_id (UUID): Optional generator ID.
        id (UUID): Unique identifier for the stack.
        name (str): Name of the stack.
        updated_at (datetime.datetime): Timestamp when the stack was last updated.
        deleted_at (datetime.datetime | None | Unset): Timestamp for soft deletion, if applicable.
        description (None | str | Unset): Optional description of the stack.
    """

    created_at: datetime.datetime
    generator_id: UUID
    id: UUID
    name: str
    updated_at: datetime.datetime
    deleted_at: datetime.datetime | None | Unset = UNSET
    description: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        generator_id = str(self.generator_id)

        id = str(self.id)

        name = self.name

        updated_at = self.updated_at.isoformat()

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

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "generator_id": generator_id,
                "id": id,
                "name": name,
                "updated_at": updated_at,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at
        if description is not UNSET:
            field_dict["description"] = description

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        generator_id = UUID(d.pop("generator_id"))

        id = UUID(d.pop("id"))

        name = d.pop("name")

        updated_at = isoparse(d.pop("updated_at"))

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

        stack = cls(
            created_at=created_at,
            generator_id=generator_id,
            id=id,
            name=name,
            updated_at=updated_at,
            deleted_at=deleted_at,
            description=description,
        )

        stack.additional_properties = d
        return stack

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
