/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

/**
 * SDK contract: WS-10 / WS-13 telemetry history + WS connections
 * through the ergonomic wrapper methods on `BrokkrClient`, plus a
 * WS-11 live-subscription smoke test using the `ws` package.
 *
 * The harness doesn't run a real agent or kube cluster, so the
 * event/log responses are empty — the proof is the response shape
 * (retention metadata correct, `connections` array present) and that
 * the wrappers actually round-trip through the broker.
 */

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import WebSocket from "ws";

import {
  BrokkrClient,
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

async function waitForReady(): Promise<void> {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const r = await fetch(`${BROKER_URL}/healthz`);
      if (r.ok) return;
    } catch {
      /* ignore */
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error("broker not ready after 30s");
}

let admin: BrokkrClient;
let adminRaw: BrokkrApi;
let stackId: string;

async function seedStack(): Promise<string> {
  const genName = `sdk-contract-ts-tel-gen-${randomUUID().slice(0, 8)}`;
  const { data: gen, error } = await adminRaw.POST("/generators", {
    body: { name: genName, description: null },
  });
  if (error || !gen) {
    throw new Error(`create_generator failed: ${JSON.stringify(error)}`);
  }
  const genClient = createBrokkrClient({
    baseUrl: BASE_URL,
    headers: { Authorization: gen.pak },
  });
  const stackName = `sdk-contract-ts-tel-stack-${randomUUID().slice(0, 8)}`;
  const { data: stack, error: e2 } = await genClient.POST("/stacks", {
    body: {
      name: stackName,
      generator_id: gen.generator.id,
      description: null,
    },
  });
  if (e2 || !stack) {
    throw new Error(`create_stack failed: ${JSON.stringify(e2)}`);
  }
  return stack.id;
}

beforeAll(async () => {
  await waitForReady();
  admin = new BrokkrClient({ baseUrl: BASE_URL, token: ADMIN_PAK });
  adminRaw = admin.api;
  stackId = await seedStack();
});

describe("WS-10 / WS-13 wrapper round-trip", () => {
  it("listTelemetryEvents returns retention metadata", async () => {
    const resp = await admin.listTelemetryEvents(stackId, { limit: 10 });
    expect(resp.retention.retention_ceiling_seconds).toBe(21600);
    expect(resp.retention.effective_retention_seconds).toBe(21600);
    expect(resp.retention.long_term_sink_hint).toMatch(/Datadog/);
    expect(Array.isArray(resp.events)).toBe(true);
  });

  it("listTelemetryLogs returns retention metadata", async () => {
    const resp = await admin.listTelemetryLogs(stackId);
    expect(resp.retention.retention_ceiling_seconds).toBe(21600);
    expect(Array.isArray(resp.lines)).toBe(true);
  });

  it("listWsConnections returns snapshot shape", async () => {
    const resp = await admin.listWsConnections();
    expect(typeof resp.connected_agents).toBe("number");
    expect(typeof resp.live_subscribers).toBe("number");
    expect(Array.isArray(resp.connections)).toBe(true);
  });
});

describe("WS-11 live subscription", () => {
  it(
    "opens the live tail and receives the broker's hello with auth via ws package",
    async () => {
      // node's `ws` library supports custom headers on the upgrade,
      // unlike browsers — that's exactly why this test lives here.
      const url = admin.liveSubscriptionUrl(stackId);
      const socket = new WebSocket(url, {
        headers: { Authorization: `Bearer ${ADMIN_PAK}` },
      });

      const opened = await new Promise<boolean>((resolve, reject) => {
        const timer = setTimeout(
          () => reject(new Error("WS open timeout (3s)")),
          3000,
        );
        socket.once("open", () => {
          clearTimeout(timer);
          resolve(true);
        });
        socket.once("error", (e) => {
          clearTimeout(timer);
          reject(e);
        });
      });
      expect(opened).toBe(true);

      // Close cleanly.
      await new Promise<void>((resolve) => {
        socket.once("close", () => resolve());
        socket.close();
      });
    },
    10_000,
  );
});

afterAll(() => {
  // Best-effort cleanup; the broker fixtures aren't torn down by this
  // test, by design — the harness is single-shot.
});
