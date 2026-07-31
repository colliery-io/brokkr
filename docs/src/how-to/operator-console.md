# Using the Operator Console

The Operator Console is the read-only web view of a Brokkr deployment. It is served by the broker itself, at the broker's own root URL, and it needs no credential from you — open the address in a browser and you are looking at the fleet.

That convenience is also the thing to understand before you expose it. **The page hands every visitor a working read-only administrative credential.** Whoever can reach the broker's port can read the whole deployment. This guide covers how to open the console, what each view shows, how to scope it to one tenant, and — first — what its authentication model means for you.

## Before You Start

- **A broker built with the console.** The published broker image includes it. A broker you compiled yourself from a plain `cargo build` does not: it serves a short placeholder page saying the console was not compiled in, and its JSON API works normally. See [Local Development Environment](../getting-started/development.md) if you want the console in a locally built broker.
- **Network access to the broker's HTTP port** (`3000` by default) from your browser. That is the same port the API listens on; there is no separate UI port and no separate UI container.
- A modern browser. The console is a WebAssembly application and will not run with scripting disabled.

## Step 1: Open the Console

Browse to the broker's root URL:

```
http://<broker-host>:3000/
```

If the broker is inside a cluster and not exposed, a port-forward is the safest way to reach it:

```bash
kubectl port-forward -n brokkr svc/brokkr-broker 3000:3000
```

Then open <http://localhost:3000/>.

You are not asked to sign in or to paste a key. The console loads and immediately begins reading the API.

Every path the API does not own falls back to the console, so bookmarking any console path works. Paths under `/api` and `/internal` always belong to the API and return a normal 404 when they do not exist — the console never shadows them.

## Step 2: Know What the Console Is Authenticating As

This is the part worth reading before anyone else gets the URL.

Each time a broker process starts, it mints one random read-only credential and holds it in memory. It is never written to the database, never logged, and cannot be rotated or retrieved through the CLI; restarting the broker simply replaces it. When the broker serves the console page, it embeds that credential in the HTML. The console reads it out of the page and uses it for every API call it makes.

The consequences follow directly:

- **Network reachability of the broker URL is the console's entire authentication boundary.** Anyone who can fetch the page holds a working read-only admin credential. There is no second factor, no login, and no per-user identity.
- **That credential sees everything readable, not just what the console draws.** It satisfies admin-level read checks across all tenants — the fleet rollup, every tenant's stacks, telemetry, work orders, and the [audit log](../reference/audit-logs.md), which the console itself has no screen for. Someone who copies the token out of the page can query any of it directly.
- **It cannot change anything.** Reads (`GET`/`HEAD`) are allowed anywhere. Exactly two `POST` routes are allowlisted because they do not alter desired state: credential introspection, and requesting a diagnostic for a deployment object. Every other mutation is refused with `403 Forbidden`. If you see a 403 from the console, that is the design working, not a bug.
- **Builds without the embedded console carry no such credential at all** — the placeholder page has nothing to hand out.

