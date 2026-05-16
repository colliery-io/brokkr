/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

import { describe, expect, expectTypeOf, it } from "vitest";

import {
  type Agent,
  type ErrorResponse,
  type WorkOrder,
  type paths,
  createBrokkrClient,
} from "./index.js";

describe("brokkr-client surface", () => {
  it("constructs without options", () => {
    const c = createBrokkrClient();
    expect(c).toBeDefined();
    expect(typeof c.GET).toBe("function");
    expect(typeof c.POST).toBe("function");
    expect(typeof c.PUT).toBe("function");
    expect(typeof c.DELETE).toBe("function");
    expect(typeof c.PATCH).toBe("function");
  });

  it("accepts baseUrl + Authorization header", () => {
    const c = createBrokkrClient({
      baseUrl: "http://localhost:3000/api/v1",
      headers: { Authorization: "Bearer bk_admin_test" },
    });
    expect(c).toBeDefined();
  });

  it("exposes typed paths for baseline operations", () => {
    // Compile-time checks — every path must exist in the schema's `paths` map.
    expectTypeOf<paths>().toHaveProperty("/agents");
    expectTypeOf<paths>().toHaveProperty("/agents/{id}");
    expectTypeOf<paths>().toHaveProperty("/stacks");
    expectTypeOf<paths>().toHaveProperty("/work-orders");
    expectTypeOf<paths>().toHaveProperty("/work-orders/{id}/claim");
    expectTypeOf<paths>().toHaveProperty("/work-orders/{id}/complete");
    expectTypeOf<paths>().toHaveProperty("/auth/pak");
    expectTypeOf<paths>().toHaveProperty("/agents/{id}/health-status");
    expectTypeOf<paths>().toHaveProperty("/webhooks");
    expectTypeOf<paths>().toHaveProperty("/agents/{agent_id}/webhooks/pending");
  });

  it("ErrorResponse carries the canonical fields", () => {
    const err: ErrorResponse = {
      code: "agent_not_found",
      message: "agent not found",
    };
    expect(err.code).toBe("agent_not_found");
    expectTypeOf<ErrorResponse["code"]>().toEqualTypeOf<string>();
    expectTypeOf<ErrorResponse["message"]>().toEqualTypeOf<string>();
  });

  it("Agent and WorkOrder types include the fields the agent migration depends on", () => {
    // These spot-checks would fail at compile time if the spec drifted.
    expectTypeOf<Agent>().toHaveProperty("id");
    expectTypeOf<Agent>().toHaveProperty("name");
    expectTypeOf<Agent>().toHaveProperty("cluster_name");
    expectTypeOf<WorkOrder>().toHaveProperty("id");
    expectTypeOf<WorkOrder>().toHaveProperty("status");
    expectTypeOf<WorkOrder>().toHaveProperty("work_type");
  });
});
