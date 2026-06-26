"""
SDK contract: UAT walkthrough through the generated Python SDK.

Each step exercises the typed `brokkr_broker_client.api.*` endpoint modules
+ `models` against a running broker. No hand-rolled HTTP. Mirrors the Rust
and TypeScript suites in `tests/sdk-contract/{rust,typescript}/`.
"""

from __future__ import annotations

from uuid import UUID

from brokkr_broker_client.api.agent_targets import add_target, remove_target
from brokkr_broker_client.api.agents import create_agent, delete_agent
from brokkr_broker_client.api.generators import create_generator, delete_generator, register_agent
from brokkr_broker_client.models.agent_registration_body import AgentRegistrationBody
from brokkr_broker_client.api.stacks import (
    create_deployment_object,
    create_stack,
    delete_stack,
    get_stack,
    list_stacks,
    stacks_add_annotation,
    stacks_add_label,
)
from brokkr_broker_client.models import (
    AgentTarget,
    CreateAgentResponse,
    CreateDeploymentObjectRequest,
    CreateGeneratorResponse,
    DeploymentObject,
    ErrorResponse,
    NewAgent,
    NewAgentTarget,
    NewGenerator,
    NewStack,
    NewStackAnnotation,
    Stack,
    StackAnnotation,
    StackLabel,
)

from conftest import DEMO_YAML, make_client, unique


def test_uat_walkthrough(admin_client, base_url):
    """Full UAT walkthrough: admin bootstrap + generator-driven flow."""
    # ----- Step 1: admin creates a generator (capture generator PAK) -----
    gen_name = unique("sdk-contract-py-gen")
    gen_resp = create_generator.sync(
        client=admin_client,
        body=NewGenerator(name=gen_name, description="python sdk contract"),
    )
    assert isinstance(gen_resp, CreateGeneratorResponse), f"got {type(gen_resp).__name__}: {gen_resp!r}"
    generator_id: UUID = gen_resp.generator.id
    generator_pak: str = gen_resp.pak
    print(f"  → generator_id={generator_id}")

    # ----- Step 2: admin creates an agent -----
    agent_resp = create_agent.sync(
        client=admin_client,
        body=NewAgent(name=unique("sdk-contract-py-agent"), cluster_name="sdk-contract-py-cluster"),
    )
    assert isinstance(agent_resp, CreateAgentResponse), (
        f"expected CreateAgentResponse, got {type(agent_resp).__name__}: {agent_resp!r}"
    )
    assert agent_resp.initial_pak, "create_agent returned empty initial_pak"
    agent_id: UUID = agent_resp.agent.id
    print(f"  → agent_id={agent_id}")

    # Switch to generator PAK for the rest of the flow.
    gen_client = make_client(base_url, generator_pak)

    # ----- Step 3: generator creates a stack -----
    stack_name = unique("sdk-contract-py-stack")
    stack = create_stack.sync(
        client=gen_client,
        body=NewStack(name=stack_name, generator_id=generator_id, description="python sdk contract"),
    )
    assert isinstance(stack, Stack), f"create_stack returned {type(stack).__name__}: {stack!r}"
    stack_id = stack.id
    print(f"  → stack_id={stack_id}")

    try:
        # ----- Step 4: stack label (BROKKR-T-0152 — JSON-string body) -----
        label = stacks_add_label.sync(stack_id, client=gen_client, body="contract-test")
        assert isinstance(label, StackLabel), (
            f"stacks_add_label returned {type(label).__name__}: {label!r}"
        )
        print(f"    label={label.label}")

        # ----- Step 5: stack annotation -----
        ann = stacks_add_annotation.sync(
            stack_id,
            client=gen_client,
            body=NewStackAnnotation(stack_id=stack_id, key="purpose", value="sdk-contract"),
        )
        assert isinstance(ann, StackAnnotation), (
            f"stacks_add_annotation returned {type(ann).__name__}: {ann!r}"
        )

        # ----- Step 6: deployment object -----
        dep = create_deployment_object.sync(
            stack_id,
            client=gen_client,
            body=CreateDeploymentObjectRequest(
                yaml_content=DEMO_YAML,
                is_deletion_marker=False,
            ),
        )
        assert isinstance(dep, DeploymentObject), (
            f"create_deployment_object returned {type(dep).__name__}: {dep!r}"
        )
        print(f"    deployment_id={dep.id}")

        # ----- Step 6.5: register agent with generator before targeting -----
        register_agent.sync(
            generator_id,
            client=admin_client,
            body=AgentRegistrationBody(agent_id=agent_id),
        )

        # ----- Step 7: target stack to agent (BROKKR-T-0153 — generator PAK now allowed) -----
        tgt = add_target.sync(
            agent_id,
            client=gen_client,
            body=NewAgentTarget(agent_id=agent_id, stack_id=stack_id),
        )
        assert isinstance(tgt, AgentTarget), f"add_target returned {type(tgt).__name__}: {tgt!r}"
        print(f"    target_id={tgt.id}")

        # ----- Step 7.5: list_stacks as the generator (BROKKR-T-0155). -----
        listed = list_stacks.sync(client=gen_client)
        assert isinstance(listed, list), (
            f"list_stacks (as generator) returned {type(listed).__name__}: {listed!r}"
        )
        assert any(s.id == stack_id for s in listed), (
            f"list_stacks (as generator) missing own stack {stack_id}; got {[s.id for s in listed]}"
        )
        assert all(s.generator_id == generator_id for s in listed), (
            "list_stacks (as generator) leaked stacks from another generator"
        )

        # ----- Step 8: GET the stack and verify shape -----
        fetched = get_stack.sync(stack_id, client=gen_client)
        assert isinstance(fetched, Stack), f"get_stack returned {type(fetched).__name__}: {fetched!r}"
        assert fetched.id == stack_id
        assert fetched.name == stack_name
        assert fetched.generator_id == generator_id
    finally:
        # Cleanup (best-effort).
        try:
            remove_target.sync_detailed(agent_id, stack_id, client=admin_client)
        except Exception:
            pass
        try:
            delete_stack.sync_detailed(stack_id, client=admin_client)
        except Exception:
            pass
        try:
            delete_agent.sync_detailed(agent_id, client=admin_client)
        except Exception:
            pass
        try:
            delete_generator.sync_detailed(generator_id, client=admin_client)
        except Exception:
            pass


