from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.agent_pod_log import AgentPodLog
    from ..models.retention_info import RetentionInfo


T = TypeVar("T", bound="PodLogHistoryResponse")


@_attrs_define
class PodLogHistoryResponse:
    """
    Attributes:
        lines (list[AgentPodLog]):
        retention (RetentionInfo):
    """

    lines: list[AgentPodLog]
    retention: RetentionInfo
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        lines = []
        for lines_item_data in self.lines:
            lines_item = lines_item_data.to_dict()
            lines.append(lines_item)

        retention = self.retention.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "lines": lines,
                "retention": retention,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.agent_pod_log import AgentPodLog
        from ..models.retention_info import RetentionInfo

        d = dict(src_dict)
        lines = []
        _lines = d.pop("lines")
        for lines_item_data in _lines:
            lines_item = AgentPodLog.from_dict(lines_item_data)

            lines.append(lines_item)

        retention = RetentionInfo.from_dict(d.pop("retention"))

        pod_log_history_response = cls(
            lines=lines,
            retention=retention,
        )

        pod_log_history_response.additional_properties = d
        return pod_log_history_response

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
