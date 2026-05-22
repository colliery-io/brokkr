from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.agent import Agent


T = TypeVar("T", bound="CreateAgentResponse")


@_attrs_define
class CreateAgentResponse:
    """Response body for [`create_agent`]: the newly-created agent plus the
    one-time initial PAK shown only at creation.

        Attributes:
            agent (Agent): Represents an agent in the database.
            initial_pak (str):
    """

    agent: Agent
    initial_pak: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent = self.agent.to_dict()

        initial_pak = self.initial_pak

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent": agent,
                "initial_pak": initial_pak,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.agent import Agent

        d = dict(src_dict)
        agent = Agent.from_dict(d.pop("agent"))

        initial_pak = d.pop("initial_pak")

        create_agent_response = cls(
            agent=agent,
            initial_pak=initial_pak,
        )

        create_agent_response.additional_properties = d
        return create_agent_response

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
