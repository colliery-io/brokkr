/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

/**
 * SDK contract: UAT walkthrough through the generated TypeScript SDK.
 *
 * Exercises the typed `@colliery-io/brokkr-client` against a running broker
 * using only its `openapi-fetch`-based surface. Mirrors the Rust + Python
 * suites under `tests/sdk-contract/`.
 */

import { beforeAll, describe, expect, it } from "vitest";

import {
  type BrokkrApi,
  type ErrorResponse,
  createBrokkrClient,
} from "@colliery-io/brokkr-client";
import { randomUUID } from "node:crypto";

const BROKER_URL = (process.env.BROKER_URL ?? "http://localhost:3000").replace(
  /\/+$/,
  "",
);
const BASE_URL = `${BROKER_URL}/api/v1`;
const ADMIN_PAK =
  process.env.ADMIN_PAK ?? "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8";

const DEMO_YAML = `apiVersion: v1
kind: Namespace
metadata:
  name: sdk-contract-ts-ns
  labels:
    app: sdk-contract-ts
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: sdk-contract-ts-config
  namespace: sdk-contract-ts-ns
data:
  KEY: "value"
`;

function unique(prefix: string): string {
  return `${prefix}-${randomUUID().replace(/-/g, "").slice(0, 8)}`;
}

/**
 * The Brokkr broker reads the raw PAK from `Authorization` — no `Bearer `
 * prefix is stripped. Matches the Rust + Python SDK behaviour.
 */
function clientFor(pak: string): BrokkrApi {
  return createBrokkrClient({
    baseUrl: BASE_URL,
    headers: { Authorization: pak },
  });
}

async function waitForBroker(timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastErr: unknown;
  while (Date.now() < deadline) {
    try {
      const resp = await fetch(`${BROKER_URL}/healthz`);
      if (resp.ok) return;
    } catch (e) {
      lastErr = e;
    }
    await new Promise((r) => setTimeout(r, 1_000));
  }
  throw new Error(`broker /healthz never returned 2xx: ${String(lastErr)}`);
}

beforeAll(async () => {
  await waitForBroker();
});

