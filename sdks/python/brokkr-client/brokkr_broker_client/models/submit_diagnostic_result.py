from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="SubmitDiagnosticResult")


@_attrs_define
class SubmitDiagnosticResult:
    """Request body for submitting diagnostic results.

    Attributes:
        collected_at (datetime.datetime): When the diagnostics were collected.
        events (str): JSON-encoded Kubernetes events.
        pod_statuses (str): JSON-encoded pod statuses.
        log_tails (None | str | Unset): JSON-encoded log tails (optional).
    """

    collected_at: datetime.datetime
    events: str
    pod_statuses: str
    log_tails: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        collected_at = self.collected_at.isoformat()

        events = self.events

        pod_statuses = self.pod_statuses

        log_tails: None | str | Unset
        if isinstance(self.log_tails, Unset):
            log_tails = UNSET
        else:
            log_tails = self.log_tails

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "collected_at": collected_at,
                "events": events,
                "pod_statuses": pod_statuses,
            }
        )
        if log_tails is not UNSET:
            field_dict["log_tails"] = log_tails

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        collected_at = isoparse(d.pop("collected_at"))

        events = d.pop("events")

        pod_statuses = d.pop("pod_statuses")

        def _parse_log_tails(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        log_tails = _parse_log_tails(d.pop("log_tails", UNSET))

        submit_diagnostic_result = cls(
            collected_at=collected_at,
            events=events,
            pod_statuses=pod_statuses,
            log_tails=log_tails,
        )

        submit_diagnostic_result.additional_properties = d
        return submit_diagnostic_result

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
