# Welcome to Brokkr

Brokkr is a control plane for Kubernetes that lets you dynamically create and manage applications across clusters. Define what you need, fire it off, and the controller loop takes care of the rest — your applications get created, configured, and reconciled automatically.

## Use Cases

### On-Demand Application Provisioning

A customer needs a new service spun up? A new tenant needs their own stack? Just create the deployment through Brokkr and it flows through the controller loop to your clusters. No manual kubectl, no waiting on CI pipelines — your infrastructure adapts to your needs in real time.

### Dynamic Service Management

As your requirements change, Brokkr lets you define, reconfigure, and scale the services running across your clusters. Generators can programmatically create deployment objects, templates let you stamp out standardized configurations, and the agent reconciliation loop keeps everything in the desired state.

### Multi-Cluster Orchestration

Manage applications across multiple Kubernetes clusters from a single control plane. Target specific clusters with labels, push updates to all of them at once, and let each agent independently reconcile its own state. Brokkr handles the coordination so you can focus on what to deploy, not where and how.

## Explore Brokkr

- **[Getting Started](./getting-started/README.md)** — Install, configure, and get Brokkr running
- **[How-To Guides](./how-to/README.md)** — Practical guides for common tasks
- **[Explanation](./explanation/README.md)** — Architecture, concepts, and design decisions
- **[Reference](./reference/README.md)** — API reference and technical details

## What Makes Brokkr Different?

While tools like FluxCD and ArgoCD excel at GitOps-based state management, Brokkr takes a different approach — it's built for dynamic, on-demand application lifecycle management rather than static manifest synchronization.

### Programmatic Resource Creation

Brokkr's generators and templates let external systems programmatically create Kubernetes resources through an API. CI/CD pipelines, customer provisioning systems, or internal tools can fire off deployments without touching git repos or manifest files.

### Controller Loop Reconciliation

Every agent runs its own reconciliation loop, continuously pulling its target state from the broker and applying it to its cluster. Resources drift? The agent corrects it. New deployment object pushed? The agent picks it up on the next poll.

### Built for Dynamic Workloads

Where GitOps tools work best with a known, static set of manifests, Brokkr is designed for environments where the set of applications changes frequently — multi-tenant platforms, on-demand infrastructure, and systems where what needs to run is determined at runtime, not at commit time.

## When to Use Brokkr — and When Not To

**Brokkr is a good fit when:**

- What needs to run is decided at **runtime, by an API call** — multi-tenant platforms, on-demand provisioning from CI or an internal portal, or any system that programmatically creates workloads.
- You're pushing the same workloads to **many clusters** and want label/annotation-based targeting from one control plane.
- Your agents live in **restricted or firewalled networks** — the pull model means each agent needs only outbound connectivity to the broker, and an offline agent catches up when it reconnects.

**Brokkr is probably not the right tool when:**

- You have a **known, static set of manifests** you're happy syncing from a Git repository. Tools like ArgoCD and Flux are purpose-built for that and will fit better.
- You want **Git to be the source of truth** for desired state — with pull-request review and Git history of every change. Brokkr's source of truth is the broker (an API plus its database), so you trade Git's review/history workflow for programmatic, runtime-driven control. Brokkr records changes in an [audit log](./reference/audit-logs.md) rather than in Git.

In short: Brokkr complements GitOps rather than replacing it. Reach for it when deployments are driven by systems and events, not by commits.

## Working with Brokkr Day to Day

Once Brokkr is running, the recommended everyday workflow is the **`brokkr` CLI**: point it at a folder of manifests and run [`brokkr apply`](./how-to/cli-apply.md) — it's idempotent and CI-safe. For programmatic use, the [SDKs](./how-to/sdks/README.md) (Rust, Python, TypeScript) expose the same operations. The raw [REST API](./reference/api/README.md) sits underneath both.
