"""
Pytest fixtures for the Brokkr Python SDK contract test suite.

The suite exercises the generated `brokkr-client-generated` Python SDK
(module `brokkr_broker_client`) against a running broker. All HTTP goes
through the SDK's `AuthenticatedClient` + endpoint modules — no hand-rolled
requests/httpx calls in test code.
"""

from __future__ import annotations

import os
import time
import uuid

import httpx
import pytest

from brokkr_broker_client import AuthenticatedClient

DEFAULT_BROKER_URL = "http://localhost:3000"
DEFAULT_ADMIN_PAK = "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8"


@pytest.fixture(scope="session")
def broker_url() -> str:
    return os.environ.get("BROKER_URL", DEFAULT_BROKER_URL).rstrip("/")


@pytest.fixture(scope="session")
def base_url(broker_url: str) -> str:
    return f"{broker_url}/api/v1"


@pytest.fixture(scope="session")
def admin_pak() -> str:
    return os.environ.get("ADMIN_PAK", DEFAULT_ADMIN_PAK)


@pytest.fixture(scope="session", autouse=True)
def wait_for_broker(broker_url: str) -> None:
    """Block until `/healthz` returns 2xx, or fail fast after 30 s."""
    start = time.monotonic()
    last_err: BaseException | None = None
    while time.monotonic() - start < 30:
        try:
            r = httpx.get(f"{broker_url}/healthz", timeout=2.0)
            if r.is_success:
                return
        except Exception as exc:  # noqa: BLE001 - test bootstrap
            last_err = exc
        time.sleep(1)
    raise RuntimeError(f"broker not ready after 30s: {last_err!r}")


@pytest.fixture(scope="session")
def admin_client(base_url: str, admin_pak: str) -> AuthenticatedClient:
    # The Brokkr broker reads the raw PAK from the `Authorization` header
    # without stripping a `Bearer ` prefix (matches the Rust SDK wrapper).
    return AuthenticatedClient(base_url=base_url, token=admin_pak, prefix="")


def make_client(base_url: str, pak: str) -> AuthenticatedClient:
    """Build an AuthenticatedClient that sends `Authorization: <pak>` (no prefix)."""
    return AuthenticatedClient(base_url=base_url, token=pak, prefix="")


def unique(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:8]}"


DEMO_YAML = """apiVersion: v1
kind: Namespace
metadata:
  name: sdk-contract-python-ns
  labels:
    app: sdk-contract-python
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: sdk-contract-python-config
  namespace: sdk-contract-python-ns
data:
  KEY: "value"
"""
