from __future__ import annotations

from collections.abc import Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

from ..types import UNSET, Unset

if TYPE_CHECKING:
    from ..models.diagnostic_request import DiagnosticRequest
    from ..models.diagnostic_result import DiagnosticResult


T = TypeVar("T", bound="DiagnosticResponse")


@_attrs_define
class DiagnosticResponse:
    """Response containing a diagnostic request with optional result.

    Attributes:
        request (DiagnosticRequest): A diagnostic request record from the database.
        result (DiagnosticResult | Unset): A diagnostic result record from the database.
    """

    request: DiagnosticRequest
    result: DiagnosticResult | Unset = UNSET
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        request = self.request.to_dict()

        result: dict[str, Any] | Unset = UNSET
        if not isinstance(self.result, Unset):
            result = self.result.to_dict()

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "request": request,
            }
        )
        if result is not UNSET:
            field_dict["result"] = result

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.diagnostic_request import DiagnosticRequest
        from ..models.diagnostic_result import DiagnosticResult

        d = dict(src_dict)
        request = DiagnosticRequest.from_dict(d.pop("request"))

        _result = d.pop("result", UNSET)
        result: DiagnosticResult | Unset
        if isinstance(_result, Unset):
            result = UNSET
        else:
            result = DiagnosticResult.from_dict(_result)

        diagnostic_response = cls(
            request=request,
            result=result,
        )

        diagnostic_response.additional_properties = d
        return diagnostic_response

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
