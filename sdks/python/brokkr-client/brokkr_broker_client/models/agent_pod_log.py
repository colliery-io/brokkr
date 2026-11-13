from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar, cast
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

T = TypeVar("T", bound="AgentPodLog")


@_attrs_define
class AgentPodLog:
    """
    Attributes:
        agent_id (UUID):
        container (str):
        created_at (datetime.datetime):
        id (UUID):
        line (str):
        namespace (str):
        pod (str):
        ts (datetime.datetime):
        stack_id (None | Unset | UUID): The stack the row belongs to. No value: it belongs to a work order.
        work_order_id (None | Unset | UUID): The work order the row belongs to. No value: it belongs to a stack.
            Exactly one of `stack_id` and `work_order_id` has a value (BROKKR-T-0405).
    """

    agent_id: UUID
    container: str
    created_at: datetime.datetime
    id: UUID
    line: str
    namespace: str
    pod: str
    ts: datetime.datetime
    stack_id: None | Unset | UUID = UNSET
    work_order_id: None | Unset | UUID = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        container = self.container

        created_at = self.created_at.isoformat()

        id = str(self.id)

        line = self.line

        namespace = self.namespace

        pod = self.pod

        ts = self.ts.isoformat()

        stack_id: None | str | Unset
        if isinstance(self.stack_id, Unset):
            stack_id = UNSET
        elif isinstance(self.stack_id, UUID):
            stack_id = str(self.stack_id)
        else:
            stack_id = self.stack_id

        work_order_id: None | str | Unset
        if isinstance(self.work_order_id, Unset):
            work_order_id = UNSET
        elif isinstance(self.work_order_id, UUID):
            work_order_id = str(self.work_order_id)
        else:
            work_order_id = self.work_order_id

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "container": container,
                "created_at": created_at,
                "id": id,
                "line": line,
                "namespace": namespace,
                "pod": pod,
                "ts": ts,
            }
        )
        if stack_id is not UNSET:
            field_dict["stack_id"] = stack_id
        if work_order_id is not UNSET:
            field_dict["work_order_id"] = work_order_id

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        container = d.pop("container")

        created_at = isoparse(d.pop("created_at"))

        id = UUID(d.pop("id"))

        line = d.pop("line")

        namespace = d.pop("namespace")

        pod = d.pop("pod")

        ts = isoparse(d.pop("ts"))

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

        def _parse_work_order_id(data: object) -> None | Unset | UUID:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, str):
                    raise TypeError()
                work_order_id_type_0 = UUID(data)

                return work_order_id_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(None | Unset | UUID, data)

        work_order_id = _parse_work_order_id(d.pop("work_order_id", UNSET))

        agent_pod_log = cls(
            agent_id=agent_id,
            container=container,
            created_at=created_at,
            id=id,
            line=line,
            namespace=namespace,
            pod=pod,
            ts=ts,
            stack_id=stack_id,
            work_order_id=work_order_id,
        )

        agent_pod_log.additional_properties = d
        return agent_pod_log

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