describe("brokkr SDK contract — TypeScript", () => {
  it("UAT walkthrough via generator PAK", async () => {
    const admin = clientFor(ADMIN_PAK);

    // Step 1: admin creates generator
    const genName = unique("sdk-contract-ts-gen");
    const genRes = await admin.POST("/generators", {
      body: { name: genName, description: "typescript sdk contract" },
    });
    expect(genRes.error, `create_generator error: ${JSON.stringify(genRes.error)}`).toBeUndefined();
    expect(genRes.response.status).toBe(201);
    expect(genRes.data).toBeDefined();
    const generatorId = genRes.data!.generator.id;
    const generatorPak = genRes.data!.pak;

    // Step 2: admin creates agent (now typed as CreateAgentResponse)
    const agentRes = await admin.POST("/agents", {
      body: {
        name: unique("sdk-contract-ts-agent"),
        cluster_name: "sdk-contract-ts-cluster",
      },
    });
    expect(agentRes.error).toBeUndefined();
    expect(agentRes.response.status).toBe(201);
    expect(agentRes.data).toBeDefined();
    const agentId = agentRes.data!.agent.id;
    expect(agentRes.data!.initial_pak).toBeTruthy();
    expect(agentId).toMatch(/^[0-9a-f-]{36}$/);

    const gen = clientFor(generatorPak);

    try {
      // Step 3: generator creates stack
      const stackName = unique("sdk-contract-ts-stack");
      const stackRes = await gen.POST("/stacks", {
        body: {
          name: stackName,
          generator_id: generatorId,
          description: "typescript sdk contract",
        },
      });
      expect(stackRes.error, `create_stack error: ${JSON.stringify(stackRes.error)}`).toBeUndefined();
      expect(stackRes.response.status).toBe(201);
      const stackId = stackRes.data!.id;

      // Step 4: stack label (BROKKR-T-0152 — JSON-string body)
      const labelRes = await gen.POST("/stacks/{id}/labels", {
        params: { path: { id: stackId } },
        body: "contract-test",
      });
      expect(labelRes.error, `stacks_add_label error: ${JSON.stringify(labelRes.error)}`).toBeUndefined();
      expect(labelRes.response.status).toBe(201);
      expect(labelRes.data!.label).toBe("contract-test");

      // Step 5: stack annotation
      const annRes = await gen.POST("/stacks/{id}/annotations", {
        params: { path: { id: stackId } },
        body: { stack_id: stackId, key: "purpose", value: "sdk-contract" },
      });
      expect(annRes.error, `stacks_add_annotation error: ${JSON.stringify(annRes.error)}`).toBeUndefined();
      expect(annRes.response.status).toBe(201);

      // Step 6: deployment object (typed CreateDeploymentObjectRequest body)
      const depRes = await gen.POST("/stacks/{id}/deployment-objects", {
        params: { path: { id: stackId } },
        body: {
          yaml_content: DEMO_YAML,
          is_deletion_marker: false,
        },
      });
      expect(depRes.error, `create_deployment_object error: ${JSON.stringify(depRes.error)}`).toBeUndefined();
      expect(depRes.response.status).toBe(201);

      // Step 7: target stack to agent (BROKKR-T-0153 — generator PAK allowed)
      const targetRes = await gen.POST("/agents/{id}/targets", {
        params: { path: { id: agentId } },
        body: { agent_id: agentId, stack_id: stackId },
      });
      expect(targetRes.error, `add_target error: ${JSON.stringify(targetRes.error)}`).toBeUndefined();
      expect(targetRes.response.status).toBe(201);

      // Step 7.5: list_stacks as the generator (BROKKR-T-0155 — filtered to own)
      const listRes = await gen.GET("/stacks", {});
      expect(listRes.error, `list_stacks error: ${JSON.stringify(listRes.error)}`).toBeUndefined();
      expect(listRes.response.status).toBe(200);
      expect(listRes.data).toBeDefined();
      const stackIds = listRes.data!.map((s) => s.id);
      expect(stackIds, `generator did not see own stack ${stackId}; got ${stackIds.join(", ")}`).toContain(stackId);
      for (const s of listRes.data!) {
        expect(s.generator_id, `list_stacks leaked stack ${s.id} from another generator`).toBe(generatorId);
      }

      // Step 8: GET the stack and verify shape
      const getStack = await gen.GET("/stacks/{id}", {
        params: { path: { id: stackId } },
      });
      expect(getStack.error).toBeUndefined();
      expect(getStack.response.status).toBe(200);
      expect(getStack.data!.id).toBe(stackId);
      expect(getStack.data!.name).toBe(stackName);
      expect(getStack.data!.generator_id).toBe(generatorId);
    } finally {
      // Best-effort cleanup.
      await admin
        .DELETE("/agents/{id}/targets/{stack_id}", {
          params: { path: { id: agentId, stack_id: "" } },
        })
        .catch(() => undefined);
      await admin
        .DELETE("/agents/{id}", { params: { path: { id: agentId } } })
        .catch(() => undefined);
      await admin
        .DELETE("/generators/{id}", { params: { path: { id: generatorId } } })
        .catch(() => undefined);
    }
  });

  it("negative path: generator targeting a stack it does not own → typed 403", async () => {
    const admin = clientFor(ADMIN_PAK);

    const genA = await admin.POST("/generators", {
      body: { name: unique("sdk-contract-ts-gen-a") },
    });
    expect(genA.error).toBeUndefined();
    const genB = await admin.POST("/generators", {
      body: { name: unique("sdk-contract-ts-gen-b") },
    });
    expect(genB.error).toBeUndefined();

    const stackB = await admin.POST("/stacks", {
      body: {
        name: unique("sdk-contract-ts-stack-b"),
        generator_id: genB.data!.generator.id,
      },
    });
    expect(stackB.error).toBeUndefined();

    const agentRes = await admin.POST("/agents", {
      body: {
        name: unique("sdk-contract-ts-agent-x"),
        cluster_name: "sdk-contract-ts-cluster",
      },
    });
    expect(agentRes.error).toBeUndefined();
    expect(agentRes.data).toBeDefined();
    const agentId = agentRes.data!.agent.id;

    const genAClient = clientFor(genA.data!.pak);

    try {
      const targetRes = await genAClient.POST("/agents/{id}/targets", {
        params: { path: { id: agentId } },
        body: { agent_id: agentId, stack_id: stackB.data!.id },
      });

      expect(targetRes.data).toBeUndefined();
      expect(targetRes.response.status).toBe(403);
      const err = targetRes.error as ErrorResponse;
      expect(err).toBeDefined();
      expect(err.code).toBe("target_generator_mismatch");
      expect(typeof err.message).toBe("string");
    } finally {
      await admin
        .DELETE("/stacks/{id}", { params: { path: { id: stackB.data!.id } } })
        .catch(() => undefined);
      await admin
        .DELETE("/agents/{id}", { params: { path: { id: agentId } } })
        .catch(() => undefined);
      await admin
        .DELETE("/generators/{id}", {
          params: { path: { id: genA.data!.generator.id } },
        })
        .catch(() => undefined);
      await admin
        .DELETE("/generators/{id}", {
          params: { path: { id: genB.data!.generator.id } },
        })
        .catch(() => undefined);
    }
  });
});
