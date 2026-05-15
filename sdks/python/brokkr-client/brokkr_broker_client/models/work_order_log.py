from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="WorkOrderLog")


@_attrs_define
class WorkOrderLog:
    r"""Represents a completed work order in the audit log.

    Example:
        {'claimed_at': '2023-01-01T00:01:00Z', 'claimed_by': '123e4567-e89b-12d3-a456-426614174001', 'completed_at':
            '2023-01-01T00:05:00Z', 'created_at': '2023-01-01T00:00:00Z', 'id': '123e4567-e89b-12d3-a456-426614174000',
            'result_message': 'sha256:abc123...', 'retries_attempted': 0, 'success': True, 'work_type': 'build',
            'yaml_content': 'apiVersion: shipwright.io/v1beta1\nkind: Build\n...'}

    Attributes:
        completed_at (datetime.datetime): Timestamp when the work order completed. Example: 2023-01-01T00:05:00Z.
        created_at (datetime.datetime): Timestamp when the work order was created. Example: 2023-01-01T00:00:00Z.
        id (UUID): Original work order ID. Example: 123e4567-e89b-12d3-a456-426614174000.
        retries_attempted (int): Number of retry attempts before completion.
        success (bool): Whether the work completed successfully. Example: True.
        work_type (str): Type of work. Example: build.
        yaml_content (str): Original YAML content for debugging/reconstruction. Example: apiVersion:
            shipwright.io/v1beta1
            kind: Build
            ....
        claimed_at (datetime.datetime | None | Unset): Timestamp when the work order was claimed. Example:
            2023-01-01T00:01:00Z.
        claimed_by (None | Unset | UUID): ID of the agent that executed this work order. Example:
            123e4567-e89b-12d3-a456-426614174001.
        result_message (None | str | Unset): Result message (image digest on success, error details on failure).
            Example: sha256:abc123....
    """

    completed_at: datetime.datetime
    created_at: datetime.datetime
    id: UUID
    retries_attempted: int
    success: bool
    work_type: str
    yaml_content: str
    claimed_at: datetime.datetime | None | Unset = UNSET
    claimed_by: None | Unset | UUID = UNSET
    result_message: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        completed_at = self.completed_at.isoformat()

        created_at = self.created_at.isoformat()

        id = str(self.id)

        retries_attempted = self.retries_attempted

        success = self.success

        work_type = self.work_type

        yaml_content = self.yaml_content

        claimed_at: None | str | Unset
        if isinstance(self.claimed_at, Unset):
            claimed_at = UNSET
        elif isinstance(self.claimed_at, datetime.datetime):
            claimed_at = self.claimed_at.isoformat()
        else:
            claimed_at = self.claimed_at

        claimed_by: None | str | Unset
        if isinstance(self.claimed_by, Unset):
            claimed_by = UNSET
        elif isinstance(self.claimed_by, UUID):
            claimed_by = str(self.claimed_by)
        else:
            claimed_by = self.claimed_by

        result_message: None | str | Unset
        if isinstance(self.result_message, Unset):
            result_message = UNSET
        else:
            result_message = self.result_message

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "completed_at": completed_at,
                "created_at": created_at,
                "id": id,
                "retries_attempted": retries_attempted,
                "success": success,
                "work_type": work_type,
                "yaml_content": yaml_content,
            }
        )
        if claimed_at is not UNSET:
            field_dict["claimed_at"] = claimed_at
        if claimed_by is not UNSET:
            field_dict["claimed_by"] = claimed_by
        if result_message is not UNSET:
            field_dict["result_message"] = result_message

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        completed_at = isoparse(d.pop("completed_at"))

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        retries_attempted = d.pop("retries_attempted")

        success = d.pop("success")

        work_type = d.pop("work_type")

        yaml_content = d.pop("yaml_content")

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

        def _parse_claimed_by(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                claimed_by_type_0 = UUID(data)

                return claimed_by_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        claimed_by = _parse_claimed_by(d.pop("claimed_by", UNSET))

        def _parse_result_message(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        result_message = _parse_result_message(d.pop("result_message", UNSET))

        work_order_log = cls(
            completed_at=completed_at,
            created_at=created_at,
            id=id,
            retries_attempted=retries_attempted,
            success=success,
            work_type=work_type,
            yaml_content=yaml_content,
            claimed_at=claimed_at,
            claimed_by=claimed_by,
            result_message=result_message,
        )

        work_order_log.additional_properties = d
        return work_order_log

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
