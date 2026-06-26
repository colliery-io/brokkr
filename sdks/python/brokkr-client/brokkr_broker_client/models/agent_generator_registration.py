from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

T = TypeVar("T", bound="AgentGeneratorRegistration")


@_attrs_define
class AgentGeneratorRegistration:
    """A registration linking an agent to a generator scope.

    Attributes:
        agent_id (UUID):
        generator_id (UUID):
        id (UUID):
        registered_at (datetime.datetime):
    """

    agent_id: UUID
    generator_id: UUID
    id: UUID
    registered_at: datetime.datetime
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        generator_id = str(self.generator_id)

        id = str(self.id)

        registered_at = self.registered_at.isoformat()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "generator_id": generator_id,
                "id": id,
                "registered_at": registered_at,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        generator_id = UUID(d.pop("generator_id"))

        id = UUID(d.pop("id"))

        registered_at = isoparse(d.pop("registered_at"))

        agent_generator_registration = cls(
            agent_id=agent_id,
            generator_id=generator_id,
            id=id,
            registered_at=registered_at,
        )

        agent_generator_registration.additional_properties = d
        return agent_generator_registration

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
