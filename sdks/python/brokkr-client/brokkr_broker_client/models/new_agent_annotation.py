from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="NewAgentAnnotation")


@_attrs_define
class NewAgentAnnotation:
    """Represents a new agent annotation to be inserted into the database.

    Attributes:
        agent_id (UUID): ID of the agent this annotation belongs to.
        key (str): Key of the annotation (max 64 characters, no whitespace).
        value (str): Value of the annotation (max 64 characters, no whitespace).
    """

    agent_id: UUID
    key: str
    value: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        key = self.key

        value = self.value

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "key": key,
                "value": value,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        key = d.pop("key")

        value = d.pop("value")

        new_agent_annotation = cls(
            agent_id=agent_id,
            key=key,
            value=value,
        )

        new_agent_annotation.additional_properties = d
        return new_agent_annotation

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
