# @colliery-io/brokkr-client

TypeScript client for the Brokkr broker API.

```bash
npm install @colliery-io/brokkr-client
```

The package has two layers:

- `BrokkrClient`, an ergonomic wrapper. It sends your PAK on every request,
  gives typed `BrokkrError` errors with a stable `code`, has an opt-in
  `retry` helper, and has `apply` for a folder of manifests.
- `client.api`, a typed `openapi-fetch` client. `openapi-typescript`
  generates its types from the broker's OpenAPI specification. Use it for any
  operation that the wrapper does not cover.

## Version

The SDK version matches the broker version. Install the release that has the
same version as your broker. See the
[releases page](https://github.com/colliery-io/brokkr/releases).

## First deployment

This example makes a folder of manifests the desired state of a stack, then
prints the result. It uses a **generator PAK**. A generator owns the stacks
that it creates. An admin can create a generator with `brokkr-broker create
generator --name <name>` or `POST /api/v1/generators`. The broker shows the
PAK only once.

Put one manifest in a folder:

```yaml
# manifests/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hello-brokkr
  namespace: default
data:
  greeting: hello
```

Then apply the folder (Node, ES module):

```typescript
import { BrokkrClient } from "@colliery-io/brokkr-client";

const client = new BrokkrClient({
  baseUrl: "http://localhost:3000", // broker URL; the client adds /api/v1
  token: "<generator PAK>",
});

const result = await client.apply("hello-brokkr", "./manifests", ["env:dev"]);
console.log("status:", result.status); // created | updated | unchanged
if (result.status !== "unchanged") {
  console.log("stack:", result.deploymentObject.stack_id);
  console.log("sequence id:", result.deploymentObject.sequence_id);
}
```

The base URL can be the broker root (`http://localhost:3000`) or end in
`/api/v1`. The client adds `/api/v1` when it is absent and never adds it twice.
The `brokkr` CLI uses the same rule.

`apply` creates the stack if it does not exist, adds the targeting labels
(`env:dev`), and submits a new revision only when the folder changed. Run it
again with no change and the status is `unchanged`. Agents whose labels match
the stack's targeting deploy the revision. `apply` and `submitManifests` read
files, so they work only in Node.

With an admin PAK, give the generator that owns the stack as the fourth
argument: `client.apply("hello-brokkr", "./manifests", [], "<generator name>")`.

## More

For errors, retry, telemetry and the live-tail URL, see the
[TypeScript SDK documentation](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/sdks/typescript.md).