**Mitigate this at the network layer.** Restrict who can reach port `3000` with a NetworkPolicy or firewall rules; if the console must be available beyond the cluster, put authentication in front of it at the ingress (OIDC or basic auth on the ingress controller, for example). Treat the broker port as sensitive for read access, the same way you would treat `/metrics`. The [Security Hardening](./security-hardening.md) checklist carries this item, and the [Security Model](../explanation/security-model.md#read-only-console-authentication-the-ui-pak) explains the credential class and the threat model behind the decision.

> **The embedded read-only credential is the only identity a console session has.** Earlier versions also honoured a write-capable PAK placed in the browser's local storage under `brokkr_pak`, and preferred it over the embedded one — so setting it silently made the whole console write-capable. That override was removed in 0.9.0. The one action that needs more than read access, [creating a tenant](#step-6-create-a-tenant), asks for an admin PAK at the moment you use it and does not keep it.

## Step 3: Find Your Way Around

The sidebar groups seven views:

**Monitor**

- **Overview** — the at-a-glance landing view: headline counts, a fleet-health bar, a broker-throughput sparkline drawn from the broker's Prometheus metrics, and a recent activity feed of agent events.
- **Fleet** — every agent, grouped into a panel per cluster, with counts of active, degraded, and failing agents across the top. Each row shows the agent's status and health pills, whether it holds an internal WebSocket connection to the broker, and how long ago it last checked in. Clicking a row opens the agent detail panel, which is also where diagnostics live (Step 5).
- **Deployments** — the stacks this broker knows about. Clicking one opens its detail, including the per-deployment-object health rollup for that stack.
- **Telemetry** — two tabs. *Kube events* lists the agent lifecycle events (applies, heartbeats, reconciles) the broker has retained; clicking one opens its detail. *Pod logs* is a placeholder — pod log tails are per-stack and there is no global feed, so the tab explains that rather than showing data. Both sit behind a short retention window, which the view states on screen: this is a window into recent activity, not a log archive. For anything longer-lived, ship logs onward as described in [Streaming Pod Logs & Live Tail](./log-streaming.md).

**Operations**

- **Work orders** — active work orders above the completed history; clicking a history row opens its detail. The active list is admin-gated, and the panel says so plainly if the broker declines to serve it.
- **Webhooks** — the webhook subscriptions and their configured events. The API deliberately redacts subscription URLs, so the console shows only whether one is set. Clicking a subscription lists its recent delivery attempts.

**System**

- **Broker health** — headline broker metrics (active agents, connected agents, HTTP request volume, live subscribers, stack and deployment-object counts) drawn from the broker's `/metrics` endpoint, alongside the current internal WebSocket connections.

### How the Views Refresh

The console reads the REST API and nothing else — it opens no WebSocket and streams nothing. Most views simply re-read their endpoints on a short timer, so the page stays roughly current on its own without you reloading it.

The header carries a wall clock and a Live/Paused control. **The Live/Paused control is not yet wired to anything**: switching it to Paused does not stop the periodic re-reads. Treat it as decoration for now.

If you want a genuine live stream of fleet state, that is an API capability rather than a console one — see [Monitoring Your Agent Fleet](./fleet-monitoring.md).

## Step 4: Scope the View to One Tenant

On a broker shared between tenants, the sidebar shows a **Tenant** selector below the navigation. It lists the named PAK owners (generators) the broker knows about, plus **All**. Choosing one narrows the Overview, Fleet, Deployments, and Telemetry views to that tenant's resources; the choice is remembered in your browser between visits, and falls back to **All** if the tenant it named has since been removed.

On a single-tenant install there is nothing to choose, so the selector is hidden entirely. It is also hidden if the tenant listing cannot be read — the views still work, unscoped.

**The tenant selector is a view filter, not an authorization boundary.** It trims what the console displays; it does not restrict what the console's credential is permitted to read, and switching back to **All** is a click away for anyone using the page. Real isolation comes from generator ownership, described in [Setting Up Multi-Tenant Operation](./multi-tenant-setup.md) and the [Multi-Tenancy reference](../reference/multi-tenancy.md). Never treat the selector as a way to show one tenant's operators only their own data.

## Step 5: Run a Diagnostic on an Agent

Diagnostics are the console's one action, and they are read-shaped: you are asking an agent to go look at its cluster and report back, not changing any desired state.

1. Open the **Fleet** view and click the agent you want to inspect.
2. In the agent detail panel, find the **deployment object** picker. It lists everything currently targeted at that agent. If the agent has no deployment objects, the panel says so instead of offering a dead button — there is nothing to diagnose.
3. Pick an object and choose **Run diagnostic**. The console tells you the request was accepted and begins waiting for it.
4. The result appears in the same panel once the agent submits it: pod statuses for that deployment object, recent events from the namespaces it searched, and log tails. An empty pod list is a legitimate answer for an object that applies no workloads, and the panel labels it as such rather than as an error.

Collection is not instantaneous — the agent picks up pending requests on its own timer and then has to query the Kubernetes API. The console waits a bounded amount of time and then stops, offering a **Check again** button. Stopping is not evidence that collection failed; it only means no result had arrived yet.

For the request lifecycle, retention, and how to drive the same capability from the API or CLI, see [Running On-Demand Diagnostics](./diagnostics.md) and the [Diagnostics reference](../reference/diagnostics.md).

## Step 6: Create a Tenant

**Tenants** lists the generators on this broker, and is the one place the console writes anything. Because a tenant's PAK is a credential, the console will not mint one on the strength of the read-only identity it carries — you supply an admin PAK for that single request.

1. Open **Tenants** in the sidebar and choose **+ New tenant**.
2. Give the tenant a name, and a description if it helps whoever inherits it.
3. Paste an **admin PAK**. This is the only step in the console that asks for a credential.
4. Choose **Create tenant**.

The new tenant's PAK is then shown once, with a **Copy** button.

> **Copy it before you close the dialog.** The broker stores only a hash, so a PAK that is not captured cannot be recovered — the only remedy is [rotating it](./pak-management.md#rotating-generator-paks). This is the same one-shot behaviour as `brokkr-broker create generator`.

The admin PAK you pasted is held in memory for that one request and cleared as soon as it finishes, whether it succeeded or not. It is never written to browser storage and never logged, so creating a second tenant asks for it again — deliberately, so that a browser left open is not a standing admin credential.

If the PAK is rejected, the dialog says whether it was not an admin credential (403) or the name was already taken (409). Nothing is created in either case.

The equivalent outside the console is `POST /api/v1/generators` or `brokkr-broker create generator`; see [Generators](../reference/generators.md). Creating **agents** is not available here — only tenants.

## Running More Than One Broker Replica

Each broker process mints its own console credential, and a credential minted by one replica is rejected by the others. A browser served the console page by replica A and then load-balanced onto replica B will find its API calls refused.

**Enable session affinity (sticky sessions) on whatever fronts your brokers** whenever more than one replica serves console traffic, so a browser keeps talking to the replica that served it the page. Reloading the page also fixes an individual session, since the reload picks up the current replica's credential — but affinity is what makes the console usable rather than intermittent. Agent, generator, and admin PAK authentication is unaffected; those credentials verify identically on every replica. See [Horizontal Broker Scaling](../explanation/architecture.md#horizontal-broker-scaling).

## The Console Is Not `examples/ui-slim`

The repository also contains `examples/ui-slim`, a small React application. It is a **demonstration of what you can build against the API — not a supported product, and not the console.** The differences that matter:

| | Operator Console | `examples/ui-slim` |
|---|---|---|
| Shipped in the broker image | Yes, served at the broker's root URL | No; run separately |
| Credential | Embedded automatically, read-only | You supply an admin PAK by hand |
| Capabilities | Read-only, plus diagnostics | Demonstrates writes as well |
| Supported | Yes | No |

`ui-slim` is also the browser consumer of the fleet live-tail WebSocket; the console does not use it. If you are evaluating Brokkr and want the supported view, use the console.

## Troubleshooting

**The root URL shows a short "compiled without the bundled operator console" page.** The broker was built without the console. Use the published image, or build the broker with the console enabled as described in [Local Development Environment](../getting-started/development.md).

**Views show an error where data should be.** The console reports API failures inline with a retry control. Confirm the broker is healthy (`/healthz`, `/readyz`) and check the broker logs; the console is only as available as the API behind it.

**Everything works until the page sits open for a while, then calls start failing.** The broker probably restarted, invalidating the credential in the loaded page — or you are behind a load balancer without session affinity. Reload the page; if it recurs, fix the affinity as described above.

**An action you expected is refused with 403.** The console authenticates read-only by design. Perform the change with an admin PAK through the [CLI](../reference/cli.md), an [SDK](./sdks/README.md), or the API.

## Related

- [Security Model — Read-Only Console Authentication](../explanation/security-model.md#read-only-console-authentication-the-ui-pak) — the credential class, the threat model, and why network reach is the boundary
- [Security Hardening](./security-hardening.md) — the pre-exposure checklist, including restricting the console surface
- [Network Configuration](./network-configuration.md) and [Network Ports](../reference/network-ports.md) — what shares port 3000
- [Monitoring Your Agent Fleet](./fleet-monitoring.md) — the API surface the Fleet view reads, including the live stream the console does not use
