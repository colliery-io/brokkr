from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="WebhookDelivery")


@_attrs_define
class WebhookDelivery:
    """A webhook delivery record from the database.

    Attributes:
        attempts (int): Number of delivery attempts.
        created_at (datetime.datetime): When the delivery was created.
        event_id (UUID): The event ID (idempotency key).
        event_type (str): The event type being delivered.
        id (UUID): Unique identifier for the delivery.
        payload (str): JSON-encoded event payload.
        status (str): Delivery status: pending, acquired, success, failed, dead.
        subscription_id (UUID): The subscription this delivery belongs to.
        acquired_by (None | Unset | UUID): Agent ID that acquired this delivery (NULL = broker).
        acquired_until (datetime.datetime | None | Unset): TTL for the acquisition - release if exceeded.
        completed_at (datetime.datetime | None | Unset): When the delivery completed (success or dead).
        last_attempt_at (datetime.datetime | None | Unset): When the last delivery attempt was made.
        last_error (None | str | Unset): Error message from last failed attempt.
        next_retry_at (datetime.datetime | None | Unset): When to retry after failure.
        target_labels (list[None | str] | None | Unset): Labels for delivery targeting (copied from subscription).
    """

    attempts: int
    created_at: datetime.datetime
    event_id: UUID
    event_type: str
    id: UUID
    payload: str
    status: str
    subscription_id: UUID
    acquired_by: None | Unset | UUID = UNSET
    acquired_until: datetime.datetime | None | Unset = UNSET
    completed_at: datetime.datetime | None | Unset = UNSET
    last_attempt_at: datetime.datetime | None | Unset = UNSET
    last_error: None | str | Unset = UNSET
    next_retry_at: datetime.datetime | None | Unset = UNSET
    target_labels: list[None | str] | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        attempts = self.attempts

        created_at = self.created_at.isoformat()

        event_id = str(self.event_id)

        event_type = self.event_type

        id = str(self.id)

        payload = self.payload

        status = self.status

        subscription_id = str(self.subscription_id)

        acquired_by: None | str | Unset
        if isinstance(self.acquired_by, Unset):
            acquired_by = UNSET
        elif isinstance(self.acquired_by, UUID):
            acquired_by = str(self.acquired_by)
        else:
            acquired_by = self.acquired_by

        acquired_until: None | str | Unset
        if isinstance(self.acquired_until, Unset):
            acquired_until = UNSET
        elif isinstance(self.acquired_until, datetime.datetime):
            acquired_until = self.acquired_until.isoformat()
        else:
            acquired_until = self.acquired_until

        completed_at: None | str | Unset
        if isinstance(self.completed_at, Unset):
            completed_at = UNSET
        elif isinstance(self.completed_at, datetime.datetime):
            completed_at = self.completed_at.isoformat()
        else:
            completed_at = self.completed_at

        last_attempt_at: None | str | Unset
        if isinstance(self.last_attempt_at, Unset):
            last_attempt_at = UNSET
        elif isinstance(self.last_attempt_at, datetime.datetime):
            last_attempt_at = self.last_attempt_at.isoformat()
        else:
            last_attempt_at = self.last_attempt_at

        last_error: None | str | Unset
        if isinstance(self.last_error, Unset):
            last_error = UNSET
        else:
            last_error = self.last_error

        next_retry_at: None | str | Unset
        if isinstance(self.next_retry_at, Unset):
            next_retry_at = UNSET
        elif isinstance(self.next_retry_at, datetime.datetime):
            next_retry_at = self.next_retry_at.isoformat()
        else:
            next_retry_at = self.next_retry_at

        target_labels: list[None | str] | None | Unset
        if isinstance(self.target_labels, Unset):
            target_labels = UNSET
        elif isinstance(self.target_labels, list):
            target_labels = []
            for target_labels_type_0_item_data in self.target_labels:
                target_labels_type_0_item: None | str
                target_labels_type_0_item = target_labels_type_0_item_data
                target_labels.append(target_labels_type_0_item)

        else:
            target_labels = self.target_labels

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "attempts": attempts,
                "created_at": created_at,
                "event_id": event_id,
                "event_type": event_type,
                "id": id,
                "payload": payload,
                "status": status,
                "subscription_id": subscription_id,
            }
        )
        if acquired_by is not UNSET:
            field_dict["acquired_by"] = acquired_by
        if acquired_until is not UNSET:
            field_dict["acquired_until"] = acquired_until
        if completed_at is not UNSET:
            field_dict["completed_at"] = completed_at
        if last_attempt_at is not UNSET:
            field_dict["last_attempt_at"] = last_attempt_at
        if last_error is not UNSET:
            field_dict["last_error"] = last_error
        if next_retry_at is not UNSET:
            field_dict["next_retry_at"] = next_retry_at
        if target_labels is not UNSET:
            field_dict["target_labels"] = target_labels

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        attempts = d.pop("attempts")

        created_at = isoparse(d.pop("created_at"))

        event_id = UUID(d.pop("event_id"))

        event_type = d.pop("event_type")

        id = UUID(d.pop("id"))

        payload = d.pop("payload")

        status = d.pop("status")

        subscription_id = UUID(d.pop("subscription_id"))

        def _parse_acquired_by(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                acquired_by_type_0 = UUID(data)

                return acquired_by_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        acquired_by = _parse_acquired_by(d.pop("acquired_by", UNSET))

        def _parse_acquired_until(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                acquired_until_type_0 = isoparse(data)

                return acquired_until_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        acquired_until = _parse_acquired_until(d.pop("acquired_until", UNSET))

        def _parse_completed_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                completed_at_type_0 = isoparse(data)

                return completed_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        completed_at = _parse_completed_at(d.pop("completed_at", UNSET))

        def _parse_last_attempt_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                last_attempt_at_type_0 = isoparse(data)

                return last_attempt_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        last_attempt_at = _parse_last_attempt_at(d.pop("last_attempt_at", UNSET))

        def _parse_last_error(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        last_error = _parse_last_error(d.pop("last_error", UNSET))

        def _parse_next_retry_at(data: object) -> datetime.datetime | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                next_retry_at_type_0 = isoparse(data)

                return next_retry_at_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(datetime.datetime | None | Unset, data)

        next_retry_at = _parse_next_retry_at(d.pop("next_retry_at", UNSET))

        def _parse_target_labels(data: object) -> list[None | str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                target_labels_type_0 = []
                _target_labels_type_0 = data
                for target_labels_type_0_item_data in _target_labels_type_0:

                    def _parse_target_labels_type_0_item(data: object) -> None | str:
                        if data is None:
                            return data
                        return cast(None | str, data)

                    target_labels_type_0_item = _parse_target_labels_type_0_item(target_labels_type_0_item_data)

                    target_labels_type_0.append(target_labels_type_0_item)

                return target_labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[None | str] | None | Unset, data)

        target_labels = _parse_target_labels(d.pop("target_labels", UNSET))

        webhook_delivery = cls(
            attempts=attempts,
            created_at=created_at,
            event_id=event_id,
            event_type=event_type,
            id=id,
            payload=payload,
            status=status,
            subscription_id=subscription_id,
            acquired_by=acquired_by,
            acquired_until=acquired_until,
            completed_at=completed_at,
            last_attempt_at=last_attempt_at,
            last_error=last_error,
            next_retry_at=next_retry_at,
            target_labels=target_labels,
        )

        webhook_delivery.additional_properties = d
        return webhook_delivery

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
