from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="WebhookFilters")


@_attrs_define
class WebhookFilters:
    """Filters for webhook subscriptions.

    A filter narrows an event-type match further, using field values carried in
    the event payload. Evaluated by [`WebhookFilters::matches`] at emission time
    (see `brokkr_broker::utils::event_bus::emit_event`).

    Semantics (BROKKR-T-0288):

    * Every filter field that is set must match: the fields are ANDed.
    * **An event that does not carry a filtered field does not match.** A
      subscription filtering on `stack_id` therefore receives nothing for event
      types whose payload has no `stack_id` (`agent.*` and `workorder.*`; the
      whole `deployment.*` family does carry it). This is deliberate: a filter
      is a narrowing
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

        Attributes:
            agent_id (None | Unset | UUID): Filter by specific agent ID.
            stack_id (None | Unset | UUID): Filter by specific stack ID.
    """

    agent_id: None | Unset | UUID = UNSET
    stack_id: None | Unset | UUID = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id: None | str | Unset
        if isinstance(self.agent_id, Unset):
            agent_id = UNSET
        elif isinstance(self.agent_id, UUID):
            agent_id = str(self.agent_id)
        else:
            agent_id = self.agent_id

        stack_id: None | str | Unset
        if isinstance(self.stack_id, Unset):
            stack_id = UNSET
        elif isinstance(self.stack_id, UUID):
            stack_id = str(self.stack_id)
        else:
            stack_id = self.stack_id

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update({})
        if agent_id is not UNSET:
            field_dict["agent_id"] = agent_id
        if stack_id is not UNSET:
            field_dict["stack_id"] = stack_id

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)

        def _parse_agent_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                agent_id_type_0 = UUID(data)

                return agent_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        agent_id = _parse_agent_id(d.pop("agent_id", UNSET))

        def _parse_stack_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                stack_id_type_0 = UUID(data)

                return stack_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        stack_id = _parse_stack_id(d.pop("stack_id", UNSET))

        webhook_filters = cls(
            agent_id=agent_id,
            stack_id=stack_id,
        )

        webhook_filters.additional_properties = d
        return webhook_filters

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
