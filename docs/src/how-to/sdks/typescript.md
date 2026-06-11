# TypeScript SDK

The `@colliery-io/brokkr-client` package combines a fully typed `openapi-fetch` client (types generated from `openapi/brokkr-v1.json` by `openapi-typescript`) with an ergonomic `BrokkrClient` wrapper that adds a single-credential constructor, a typed `BrokkrError`, an opt-in retry helper, and convenience methods for the stack-telemetry surface.

## Install

```bash
npm install @colliery-io/brokkr-client
```

For in-tree workspace development, build from the local checkout:

```bash
npm --prefix sdks/typescript/brokkr-client install
npm --prefix sdks/typescript/brokkr-client run build
```

## Construct a client

```typescript
import { BrokkrClient } from "@colliery-io/brokkr-client";

const client = new BrokkrClient({
  baseUrl: "https://broker.example.com/api/v1",
  token: "brokkr_BRabcd1234_AgentLongTokenExample0001", // PAK
});
```

Options (`BrokkrClientOptions` in `sdks/typescript/brokkr-client/src/client.ts`): `baseUrl` (conventionally includes `/api/v1`), optional `token` (injected as `Authorization: Bearer <token>` on every request), `requestTimeoutMs` (default 30000), `maxRetries` (default 3), and `initialBackoffMs` (default 200).

## Call one endpoint

The typed `openapi-fetch` client is reachable via `client.api`. Every operation is a `GET`/`POST`/`PUT`/`DELETE` call keyed by path:

```typescript
const { data, error, response } = await client.api.GET("/agents", {});
if (data) {
  console.log(`${data.length} agents`);
}
```

## Submit a folder of manifests (Node)

You usually have a *folder* of manifests, not a hand-built YAML blob. `submitManifests` reads the folder (top-level `*.yaml`/`*.yml`, sorted) or a single file, concatenates it into one multi-document stream, validates each document has `apiVersion`+`kind`, and submits it as a new deployment object on an existing stack. These helpers are **Node-only** (they read the filesystem via dynamic imports, so the browser bundle stays clean):

```typescript
const object = await client.submitManifests(stackId, "./manifests");
console.log("submitted revision", object.sequence_id);
```

For the control-plane loop, `apply` is idempotent: it creates the stack by name if needed, applies targeting labels for fan-out, and submits a new revision **only when the bundle changed**. It requires a generator PAK (the stack is owned by that generator):

```typescript
import type { ApplyResult } from "@colliery-io/brokkr-client";

const result: ApplyResult = await client.apply("payments", "./manifests", ["env:prod", "region:us"]);
console.log(result.status); // "created" | "updated" | "unchanged"
```

A stack's desired state is the single latest deployment object, and the agent reconciles + prunes — so removing a file and re-applying deletes that resource on the next reconcile. Ordering is forgiving: the agent front-loads `Namespace`/`CustomResourceDefinition` objects.

## Handle one error

`BrokkrError` carries the broker's stable wire `code` plus the HTTP `status` and the typed `ErrorResponse` body:

```typescript
import { BrokkrError } from "@colliery-io/brokkr-client";

try {
  const agent = await client.retry((api) =>
    api.GET("/agents/{id}", { params: { path: { id: agentId } } }),
  );
} catch (err) {
  if (err instanceof BrokkrError && err.code === "agent_not_found") {
    console.log("no such agent");
  } else {
    throw err;
  }
}
```

See [stable error codes](../../reference/error-codes.md) for the full list. `err.isRetryable()` returns `true` for transport failures and HTTP `408/429/502/503/504`.

## Retry on transient failures

`client.retry` re-runs a closure with exponential backoff (200 ms initial, doubling, capped at 10 s; 3 retries by default), unwraps the `openapi-fetch` result tuple, and throws `BrokkrError` on failure. Wrap only operations that are safe to repeat — typically idempotent GETs.

## Stack telemetry and live tail

The wrapper has dedicated methods for the telemetry surface:

```typescript
// The same PAK used to construct the client
const pak = "brokkr_BRabcd1234_AgentLongTokenExample0001";

// Retained history (6-hour ceiling); responses include retention metadata
const logs = await client.listTelemetryLogs(stackId, { limit: 1000 });
const events = await client.listTelemetryEvents(stackId, {
  since: "2026-06-09T12:00:00Z",
});

// Admin-only snapshot of connected agents on the internal WS channel
const conns = await client.listWsConnections();

// Compute the live-tail WebSocket URL (http→ws, https→wss)
const ws = new WebSocket(client.liveSubscriptionUrl(stackId), [
  `brokkr.pak.${pak}`, // browser auth subprotocol; use a header in Node
  "brokkr.v1",
]);
```

`liveSubscriptionUrl` only computes the URL — you bring the `WebSocket` implementation for your runtime (browser global, or the `ws` package in Node, where you can pass the `Authorization` header instead of the subprotocol). Frame shapes are documented in the [WebSocket Protocol reference](../../reference/ws-protocol.md).

## When you need to drop to the raw client

For anything the wrapper doesn't cover, use `client.api` directly — it is the same typed `openapi-fetch` client and still injects the auth header. The generated types live in `sdks/typescript/brokkr-client/src/schema.d.ts` (never hand-edit; regenerate with `angreal openapi gen-typescript`).
