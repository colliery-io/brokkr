/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

/**
 * Ergonomic wrapper around the `openapi-fetch` typed client (from
 * `createBrokkrClient`).
 *
 * Mirrors the Rust (`crates/brokkr-client::BrokkrClient`) and Python
 * (`sdks/python/brokkr` `BrokkrClient`) wrappers so consumers crossing
 * languages see the same contract: one credential, opt-in retry, typed
 * `BrokkrError`. Pagination iterators are intentionally absent — no v1
 * endpoint paginates.
 */

import {
  type AuthResponse,
  type BrokkrApi,
  type DeploymentObject,
  type K8sEventHistoryResponse,
  type PodLogHistoryResponse,
  type Stack,
  type WsConnectionsResponse,
  createBrokkrClient,
} from "./index.js";
import { BrokkrError } from "./error.js";

/** Outcome of {@link BrokkrClient.apply}. */
export type ApplyResult =
  | { status: "created"; deploymentObject: DeploymentObject }
  | { status: "updated"; deploymentObject: DeploymentObject }
  | { status: "unchanged" };

export interface TelemetryHistoryQuery {
  /** Earliest `created_at` to include (ISO-8601). Clamped server-side
   * to the 6h retention ceiling. */
  since?: string;
  /** Maximum rows to return. Server default 500, capped at 5000. */
  limit?: number;
}

export interface BrokkrClientOptions {
  baseUrl: string;
  /** PAK; injected as `Authorization: Bearer <token>` on every request. */
  token?: string;
  /** Per-request timeout. Default 30s. */
  requestTimeoutMs?: number;
  /** Max retry attempts for `retry()`. Default 3 (= 1 initial + 3 retries). */
  maxRetries?: number;
  /** Initial backoff for `retry()`. Default 200ms. */
  initialBackoffMs?: number;
}

const DEFAULTS = {
  requestTimeoutMs: 30_000,
  maxRetries: 3,
  initialBackoffMs: 200,
  maxBackoffMs: 10_000,
} as const;

/** Result tuple returned by every openapi-fetch method. */
type FetchResult<T> = {
  data?: T;
  error?: unknown;
  response: Response;
};

export class BrokkrClient {
  readonly api: BrokkrApi;
  readonly maxRetries: number;
  readonly initialBackoffMs: number;
  readonly baseUrl: string;

