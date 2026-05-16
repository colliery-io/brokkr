"""
Surface tests for the generated Python SDK.

These do not talk to a real broker (integration tests against a running
service live with the ergonomic wrapper added by T-C2). The point of this
file is to assert that the generated package exposes the contract the wrapper
will build on — same role as `crates/brokkr-client/tests/surface.rs`.
"""

from __future__ import annotations

import inspect

from brokkr_broker_client import AuthenticatedClient, Client
from brokkr_broker_client.api.agents import (
    create_agent,
    get_agent,
    list_agents,
)
from brokkr_broker_client.api.auth import verify_pak
from brokkr_broker_client.api.health import update_health_status
from brokkr_broker_client.api.stacks import create_stack, list_stacks
from brokkr_broker_client.api.webhooks import (
    get_pending_agent_webhooks,
    list_webhooks,
)
from brokkr_broker_client.api.work_orders import (
    claim_work_order,
    complete_work_order,
    create_work_order,
    list_work_orders,
)
from brokkr_broker_client.models import (
    ErrorResponse,
)


def test_clients_construct() -> None:
    Client(base_url="http://localhost:3000/api/v1")
    AuthenticatedClient(base_url="http://localhost:3000/api/v1", token="pak_test")


def test_endpoints_expose_sync_and_async() -> None:
    for ep in (
        list_agents,
        create_agent,
        get_agent,
        list_stacks,
        create_stack,
        list_work_orders,
        create_work_order,
        claim_work_order,
        complete_work_order,
        verify_pak,
        update_health_status,
        list_webhooks,
        get_pending_agent_webhooks,
    ):
        assert hasattr(ep, "sync"), f"{ep.__name__} missing sync()"
        assert hasattr(ep, "asyncio"), f"{ep.__name__} missing asyncio()"


def test_error_response_shape() -> None:
    err = ErrorResponse(code="agent_not_found", message="agent not found")
    assert err.code == "agent_not_found"
    assert err.message == "agent not found"
    field_names = {f.name for f in ErrorResponse.__attrs_attrs__}
    assert {"code", "message", "details"}.issubset(field_names)


def test_list_agents_return_type_includes_error_response() -> None:
    # The generator should fold ErrorResponse into the return union so callers
    # can match on it without re-parsing the body. This is critical for the
    # ergonomic wrapper (T-C2) — it needs the typed error to flow through.
    sig = inspect.signature(list_agents.sync)
    return_annotation = str(sig.return_annotation)
    assert "ErrorResponse" in return_annotation
    assert "Agent" in return_annotation
