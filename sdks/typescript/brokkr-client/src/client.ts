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
  type BrokkrApi,
  createBrokkrClient,
} from "./index.js";
import { BrokkrError } from "./error.js";

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

  constructor(options: BrokkrClientOptions) {
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