  constructor(options: BrokkrClientOptions) {
    this.baseUrl = options.baseUrl;
    const maxRetries = options.maxRetries ?? DEFAULTS.maxRetries;
    const initialBackoffMs =
      options.initialBackoffMs ?? DEFAULTS.initialBackoffMs;
    if (maxRetries < 0) {
      throw new RangeError("maxRetries must be >= 0");
    }
    if (initialBackoffMs <= 0) {
      throw new RangeError("initialBackoffMs must be > 0");
    }

    const headers: Record<string, string> = {};
    if (options.token !== undefined) {
      headers["Authorization"] = `Bearer ${options.token}`;
    }

    const timeoutMs = options.requestTimeoutMs ?? DEFAULTS.requestTimeoutMs;
    const customFetch: typeof fetch = (input, init) => {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), timeoutMs);
      const signal = init?.signal
        ? mergeSignals([init.signal, controller.signal])
        : controller.signal;
      return fetch(input, { ...init, signal }).finally(() =>
        clearTimeout(timer),
      );
    };

    this.api = createBrokkrClient({
      baseUrl: options.baseUrl,
      headers,
      fetch: customFetch,
    });
    this.maxRetries = maxRetries;
    this.initialBackoffMs = initialBackoffMs;
  }

  // -------------------------------------------------------------------
  // Ergonomic methods for the internal-WS-channel surface
  // (BROKKR-I-0019). These wrap the openapi-fetch builders for the
  // most-common calls. The retention metadata in the responses is
  // part of the typed return — surface it in any UI that consumes
  // this SDK per ADR-0008 / project_log_retention_stance.
  // -------------------------------------------------------------------

  /** Paginated kube-event history for a stack (6h window). */
  async listTelemetryEvents(
    stackId: string,
    query: TelemetryHistoryQuery = {},
  ): Promise<K8sEventHistoryResponse> {
    return this.retry<K8sEventHistoryResponse>((api) =>
      api.GET("/stacks/{id}/events", {
        params: { path: { id: stackId }, query },
      }),
    );
  }

  /** Paginated pod-log history for a stack (6h window). */
  async listTelemetryLogs(
    stackId: string,
    query: TelemetryHistoryQuery = {},
  ): Promise<PodLogHistoryResponse> {
    return this.retry<PodLogHistoryResponse>((api) =>
      api.GET("/stacks/{id}/logs", {
        params: { path: { id: stackId }, query },
      }),
    );
  }

  /** Admin-only snapshot of currently-connected agents on the
   * internal WS channel. For continuous monitoring prefer scraping
   * the `brokkr_ws_connected_agents` Prometheus gauge. */
  async listWsConnections(): Promise<WsConnectionsResponse> {
    return this.retry<WsConnectionsResponse>((api) =>
      api.GET("/admin/ws/connections", {}),
    );
  }

  // -------------------------------------------------------------------
  // Manifest submission helpers (BROKKR-I-0021). Node-only: they read the
  // filesystem via dynamic imports so the browser bundle stays clean.
  // -------------------------------------------------------------------

  /**
   * Read a folder (top-level `*.yaml`/`*.yml`, sorted) or a single file of
   * manifests, concatenate into one multi-document YAML stream, and submit it
   * as a new deployment object on an existing stack. Node-only.
   */
  async submitManifests(
    stackId: string,
    path: string,
  ): Promise<DeploymentObject> {
    const yaml = await readManifests(path);
    // Single attempt: submitting a deployment object is not idempotent, so a
    // retry after a lost response could double-submit a revision.
    const res = await this.api.POST("/stacks/{id}/deployment-objects", {
      params: { path: { id: stackId } },
      body: { yaml_content: yaml, is_deletion_marker: false },
    });
    if (res.error !== undefined) {
      throw BrokkrError.fromOpenapiFetch(res.error, res.response);
    }
    return res.data as DeploymentObject;
  }

  /**
   * Idempotently make a folder of manifests the desired state of the stack
   * named `stackName`, creating the stack if needed, applying `targeting`
   * labels for fan-out, and submitting a new revision only when the bundle
   * changed. Requires a generator PAK. Node-only.
   */
  async apply(
    stackName: string,
    path: string,
    targeting: string[] = [],
  ): Promise<ApplyResult> {
    const yaml = await readManifests(path);
    const checksum = await sha256Hex(yaml);

    // POST /auth/pak is a pure read (verify the PAK) with no side effect, so
    // retrying it is safe.
    const auth = await this.retry<AuthResponse>((api) =>
      api.POST("/auth/pak", {}),
    );
    const generatorId = auth.generator;
    if (!generatorId) {
      throw new BrokkrError({
        message:
          "apply by name requires a generator PAK; admin callers should create the stack explicitly and use submitManifests",
      });
    }

    const stacks = await this.retry<Stack[]>((api) => api.GET("/stacks", {}));
    let stack = stacks.find((s) => s.name === stackName);
    if (!stack) {
      // Single attempt: creating a stack is not idempotent.
      const stackRes = await this.api.POST("/stacks", {
        body: { name: stackName, generator_id: generatorId },
      });
      if (stackRes.error !== undefined) {
        throw BrokkrError.fromOpenapiFetch(stackRes.error, stackRes.response);
      }
      stack = stackRes.data as Stack;
    }
    const stackId = stack.id;

    // Targeting labels: a 409 means the label is already present — fine.
    for (const label of targeting) {
      const res = await this.api.POST("/stacks/{id}/labels", {
        params: { path: { id: stackId } },
        body: label,
      });
      if (res.error !== undefined && res.response.status !== 409) {
        throw BrokkrError.fromOpenapiFetch(res.error, res.response);
      }
    }

    const objects = await this.retry<DeploymentObject[]>((api) =>
      api.GET("/stacks/{id}/deployment-objects", {
        params: { path: { id: stackId } },
      }),
    );
    const latest = objects.reduce<DeploymentObject | undefined>(
      (acc, o) => (acc && acc.sequence_id >= o.sequence_id ? acc : o),
      undefined,
    );
    if (latest && latest.yaml_checksum === checksum) {
      return { status: "unchanged" };
    }
    const hadPrior = objects.length > 0;

    // Single attempt: submitting a revision is not idempotent.
    const objRes = await this.api.POST("/stacks/{id}/deployment-objects", {
      params: { path: { id: stackId } },
      body: { yaml_content: yaml, is_deletion_marker: false },
    });
    if (objRes.error !== undefined) {
      throw BrokkrError.fromOpenapiFetch(objRes.error, objRes.response);
    }
    const object = objRes.data as DeploymentObject;
    return hadPrior
      ? { status: "updated", deploymentObject: object }
      : { status: "created", deploymentObject: object };
  }

  /**
   * Open a live WebSocket subscription to a stack's event + log tail.
   * The URL is computed from the configured `baseUrl` (http→ws,
   * https→wss). The caller is responsible for providing a
   * `WebSocket` constructor compatible with their runtime (browser:
   * `globalThis.WebSocket`; node: `ws` package).
   *
   * Frames are `WsMessage` JSON-encoded text — see the broker docs
   * for the wire schema. Lagged subscribers receive a
   * `log_gap` frame so the UI can render a visible gap.
   */
  liveSubscriptionUrl(stackId: string): string {
    // The broker mounts the live subscription at /api/v1/stacks/{id}/live,
    // even though the OpenAPI schema strips that prefix from operation
    // paths. baseUrl conventionally includes /api/v1 (matching how
    // BrokkrClient is constructed everywhere else), so strip it once
    // before re-appending the canonical full path.
    const trimmed = this.baseUrl.replace(/\/+$/, "");
    const root = trimmed.endsWith("/api/v1")
      ? trimmed.slice(0, -"/api/v1".length)
      : trimmed;
    const wsRoot = root.replace(/^http:/, "ws:").replace(/^https:/, "wss:");
    return `${wsRoot}/api/v1/stacks/${stackId}/live`;
  }

  /**
   * Run `op(api)` with exponential backoff on retryable failures.
   *
   * `op` is an async callback that takes the generated client and returns the
   * operation's parsed `data`. The wrapper unwraps the `{ data, error,
   * response }` tuple: when `error` is set or `data` is undefined despite a
   * non-success status, it builds a `BrokkrError` and decides whether to
   * retry.
   *
   * Non-idempotent POSTs should generally NOT be wrapped. Callers opt in per
   * call.
   */
  async retry<T>(op: (api: BrokkrApi) => Promise<FetchResult<T>>): Promise<T> {
    let attempt = 0;
    // We loop instead of recursing so stack depth stays constant for large maxRetries.
    // eslint-disable-next-line no-constant-condition
    while (true) {
      let result: FetchResult<T> | undefined;
      let transportErr: unknown;
      try {
        result = await op(this.api);
      } catch (e) {
        transportErr = e;
      }

      const err = classify(result, transportErr);
      if (!err) {
        // Type assertion: classify returns undefined only when data is set.
        return result!.data as T;
      }

      if (!err.isRetryable() || attempt >= this.maxRetries) {
        throw err;
      }

      const backoff = Math.min(
        this.initialBackoffMs * 2 ** attempt,
        DEFAULTS.maxBackoffMs,
      );
      await sleep(backoff);
      attempt += 1;
    }
  }
}

