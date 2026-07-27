from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.webhook_filters import WebhookFilters


T = TypeVar("T", bound="CreateWebhookRequest")


@_attrs_define
class CreateWebhookRequest:
    """Body of `POST /webhooks`.

    Unknown keys are ignored rather than rejected, with two deliberate
    exceptions that this endpoint used to accept and silently ignore
    (BROKKR-T-0288). Both are now **rejected with 422** by
    [`reject_removed_write_fields`] before this struct is deserialized:

    * `validate` ("send test request on creation") was documented and parsed but
      never read by `create_webhook` — it did nothing, ever. The real mechanism
      is `POST /webhooks/{id}/test`.
    * `filters.labels` was stored and echoed but never evaluated; label-based
      routing is `target_labels`, which is real.

    Rejecting is the lesser evil. A caller who sends `filters.labels` believes
    their deliveries are scoped; accepting the request and dropping the key
    leaves them with a subscription that fires on everything and a response they
    have no reason to re-read. A 422 naming the field and its replacement is
    noisy exactly once, at the moment the operator can still fix it.

    This is a **write-path** rule only. Subscription rows already stored with a
    `labels` key keep loading and keep delivering — see [`WebhookFilters`],
    whose deserialization stays tolerant of unknown keys.

        Attributes:
            event_types (list[str]):
            name (str):
            url (str):
            auth_header (None | str | Unset):
            filters (WebhookFilters | Unset): Filters for webhook subscriptions.

                A filter narrows an event-type match further, using field values carried in
                the event payload. Evaluated by [`WebhookFilters::matches`] at emission time
                (see `brokkr_broker::utils::event_bus::emit_event`).

                Semantics (BROKKR-T-0288):

                * Every filter field that is set must match: the fields are ANDed.
                * **An event that does not carry a filtered field does not match.** A
                  subscription filtering on `stack_id` therefore receives nothing for event
                  types whose payload has no `stack_id` (`agent.*`, `workorder.*`, and
                  `deployment.applied`/`deployment.failed`, which carry only
                  `deployment_object_id`). This is deliberate: a filter is a narrowing
                  statement, and widening it to "…or the event didn't say" would deliver
                  precisely the events the operator asked to exclude.
                * A JSON `null` counts as absent, not as a value. Several payloads emit
                  `"stack_id": null` / `"agent_id": null` when the source column is NULL
                  (e.g. `deployment.deleted` for an already-purged object,
                  `workorder.completed` for an unclaimed order).

                # Read/write asymmetry

                This type is the **read** path: deserialization deliberately ignores unknown
                keys, so rows written before a field was removed still load — notably the
                dropped `labels` filter, which was never evaluated and is superseded by
                `target_labels` delivery routing. A legacy row must keep delivering exactly
                as it does today; a broker that refused to load it would silently stop the
                subscription instead. **Do not add `deny_unknown_fields` here.**

                The **write** path is strict, and intentionally not symmetric: `POST` and
                `PUT /webhooks` reject a body carrying `filters.labels` with a 422 that names
                `target_labels` as the replacement (see
                `brokkr_broker::api::v1::webhooks::reject_removed_write_fields`). Accepting
                the key and dropping it would leave the caller believing their deliveries
                were scoped when they were not — the failure mode that motivated
                BROKKR-T-0288 in the first place. Rejection is enforced on the raw request
                body, above this type, precisely so the tolerant deserialization below is
                preserved.
            max_retries (int | None | Unset):
            target_labels (list[str] | None | Unset):
            timeout_seconds (int | None | Unset):
    """

    event_types: list[str]
    name: str
    url: str
    auth_header: None | str | Unset = UNSET
    filters: WebhookFilters | Unset = UNSET
    max_retries: int | None | Unset = UNSET
    target_labels: list[str] | None | Unset = UNSET
    timeout_seconds: int | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        event_types = self.event_types

        name = self.name

        url = self.url

        auth_header: None | str | Unset
        if isinstance(self.auth_header, Unset):
            auth_header = UNSET
        else:
            auth_header = self.auth_header

        filters: dict[str, Any] | Unset = UNSET
        if not isinstance(self.filters, Unset):
            filters = self.filters.to_dict()

        max_retries: int | None | Unset
        if isinstance(self.max_retries, Unset):
            max_retries = UNSET
        else:
            max_retries = self.max_retries

        target_labels: list[str] | None | Unset
        if isinstance(self.target_labels, Unset):
            target_labels = UNSET
        elif isinstance(self.target_labels, list):
            target_labels = self.target_labels

        else:
            target_labels = self.target_labels

        timeout_seconds: int | None | Unset
        if isinstance(self.timeout_seconds, Unset):
            timeout_seconds = UNSET
        else:
            timeout_seconds = self.timeout_seconds

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "event_types": event_types,
                "name": name,
                "url": url,
            }
        )
        if auth_header is not UNSET:
            field_dict["auth_header"] = auth_header
        if filters is not UNSET:
            field_dict["filters"] = filters
        if max_retries is not UNSET:
            field_dict["max_retries"] = max_retries
        if target_labels is not UNSET:
            field_dict["target_labels"] = target_labels
        if timeout_seconds is not UNSET:
            field_dict["timeout_seconds"] = timeout_seconds

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.webhook_filters import WebhookFilters

        d = dict(src_dict)
        event_types = cast(list[str], d.pop("event_types"))

        name = d.pop("name")

        url = d.pop("url")

        def _parse_auth_header(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        auth_header = _parse_auth_header(d.pop("auth_header", UNSET))

        _filters = d.pop("filters", UNSET)
        filters: WebhookFilters | Unset
        if isinstance(_filters, Unset):
            filters = UNSET
        else:
            filters = WebhookFilters.from_dict(_filters)

        def _parse_max_retries(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        max_retries = _parse_max_retries(d.pop("max_retries", UNSET))

        def _parse_target_labels(data: object) -> list[str] | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, list):
                    raise TypeError()
                target_labels_type_0 = cast(list[str], data)

                return target_labels_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(list[str] | None | Unset, data)

        target_labels = _parse_target_labels(d.pop("target_labels", UNSET))

        def _parse_timeout_seconds(data: object) -> int | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(int | None | Unset, data)

        timeout_seconds = _parse_timeout_seconds(d.pop("timeout_seconds", UNSET))

        create_webhook_request = cls(
            event_types=event_types,
            name=name,
            url=url,
            auth_header=auth_header,
            filters=filters,
            max_retries=max_retries,
            target_labels=target_labels,
            timeout_seconds=timeout_seconds,
        )

        create_webhook_request.additional_properties = d
        return create_webhook_request

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
