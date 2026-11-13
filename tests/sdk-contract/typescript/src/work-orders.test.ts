/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

/**
 * SDK contract: a generator launches a work order with its own PAK
 * (BROKKR-T-0408, BROKKR-A-0013).
 *
 * A work order is a job that runs one time. The generator that creates it
 * owns it. It reaches only the agents registered with that generator, and
 * only its owner and an admin can read it.
 *
 * The harness runs no agent that does the work, so the work order stays
 * PENDING and its logs and events are empty. The proof is the owner field,
 * the shape of each answer, and the two refusals.
 */

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import {
  BrokkrClient,
  BrokkrError,
  createBrokkrClient,
  type BrokkrApi,
} from "@colliery-io/brokkr-client";
import { randomUUID } from "node:crypto";

const BROKER_URL = (process.env.BROKER_URL ?? "http://localhost:3000").replace(
  /\/+$/,
  "",
);
const BASE_URL = `${BROKER_URL}/api/v1`;
const ADMIN_PAK =
  process.env.ADMIN_PAK ?? "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8";

// The job of the work order: a Kubernetes Job that prints one line.
const JOB_YAML = `apiVersion: batch/v1
kind: Job
metadata:
  name: sdk-contract-ts-job
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
`;

function unique(prefix: string): string {
  return `${prefix}-${randomUUID().replace(/-/g, "").slice(0, 8)}`;
}

/** The broker reads the raw PAK from `Authorization`, with no `Bearer `. */
function clientFor(pak: string): BrokkrApi {
  return createBrokkrClient({
    baseUrl: BASE_URL,
    headers: { Authorization: pak },
  });
}

async function waitForBroker(timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const resp = await fetch(`${BROKER_URL}/healthz`);
      if (resp.ok) return;
    } catch {
      /* not up yet */
    }
    await new Promise((r) => setTimeout(r, 1_000));
  }
  throw new Error("broker /healthz never returned 2xx");
}

const admin = clientFor(ADMIN_PAK);

interface Tenant {
  id: string;
  pak: string;
}

async function newGenerator(prefix: string): Promise<Tenant> {
  const { data, error } = await admin.POST("/generators", {
    body: { name: unique(prefix), description: null },
  });
  if (error || !data) {
    throw new Error(`create_generator failed: ${JSON.stringify(error)}`);
  }
  return { id: data.generator.id, pak: data.pak };
}

async function newAgent(prefix: string): Promise<string> {
  const { data, error } = await admin.POST("/agents", {
    body: { name: unique(prefix), cluster_name: "ts-wo-cluster" },
  });
  if (error || !data) {
    throw new Error(`create_agent failed: ${JSON.stringify(error)}`);
  }
  return data.agent.id;
}

let owner: Tenant;
let other: Tenant;
let agentId: string;
let strayId: string;

beforeAll(async () => {
  await waitForBroker();
  // The admin sets up: two generators, and two agents. One agent is
  // registered with the first generator; the other is with no generator.
  owner = await newGenerator("ts-wo-owner");
  other = await newGenerator("ts-wo-other");
  agentId = await newAgent("ts-wo-agent");
  strayId = await newAgent("ts-wo-stray");
  const reg = await admin.POST("/generators/{id}/register", {
    params: { path: { id: owner.id } },
    body: { agent_id: agentId },
  });
  expect(reg.error, `register_agent: ${JSON.stringify(reg.error)}`).toBeUndefined();
});

afterAll(async () => {
  // Best effort. A generator delete takes its waiting work orders with it.
  for (const id of [agentId, strayId]) {
    await admin
      .DELETE("/agents/{id}", { params: { path: { id } } })
      .catch(() => undefined);
  }
  for (const t of [owner, other]) {
    if (!t) continue;
    await admin
      .DELETE("/generators/{id}", { params: { path: { id: t.id } } })
      .catch(() => undefined);
  }
});

describe("a generator launches a work order with its own PAK", () => {
  it("creates, lists, reads, reads the logs and events, and deletes", async () => {
    // The wrapper, built with the generator's PAK. `client.api` is the typed
    // client for the generated calls.
    const client = new BrokkrClient({ baseUrl: BASE_URL, token: owner.pak });
    const gen = client.api;

    // The generator launches the job on its registered agent.
    const created = await gen.POST("/work-orders", {
      body: {
        work_type: "custom",
        yaml_content: JOB_YAML,
        targeting: { agent_ids: [agentId] },
      },
    });
    expect(created.error, `create_work_order: ${JSON.stringify(created.error)}`).toBeUndefined();
    expect(created.response.status).toBe(201);
    const workOrder = created.data!;
    expect(workOrder.status).toBe("PENDING");
    // The generator that created it owns it.
    expect(workOrder.owner_generator_id).toBe(owner.id);

    // It lists and reads its own work order.
    const listed = await gen.GET("/work-orders", {});
    expect(listed.data!.some((w) => w.id === workOrder.id)).toBe(true);
    const read = await gen.GET("/work-orders/{id}", {
      params: { path: { id: workOrder.id } },
    });
    expect(read.data!.owner_generator_id).toBe(owner.id);

    // It reads the logs and the events of the job. No agent runs the job
    // here, so both are empty; the retention block is the shape.
    const events = await client.listWorkOrderEvents(workOrder.id, { limit: 10 });
    expect(events.retention.retention_ceiling_seconds).toBe(21600);
    expect(events.events).toEqual([]);
    const logs = await client.listWorkOrderLogs(workOrder.id);
    expect(logs.retention.retention_ceiling_seconds).toBe(21600);
    expect(logs.lines).toEqual([]);

    // Another generator sees none of it.
    const stranger = clientFor(other.pak);
    const theirList = await stranger.GET("/work-orders", {});
    expect(theirList.data!.some((w) => w.id === workOrder.id)).toBe(false);
    const refused = await stranger.GET("/work-orders/{id}", {
      params: { path: { id: workOrder.id } },
    });
    expect(refused.response.status).toBe(403);
    expect(refused.error?.code).toBe("work_order_not_owned");
    const theirs = new BrokkrClient({ baseUrl: BASE_URL, token: other.pak });
    const err = await theirs.listWorkOrderLogs(workOrder.id).catch((e) => e);
    expect(err).toBeInstanceOf(BrokkrError);
    expect((err as BrokkrError).code).toBe("work_order_not_owned");
    expect((err as BrokkrError).status).toBe(403);

    // The generator deletes its work order.
    const deleted = await gen.DELETE("/work-orders/{id}", {
      params: { path: { id: workOrder.id } },
    });
    expect(deleted.response.status).toBe(204);
    const gone = await gen.GET("/work-orders/{id}", {
      params: { path: { id: workOrder.id } },
    });
    expect(gone.response.status).toBe(404);
  });

  it("refuses a target agent that is not registered with the generator", async () => {
    const gen = clientFor(owner.pak);
    const before = (await gen.GET("/work-orders", {})).data!.length;

    const refused = await gen.POST("/work-orders", {
      body: {
        work_type: "custom",
        yaml_content: JOB_YAML,
        targeting: { agent_ids: [strayId] },
      },
    });
    expect(refused.response.status).toBe(403);
    expect(refused.error?.code).toBe("agent_not_registered");

    // No work order is made.
    const after = (await gen.GET("/work-orders", {})).data!.length;
    expect(after).toBe(before);
  });
});