function classify<T>(
  result: FetchResult<T> | undefined,
  transportErr: unknown,
): BrokkrError | undefined {
  if (transportErr !== undefined) {
    return BrokkrError.fromTransport(transportErr);
  }
  if (!result) {
    // Shouldn't happen — classify is only called after the try/catch — but be
    // defensive.
    return new BrokkrError({ message: "no response" });
  }
  if (result.error !== undefined) {
    return BrokkrError.fromOpenapiFetch(result.error, result.response);
  }
  if (!result.response.ok) {
    return new BrokkrError({
      message: `HTTP ${result.response.status}`,
      status: result.response.status,
    });
  }
  if (result.data === undefined) {
    // Some operations legitimately return undefined data (e.g. 204 No
    // Content). Treat as success — but the caller's `T` annotation needs to
    // tolerate `undefined`.
    return undefined;
  }
  return undefined;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Merge multiple AbortSignals into one. Aborted by the first to abort. */
function mergeSignals(signals: AbortSignal[]): AbortSignal {
  const controller = new AbortController();
  for (const sig of signals) {
    if (sig.aborted) {
      controller.abort(sig.reason);
      return controller.signal;
    }
    sig.addEventListener("abort", () => controller.abort(sig.reason), {
      once: true,
    });
  }
  return controller.signal;
}

// -------------------------------------------------------------------
// Node-only manifest helpers (BROKKR-T-0197). Dynamic imports keep
// `node:*` out of the top-level module graph so browser bundles that
// never call submitManifests/apply stay clean.
// -------------------------------------------------------------------

/**
 * Read a manifest path into one validated multi-document YAML stream. `path`
 * may be a single file or a directory; for a directory, top-level
 * `*.yaml`/`*.yml` files are concatenated in sorted-name order. Each non-empty
 * document is checked for `apiVersion` and `kind` (the broker validates the
 * full parse on ingest).
 */
export async function readManifests(path: string): Promise<string> {
  const fs = await import("node:fs/promises");
  const nodePath = await import("node:path");

  const stat = await fs.stat(path).catch(() => undefined);
  if (!stat) {
    throw new BrokkrError({ message: `path not found: ${path}` });
  }

  let files: string[];
  if (stat.isDirectory()) {
    const entries = await fs.readdir(path);
    files = entries
      .filter((name) => name.endsWith(".yaml") || name.endsWith(".yml"))
      .sort()
      .map((name) => nodePath.join(path, name));
  } else {
    files = [path];
  }
  if (files.length === 0) {
    throw new BrokkrError({
      message: `no .yaml/.yml manifests found in ${path}`,
    });
  }

  const parts: string[] = [];
  for (const file of files) {
    const content = await fs.readFile(file, "utf8");
    for (const doc of content.split(/^---\s*$/m)) {
      if (doc.trim() === "") continue;
      const hasApiVersion = /^apiVersion:/m.test(doc);
      const hasKind = /^kind:/m.test(doc);
      if (!hasApiVersion || !hasKind) {
        throw new BrokkrError({
          message: `${file}: every manifest document must have apiVersion and kind`,
        });
      }
    }
    parts.push(content.replace(/\s+$/, ""));
  }
  return `${parts.join("\n---\n")}\n`;
}

/**
 * Lowercase hex SHA-256, matching the broker's deployment-object checksum so
 * {@link BrokkrClient.apply} can detect an unchanged bundle. Node-only.
 */
export async function sha256Hex(content: string): Promise<string> {
  const { createHash } = await import("node:crypto");
  return createHash("sha256").update(content, "utf8").digest("hex");
}
