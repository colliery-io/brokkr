from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="DiagnosticRequest")


@_attrs_define
class DiagnosticRequest:
    """A diagnostic request record from the database.

    Attributes:
        agent_id (UUID): The agent that should handle this request.
        created_at (datetime.datetime): When the request was created.
        deployment_object_id (UUID): The deployment object to gather diagnostics for.
        expires_at (datetime.datetime): When the request expires and should be cleaned up.
        id (UUID): Unique identifier for the diagnostic request.
        status (str): Status: pending, claimed, completed, failed, expired.
        claimed_at (datetime.datetime | None | Unset): When the agent claimed the request.
        completed_at (datetime.datetime | None | Unset): When the request was completed.
        requested_by (None | str | Unset): Who requested the diagnostics (e.g., operator username).
    """

    agent_id: UUID
    created_at: datetime.datetime
    deployment_object_id: UUID
    expires_at: datetime.datetime
    id: UUID
    status: str
    claimed_at: datetime.datetime | None | Unset = UNSET
    completed_at: datetime.datetime | None | Unset = UNSET
    requested_by: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        created_at = self.created_at.isoformat()

        deployment_object_id = str(self.deployment_object_id)

        expires_at = self.expires_at.isoformat()

        id = str(self.id)

        status = self.status

        claimed_at: None | str | Unset
        if isinstance(self.claimed_at, Unset):
            claimed_at = UNSET
        elif isinstance(self.claimed_at, datetime.datetime):
            claimed_at = self.claimed_at.isoformat()
        else:
            claimed_at = self.claimed_at

        completed_at: None | str | Unset
        if isinstance(self.completed_at, Unset):
            completed_at = UNSET
        elif isinstance(self.completed_at, datetime.datetime):
            completed_at = self.completed_at.isoformat()
        else:
            completed_at = self.completed_at

        requested_by: None | str | Unset
        if isinstance(self.requested_by, Unset):
            requested_by = UNSET
        else:
            requested_by = self.requested_by

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "created_at": created_at,
                "deployment_object_id": deployment_object_id,
                "expires_at": expires_at,
                "id": id,
                "status": status,
            }
        )
        if claimed_at is not UNSET:
            field_dict["claimed_at"] = claimed_at
        if completed_at is not UNSET:
            field_dict["completed_at"] = completed_at
        if requested_by is not UNSET:
            field_dict["requested_by"] = requested_by

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        created_at = isoparse(d.pop("created_at"))

        deployment_object_id = UUID(d.pop("deployment_object_id"))

        expires_at = isoparse(d.pop("expires_at"))

        id = UUID(d.pop("id"))

        status = d.pop("status")

        def _parse_claimed_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                claimed_at_type_0 = isoparse(data)

                return claimed_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        claimed_at = _parse_claimed_at(d.pop("claimed_at", UNSET))

        def _parse_completed_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                completed_at_type_0 = isoparse(data)

                return completed_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        completed_at = _parse_completed_at(d.pop("completed_at", UNSET))

        def _parse_requested_by(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        requested_by = _parse_requested_by(d.pop("requested_by", UNSET))

        diagnostic_request = cls(
            agent_id=agent_id,
            created_at=created_at,
            deployment_object_id=deployment_object_id,
            expires_at=expires_at,
            id=id,
            status=status,
            claimed_at=claimed_at,
            completed_at=completed_at,
            requested_by=requested_by,
        )

        diagnostic_request.additional_properties = d
        return diagnostic_request

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
