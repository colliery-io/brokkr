/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

// Broker API surface for the ui-slim demo.
//
// Migrated in T-D3 to use `@colliery-io/brokkr-client` (the workspace's
// TypeScript SDK) for every v1 broker call. The handful of non-v1 endpoints —
// the Prometheus /metrics endpoint and the demo's webhook-catcher — continue
// to use raw fetch because they're intentionally outside the SDK's surface.
//
// The exported function names and signatures match the pre-migration API.js
// so the React components didn't need to change.

import { BrokkrClient } from "@colliery-io/brokkr-client";

const BROKER_URL =
  process.env.REACT_APP_BROKER_URL || "http://localhost:30300";
const ADMIN_PAK = process.env.REACT_APP_ADMIN_PAK || "";

// Single client instance for the lifetime of the page. Rotating PAKs at
// runtime would require reconstruction; the demo currently reads the PAK
// once from the env at build time, matching the pre-migration behavior.
const client = new BrokkrClient({
  baseUrl: `${BROKER_URL}/api/v1`,
  token: ADMIN_PAK,
});

const sha256 = async (str) => {
  const data = new TextEncoder().encode(str);
  const hash = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
};

// Custom error class carrying the broker's typed `ErrorResponse` so
// components can pattern-match on `e.code` (machine-readable, stable)
// rather than substring-matching on `e.message`.
export class ApiError extends Error {
  constructor({ message, code, status, response }) {
    super(message);
    this.name = "ApiError";
    this.code = code;
    this.status = status;
    this.response = response;
  }
}

// Unwrap openapi-fetch's `{ data, error, response }` tuple to either the
// success body (or null for 204) or throw an ApiError that preserves
// `code`/`status` for callers.
const unwrap = async (callPromise) => {
  const { data, error, response } = await callPromise;
  if (error !== undefined) {
    const body =
      typeof error === "object" && error !== null ? error : null;
    throw new ApiError({
      message:
        body?.message ??
        `${response.status}: ${JSON.stringify(error)}`,
      code: body?.code,
      status: response.status,
      response: body,
    });
  }
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new ApiError({
      message: `${response.status}: ${text}`,
      status: response.status,
    });
  }
  return response.status === 204 ? null : data;
};

// =============================================================================
// Agents
// =============================================================================

export const getAgents = () => unwrap(client.api.GET("/agents"));
export const getAgentLabels = (id) =>
  unwrap(client.api.GET("/agents/{id}/labels", { params: { path: { id } } }));
export const getAgentAnnotations = (id) =>
  unwrap(
    client.api.GET("/agents/{id}/annotations", { params: { path: { id } } }),
  );
export const getAgentTargets = (id) =>
  unwrap(client.api.GET("/agents/{id}/targets", { params: { path: { id } } }));
export const getAgentEvents = (id) =>
  unwrap(client.api.GET("/agents/{id}/events", { params: { path: { id } } }));
export const getAgentStacks = (id) =>
  unwrap(client.api.GET("/agents/{id}/stacks", { params: { path: { id } } }));
export const addAgentLabel = (id, label) =>
  unwrap(
    client.api.POST("/agents/{id}/labels", {
      params: { path: { id } },
      body: { agent_id: id, label },
    }),
  );
export const removeAgentLabel = (id, label) =>
  unwrap(
    client.api.DELETE("/agents/{id}/labels/{label}", {
      params: { path: { id, label } },
    }),
  );
export const addAgentAnnotation = (id, key, value) =>
  unwrap(
    client.api.POST("/agents/{id}/annotations", {
      params: { path: { id } },
      body: { agent_id: id, key, value },
    }),
  );
export const removeAgentAnnotation = (id, key) =>
  unwrap(
    client.api.DELETE("/agents/{id}/annotations/{key}", {
      params: { path: { id, key } },
    }),
  );
export const addAgentTarget = (id, stackId) =>
  unwrap(
    client.api.POST("/agents/{id}/targets", {
      params: { path: { id } },
      body: { agent_id: id, stack_id: stackId },
    }),
  );
export const removeAgentTarget = (id, stackId) =>
  unwrap(
    client.api.DELETE("/agents/{id}/targets/{stack_id}", {
      params: { path: { id, stack_id: stackId } },
    }),
  );
export const createAgent = (name, cluster) =>
  unwrap(client.api.POST("/agents", { body: { name, cluster_name: cluster } }));
