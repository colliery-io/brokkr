from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="WorkOrder")


@_attrs_define
class WorkOrder:
    r"""Represents an active work order in the queue.

    Example:
        {'backoff_seconds': 60, 'claim_timeout_seconds': 3600, 'claimed_at': None, 'claimed_by': None, 'created_at':
            '2023-01-01T00:00:00Z', 'id': '123e4567-e89b-12d3-a456-426614174000', 'last_error': None, 'last_error_at': None,
            'max_retries': 3, 'next_retry_after': None, 'retry_count': 0, 'status': 'PENDING', 'updated_at':
            '2023-01-01T00:00:00Z', 'work_type': 'build', 'yaml_content': 'apiVersion: shipwright.io/v1beta1\nkind:
            Build\n...'}

    Attributes:
        backoff_seconds (int): Base backoff seconds for exponential retry calculation. Example: 60.
        claim_timeout_seconds (int): Seconds before a claimed work order is considered stale. Example: 3600.
        created_at (datetime.datetime): Timestamp when the work order was created. Example: 2023-01-01T00:00:00Z.
        id (UUID): Unique identifier for the work order. Example: 123e4567-e89b-12d3-a456-426614174000.
        max_retries (int): Maximum number of retry attempts. Example: 3.
        retry_count (int): Current retry count.
        status (str): Queue status: PENDING, CLAIMED, or RETRY_PENDING. Example: PENDING.
        updated_at (datetime.datetime): Timestamp when the work order was last updated. Example: 2023-01-01T00:00:00Z.
        work_type (str): Type of work (e.g., "build", "test", "backup"). Example: build.
        yaml_content (str): Multi-document YAML content (e.g., Build + WorkOrder definitions). Example: apiVersion:
            shipwright.io/v1beta1
            kind: Build
            ....
        claimed_at (datetime.datetime | None | Unset): Timestamp when the work order was claimed. Example: null.
        claimed_by (None | Unset | UUID): ID of the agent that claimed this work order (if any). Example: null.
        last_error (None | str | Unset): Most recent error message from failed execution attempt. Example: null.
        last_error_at (datetime.datetime | None | Unset): Timestamp of the most recent failure. Example: null.
        next_retry_after (datetime.datetime | None | Unset): Timestamp when RETRY_PENDING work order becomes PENDING
            again. Example: null.
    """

    backoff_seconds: int
    claim_timeout_seconds: int
    created_at: datetime.datetime
    id: UUID
    max_retries: int
    retry_count: int
    status: str
    updated_at: datetime.datetime
    work_type: str
    yaml_content: str
    claimed_at: datetime.datetime | None | Unset = UNSET
    claimed_by: None | Unset | UUID = UNSET
    last_error: None | str | Unset = UNSET
    last_error_at: datetime.datetime | None | Unset = UNSET
    next_retry_after: datetime.datetime | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        backoff_seconds = self.backoff_seconds

        claim_timeout_seconds = self.claim_timeout_seconds

        created_at = self.created_at.isoformat()

        id = str(self.id)

        max_retries = self.max_retries

        retry_count = self.retry_count

        status = self.status

        updated_at = self.updated_at.isoformat()

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

        last_error: None | str | Unset
        if isinstance(self.last_error, Unset):
            last_error = UNSET
        else:
            last_error = self.last_error

        last_error_at: None | str | Unset
        if isinstance(self.last_error_at, Unset):
            last_error_at = UNSET
        elif isinstance(self.last_error_at, datetime.datetime):
            last_error_at = self.last_error_at.isoformat()
        else:
            last_error_at = self.last_error_at

        next_retry_after: None | str | Unset
        if isinstance(self.next_retry_after, Unset):
            next_retry_after = UNSET
        elif isinstance(self.next_retry_after, datetime.datetime):
            next_retry_after = self.next_retry_after.isoformat()
        else:
            next_retry_after = self.next_retry_after

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "backoff_seconds": backoff_seconds,
                "claim_timeout_seconds": claim_timeout_seconds,
                "created_at": created_at,
                "id": id,
                "max_retries": max_retries,
                "retry_count": retry_count,
                "status": status,
                "updated_at": updated_at,
                "work_type": work_type,
                "yaml_content": yaml_content,
            }
        )
        if claimed_at is not UNSET:
            field_dict["claimed_at"] = claimed_at
        if claimed_by is not UNSET:
            field_dict["claimed_by"] = claimed_by
        if last_error is not UNSET:
            field_dict["last_error"] = last_error
        if last_error_at is not UNSET:
            field_dict["last_error_at"] = last_error_at
        if next_retry_after is not UNSET:
            field_dict["next_retry_after"] = next_retry_after

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        backoff_seconds = d.pop("backoff_seconds")

        claim_timeout_seconds = d.pop("claim_timeout_seconds")

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        max_retries = d.pop("max_retries")

        retry_count = d.pop("retry_count")

        status = d.pop("status")

        updated_at = isoparse(d.pop("updated_at"))

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

        def _parse_last_error(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        last_error = _parse_last_error(d.pop("last_error", UNSET))

        def _parse_last_error_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_error_at_type_0 = isoparse(data)

                return last_error_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_error_at = _parse_last_error_at(d.pop("last_error_at", UNSET))

        def _parse_next_retry_after(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                next_retry_after_type_0 = isoparse(data)

                return next_retry_after_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        next_retry_after = _parse_next_retry_after(d.pop("next_retry_after", UNSET))

        work_order = cls(
            backoff_seconds=backoff_seconds,
            claim_timeout_seconds=claim_timeout_seconds,
            created_at=created_at,
            id=id,
            max_retries=max_retries,
            retry_count=retry_count,
            status=status,
            updated_at=updated_at,
            work_type=work_type,
            yaml_content=yaml_content,
            claimed_at=claimed_at,
            claimed_by=claimed_by,
            last_error=last_error,
            last_error_at=last_error_at,
            next_retry_after=next_retry_after,
        )

        work_order.additional_properties = d
        return work_order

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
