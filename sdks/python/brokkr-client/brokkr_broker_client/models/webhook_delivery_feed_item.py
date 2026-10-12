from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="WebhookDeliveryFeedItem")


@_attrs_define
class WebhookDeliveryFeedItem:
    """One row of the broker-wide delivery feed (BROKKR-T-0264).

    Like [`WebhookResponse`], it never carries the subscription's encrypted
    URL or auth header. It also leaves out the event payload: the feed is for
    triage, and the per-subscription list keeps the full record.

        Attributes:
            attempts (int):
            created_at (datetime.datetime):
            event_id (UUID):
            event_type (str):
            id (UUID):
            status (str): pending, acquired, success, failed or dead.
            subscription_id (UUID):
            target_labels (list[str]): Labels that select the agent that delivers; empty means the broker.
            acquired_by (None | Unset | UUID): Agent that holds the delivery now (`null` = broker or not held).
            completed_at (datetime.datetime | None | Unset):
            last_attempt_at (datetime.datetime | None | Unset):
            last_error (None | str | Unset):
            next_retry_at (datetime.datetime | None | Unset):
            subscription_name (None | str | Unset): Name of the subscription, or `null` when the subscription is gone.
    """

    attempts: int
    created_at: datetime.datetime
    event_id: UUID
    event_type: str
    id: UUID
    status: str
    subscription_id: UUID
    target_labels: list[str]
    acquired_by: None | Unset | UUID = UNSET
    completed_at: datetime.datetime | None | Unset = UNSET
    last_attempt_at: datetime.datetime | None | Unset = UNSET
    last_error: None | str | Unset = UNSET
    next_retry_at: datetime.datetime | None | Unset = UNSET
    subscription_name: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        attempts = self.attempts

        created_at = self.created_at.isoformat()

        event_id = str(self.event_id)

        event_type = self.event_type

        id = str(self.id)

        status = self.status

        subscription_id = str(self.subscription_id)

        target_labels = self.target_labels

        acquired_by: None | str | Unset
        if isinstance(self.acquired_by, Unset):
            acquired_by = UNSET
        elif isinstance(self.acquired_by, UUID):
            acquired_by = str(self.acquired_by)
        else:
            acquired_by = self.acquired_by

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

        subscription_name: None | str | Unset
        if isinstance(self.subscription_name, Unset):
            subscription_name = UNSET
        else:
            subscription_name = self.subscription_name

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "attempts": attempts,
                "created_at": created_at,
                "event_id": event_id,
                "event_type": event_type,
                "id": id,
                "status": status,
                "subscription_id": subscription_id,
                "target_labels": target_labels,
            }
        )
        if acquired_by is not UNSET:
            field_dict["acquired_by"] = acquired_by
        if completed_at is not UNSET:
            field_dict["completed_at"] = completed_at
        if last_attempt_at is not UNSET:
            field_dict["last_attempt_at"] = last_attempt_at
        if last_error is not UNSET:
            field_dict["last_error"] = last_error
        if next_retry_at is not UNSET:
            field_dict["next_retry_at"] = next_retry_at
        if subscription_name is not UNSET:
            field_dict["subscription_name"] = subscription_name

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        attempts = d.pop("attempts")

        created_at = isoparse(d.pop("created_at"))

        event_id = UUID(d.pop("event_id"))

        event_type = d.pop("event_type")

        id = UUID(d.pop("id"))

        status = d.pop("status")

        subscription_id = UUID(d.pop("subscription_id"))

        target_labels = cast(list[str], d.pop("target_labels"))

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

        def _parse_subscription_name(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        subscription_name = _parse_subscription_name(d.pop("subscription_name", UNSET))

        webhook_delivery_feed_item = cls(
            attempts=attempts,
            created_at=created_at,
            event_id=event_id,
            event_type=event_type,
            id=id,
            status=status,
            subscription_id=subscription_id,
            target_labels=target_labels,
            acquired_by=acquired_by,
            completed_at=completed_at,
            last_attempt_at=last_attempt_at,
            last_error=last_error,
            next_retry_at=next_retry_at,
            subscription_name=subscription_name,
        )

        webhook_delivery_feed_item.additional_properties = d
        return webhook_delivery_feed_item

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
