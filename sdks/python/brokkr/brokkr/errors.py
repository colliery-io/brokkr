"""Typed error wrapper for the Brokkr SDK."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from brokkr_broker_client.models import ErrorResponse

# HTTP status codes worth retrying — transient failures only. Mirrors the
# Rust wrapper's `is_retryable_status` set.
_RETRYABLE_STATUSES: frozenset[int] = frozenset({408, 429, 502, 503, 504})


@dataclass
class BrokkrError(Exception):
    """Single exception type surfaced by the wrapper.

    Carries the typed `ErrorResponse` body when the broker returned a
    documented 4xx/5xx, and a status code when one is known. Callers
    pattern-match on `code` (machine-readable, stable across versions)
    rather than `message` (human-readable, not stable).
    """

    message: str
    code: Optional[str] = None
    status: Optional[int] = None
    response: Optional[ErrorResponse] = None

    def __post_init__(self) -> None:
        super().__init__(self.message)

    def __str__(self) -> str:  # pragma: no cover - dataclass repr is fine
        if self.status is not None and self.code is not None:
            return f"{self.status} {self.code}: {self.message}"
        if self.code is not None:
            return f"{self.code}: {self.message}"
        return self.message

    def is_retryable(self) -> bool:
        """Whether this error qualifies for the wrapper's retry helper."""
        if self.status is None:
            # Transport / unknown errors are retryable by default.
            return True
        return self.status in _RETRYABLE_STATUSES

    @classmethod
    def from_response(cls, body: ErrorResponse, status: int) -> "BrokkrError":
        """Build from a generated `ErrorResponse` returned in an operation
        union (rather than raised). The generator folds documented errors
        into the return type union; the wrapper converts them to raises."""
        return cls(
            message=body.message,
            code=body.code,
            status=status,
            response=body,
        )

    @classmethod
    def from_transport(cls, exc: BaseException) -> "BrokkrError":
        """Wrap an httpx / network exception."""
        return cls(message=str(exc), code=None, status=None, response=None)
