from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="CreateDiagnosticRequest")


@_attrs_define
class CreateDiagnosticRequest:
    """Request body for creating a diagnostic request.

    Attributes:
        agent_id (UUID): The agent that should handle this request.
        requested_by (None | str | Unset): Who is requesting the diagnostics (optional).
        retention_minutes (int | None | Unset): How long the request should be retained in minutes (default 60, max
            1440).
    """

    agent_id: UUID
    requested_by: None | str | Unset = UNSET
    retention_minutes: int | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        requested_by: None | str | Unset
        if isinstance(self.requested_by, Unset):
            requested_by = UNSET
        else:
            requested_by = self.requested_by

        retention_minutes: int | None | Unset
        if isinstance(self.retention_minutes, Unset):
            retention_minutes = UNSET
        else:
            retention_minutes = self.retention_minutes

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
            }
        )
        if requested_by is not UNSET:
            field_dict["requested_by"] = requested_by
        if retention_minutes is not UNSET:
            field_dict["retention_minutes"] = retention_minutes

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        def _parse_requested_by(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        requested_by = _parse_requested_by(d.pop("requested_by", UNSET))

        def _parse_retention_minutes(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        retention_minutes = _parse_retention_minutes(d.pop("retention_minutes", UNSET))

        create_diagnostic_request = cls(
            agent_id=agent_id,
            requested_by=requested_by,
            retention_minutes=retention_minutes,
        )

        create_diagnostic_request.additional_properties = d
        return create_diagnostic_request

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
