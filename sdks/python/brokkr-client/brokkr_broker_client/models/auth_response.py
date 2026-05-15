from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

T = TypeVar("T", bound="AuthResponse")


@_attrs_define
class AuthResponse:
    """Represents the response structure for authentication information.

    Attributes:
        admin (bool): Indicates if the authenticated entity is an admin.
        agent (None | str | Unset): The string representation of the agent's UUID, if applicable.
        generator (None | str | Unset): The string representation of the generator's UUID, if applicable.
    """

    admin: bool
    agent: None | str | Unset = UNSET
    generator: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        admin = self.admin

        agent: None | str | Unset
        if isinstance(self.agent, Unset):
            agent = UNSET
        else:
            agent = self.agent

        generator: None | str | Unset
        if isinstance(self.generator, Unset):
            generator = UNSET
        else:
            generator = self.generator

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "admin": admin,
            }
        )
        if agent is not UNSET:
            field_dict["agent"] = agent
        if generator is not UNSET:
            field_dict["generator"] = generator

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        admin = d.pop("admin")

        def _parse_agent(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        agent = _parse_agent(d.pop("agent", UNSET))

        def _parse_generator(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        generator = _parse_generator(d.pop("generator", UNSET))

        auth_response = cls(
            admin=admin,
            agent=agent,
            generator=generator,
        )

        auth_response.additional_properties = d
        return auth_response

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
