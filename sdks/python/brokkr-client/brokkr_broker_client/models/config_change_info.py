from __future__ import annotations

from collections.abc import Mapping
from typing import Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

T = TypeVar("T", bound="ConfigChangeInfo")


@_attrs_define
class ConfigChangeInfo:
    """Information about a single configuration change.

    Attributes:
        key (str): The configuration key that changed.
        new_value (str): The new value (as a string representation).
        old_value (str): The previous value (as a string representation).
    """

    key: str
    new_value: str
    old_value: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        key = self.key

        new_value = self.new_value

        old_value = self.old_value

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "key": key,
                "new_value": new_value,
                "old_value": old_value,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        d = dict(src_dict)
        key = d.pop("key")

        new_value = d.pop("new_value")

        old_value = d.pop("old_value")

        config_change_info = cls(
            key=key,
            new_value=new_value,
            old_value=old_value,
        )

        config_change_info.additional_properties = d
        return config_change_info

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
