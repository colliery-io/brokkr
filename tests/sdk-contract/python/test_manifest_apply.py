"""
SDK contract: manifest folder helpers via the `brokkr` wrapper (BROKKR-T-0196).

Exercises `BrokkrClient.apply` (idempotent create -> unchanged -> updated,
targeting label) and `submit_manifests` against a running broker. Mirrors the
Rust suite's `scenario_manifest_apply`.
"""

from __future__ import annotations

import asyncio

from brokkr import ApplyResult, BrokkrClient
from brokkr_broker_client.api.generators import create_generator
from brokkr_broker_client.api.stacks import list_stacks, stacks_list_labels
from brokkr_broker_client.models import CreateGeneratorResponse, NewGenerator
from conftest import unique


def test_manifest_apply(admin_client, base_url, tmp_path):
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

    asyncio.run(_run())
