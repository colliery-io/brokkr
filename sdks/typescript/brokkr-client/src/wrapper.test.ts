/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { afterEach, describe, expect, it, vi } from "vitest";

import { BrokkrClient, BrokkrError, type ErrorResponse } from "./index.js";

/** Build a fake `fetch` that returns scripted responses in order. */
function scriptedFetch(
  steps: Array<{ status: number; body?: object } | { throws: unknown }>,
) {
  let i = 0;
  const calls: Array<{ url: string; init?: RequestInit; headers: Headers }> = [];
  const impl: typeof fetch = (input, init) => {
    let url: string;
    const headers = new Headers();
    if (typeof input === "string") {
      url = input;
    } else if (input instanceof URL) {
      url = input.toString();
    } else {
      url = (input as Request).url;
      (input as Request).headers.forEach((v, k) => headers.set(k, v));
    }
    new Headers(init?.headers).forEach((v, k) => headers.set(k, v));
    calls.push({ url, init: init ?? undefined, headers });
    const step = steps[i++];
    if (!step) {
      return Promise.reject(
        new Error(`scripted fetch ran out of steps after ${i - 1}`),
      );
    }
    if ("throws" in step) {
      return Promise.reject(step.throws);
    }
    const body = step.body !== undefined ? JSON.stringify(step.body) : "";
    return Promise.resolve(
      new Response(body, {
        status: step.status,
        headers: { "Content-Type": "application/json" },
      }),
    );
  };
  return { fetch: impl, calls };
}

const baseUrl = "http://localhost:3000/api/v1";

describe("BrokkrError", () => {
  it("retryable status set matches the cross-language contract", () => {
    for (const status of [408, 429, 502, 503, 504]) {
      const err = new BrokkrError({ message: "x", code: "transient", status });
      expect(err.isRetryable()).toBe(true);
    }
    for (const status of [400, 401, 403, 404, 409, 422, 500, 501]) {
      const err = new BrokkrError({
        message: "x",
        code: "non_transient",
        status,
      });
      expect(err.isRetryable()).toBe(false);
    }
  });

  it("transport errors default to retryable", () => {
    const err = BrokkrError.fromTransport(new TypeError("network down"));
    expect(err.isRetryable()).toBe(true);
    expect(err.status).toBeUndefined();
  });

  it("fromResponse preserves the typed ErrorResponse body", () => {
    const body: ErrorResponse = { code: "agent_not_found", message: "x" };
    const err = BrokkrError.fromResponse(body, 404);
    expect(err.code).toBe("agent_not_found");
    expect(err.status).toBe(404);
    expect(err.response).toBe(body);
  });

  it("is an Error subclass with the right name", () => {
    const err = new BrokkrError({ message: "x" });
    expect(err).toBeInstanceOf(Error);
    expect(err.name).toBe("BrokkrError");
  });
});

describe("BrokkrClient construction", () => {
  it("rejects invalid maxRetries", () => {
    expect(() => new BrokkrClient({ baseUrl, maxRetries: -1 })).toThrow(
      RangeError,
    );
  });

  it("rejects invalid initialBackoffMs", () => {
    expect(() => new BrokkrClient({ baseUrl, initialBackoffMs: 0 })).toThrow(
      RangeError,
    );
  });

  it("constructs without a token, defaulting retry config", () => {
    const c = new BrokkrClient({ baseUrl });
    expect(c.api).toBeDefined();
    expect(c.maxRetries).toBe(3);
    expect(c.initialBackoffMs).toBe(200);
  });

  it("constructs with a token + custom retry config", () => {
    const c = new BrokkrClient({
      baseUrl,
      token: "bk_admin_test",
      maxRetries: 5,
      initialBackoffMs: 50,
      requestTimeoutMs: 1_000,
    });
    expect(c.maxRetries).toBe(5);
    expect(c.initialBackoffMs).toBe(50);
  });
});