export const updateAgent = (id, updates) =>
  unwrap(
    client.api.PUT("/agents/{id}", { params: { path: { id } }, body: updates }),
  );
export const rotateAgentPak = (id) =>
  unwrap(
    client.api.POST("/agents/{id}/rotate-pak", { params: { path: { id } } }),
  );

// =============================================================================
// Stacks
// =============================================================================

export const getStacks = () => unwrap(client.api.GET("/stacks"));
export const getStackLabels = (id) =>
  unwrap(client.api.GET("/stacks/{id}/labels", { params: { path: { id } } }));
export const getStackAnnotations = (id) =>
  unwrap(
    client.api.GET("/stacks/{id}/annotations", { params: { path: { id } } }),
  );
export const getStackDeployments = (id) =>
  unwrap(
    client.api.GET("/stacks/{id}/deployment-objects", {
      params: { path: { id } },
    }),
  );
export const createStack = (name, description, generatorId) =>
  unwrap(
    client.api.POST("/stacks", {
      body: { name, description: description || null, generator_id: generatorId },
    }),
  );
export const addStackLabel = (id, label) =>
  unwrap(
    client.api.POST("/stacks/{id}/labels", {
      params: { path: { id } },
      body: label,
    }),
  );
export const removeStackLabel = (id, label) =>
  unwrap(
    client.api.DELETE("/stacks/{id}/labels/{label}", {
      params: { path: { id, label } },
    }),
  );
export const addStackAnnotation = (id, key, value) =>
  unwrap(
    client.api.POST("/stacks/{id}/annotations", {
      params: { path: { id } },
      body: { stack_id: id, key, value },
    }),
  );
export const removeStackAnnotation = (id, key) =>
  unwrap(
    client.api.DELETE("/stacks/{id}/annotations/{key}", {
      params: { path: { id, key } },
    }),
  );
export const createDeployment = async (stackId, yaml, isDeletion = false) => {
  const checksum = await sha256(yaml);
  return unwrap(
    client.api.POST("/stacks/{id}/deployment-objects", {
      params: { path: { id: stackId } },
      body: {
        yaml_content: yaml,
        yaml_checksum: checksum,
        is_deletion_marker: isDeletion,
        sequence_id: null,
      },
    }),
  );
};
export const getDeployment = (id) =>
  unwrap(
    client.api.GET("/deployment-objects/{id}", { params: { path: { id } } }),
  );

// =============================================================================
// Templates
// =============================================================================

export const getTemplates = () => unwrap(client.api.GET("/templates"));
export const getTemplateLabels = (id) =>
  unwrap(
    client.api.GET("/templates/{id}/labels", { params: { path: { id } } }),
  );
export const getTemplateAnnotations = (id) =>
  unwrap(
    client.api.GET("/templates/{id}/annotations", { params: { path: { id } } }),
  );
export const createTemplate = (name, description, content, schema) =>
  unwrap(
    client.api.POST("/templates", {
      body: {
        name,
        description: description || null,
        template_content: content,
        parameters_schema: schema,
      },
    }),
  );
export const updateTemplate = (id, description, content, schema) =>
  unwrap(
    client.api.PUT("/templates/{id}", {
      params: { path: { id } },
      body: {
        description: description || null,
        template_content: content,
        parameters_schema: schema,
      },
    }),
  );
export const deleteTemplate = (id) =>
  unwrap(client.api.DELETE("/templates/{id}", { params: { path: { id } } }));
export const addTemplateLabel = (id, label) =>
  unwrap(
    client.api.POST("/templates/{id}/labels", {
      params: { path: { id } },
      body: label,
    }),
  );
export const removeTemplateLabel = (id, label) =>
  unwrap(
    client.api.DELETE("/templates/{id}/labels/{label}", {
      params: { path: { id, label } },
    }),
  );
export const instantiateTemplate = (stackId, templateId, params) =>
  unwrap(
    client.api.POST("/stacks/{stack_id}/deployment-objects/from-template", {
      params: { path: { stack_id: stackId } },
      body: { template_id: templateId, parameters: params },
    }),
  );

// =============================================================================
// Generators
// =============================================================================

export const getGenerators = () => unwrap(client.api.GET("/generators"));
export const createGenerator = (name, description) =>
  unwrap(
    client.api.POST("/generators", {
      body: { name, description: description || null },
    }),
  );
