"""
SDK contract: WS-10 / WS-13 telemetry history + WS connections through the
ergonomic helpers in ``brokkr_broker_client.helpers`` against a running
broker.

The harness doesn't bring up a real agent or kube cluster, so the
event/log responses are empty arrays — the proof under test is the
**response shape** (retention metadata correct, ``connections`` array
present) and that the wrappers actually talk to the broker.
"""

from __future__ import annotations

from brokkr_broker_client import AuthenticatedClient
from brokkr_broker_client import helpers
from brokkr_broker_client.api.generators import create_generator
from brokkr_broker_client.api.stacks import create_stack
from brokkr_broker_client.models import NewGenerator, NewStack

from conftest import make_client, unique


def _seed_stack(admin_client: AuthenticatedClient, base_url: str):
    gen_resp = create_generator.sync(
        client=admin_client,
        body=NewGenerator(name=unique("sdk-contract-py-tel-gen")),
    )
    assert gen_resp is not None and not isinstance(gen_resp, type(None))
    generator = gen_resp.generator
    gen_client = make_client(base_url, gen_resp.pak)
    stack = create_stack.sync(
        client=gen_client,
        body=NewStack(name=unique("sdk-contract-py-tel-stack"), generator_id=generator.id),
    )
    assert stack is not None
    return stack


def test_list_telemetry_events_returns_retention_metadata(
    admin_client: AuthenticatedClient,
    base_url: str,
) -> None:
    stack = _seed_stack(admin_client, base_url)
    resp = helpers.list_telemetry_events(admin_client, stack.id, limit=10)
    assert resp.retention.retention_ceiling_seconds == 21600
    assert resp.retention.effective_retention_seconds == 21600
    assert "Datadog" in resp.retention.long_term_sink_hint
    # events list is empty on a fresh stack — the shape is the contract.
    assert isinstance(resp.events, list)


def test_list_telemetry_logs_returns_retention_metadata(
    admin_client: AuthenticatedClient,
    base_url: str,
) -> None:
    stack = _seed_stack(admin_client, base_url)
    resp = helpers.list_telemetry_logs(admin_client, stack.id)
    assert resp.retention.retention_ceiling_seconds == 21600
    assert isinstance(resp.lines, list)


def test_list_ws_connections_returns_snapshot(
    admin_client: AuthenticatedClient,
) -> None:
    resp = helpers.list_ws_connections(admin_client)
    # No agent is connected in the contract harness; the shape is the
    # proof.
    assert isinstance(resp.connected_agents, int)
    assert isinstance(resp.live_subscribers, int)
    assert isinstance(resp.connections, list)


def test_live_subscription_url_helper_round_trips_through_format(
    broker_url: str,
) -> None:
    # No real WS upgrade here — the URL helper just composes the path.
    from uuid import UUID

    stack_id = UUID("11111111-1111-1111-1111-111111111111")
    url = helpers.live_subscription_url(broker_url, stack_id)
    expected_scheme = "wss" if broker_url.startswith("https") else "ws"
    assert url.startswith(f"{expected_scheme}://")
    assert url.endswith(f"/api/v1/stacks/{stack_id}/live")