describe("BrokkrClient.retry", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("returns on first success", async () => {
    const { fetch: scripted } = scriptedFetch([{ status: 200, body: [] }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl, maxRetries: 5 });
    const data = await c.retry((api) => api.GET("/agents"));
    expect(Array.isArray(data)).toBe(true);
  });

  it("stops after maxRetries on retryable transport errors", async () => {
    const { fetch: scripted, calls } = scriptedFetch([
      { throws: new TypeError("conn reset") },
      { throws: new TypeError("conn reset") },
      { throws: new TypeError("conn reset") },
    ]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({
      baseUrl,
      maxRetries: 2,
      initialBackoffMs: 1,
    });
    await expect(c.retry((api) => api.GET("/agents"))).rejects.toBeInstanceOf(
      BrokkrError,
    );
    expect(calls.length).toBe(3);
  });

  it("short-circuits on non-retryable status", async () => {
    const body: ErrorResponse = {
      code: "agent_not_found",
      message: "agent not found",
    };
    const { fetch: scripted, calls } = scriptedFetch([{ status: 404, body }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({
      baseUrl,
      maxRetries: 5,
      initialBackoffMs: 1,
    });
    let caught: unknown;
    try {
      await c.retry((api) =>
        api.GET("/agents/{id}", {
          params: {
            path: { id: "00000000-0000-0000-0000-000000000001" },
          },
        }),
      );
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(BrokkrError);
    expect((caught as BrokkrError).code).toBe("agent_not_found");
    expect((caught as BrokkrError).status).toBe(404);
    expect(calls.length).toBe(1);
  });

  it("retries on a retryable 503 then succeeds", async () => {
    const { fetch: scripted, calls } = scriptedFetch([
      {
        status: 503,
        body: { code: "transient", message: "service unavailable" },
      },
      { status: 200, body: [] },
    ]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({
      baseUrl,
      maxRetries: 3,
      initialBackoffMs: 1,
    });
    await expect(c.retry((api) => api.GET("/agents"))).resolves.toBeDefined();
    expect(calls.length).toBe(2);
  });

  it("injects Authorization header on every request", async () => {
    const { fetch: scripted, calls } = scriptedFetch([
      { status: 200, body: [] },
    ]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl, token: "bk_admin_test_token" });
    await c.retry((api) => api.GET("/agents"));
    expect(calls[0]!.headers.get("Authorization")).toBe(
      "Bearer bk_admin_test_token",
    );
  });
});

// =============================================================================
// WS-10 / WS-13 ergonomic wrappers
// =============================================================================

describe("BrokkrClient.listTelemetryEvents", () => {
  it("calls the spec path with the right id and returns the parsed body", async () => {
    const body = {
      retention: {
        retention_ceiling_seconds: 21600,
        effective_retention_seconds: 21600,
        long_term_sink_hint: "ship to Datadog",
      },
      events: [],
    };
    const { fetch: scripted, calls } = scriptedFetch([{ status: 200, body }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl });
    const stack = "11111111-1111-1111-1111-111111111111";
    const out = await c.listTelemetryEvents(stack);
    expect(out.retention.retention_ceiling_seconds).toBe(21600);
    expect(calls[0]!.url).toContain(`/stacks/${stack}/events`);
  });

  it("threads `since` and `limit` into the query string", async () => {
    const body = {
      retention: {
        retention_ceiling_seconds: 21600,
        effective_retention_seconds: 21600,
        long_term_sink_hint: "ship to Datadog",
      },
      events: [],
    };
    const { fetch: scripted, calls } = scriptedFetch([{ status: 200, body }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl });
    await c.listTelemetryEvents("22222222-2222-2222-2222-222222222222", {
      since: "2026-05-23T00:00:00Z",
      limit: 42,
    });
    const url = new URL(calls[0]!.url);
    expect(url.searchParams.get("since")).toBe("2026-05-23T00:00:00Z");
    expect(url.searchParams.get("limit")).toBe("42");
  });
});

describe("BrokkrClient.listTelemetryLogs", () => {
  it("calls /stacks/{id}/logs and returns the parsed body", async () => {
    const body = {
      retention: {
        retention_ceiling_seconds: 21600,
        effective_retention_seconds: 21600,
        long_term_sink_hint: "ship to Datadog",
      },
      lines: [],
    };
    const { fetch: scripted, calls } = scriptedFetch([{ status: 200, body }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl });
    const stack = "33333333-3333-3333-3333-333333333333";
    const out = await c.listTelemetryLogs(stack);
    expect(out.lines).toEqual([]);
    expect(calls[0]!.url).toContain(`/stacks/${stack}/logs`);
  });
});

describe("BrokkrClient.listWsConnections", () => {
  it("calls /admin/ws/connections and returns the parsed body", async () => {
    const body = {
      connected_agents: 0,
      connections: [],
      live_subscribers: 0,
    };
    const { fetch: scripted, calls } = scriptedFetch([{ status: 200, body }]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({ baseUrl });
    const out = await c.listWsConnections();
    expect(out.connected_agents).toBe(0);
    expect(calls[0]!.url).toContain("/admin/ws/connections");
  });
});

describe("BrokkrClient.liveSubscriptionUrl", () => {
  it("strips /api/v1 from baseUrl then re-appends the full subscription path", () => {
    const c = new BrokkrClient({ baseUrl: "http://broker.test:3000/api/v1" });
    expect(c.liveSubscriptionUrl("abc-123")).toBe(
      "ws://broker.test:3000/api/v1/stacks/abc-123/live",
    );
  });

  it("handles baseUrl that's already the broker root (no /api/v1 suffix)", () => {
    const c = new BrokkrClient({ baseUrl: "http://broker.test:3000" });
    expect(c.liveSubscriptionUrl("abc-123")).toBe(
      "ws://broker.test:3000/api/v1/stacks/abc-123/live",
    );
  });

  it("swaps https://→wss://", () => {
    const c = new BrokkrClient({ baseUrl: "https://broker.example.com/api/v1" });
    expect(c.liveSubscriptionUrl("xyz")).toBe(
      "wss://broker.example.com/api/v1/stacks/xyz/live",
    );
  });

  it("tolerates a trailing slash on the base URL", () => {
    const c = new BrokkrClient({ baseUrl: "http://broker.test:3000/api/v1/" });
    expect(c.liveSubscriptionUrl("abc")).toBe(
      "ws://broker.test:3000/api/v1/stacks/abc/live",
    );
  });
});

describe("non-idempotent POSTs are single-attempt (BROKKR-T-0212)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("does not retry submitManifests even on a retryable 502", async () => {
    const dir = mkdtempSync(join(tmpdir(), "brokkr-noretry-"));
    writeFileSync(
      join(dir, "ns.yaml"),
      "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: n\n",
    );
    const body: ErrorResponse = { code: "internal_error", message: "boom" };
    // Script several 502s: with the old retry wrapper this POST would be
    // called maxRetries+1 times; single-attempt must call it exactly once.
    const { fetch: scripted, calls } = scriptedFetch([
      { status: 502, body },
      { status: 502, body },
      { status: 502, body },
      { status: 502, body },
    ]);
    vi.stubGlobal("fetch", scripted);
    const c = new BrokkrClient({
      baseUrl,
      token: "t",
      maxRetries: 5,
      initialBackoffMs: 1,
    });
    await expect(
      c.submitManifests("00000000-0000-0000-0000-000000000001", dir),
    ).rejects.toBeInstanceOf(BrokkrError);
    expect(calls.length).toBe(1);
  });
});
