from __future__ import annotations

from collections.abc import Generator, Mapping
from typing import TYPE_CHECKING, Any, TypeVar

from attrs import define as _attrs_define
from attrs import field as _attrs_field

if TYPE_CHECKING:
    from ..models.generator import Generator


T = TypeVar("T", bound="CreateGeneratorResponse")


@_attrs_define
class CreateGeneratorResponse:
    """Response for a successful generator creation or PAK rotation.

    Attributes:
        generator (Generator): Represents a generator in the Brokkr system.
        pak (str): The Pre-Authentication Key for the generator.
    """

    generator: Generator
    pak: str
    additional_properties: dict[str, Any] = _attrs_field(init=False, factory=dict)

    def to_dict(self) -> dict[str, Any]:
        generator = self.generator.to_dict()

        pak = self.pak

        field_dict: dict[str, Any] = {}
        field_dict.update(self.additional_properties)
        field_dict.update(
            {
                "generator": generator,
                "pak": pak,
            }
        )

        return field_dict

    @classmethod
    def from_dict(cls: type[T], src_dict: Mapping[str, Any]) -> T:
        from ..models.generator import Generator

        d = dict(src_dict)
        generator = Generator.from_dict(d.pop("generator"))

        pak = d.pop("pak")

        create_generator_response = cls(
            generator=generator,
            pak=pak,
        )

        create_generator_response.additional_properties = d
        return create_generator_response

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
