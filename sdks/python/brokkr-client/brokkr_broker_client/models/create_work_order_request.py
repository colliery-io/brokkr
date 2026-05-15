from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.work_order_targeting import WorkOrderTargeting


T = TypeVar("T", bound="CreateWorkOrderRequest")


@_attrs_define
class CreateWorkOrderRequest:
    """
    Attributes:
        work_type (str):
        yaml_content (str):
        backoff_seconds (int | None | Unset):
        claim_timeout_seconds (int | None | Unset):
        max_retries (int | None | Unset):
        target_agent_ids (list[UUID] | None | Unset):
        targeting (WorkOrderTargeting | Unset):
    """

    work_type: str
    yaml_content: str
    backoff_seconds: int | None | Unset = UNSET
    claim_timeout_seconds: int | None | Unset = UNSET
    max_retries: int | None | Unset = UNSET
    target_agent_ids: list[UUID] | None | Unset = UNSET
    targeting: WorkOrderTargeting | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        work_type = self.work_type

        yaml_content = self.yaml_content

        backoff_seconds: int | None | Unset
        if isinstance(self.backoff_seconds, Unset):
            backoff_seconds = UNSET
        else:
            backoff_seconds = self.backoff_seconds

        claim_timeout_seconds: int | None | Unset
        if isinstance(self.claim_timeout_seconds, Unset):
            claim_timeout_seconds = UNSET
        else:
            claim_timeout_seconds = self.claim_timeout_seconds

        max_retries: int | None | Unset
        if isinstance(self.max_retries, Unset):
            max_retries = UNSET
        else:
            max_retries = self.max_retries

        target_agent_ids: list[str] | None | Unset
        if isinstance(self.target_agent_ids, Unset):
            target_agent_ids = UNSET
        elif isinstance(self.target_agent_ids, list):
            target_agent_ids = []
            for target_agent_ids_type_0_item_data in self.target_agent_ids:
                target_agent_ids_type_0_item = str(target_agent_ids_type_0_item_data)
                target_agent_ids.append(target_agent_ids_type_0_item)

        else:
            target_agent_ids = self.target_agent_ids

        targeting: dict[str, Any] | Unset = UNSET
        if not isinstance(self.targeting, Unset):
            targeting = self.targeting.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "work_type": work_type,
                "yaml_content": yaml_content,
            }
        )
        if backoff_seconds is not UNSET:
            field_dict["backoff_seconds"] = backoff_seconds
        if claim_timeout_seconds is not UNSET:
            field_dict["claim_timeout_seconds"] = claim_timeout_seconds
        if max_retries is not UNSET:
            field_dict["max_retries"] = max_retries
        if target_agent_ids is not UNSET:
            field_dict["target_agent_ids"] = target_agent_ids
        if targeting is not UNSET:
            field_dict["targeting"] = targeting

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.work_order_targeting import WorkOrderTargeting

        d = dict(src_dict)
        work_type = d.pop("work_type")

        yaml_content = d.pop("yaml_content")

        def _parse_backoff_seconds(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        backoff_seconds = _parse_backoff_seconds(d.pop("backoff_seconds", UNSET))

        def _parse_claim_timeout_seconds(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        claim_timeout_seconds = _parse_claim_timeout_seconds(d.pop("claim_timeout_seconds", UNSET))

        def _parse_max_retries(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        max_retries = _parse_max_retries(d.pop("max_retries", UNSET))

        def _parse_target_agent_ids(data: object) -> list[UUID] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                target_agent_ids_type_0 = []
                _target_agent_ids_type_0 = data
                for target_agent_ids_type_0_item_data in _target_agent_ids_type_0:
                    target_agent_ids_type_0_item = UUID(target_agent_ids_type_0_item_data)

                    target_agent_ids_type_0.append(target_agent_ids_type_0_item)

                return target_agent_ids_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[UUID] | None | Unset, data)

        target_agent_ids = _parse_target_agent_ids(d.pop("target_agent_ids", UNSET))

        _targeting = d.pop("targeting", UNSET)
        targeting: WorkOrderTargeting | Unset
        if isinstance(_targeting, Unset):
            targeting = UNSET
        else:
            targeting = WorkOrderTargeting.from_dict(_targeting)

        create_work_order_request = cls(
            work_type=work_type,
            yaml_content=yaml_content,
            backoff_seconds=backoff_seconds,
            claim_timeout_seconds=claim_timeout_seconds,
            max_retries=max_retries,
            target_agent_ids=target_agent_ids,
            targeting=targeting,
        )

        create_work_order_request.additional_properties = d
        return create_work_order_request

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
