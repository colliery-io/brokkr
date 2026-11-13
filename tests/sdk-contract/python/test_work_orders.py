"""
SDK contract: a generator launches a work order with its own PAK
(BROKKR-T-0408, BROKKR-A-0013).

A work order is a job that runs one time. The generator that creates it owns
it. It reaches only the agents registered with that generator, and only its
owner and an admin can read it.

The harness runs no agent that does the work, so the work order stays
PENDING and its logs and events are empty. The proof is the owner field, the
shape of each answer, and the two refusals.
"""

from __future__ import annotations

import asyncio

import pytest
from brokkr import BrokkrClient, BrokkrError
from brokkr_broker_client import helpers
from brokkr_broker_client.api.agents import create_agent, delete_agent
from brokkr_broker_client.api.generators import (
    create_generator,
    delete_generator,
    register_agent,
)
from brokkr_broker_client.api.work_orders import (
    create_work_order,
    delete_work_order,
    get_work_order,
    list_work_orders,
)
from brokkr_broker_client.models import (
    CreateAgentResponse,
    CreateGeneratorResponse,
    CreateWorkOrderRequest,
    ErrorResponse,
    NewAgent,
    NewGenerator,
    WorkOrder,
    WorkOrderTargeting,
)
from brokkr_broker_client.models.agent_registration_body import AgentRegistrationBody
from conftest import make_client, unique

# The job of the work order: a Kubernetes Job that prints one line.
JOB_YAML = """apiVersion: batch/v1
kind: Job
metadata:
  name: sdk-contract-python-job
  namespace: default
spec:
  backoffLimit: 0
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: job
          image: busybox:1.36
          command: ["sh", "-c", "echo hello from a work order"]
"""


def test_generator_work_order(admin_client, base_url):
    # ----- The admin sets up: two generators, and two agents. One agent is
    # registered with the first generator; the other is with no generator. -----
    owner = create_generator.sync(
        client=admin_client, body=NewGenerator(name=unique("py-wo-owner"))
    )
    other = create_generator.sync(
        client=admin_client, body=NewGenerator(name=unique("py-wo-other"))
    )
    assert isinstance(owner, CreateGeneratorResponse)
    assert isinstance(other, CreateGeneratorResponse)

    agent = create_agent.sync(
        client=admin_client,
        body=NewAgent(name=unique("py-wo-agent"), cluster_name="py-wo-cluster"),
    )
    stray = create_agent.sync(
        client=admin_client,
        body=NewAgent(name=unique("py-wo-stray"), cluster_name="py-wo-cluster"),
    )
    assert isinstance(agent, CreateAgentResponse)
    assert isinstance(stray, CreateAgentResponse)
    register_agent.sync(
        owner.generator.id,
        client=admin_client,
        body=AgentRegistrationBody(agent_id=agent.agent.id),
    )

    owner_client = make_client(base_url, owner.pak)
    other_client = make_client(base_url, other.pak)

    try:
        # ----- The generator launches the job on its registered agent. -----
        work_order = create_work_order.sync(
            client=owner_client,
            body=CreateWorkOrderRequest(
                work_type="custom",
                yaml_content=JOB_YAML,
                targeting=WorkOrderTargeting(agent_ids=[agent.agent.id]),
            ),
        )
        assert isinstance(work_order, WorkOrder), f"create_work_order: {work_order!r}"
        assert work_order.status == "PENDING"
        # The generator that created it owns it.
        assert work_order.owner_generator_id == owner.generator.id

        # ----- It lists and reads its own work order. -----
        listed = list_work_orders.sync(client=owner_client)
        assert any(w.id == work_order.id for w in listed)
        read = get_work_order.sync(work_order.id, client=owner_client)
        assert isinstance(read, WorkOrder)
        assert read.owner_generator_id == owner.generator.id

        # ----- It reads the logs and the events of the job. No agent runs
        # the job here, so both are empty; the retention block is the shape. -----
        events = helpers.list_work_order_events(owner_client, work_order.id, limit=10)
        assert events.retention.retention_ceiling_seconds == 21600
        assert events.events == []
        logs = helpers.list_work_order_logs(owner_client, work_order.id)
        assert logs.retention.retention_ceiling_seconds == 21600
        assert logs.lines == []

        # ----- Another generator sees none of it. -----
        assert all(w.id != work_order.id for w in list_work_orders.sync(client=other_client))
        refused = get_work_order.sync_detailed(work_order.id, client=other_client)
        assert refused.status_code == 403
        assert isinstance(refused.parsed, ErrorResponse)
        assert refused.parsed.code == "work_order_not_owned"

        # ----- A target agent that is not registered with the generator is
        # refused, and no work order is made. -----
        before = len(list_work_orders.sync(client=owner_client))
        not_registered = create_work_order.sync_detailed(
            client=owner_client,
            body=CreateWorkOrderRequest(
                work_type="custom",
                yaml_content=JOB_YAML,
                targeting=WorkOrderTargeting(agent_ids=[stray.agent.id]),
            ),
        )
        assert not_registered.status_code == 403
        assert isinstance(not_registered.parsed, ErrorResponse)
        assert not_registered.parsed.code == "agent_not_registered"
        assert len(list_work_orders.sync(client=owner_client)) == before

        # ----- The same flow through the async wrapper, as the how-to shows
        # it: `client.api` for the generated calls, the wrapper for the logs
        # and the events. The wrapper raises a BrokkrError with the broker's
        # code. -----
        async def _wrapper() -> None:
            client = BrokkrClient(base_url, token=owner.pak)

            launched = await create_work_order.asyncio(
                client=client.api,
                body=CreateWorkOrderRequest(
                    work_type="custom",
                    yaml_content=JOB_YAML,
                    targeting=WorkOrderTargeting(agent_ids=[agent.agent.id]),
                ),
            )
            assert isinstance(launched, WorkOrder), f"create_work_order: {launched!r}"
            assert launched.owner_generator_id == owner.generator.id

            current = await get_work_order.asyncio(launched.id, client=client.api)
            assert isinstance(current, WorkOrder)
            assert current.status == "PENDING"

            job_logs = await client.list_work_order_logs(launched.id, limit=100)
            assert job_logs.lines == []
            job_events = await client.list_work_order_events(launched.id)
            assert job_events.events == []

            theirs = BrokkrClient(base_url, token=other.pak)
            with pytest.raises(BrokkrError) as err:
                await theirs.list_work_order_logs(launched.id)
            assert err.value.code == "work_order_not_owned"
            assert err.value.status == 403

            removed = await delete_work_order.asyncio_detailed(launched.id, client=client.api)
            assert removed.status_code == 204

        asyncio.run(_wrapper())

        # ----- The generator deletes its work order. -----
        deleted = delete_work_order.sync_detailed(work_order.id, client=owner_client)
        assert deleted.status_code == 204
        gone = get_work_order.sync_detailed(work_order.id, client=owner_client)
        assert gone.status_code == 404
    finally:
        # Cleanup (best-effort). A generator delete takes its waiting work
        # orders with it.
        for agent_id in (agent.agent.id, stray.agent.id):
            try:
                delete_agent.sync_detailed(agent_id, client=admin_client)
            except Exception:
                pass
        for generator_id in (owner.generator.id, other.generator.id):
            try:
                delete_generator.sync_detailed(generator_id, client=admin_client)
            except Exception:
                pass