export const rotateGeneratorPak = (id) =>
  unwrap(
    client.api.POST("/generators/{id}/rotate-pak", {
      params: { path: { id } },
    }),
  );

// =============================================================================
// Work Orders (Jobs)
// =============================================================================

export const getWorkOrders = (status, workType) => {
  const query = {};
  if (status) query.status = status;
  if (workType) query.work_type = workType;
  return unwrap(client.api.GET("/work-orders", { params: { query } }));
};
export const getWorkOrder = (id) =>
  unwrap(
    client.api.GET("/work-orders/{id}", { params: { path: { id } } }),
  );
export const createWorkOrder = (workType, yamlContent, targeting, options = {}) =>
  unwrap(
    client.api.POST("/work-orders", {
      body: {
        work_type: workType,
        yaml_content: yamlContent,
        targeting,
        max_retries: options.maxRetries,
        backoff_seconds: options.backoffSeconds,
        claim_timeout_seconds: options.claimTimeoutSeconds,
      },
    }),
  );
export const deleteWorkOrder = (id) =>
  unwrap(client.api.DELETE("/work-orders/{id}", { params: { path: { id } } }));
export const getWorkOrderLog = (workType, success, agentId, limit) => {
  const query = {};
  if (workType) query.work_type = workType;
  if (success !== undefined && success !== null) query.success = success;
  if (agentId) query.agent_id = agentId;
  if (limit) query.limit = limit;
  return unwrap(client.api.GET("/work-order-log", { params: { query } }));
};

// =============================================================================
// Diagnostics
// =============================================================================

export const createDiagnostic = (
  deploymentObjectId,
  agentId,
  requestedBy,
  retentionMinutes,
) =>
  unwrap(
    client.api.POST("/deployment-objects/{id}/diagnostics", {
      params: { path: { id: deploymentObjectId } },
      body: {
        agent_id: agentId,
        requested_by: requestedBy,
        retention_minutes: retentionMinutes,
      },
    }),
  );
export const getDiagnostic = (id) =>
  unwrap(client.api.GET("/diagnostics/{id}", { params: { path: { id } } }));

// =============================================================================
// Deployment Health
// =============================================================================

export const getDeploymentHealth = (id) =>
  unwrap(
    client.api.GET("/deployment-objects/{id}/health", {
      params: { path: { id } },
    }),
  );
export const getStackHealth = (id) =>
  unwrap(client.api.GET("/stacks/{id}/health", { params: { path: { id } } }));

// =============================================================================
// Webhooks
// =============================================================================

export const getWebhooks = () => unwrap(client.api.GET("/webhooks"));
export const getWebhook = (id) =>
  unwrap(client.api.GET("/webhooks/{id}", { params: { path: { id } } }));
export const createWebhook = (name, url, eventTypes, authHeader, options = {}) =>
  unwrap(
    client.api.POST("/webhooks", {
      body: {
        name,
        url,
        event_types: eventTypes,
        auth_header: authHeader || null,
        enabled: options.enabled !== false,
        max_retries: options.maxRetries || 5,
        timeout_seconds: options.timeoutSeconds || 30,
      },
    }),
  );
export const updateWebhook = (id, updates) =>
  unwrap(
    client.api.PUT("/webhooks/{id}", {
      params: { path: { id } },
      body: updates,
    }),
  );
export const deleteWebhook = (id) =>
  unwrap(client.api.DELETE("/webhooks/{id}", { params: { path: { id } } }));
export const getWebhookEventTypes = () =>
  unwrap(client.api.GET("/webhooks/event-types"));
export const getWebhookDeliveries = (id, status, limit) => {
  const query = {};
  if (status) query.status = status;
  if (limit) query.limit = limit;
  return unwrap(
    client.api.GET("/webhooks/{id}/deliveries", {
      params: { path: { id }, query },
    }),
  );
};

// =============================================================================
// Non-v1 endpoints (raw fetch, intentionally outside the SDK surface)
// =============================================================================

