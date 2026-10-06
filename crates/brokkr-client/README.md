# brokkr-client

Rust client for the Brokkr broker API.

```bash
cargo add brokkr-client
cargo add tokio --features macros,rt-multi-thread
```

The crate has two layers:

- `BrokkrClient`, an ergonomic wrapper. It sends your PAK on every request,
  gives typed `BrokkrError` errors with a stable `code()`, has an opt-in
  `retry` helper, and has `apply` for a folder of manifests.
- `client.api()`, the client that `progenitor` generates from the broker's
  OpenAPI specification. Use it for any operation that the wrapper does not
  cover.

## Version

The crate version matches the broker version. Use the release that has the
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

Then apply the folder:

```rust
use brokkr_client::{ApplyOutcome, BrokkrClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BrokkrClient::builder("http://localhost:3000") // the client adds /api/v1
        .token("<generator PAK>")
        .build()?;

    let targeting = vec!["env:dev".to_string()];
    match client.apply("hello-brokkr", "./manifests", &targeting).await? {
        ApplyOutcome::Created(obj) | ApplyOutcome::Updated(obj) => {
            println!("status: changed");
            println!("stack: {}", obj.stack_id);
            println!("sequence id: {}", obj.sequence_id);
        }
        ApplyOutcome::Unchanged => println!("status: unchanged"),
    }
    Ok(())
}
```

The base URL can be the broker root (`http://localhost:3000`) or end in
`/api/v1`. The client adds `/api/v1` when it is absent and never adds it twice.
The `brokkr` CLI uses the same rule.

`apply` creates the stack if it does not exist, adds the targeting labels
(`env:dev`), and submits a new revision only when the folder changed. Run it
again with no change and the result is `Unchanged`. Agents whose labels match
the stack's targeting deploy the revision.

With an admin PAK, use `apply_for_generator` and name the generator that owns
the stack:
`client.apply_for_generator("<generator name>", "hello-brokkr", "./manifests", &[]).await?`.

## More

For errors, retry and the raw client, see the
[Rust SDK documentation](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/sdks/rust.md).
