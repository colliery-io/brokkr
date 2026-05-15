from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="Agent")


@_attrs_define
class Agent:
    """Represents an agent in the database.

    Attributes:
        cluster_name (str): Name of the cluster the agent belongs to.
        created_at (datetime.datetime): Timestamp when the agent was created.
        id (UUID): Unique identifier for the agent.
        name (str): Name of the agent.
        status (str): Current status of the agent.
        updated_at (datetime.datetime): Timestamp when the agent was last updated.
        deleted_at (datetime.datetime | None | Unset): Timestamp for soft deletion, if applicable.
        last_heartbeat (datetime.datetime | None | Unset): Timestamp of the last heartbeat received from the agent.
    """

    cluster_name: str
    created_at: datetime.datetime
    id: UUID
    name: str
    status: str
    updated_at: datetime.datetime
    deleted_at: datetime.datetime | None | Unset = UNSET
    last_heartbeat: datetime.datetime | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        cluster_name = self.cluster_name

        created_at = self.created_at.isoformat()

        id = str(self.id)

        name = self.name

        status = self.status

        updated_at = self.updated_at.isoformat()

        deleted_at: None | str | Unset
        if isinstance(self.deleted_at, Unset):
            deleted_at = UNSET
        elif isinstance(self.deleted_at, datetime.datetime):
            deleted_at = self.deleted_at.isoformat()
        else:
            deleted_at = self.deleted_at

        last_heartbeat: None | str | Unset
        if isinstance(self.last_heartbeat, Unset):
            last_heartbeat = UNSET
        elif isinstance(self.last_heartbeat, datetime.datetime):
            last_heartbeat = self.last_heartbeat.isoformat()
        else:
            last_heartbeat = self.last_heartbeat

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "cluster_name": cluster_name,
                "created_at": created_at,
                "id": id,
                "name": name,
                "status": status,
                "updated_at": updated_at,
            }
        )
        if deleted_at is not UNSET:
            field_dict["deleted_at"] = deleted_at
        if last_heartbeat is not UNSET:
            field_dict["last_heartbeat"] = last_heartbeat

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        cluster_name = d.pop("cluster_name")

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        name = d.pop("name")

        status = d.pop("status")

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

        def _parse_last_heartbeat(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_heartbeat_type_0 = isoparse(data)

                return last_heartbeat_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_heartbeat = _parse_last_heartbeat(d.pop("last_heartbeat", UNSET))

        agent = cls(
            cluster_name=cluster_name,
            created_at=created_at,
            id=id,
            name=name,
            status=status,
            updated_at=updated_at,
            deleted_at=deleted_at,
            last_heartbeat=last_heartbeat,
        )

        agent.additional_properties = d
        return agent

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
