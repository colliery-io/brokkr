# Getting Started with Brokkr

This section covers installing and configuring Brokkr.

## Prerequisites

Prerequisites depend on which path you take — each page lists its own. The fastest way to try Brokkr, [Evaluate Brokkr Locally](./evaluate.md), needs only Docker (its one-command path bundles its own Kubernetes, so you do **not** need a cluster or the Rust toolchain).

For a real install or to build from source, you'll typically want:

- Kubernetes cluster access and `kubectl` (for [Installation](./installation.md))
- Docker (for container deployments)
- The Rust toolchain (only if building from source)

## Quick Navigation

1. [Evaluate Brokkr Locally](./evaluate.md) — get a working Brokkr in front of you fast
2. [Installation](./installation.md) — install Brokkr on your system
3. [Configuration](./configuration.md) — configure Brokkr for your environment
4. [Local Development Environment](./development.md) — run the whole stack from source, for contributors

Whichever path you take, the broker also serves a read-only **Operator Console** at its own root URL once it is running — open it in a browser to see your fleet, deployments, and telemetry without setting anything up.

## What's Next?

After completing the getting started guide, you can:

- Read [Core Concepts](../explanation/core-concepts.md) to put names to what you just deployed — stacks, generators, deployment objects, agents, and how they relate
- Follow our [tutorials](../tutorials/README.md) for hands-on learning
- Check out the [how-to guides](../how-to/README.md) for specific tasks
- Dive into the [reference documentation](../reference/README.md) for detailed information
