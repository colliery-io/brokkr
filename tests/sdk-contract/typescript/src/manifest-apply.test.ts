/**
 * SDK contract: manifest folder helpers via the `BrokkrClient` wrapper
 * (BROKKR-T-0197). Exercises idempotent apply (created -> unchanged ->
 * updated, targeting label) and submitManifests against a running broker.
 * Mirrors the Rust and Python suites.
 */
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import { beforeAll, describe, expect, it } from "vitest";

import { BrokkrClient, createBrokkrClient } from "@colliery-io/brokkr-client";

const BROKER_URL = (process.env.BROKER_URL ?? "http://localhost:3000").replace(/\/+$/, "");
const BASE_URL = `${BROKER_URL}/api/v1`;
const ADMIN_PAK =
  process.env.ADMIN_PAK ?? "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8";

function unique(prefix: string): string {
  return `${prefix}-${randomUUID().replace(/-/g, "").slice(0, 8)}`;
}

async function waitForBroker(timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      if ((await fetch(`${BROKER_URL}/healthz`)).ok) return;
    } catch {
      // retry
    }
    await new Promise((r) => setTimeout(r, 1_000));
  }
  throw new Error("broker /healthz never returned 2xx");
}

beforeAll(async () => {
  await waitForBroker();
});

describe("brokkr SDK contract — TypeScript manifest apply", () => {
  it("apply: created -> unchanged -> updated, label, submitManifests", async () => {
    // admin creates a generator -> generator PAK (apply needs a generator)
    const admin = createBrokkrClient({
      baseUrl: BASE_URL,
      headers: { Authorization: ADMIN_PAK },
    });
    const genRes = await admin.POST("/generators", {
      body: { name: unique("ts-apply-gen"), description: "apply contract" },
    });
    expect(genRes.error, JSON.stringify(genRes.error)).toBeUndefined();
    const generatorPak = (genRes.data as { pak: string }).pak;

    const wrapper = new BrokkrClient({ baseUrl: BASE_URL, token: generatorPak });

    // temp folder of manifests, unsorted on disk
    const dir = mkdtempSync(join(tmpdir(), "ts-apply-"));
    writeFileSync(join(dir, "02-cm.yaml"), "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: apply-cm\n");
    writeFileSync(join(dir, "01-ns.yaml"), "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: apply-ns\n");

    const stackName = unique("ts-apply-stack");

    const r1 = await wrapper.apply(stackName, dir, ["env:contract"]);
    expect(r1.status).toBe("created");

    const r2 = await wrapper.apply(stackName, dir, ["env:contract"]);
    expect(r2.status).toBe("unchanged");

    writeFileSync(
      join(dir, "03-svc.yaml"),
      "apiVersion: v1\nkind: Service\nmetadata:\n  name: apply-svc\nspec:\n  selector:\n    app: x\n  ports:\n  - port: 80\n",
    );
    const r3 = await wrapper.apply(stackName, dir, ["env:contract"]);
    expect(r3.status).toBe("updated");

    // stack exists with the targeting label
    const gen = createBrokkrClient({
      baseUrl: BASE_URL,
      headers: { Authorization: generatorPak },
    });
    const stacksRes = await gen.GET("/stacks", {});
    const stack = (stacksRes.data as Array<{ id: string; name: string }>).find(
      (s) => s.name === stackName,
    );
    expect(stack, "apply did not create the named stack").toBeDefined();
    const labelsRes = await gen.GET("/stacks/{id}/labels", {
      params: { path: { id: stack!.id } },
    });
    expect(
      (labelsRes.data as Array<{ label: string }>).some((l) => l.label === "env:contract"),
    ).toBe(true);

    // submitManifests against the existing stack id
    const obj = await wrapper.submitManifests(stack!.id, dir);
    expect((obj as { stack_id: string }).stack_id).toBe(stack!.id);
  });
});
