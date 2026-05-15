from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.health_summary import HealthSummary


T = TypeVar("T", bound="DeploymentObjectHealthUpdate")


@_attrs_define
class DeploymentObjectHealthUpdate:
    """Health update for a single deployment object.

    Attributes:
        checked_at (datetime.datetime): When the health was checked.
        id (UUID): The deployment object ID.
        status (str): Health status: healthy, degraded, failing, or unknown.
        summary (HealthSummary | Unset): Structured health summary for serialization/deserialization.
    """

    checked_at: datetime.datetime
    id: UUID
    status: str
    summary: HealthSummary | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        checked_at = self.checked_at.isoformat()

        id = str(self.id)

        status = self.status

        summary: dict[str, Any] | Unset = UNSET
        if not isinstance(self.summary, Unset):
            summary = self.summary.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "checked_at": checked_at,
                "id": id,
                "status": status,
            }
        )
        if summary is not UNSET:
            field_dict["summary"] = summary

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.health_summary import HealthSummary

        d = dict(src_dict)
        checked_at = isoparse(d.pop("checked_at"))

        id = UUID(d.pop("id"))

        status = d.pop("status")

        _summary = d.pop("summary", UNSET)
        summary: HealthSummary | Unset
        if isinstance(_summary, Unset):
            summary = UNSET
        else:
            summary = HealthSummary.from_dict(_summary)

        deployment_object_health_update = cls(
            checked_at=checked_at,
            id=id,
            status=status,
            summary=summary,
        )

        deployment_object_health_update.additional_properties = d
        return deployment_object_health_update

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
