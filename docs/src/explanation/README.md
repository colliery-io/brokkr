# Explanation

In-depth discussion of Brokkr's architecture, design decisions, and internal workings. These documents help you understand *why* Brokkr works the way it does.

## Start Here

- **[Core Concepts](./core-concepts.md)** — the vocabulary: brokers, agents, stacks, generators, deployment objects, and how they relate.
- **[Technical Architecture](./architecture.md)** — the components that make up a running Brokkr and how they are wired together.
- **[Security Model](./security-model.md)** — credential classes, authorization rules, and the registration consent boundary.
- **[Reconciliation](./reconciliation.md)** — how an agent turns desired state into cluster state, and what happens when an apply fails.

## The System in Depth

- **[Components](./components.md)** — a closer look at the broker's and agent's internal modules.
- **[Data Model Design](./data-model.md)** — the entities, their relationships, and the reasoning behind immutability and soft deletion.
- **[Data Flows](./data-flows.md)** — the journey of a deployment from creation through apply, plus events, webhooks, authentication, and retention.
- **[Network Flows](./network-flows.md)** — the traffic patterns between broker, agents, database, and external endpoints, and what each exposure choice implies.

## Subsystems

- **[Template Matching & Rendering](./template-system.md)** — parameterized YAML, schema validation, matching rules, and pinned versioning.
- **[Work Orders](./work-orders.md)** — one-time dispatched tasks, and why they are modelled separately from desired state.
- **[Fleet Legibility](./fleet-legibility.md)** — why the broker measures the fleet but refuses to judge it.
- **[Internal Broker ↔ Agent WebSocket Channel](./internal-ws-channel.md)** — the optimization layer over REST polling, and the telemetry retention stance.

## Project Practices

- **[Publishing Strategy](./publishing-strategy.md)** — how Brokkr's artifacts are versioned and released.
