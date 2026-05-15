from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.work_order_targeting_annotations_type_0 import WorkOrderTargetingAnnotationsType0


T = TypeVar("T", bound="WorkOrderTargeting")


@_attrs_define
class WorkOrderTargeting:
    """
    Attributes:
        agent_ids (list[UUID] | None | Unset):
        annotations (None | Unset | WorkOrderTargetingAnnotationsType0):
        labels (list[str] | None | Unset):
    """

    agent_ids: list[UUID] | None | Unset = UNSET
    annotations: None | Unset | WorkOrderTargetingAnnotationsType0 = UNSET
    labels: list[str] | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        from ..models.work_order_targeting_annotations_type_0 import WorkOrderTargetingAnnotationsType0

        agent_ids: list[str] | None | Unset
        if isinstance(self.agent_ids, Unset):
            agent_ids = UNSET
        elif isinstance(self.agent_ids, list):
            agent_ids = []
            for agent_ids_type_0_item_data in self.agent_ids:
                agent_ids_type_0_item = str(agent_ids_type_0_item_data)
                agent_ids.append(agent_ids_type_0_item)

        else:
            agent_ids = self.agent_ids

        annotations: dict[str, Any] | None | Unset
        if isinstance(self.annotations, Unset):
            annotations = UNSET
        elif isinstance(self.annotations, WorkOrderTargetingAnnotationsType0):
            annotations = self.annotations.to_dict()
        else:
            annotations = self.annotations

        labels: list[str] | None | Unset
        if isinstance(self.labels, Unset):
            labels = UNSET
        elif isinstance(self.labels, list):
            labels = self.labels

        else:
            labels = self.labels

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update({})
        if agent_ids is not UNSET:
            field_dict["agent_ids"] = agent_ids
        if annotations is not UNSET:
            field_dict["annotations"] = annotations
        if labels is not UNSET:
            field_dict["labels"] = labels

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.work_order_targeting_annotations_type_0 import WorkOrderTargetingAnnotationsType0

        d = dict(src_dict)

        def _parse_agent_ids(data: object) -> list[UUID] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                agent_ids_type_0 = []
                _agent_ids_type_0 = data
                for agent_ids_type_0_item_data in _agent_ids_type_0:
                    agent_ids_type_0_item = UUID(agent_ids_type_0_item_data)

                    agent_ids_type_0.append(agent_ids_type_0_item)

                return agent_ids_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[UUID] | None | Unset, data)

        agent_ids = _parse_agent_ids(d.pop("agent_ids", UNSET))

        def _parse_annotations(data: object) -> None | Unset | WorkOrderTargetingAnnotationsType0:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, dict):
                    raise TypeError()
                annotations_type_0 = WorkOrderTargetingAnnotationsType0.from_dict(data)

                return annotations_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | WorkOrderTargetingAnnotationsType0, data)

        annotations = _parse_annotations(d.pop("annotations", UNSET))

        def _parse_labels(data: object) -> list[str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                labels_type_0 = cast(list[str], data)

                return labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[str] | None | Unset, data)

        labels = _parse_labels(d.pop("labels", UNSET))

        work_order_targeting = cls(
            agent_ids=agent_ids,
            annotations=annotations,
            labels=labels,
        )

        work_order_targeting.additional_properties = d
        return work_order_targeting

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
