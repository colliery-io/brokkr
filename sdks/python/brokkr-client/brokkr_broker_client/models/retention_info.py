from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="RetentionInfo")


@_attrs_define
class RetentionInfo:
    """
    Attributes:
        effective_retention_seconds (int): Effective retention window for the stack. <= ceiling.
        long_term_sink_hint (str): Recommended sink for long-term centralisation. Brokkr is NOT a
            log warehouse — see project_log_retention_stance.
        retention_ceiling_seconds (int): Hard upper bound on retention. Never exceeds 21600 (6h).
        oldest_available_ts (datetime.datetime | None | Unset): Server-side timestamp of the oldest row currently
            retained for
            this stack, or null when no rows exist in the window.
    """

    effective_retention_seconds: int
    long_term_sink_hint: str
    retention_ceiling_seconds: int
    oldest_available_ts: datetime.datetime | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        effective_retention_seconds = self.effective_retention_seconds

        long_term_sink_hint = self.long_term_sink_hint

        retention_ceiling_seconds = self.retention_ceiling_seconds

        oldest_available_ts: None | str | Unset
        if isinstance(self.oldest_available_ts, Unset):
            oldest_available_ts = UNSET
        elif isinstance(self.oldest_available_ts, datetime.datetime):
            oldest_available_ts = self.oldest_available_ts.isoformat()
        else:
            oldest_available_ts = self.oldest_available_ts

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "effective_retention_seconds": effective_retention_seconds,
                "long_term_sink_hint": long_term_sink_hint,
                "retention_ceiling_seconds": retention_ceiling_seconds,
            }
        )
        if oldest_available_ts is not UNSET:
            field_dict["oldest_available_ts"] = oldest_available_ts

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        effective_retention_seconds = d.pop("effective_retention_seconds")

        long_term_sink_hint = d.pop("long_term_sink_hint")

        retention_ceiling_seconds = d.pop("retention_ceiling_seconds")

        def _parse_oldest_available_ts(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                oldest_available_ts_type_0 = isoparse(data)

                return oldest_available_ts_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        oldest_available_ts = _parse_oldest_available_ts(d.pop("oldest_available_ts", UNSET))

        retention_info = cls(
            effective_retention_seconds=effective_retention_seconds,
            long_term_sink_hint=long_term_sink_hint,
            retention_ceiling_seconds=retention_ceiling_seconds,
            oldest_available_ts=oldest_available_ts,
        )

        retention_info.additional_properties = d
        return retention_info

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
