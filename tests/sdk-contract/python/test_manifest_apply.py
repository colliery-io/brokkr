"""
SDK contract: manifest folder helpers via the `brokkr` wrapper (BROKKR-T-0196).

Exercises `BrokkrClient.apply` (idempotent create -> unchanged -> updated,
targeting label) and `submit_manifests` against a running broker. Mirrors the
Rust suite's `scenario_manifest_apply`.
"""

from __future__ import annotations

import asyncio

import pytest
from brokkr import ApplyResult, BrokkrClient, BrokkrError
from brokkr_broker_client.api.generators import create_generator
from brokkr_broker_client.api.stacks import list_stacks, stacks_list_labels
from brokkr_broker_client.models import CreateGeneratorResponse, NewGenerator
from conftest import make_client, unique


def test_manifest_apply(admin_client, base_url, admin_pak, tmp_path):
    # admin creates a generator -> generator PAK (apply needs a generator)
    gen_name = unique("py-apply-gen")
    gen_resp = create_generator.sync(
        client=admin_client,
        body=NewGenerator(name=gen_name, description="apply contract"),
    )
    assert isinstance(gen_resp, CreateGeneratorResponse)
    generator_pak = gen_resp.pak

    wrapper = BrokkrClient(base_url, token=generator_pak)

    # a temp folder of manifests, unsorted on disk
    (tmp_path / "02-cm.yaml").write_text(
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: apply-cm\n"
    )
    (tmp_path / "01-ns.yaml").write_text(
        "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: apply-ns\n"
    )

    stack_name = unique("py-apply-stack")

    # All async calls must share one event loop: the wrapper holds an
    # httpx.AsyncClient bound to the loop it first runs on, so issuing each
    # apply() under its own asyncio.run() would reuse a closed loop.
    async def _run() -> None:
        # first apply -> created (stack auto-created, label set)
        r1: ApplyResult = await wrapper.apply(stack_name, tmp_path, ["env:contract"])
        assert r1.status == "created", r1.status

        # same folder -> unchanged (re-adds the label -> tolerated 409)
        r2 = await wrapper.apply(stack_name, tmp_path, ["env:contract"])
        assert r2.status == "unchanged", r2.status

        # mutate folder -> updated
        (tmp_path / "03-svc.yaml").write_text(
            "apiVersion: v1\nkind: Service\nmetadata:\n  name: apply-svc\nspec:\n"
            "  selector:\n    app: x\n  ports:\n  - port: 80\n"
        )
        r3 = await wrapper.apply(stack_name, tmp_path, ["env:contract"])
        assert r3.status == "updated", r3.status

        # the named stack exists and carries the targeting label
        stacks = list_stacks.sync(client=admin_client)
        stack = next((s for s in stacks if s.name == stack_name), None)
        assert stack is not None, "apply did not create the named stack"
        labels = stacks_list_labels.sync(stack.id, client=admin_client)
        assert any(label.label == "env:contract" for label in labels)

        # submit_manifests against the existing stack id returns a new object
        obj = await wrapper.submit_manifests(stack.id, tmp_path)
        assert obj.stack_id == stack.id

        # Admin form (BROKKR-T-0332): an admin PAK applies on behalf of a
        # generator, named by name or by id. The stack belongs to that generator.
        admin_wrapper = BrokkrClient(base_url, token=admin_pak)
        admin_stack = unique("py-admin-apply-stack")
        a1 = await admin_wrapper.apply(admin_stack, tmp_path, generator=gen_name)
        assert a1.status == "created", a1.status
        a2 = await admin_wrapper.apply(
            admin_stack, tmp_path, generator=str(gen_resp.generator.id)
        )
        assert a2.status == "unchanged", a2.status
        owned = list_stacks.sync(client=make_client(base_url, generator_pak))
        assert any(s.name == admin_stack for s in owned), "generator does not own the stack"

        # Without a generator, an admin is refused; the error names the flag.
        with pytest.raises(BrokkrError, match="--generator"):
            await admin_wrapper.apply(admin_stack, tmp_path)
        with pytest.raises(BrokkrError, match="no tenant named"):
            await admin_wrapper.apply(admin_stack, tmp_path, generator="no-such-generator")
        # A tenant cannot apply for another tenant; its own name is fine.
        with pytest.raises(BrokkrError, match="cannot apply for"):
            await wrapper.apply(admin_stack, tmp_path, generator="no-such-generator")
        a3 = await wrapper.apply(admin_stack, tmp_path, generator=gen_name)
        assert a3.status == "unchanged", a3.status

    asyncio.run(_run())
