/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 */

import type { ErrorResponse } from "./index.js";

/** HTTP status codes worth retrying — transient failures only. Mirrors the
 * Rust (T-C1) and Python (T-C2) wrappers exactly. */
const RETRYABLE_STATUSES = new Set([408, 429, 502, 503, 504]);

/** Public-facing error type for the Brokkr SDK wrapper.
 *
 * Pattern-match on `code` (machine-readable, stable across versions) rather
 * than `message` (human-readable, not stable). The original typed
 * `ErrorResponse` body, when one was returned, is preserved on `response`.
 */
export class BrokkrError extends Error {
  readonly code: string | undefined;
  readonly status: number | undefined;
  readonly response: ErrorResponse | undefined;

  constructor(args: {
    message: string;
    code?: string;
    status?: number;
    response?: ErrorResponse;
  }) {
    super(args.message);
    this.name = "BrokkrError";
    this.code = args.code;
    this.status = args.status;
    this.response = args.response;
    // Preserve the prototype chain when transpiled.
    Object.setPrototypeOf(this, BrokkrError.prototype);
  }

  /** Whether this error qualifies for the wrapper's `retry()` helper.
   *  Transport errors with unknown status default to retryable, matching the
   *  Python wrapper. */
  isRetryable(): boolean {
    if (this.status === undefined) return true;
    return RETRYABLE_STATUSES.has(this.status);
  }

  /** Build from a typed `ErrorResponse` body and HTTP status. */
  static fromResponse(response: ErrorResponse, status: number): BrokkrError {
    return new BrokkrError({
      message: response.message,
      code: response.code,
      status,
      response,
    });
  }

  /** Build from a raw `fetch` / network failure (no HTTP response). */
  static fromTransport(cause: unknown): BrokkrError {
    const message =
      cause instanceof Error ? cause.message : String(cause);
    return new BrokkrError({ message });
  }

  /** Build from an openapi-fetch `{ error, response }` tuple — the most
   *  common construction path used by the wrapper internally. */
  static fromOpenapiFetch(
    error: unknown,
    response: Response,
  ): BrokkrError {
    if (
      error &&
      typeof error === "object" &&
      "code" in error &&
      "message" in error
    ) {
      return BrokkrError.fromResponse(error as ErrorResponse, response.status);
    }
    return new BrokkrError({
      message:
        typeof error === "object" && error !== null
          ? JSON.stringify(error)
          : String(error ?? `HTTP ${response.status}`),
      status: response.status,
    });
  }
}