// Prometheus metrics endpoint — text/plain, not part of the v1 spec.
export const getMetrics = async () => {
  const res = await fetch(`${BROKER_URL}/metrics`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.text();
};

// Webhook catcher (demo-only test sink, runs in docker-compose on port 8090).
// Not part of the broker; uses bare fetch.
const WEBHOOK_CATCHER_URL = "http://localhost:8090";

export const getWebhookCatcherStats = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

export const clearWebhookCatcher = async () => {
  const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`, {
    method: "DELETE",
  });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.json();
};

// =============================================================================
// Demo Build YAML constant (unchanged)
// =============================================================================

// Shipwright Build YAML for the demo.
//
// The `spec.output.image` field is required by the v1beta1 Build CRD; without
// it the k8s API rejects the resource with a 422 ("spec.output: Required
// value"). We use ttl.sh as an ephemeral registry — images auto-delete after
// 1h, no credentials required, matches what `examples/work-orders/simple-build.yaml`
// and the helm e2e suite use.
export const getDemoBuildYaml = () => `
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: demo-build
  namespace: default
spec:
  source:
    type: Git
    git:
      url: https://github.com/shipwright-io/sample-go
      revision: main
    contextDir: docker-build
  strategy:
    name: kaniko
    kind: ClusterBuildStrategy
  output:
    image: ttl.sh/brokkr-demo:1h
  timeout: 15m`;

// =============================================================================
// Additional broker delete operations (v1, through the SDK)
// =============================================================================

export const deleteStack = (id) =>
  unwrap(client.api.DELETE("/stacks/{id}", { params: { path: { id } } }));
export const deleteAgent = (id) =>
  unwrap(client.api.DELETE("/agents/{id}", { params: { path: { id } } }));
export const deleteGenerator = (id) =>
  unwrap(client.api.DELETE("/generators/{id}", { params: { path: { id } } }));

// =============================================================================
// Demo helpers (pure JS — no broker calls)
// =============================================================================

export const createBuildWorkOrder = async (
  imageTag = "latest",
  agentId = null,
) => {
  const buildYaml = getDemoBuildYaml();
  const targeting = agentId ? { agent_ids: [agentId] } : { labels: [] };
  return createWorkOrder("build", buildYaml, targeting, {
    maxRetries: 3,
    backoffSeconds: 30,
  });
};

export const getWebhookCatcherDeploymentYaml = (imageTag = "latest") => `
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webhook-catcher
  namespace: default
  labels:
    app: webhook-catcher
spec:
  replicas: 1
  selector:
    matchLabels:
      app: webhook-catcher
  template:
    metadata:
      labels:
        app: webhook-catcher
    spec:
      containers:
      - name: webhook-catcher
        image: registry:5000/webhook-catcher:${imageTag}
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: webhook-catcher
  namespace: default
spec:
  selector:
    app: webhook-catcher
  ports:
  - port: 8080
    targetPort: 8080
`.trim();

// Parse Prometheus metrics text format into an object.
export const parseMetrics = (metricsText) => {
  const metrics = {};
  const lines = metricsText.split("\n");
  for (const line of lines) {
    if (line.startsWith("#") || !line.trim()) continue;
    const match = line.match(/^([a-zA-Z_:][a-zA-Z0-9_:]*)(\{[^}]*\})?\s+(.+)$/);
    if (match) {
      const [, name, labelsStr, value] = match;
      const labels = {};
      if (labelsStr) {
        const labelMatches = labelsStr.matchAll(
          /([a-zA-Z_][a-zA-Z0-9_]*)="([^"]*)"/g,
        );
        for (const [, key, val] of labelMatches) {
          labels[key] = val;
        }
      }
      if (!metrics[name]) metrics[name] = [];
      metrics[name].push({ labels, value: parseFloat(value) });
    }
  }
  return metrics;
};

// Check environment prerequisites for the demo.
// `/healthz` is outside the v1 spec; uses bare fetch.
export const checkEnvironment = async () => {
  const result = {
    brokerHealthy: false,
    integrationAgent: null,
    shipwrightReady: false,
    buildStrategies: 0,
    errors: [],
  };
  try {
    const res = await fetch(`${BROKER_URL}/healthz`);
    result.brokerHealthy = res.ok;
  } catch (e) {
    result.errors.push("Broker API not reachable");
  }
  try {
    const agents = await getAgents();
    result.integrationAgent = agents.find(
      (a) => a.name === "brokkr-integration-test-agent",
    );
    if (!result.integrationAgent) {
      result.errors.push("Integration test agent not found");
    }
  } catch (e) {
    result.errors.push("Failed to fetch agents: " + e.message);
  }
  if (result.integrationAgent?.status === "ACTIVE") {
    result.shipwrightReady = true;
    result.buildStrategies = 3;
  }
  return result;
};

export const getWebhookCatcherEvents = async () => {
  try {
    const res = await fetch(`${WEBHOOK_CATCHER_URL}/messages`);
    if (!res.ok) return { count: 0, messages: [] };
    return res.json();
  } catch (e) {
    return { count: 0, messages: [] };
  }
};

export const pollForCondition = async (
  checkFn,
  intervalMs = 2000,
  timeoutMs = 60000,
) => {
  const startTime = Date.now();
  while (Date.now() - startTime < timeoutMs) {
    const result = await checkFn();
    if (result.done) return result;
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  throw new Error("Polling timed out");
};

export const pollAgentStatus = async (agentId, timeoutMs = 120000) => {
  return pollForCondition(
    async () => {
      try {
        const agents = await getAgents();
        const agent = agents.find((a) => a.id === agentId);
        if (!agent) return { done: false, agent: null };
        if (agent.status === "ACTIVE" && agent.last_heartbeat) {
          const heartbeatAge =
            Date.now() - new Date(agent.last_heartbeat).getTime();
          if (heartbeatAge < 60000) {
            return { done: true, agent };
          }
        }
        return { done: false, agent };
      } catch (e) {
        return { done: false, agent: null, error: e.message };
      }
    },
    3000,
    timeoutMs,
  );
};

export const pollWorkOrderStatus = async (workOrderId, timeoutMs = 300000) => {
  return pollForCondition(
    async () => {
      try {
        const wo = await getWorkOrder(workOrderId);
        const terminalStates = ["COMPLETED", "FAILED", "CANCELLED"];
        if (terminalStates.includes(wo.status)) {
          return { done: true, workOrder: wo };
        }
        return { done: false, workOrder: wo };
      } catch (e) {
        return { done: false, workOrder: null, error: e.message };
      }
    },
    3000,
    timeoutMs,
  );
};

// Orchestrated cleanup of demo resources.
export const cleanupDemo = async (resources, onProgress) => {
  const log = (step, status) => onProgress?.(step, status);
  const errors = [];

  if (resources.agentTargets?.length) {
    log("Removing agent targets...", "active");
    for (const target of resources.agentTargets) {
      try {
        await removeAgentTarget(target.agentId, target.stackId);
      } catch (e) {
        errors.push(`Failed to remove target: ${e.message}`);
      }
    }
    log("Agent targets removed", "done");
  }

  if (resources.webhookIds?.length) {
    log("Removing webhook subscriptions...", "active");
    for (const id of resources.webhookIds) {
      try {
        await deleteWebhook(id);
      } catch (e) {
        errors.push(`Failed to delete webhook ${id}: ${e.message}`);
      }
    }
    log("Webhooks removed", "done");
  }

  if (resources.deploymentIds?.length) {
    log("Creating deletion markers...", "active");
    for (const deployment of resources.deploymentIds) {
      try {
        await createDeployment(deployment.stackId, deployment.yaml, true);
      } catch (e) {
        errors.push(`Failed to create deletion marker: ${e.message}`);
      }
    }
    log("Deletion markers created", "done");
  }

  if (resources.stackIds?.length) {
    log("Deleting stacks...", "active");
    for (const id of resources.stackIds) {
      try {
        await deleteStack(id);
      } catch (e) {
        errors.push(`Failed to delete stack ${id}: ${e.message}`);
      }
    }
    log("Stacks deleted", "done");
  }

  if (resources.templateIds?.length) {
    log("Deleting templates...", "active");
    for (const id of resources.templateIds) {
      try {
        await deleteTemplate(id);
      } catch (e) {
        errors.push(`Failed to delete template ${id}: ${e.message}`);
      }
    }
    log("Templates deleted", "done");
  }

  if (resources.agentIds?.length) {
    log("Deactivating demo agents...", "active");
    for (const id of resources.agentIds) {
      try {
        await updateAgent(id, { status: "DEACTIVATED" });
      } catch (e) {
        errors.push(`Failed to deactivate agent ${id}: ${e.message}`);
      }
    }
    log("Demo agents deactivated", "done");
  }

  log("Clearing webhook catcher...", "active");
  try {
    await clearWebhookCatcher();
  } catch (e) {
    // not critical
  }
  log("Webhook catcher cleared", "done");

  return { success: errors.length === 0, errors };
};
