from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="AgentTarget")


@_attrs_define
class AgentTarget:
    """Represents an agent target in the database.

    Attributes:
        agent_id (UUID): ID of the agent associated with this target.
        id (UUID): Unique identifier for the agent target.
        stack_id (UUID): ID of the stack associated with this target.
    """

    agent_id: UUID
    id: UUID
    stack_id: UUID
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        id = str(self.id)

        stack_id = str(self.stack_id)

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "id": id,
                "stack_id": stack_id,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        id = UUID(d.pop("id"))

        stack_id = UUID(d.pop("stack_id"))

        agent_target = cls(
            agent_id=agent_id,
            id=id,
            stack_id=stack_id,
        )

        agent_target.additional_properties = d
        return agent_target

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
