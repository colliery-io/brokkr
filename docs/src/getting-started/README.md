# Getting Started with Brokkr

This section covers installing and configuring Brokkr.

## Prerequisites

This is the one list of prerequisites for Brokkr. The other pages link here. Find the path that you will take, and install the tools in its list.

### To evaluate with `angreal local up`

This is Path A of [Evaluate Brokkr Locally](./evaluate.md). It builds Brokkr from source inside Docker and bundles its own Kubernetes (k3s). You do not need a cluster or a Rust toolchain.

- **Docker** with Docker Compose
- **Git**
- **[Angreal](https://pypi.org/project/angreal/)**, the project's task runner: `pip install angreal`
- **`curl`** and **`jq`**

### To evaluate or install with Helm

This is Path B of [Evaluate Brokkr Locally](./evaluate.md) and the [Installation](./installation.md) guide. These paths use the published images. You do not need a source checkout or a Rust toolchain.

- **A Kubernetes cluster**, v1.29 or later (the agent chart declares `kubeVersion: ">=1.29.0-0"`). For an evaluation, a local [kind](https://kind.sigs.k8s.io/) or [k3d](https://k3d.io/) cluster is sufficient.
- **kubectl**, configured to reach that cluster
- **Helm** 3.8 or later ([installation guide](https://helm.sh/docs/intro/install/))
- **Docker**, to run kind or k3d, and to run `brokkr-broker generate-pak` from the published image
- **`curl`** and **`jq`**

### To develop Brokkr

This is the [Local Development Environment](./development.md). It builds and tests Brokkr from source.

- **Rust 1.90 or later** (the workspace uses edition 2024)
- **PostgreSQL client tools**: `libpq`, which the build links, and `psql`. The database itself runs in Docker.
- **Docker** with Docker Compose
- **Git**
- **[Angreal](https://pypi.org/project/angreal/)**, the project's task runner: `pip install angreal`

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
