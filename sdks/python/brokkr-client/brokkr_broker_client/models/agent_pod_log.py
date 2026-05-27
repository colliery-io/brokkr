from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

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
        stack_id (UUID):
        ts (datetime.datetime):
    """

    agent_id: UUID
    container: str
    created_at: datetime.datetime
    id: UUID
    line: str
    namespace: str
    pod: str
    stack_id: UUID
    ts: datetime.datetime
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        container = self.container

        created_at = self.created_at.isoformat()

        id = str(self.id)

        line = self.line

        namespace = self.namespace

        pod = self.pod

        stack_id = str(self.stack_id)

        ts = self.ts.isoformat()

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
                "stack_id": stack_id,
                "ts": ts,
            }
        )

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

        stack_id = UUID(d.pop("stack_id"))

        ts = isoparse(d.pop("ts"))

        agent_pod_log = cls(
            agent_id=agent_id,
            container=container,
            created_at=created_at,
            id=id,
            line=line,
            namespace=namespace,
            pod=pod,
            stack_id=stack_id,
            ts=ts,
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
