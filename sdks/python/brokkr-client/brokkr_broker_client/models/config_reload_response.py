from __future__ import annotations

import datetime
from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field
from dateutil.parser import isoparse

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.config_change_info import ConfigChangeInfo


T = TypeVar("T", bound="ConfigReloadResponse")


@_attrs_define
class ConfigReloadResponse:
    """Response structure for configuration reload operations.

    Attributes:
        changes (list[ConfigChangeInfo]): List of configuration changes detected during reload.
        reloaded_at (datetime.datetime): Timestamp when the configuration was reloaded.
        success (bool): Indicates whether the reload was successful.
        message (None | str | Unset): Optional message providing additional context.
    """

    changes: list[ConfigChangeInfo]
    reloaded_at: datetime.datetime
    success: bool
    message: None | str | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        changes = []
        for changes_item_data in self.changes:
            changes_item = changes_item_data.to_dict()
            changes.append(changes_item)

        reloaded_at = self.reloaded_at.isoformat()

        success = self.success

        message: None | str | Unset
        if isinstance(self.message, Unset):
            message = UNSET
        else:
            message = self.message

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "changes": changes,
                "reloaded_at": reloaded_at,
                "success": success,
            }
        )
        if message is not UNSET:
            field_dict["message"] = message

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.config_change_info import ConfigChangeInfo

        d = dict(src_dict)
        changes = []
        _changes = d.pop("changes")
        for changes_item_data in _changes:
            changes_item = ConfigChangeInfo.from_dict(changes_item_data)

            changes.append(changes_item)

        reloaded_at = isoparse(d.pop("reloaded_at"))

        success = d.pop("success")

        def _parse_message(data: object) -> None | str | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            return cast(None | str | Unset, data)

        message = _parse_message(d.pop("message", UNSET))

        config_reload_response = cls(
            changes=changes,
            reloaded_at=reloaded_at,
            success=success,
            message=message,
        )

        config_reload_response.additional_properties = d
        return config_reload_response

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