def test_target_generator_mismatch_returns_typed_403(admin_client, base_url):
    """Generator A cannot target a stack owned by generator B → typed 403."""
    gen_a = create_generator.sync(
        client=admin_client, body=NewGenerator(name=unique("sdk-contract-py-gen-a"))
    )
    gen_b = create_generator.sync(
        client=admin_client, body=NewGenerator(name=unique("sdk-contract-py-gen-b"))
    )
    assert isinstance(gen_a, CreateGeneratorResponse)
    assert isinstance(gen_b, CreateGeneratorResponse)

    stack_b = create_stack.sync(
        client=admin_client,
        body=NewStack(name=unique("sdk-contract-py-stack-b"), generator_id=gen_b.generator.id),
    )
    assert isinstance(stack_b, Stack)

    agent_resp = create_agent.sync(
        client=admin_client,
        body=NewAgent(name=unique("sdk-contract-py-agent-x"), cluster_name="sdk-contract-py-cluster"),
    )
    assert isinstance(agent_resp, CreateAgentResponse)
    agent_id: UUID = agent_resp.agent.id

    gen_a_client = make_client(base_url, gen_a.pak)

    # Register agent with Gen A so the mismatch check (not the registration
    # check) is what fires when targeting Gen B's stack.
    register_agent.sync(
        gen_a.generator.id,
        client=admin_client,
        body=AgentRegistrationBody(agent_id=agent_id),
    )

    try:
        # `sync_detailed` gives us status_code + parsed body, both typed.
        resp = add_target.sync_detailed(
            agent_id,
            client=gen_a_client,
            body=NewAgentTarget(agent_id=agent_id, stack_id=stack_b.id),
        )
        assert resp.status_code == 403, f"expected 403, got {resp.status_code}: {resp.content!r}"
        assert isinstance(resp.parsed, ErrorResponse), (
            f"expected ErrorResponse, got {type(resp.parsed).__name__}: {resp.parsed!r}"
        )
        assert resp.parsed.code == "target_generator_mismatch", (
            f"expected code=target_generator_mismatch, got {resp.parsed.code!r}"
        )
    finally:
        try:
            delete_stack.sync_detailed(stack_b.id, client=admin_client)
        except Exception:
            pass
        try:
            delete_agent.sync_detailed(agent_id, client=admin_client)
        except Exception:
            pass
        try:
            delete_generator.sync_detailed(gen_a.generator.id, client=admin_client)
        except Exception:
            pass
        try:
            delete_generator.sync_detailed(gen_b.generator.id, client=admin_client)
        except Exception:
            pass
