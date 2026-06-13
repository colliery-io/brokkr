from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="HeartbeatReport")


@_attrs_define
class HeartbeatReport:
    """Optional heartbeat report body (BROKKR-T-0227).

    A plain heartbeat carries no body; agents that probe their own Kubernetes
    API attach this to self-report reachability. Both fields are optional so a
    body may carry only what the agent could measure, and the entire body may
    be omitted (legacy/no-body heartbeats still work).

        Attributes:
            k8s_api_latency_ms (int | None | Unset): Measured latency (milliseconds) of the reachability probe, if any.
            k8s_reachable (bool | None | Unset): Whether the agent can reach its own Kubernetes API.
    """

    k8s_api_latency_ms: int | None | Unset = UNSET
    k8s_reachable: bool | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        k8s_api_latency_ms: int | None | Unset
        if isinstance(self.k8s_api_latency_ms, Unset):
            k8s_api_latency_ms = UNSET
        else:
            k8s_api_latency_ms = self.k8s_api_latency_ms

        k8s_reachable: bool | None | Unset
        if isinstance(self.k8s_reachable, Unset):
            k8s_reachable = UNSET
        else:
            k8s_reachable = self.k8s_reachable

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update({})
        if k8s_api_latency_ms is not UNSET:
            field_dict["k8s_api_latency_ms"] = k8s_api_latency_ms
        if k8s_reachable is not UNSET:
            field_dict["k8s_reachable"] = k8s_reachable

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)

        def _parse_k8s_api_latency_ms(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        k8s_api_latency_ms = _parse_k8s_api_latency_ms(d.pop("k8s_api_latency_ms", UNSET))

        def _parse_k8s_reachable(data: object) -> bool | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(bool | None | Unset, data)

        k8s_reachable = _parse_k8s_reachable(d.pop("k8s_reachable", UNSET))

        heartbeat_report = cls(
            k8s_api_latency_ms=k8s_api_latency_ms,
            k8s_reachable=k8s_reachable,
        )

        heartbeat_report.additional_properties = d
        return heartbeat_report

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
