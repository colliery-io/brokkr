from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import Any, TypeVar
from uuid import UUID

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

T = TypeVar("T", bound="WsConnectionInfo")


@_attrs_define
class WsConnectionInfo:
    """
    Attributes:
        agent_id (UUID):
        connected_since (datetime.datetime):
        messages_in (int):
        messages_out (int):
    """

    agent_id: UUID
    connected_since: datetime.datetime
    messages_in: int
    messages_out: int
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        agent_id = str(self.agent_id)

        connected_since = self.connected_since.isoformat()

        messages_in = self.messages_in

        messages_out = self.messages_out

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "agent_id": agent_id,
                "connected_since": connected_since,
                "messages_in": messages_in,
                "messages_out": messages_out,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        agent_id = UUID(d.pop("agent_id"))

        connected_since = isoparse(d.pop("connected_since"))

        messages_in = d.pop("messages_in")

        messages_out = d.pop("messages_out")

        ws_connection_info = cls(
            agent_id=agent_id,
            connected_since=connected_since,
            messages_in=messages_in,
            messages_out=messages_out,
        )

        ws_connection_info.additional_properties = d
        return ws_connection_info

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
