from __future__ import annotations

import datetime
from collections.abc import Generator, Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="Generator")


@_attrs_define
class Generator:
    """Represents a generator in the Brokkr system.

    Attributes:
        created_at (datetime.datetime): Timestamp of when the generator was created.
        id (UUID): Unique identifier for the generator.
        is_active (bool): Indicates whether the generator is currently active.
        name (str): Name of the generator.
        updated_at (datetime.datetime): Timestamp of when the generator was last updated.
        deleted_at (datetime.datetime | None | Unset): Timestamp of when the generator was deleted, if applicable.
        description (None | str | Unset): Optional description of the generator.
        last_active_at (datetime.datetime | None | Unset): Timestamp of when the generator was last active.
    """

    created_at: datetime.datetime
    id: UUID
    is_active: bool
    name: str
    updated_at: datetime.datetime
    deleted_at: datetime.datetime | None | Unset = UNSET
    description: None | str | Unset = UNSET
    last_active_at: datetime.datetime | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        created_at = self.created_at.isoformat()

        id = str(self.id)

        is_active = self.is_active

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

        last_active_at: None | str | Unset
        if isinstance(self.last_active_at, Unset):
            last_active_at = UNSET
        elif isinstance(self.last_active_at, datetime.datetime):
            last_active_at = self.last_active_at.isoformat()
        else:
            last_active_at = self.last_active_at

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "created_at": created_at,
                "id": id,
                "is_active": is_active,
                "name": name,
                "updated_at": updated_at,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at
        if description is not UNSET:
            field_dict["description"] = description
        if last_active_at is not UNSET:
            field_dict["last_active_at"] = last_active_at

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        is_active = d.pop("is_active")

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

        def _parse_last_active_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_active_at_type_0 = isoparse(data)

                return last_active_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_active_at = _parse_last_active_at(d.pop("last_active_at", UNSET))

        generator = cls(
            created_at=created_at,
            id=id,
            is_active=is_active,
            name=name,
            updated_at=updated_at,
            deleted_at=deleted_at,
            description=description,
            last_active_at=last_active_at,
        )

        generator.additional_properties = d
        return generator

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
