"""Surface tests for the hand-written ergonomic helpers in
``brokkr_broker_client.helpers``.

We don't talk to a real broker here; the contract under test is that
the helpers exist with the expected signatures and that
``live_subscription_url`` builds the right URL. End-to-end coverage
against a running broker lives in
``crates/brokkr-broker/tests/integration/api/ws.rs``.
"""

from __future__ import annotations

import inspect
from uuid import UUID

from brokkr_broker_client import helpers


def test_helpers_module_exposes_expected_surface() -> None:
    expected = {
        "list_telemetry_events",
        "list_telemetry_events_async",
        "list_telemetry_logs",
        "list_telemetry_logs_async",
        "list_ws_connections",
        "list_ws_connections_async",
        "live_subscription_url",
    }
    assert set(helpers.__all__) == expected
    for name in expected:
        assert hasattr(helpers, name), f"helpers.{name} missing"


def test_live_subscription_url_http_to_ws() -> None:
    stack = UUID("11111111-1111-1111-1111-111111111111")
    assert (
        helpers.live_subscription_url("http://broker.test:3000", stack)
        == f"ws://broker.test:3000/api/v1/stacks/{stack}/live"
    )


def test_live_subscription_url_https_to_wss() -> None:
    stack = UUID("22222222-2222-2222-2222-222222222222")
    assert (
        helpers.live_subscription_url("https://broker.example.com", stack)
        == f"wss://broker.example.com/api/v1/stacks/{stack}/live"
    )


def test_live_subscription_url_strips_trailing_slash() -> None:
    stack = UUID("33333333-3333-3333-3333-333333333333")
    assert (
        helpers.live_subscription_url("http://broker.test:3000/", stack)
        == f"ws://broker.test:3000/api/v1/stacks/{stack}/live"
    )


def test_history_helper_signatures_include_keyword_filters() -> None:
    # Forces a regression test if the helpers grow positional args that
    # would change the call shape for downstream callers.
    sig = inspect.signature(helpers.list_telemetry_events)
    params = list(sig.parameters)
    assert params[:2] == ["client", "stack_id"]
    assert "since" in sig.parameters
    assert "limit" in sig.parameters


def test_list_ws_connections_takes_only_a_client() -> None:
    sig = inspect.signature(helpers.list_ws_connections)
    assert list(sig.parameters) == ["client"]
