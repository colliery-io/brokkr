from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar, cast

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.error_response_details_type_0 import ErrorResponseDetailsType0


T = TypeVar("T", bound="ErrorResponse")


@_attrs_define
class ErrorResponse:
    """Wire format for every 4xx/5xx response body in the v1 API.

    Attributes:
        code (str): Stable, machine-readable error code (e.g. `agent_not_found`).
        message (str): Human-readable message; not stable and may change between versions.
        details (ErrorResponseDetailsType0 | None | Unset): Optional structured context. Keys and value shapes are
            documented per
            error code when present.
    """

    code: str
    message: str
    details: ErrorResponseDetailsType0 | None | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        from ..models.error_response_details_type_0 import ErrorResponseDetailsType0

        code = self.code

        message = self.message

        details: dict[str, Any] | None | Unset
        if isinstance(self.details, Unset):
            details = UNSET
        elif isinstance(self.details, ErrorResponseDetailsType0):
            details = self.details.to_dict()
        else:
            details = self.details

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "code": code,
                "message": message,
            }
        )
        if details is not UNSET:
            field_dict["details"] = details

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.error_response_details_type_0 import ErrorResponseDetailsType0

        d = dict(src_dict)
        code = d.pop("code")

        message = d.pop("message")

        def _parse_details(data: object) -> ErrorResponseDetailsType0 | None | Unset:
            if data is None:
                return data
            if isinstance(data, Unset):
                return data
            try:
                if not isinstance(data, dict):
                    raise TypeError()
                details_type_0 = ErrorResponseDetailsType0.from_dict(data)

                return details_type_0
            except (TypeError, ValueError, AttributeError, KeyError):
                pass
            return cast(ErrorResponseDetailsType0 | None | Unset, data)

        details = _parse_details(d.pop("details", UNSET))

        error_response = cls(
            code=code,
            message=message,
            details=details,
        )

        error_response.additional_properties = d
        return error_response

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
