from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.resource_health import ResourceHealth


T = TypeVar("T", bound="HealthSummary")


@_attrs_define
class HealthSummary:
    """Structured health summary for serialization/deserialization.

    Attributes:
        conditions (list[str]): List of detected problematic conditions (e.g., ImagePullBackOff).
        pods_ready (int): Number of pods in ready state.
        pods_total (int): Total number of pods.
        resources (list[ResourceHealth] | None | Unset): Optional detailed resource status.
    """

    conditions: list[str]
    pods_ready: int
    pods_total: int
    resources: list[ResourceHealth] | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        conditions = self.conditions

        pods_ready = self.pods_ready

        pods_total = self.pods_total

        resources: list[dict[str, Any]] | None | Unset
        if isinstance(self.resources, Unset):
            resources = UNSET
        elif isinstance(self.resources, list):
            resources = []
            for resources_type_0_item_data in self.resources:
                resources_type_0_item = resources_type_0_item_data.to_dict()
                resources.append(resources_type_0_item)

        else:
            resources = self.resources

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "conditions": conditions,
                "pods_ready": pods_ready,
                "pods_total": pods_total,
            }
        )
        if resources is not UNSET:
            field_dict["resources"] = resources

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.resource_health import ResourceHealth

        d = dict(src_dict)
        conditions = cast(list[str], d.pop("conditions"))

        pods_ready = d.pop("pods_ready")

        pods_total = d.pop("pods_total")

        def _parse_resources(data: object) -> list[ResourceHealth] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                resources_type_0 = []
                _resources_type_0 = data
                for resources_type_0_item_data in _resources_type_0:
                    resources_type_0_item = ResourceHealth.from_dict(resources_type_0_item_data)

                    resources_type_0.append(resources_type_0_item)

                return resources_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[ResourceHealth] | None | Unset, data)

        resources = _parse_resources(d.pop("resources", UNSET))

        health_summary = cls(
            conditions=conditions,
            pods_ready=pods_ready,
            pods_total=pods_total,
            resources=resources,
        )

        health_summary.additional_properties = d
        return health_summary

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
