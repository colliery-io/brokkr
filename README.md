# Brokkr

Brokkr is an environment-aware control plane for dynamically distributing Kubernetes objects across multiple clusters. It provides a centralized way to manage deployments across heterogeneous Kubernetes environments while respecting the unique characteristics and constraints of each target cluster.

## Overview

Modern infrastructure often spans multiple Kubernetes clusters across different cloud providers, regions, or environments. Managing deployments consistently across these clusters while adapting to their individual requirements is challenging. Brokkr solves this by providing a broker-agent architecture where a central broker service orchestrates resource distribution to lightweight agents running in each target cluster.

The broker maintains the desired state of your applications as "stacks" containing Kubernetes resources. Agents poll the broker for updates and apply resources to their local clusters. This pull-based model means clusters can be behind firewalls or in restricted networks and still receive deployments reliably. Agents also report back health status and deployment results, giving you visibility into the state of your applications across all clusters from a single point.

Brokkr supports sophisticated targeting through labels and annotations, allowing you to direct deployments to specific clusters or groups of clusters. Templates with JSON Schema validation enable standardized deployments while allowing per-environment customization. The system tracks deployment health, emits events via webhooks, and provides comprehensive APIs for integration with CI/CD pipelines and external tooling.

## Quick Start

### Prerequisites

Running Brokkr locally requires Rust 1.8+, PostgreSQL, Docker with Docker Compose, and the [Angreal](https://angreal.github.io/) task runner which you can install via `pip install angreal`.

### Running Locally

Clone the repository and start the development environment:

```bash
git clone https://github.com/your-org/brokkr.git
cd brokkr
angreal local up
```

This starts the broker API at http://localhost:3000 and the admin UI at http://localhost:3001.

### Creating Your First Deployment

Create a stack to hold your application's resources:

```bash
curl -X POST http://localhost:3000/api/v1/stacks \
  -H "Authorization: Bearer <admin-pak>" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-app", "description": "My application stack"}'
```

Verify agents are connected and ready to receive deployments:

```bash
curl http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <admin-pak>"
```

## Documentation

Comprehensive documentation is available in the [docs/](./docs/) directory or can be built locally with `angreal local docs`. The documentation follows the Divio documentation system, organized into four categories:

The **Getting Started** guide covers installation options including Helm charts for production and development environment setup. **Tutorials** provide step-by-step walkthroughs of common tasks like deploying your first application or setting up multi-cluster targeting. **How-To Guides** give focused instructions for specific tasks such as configuring webhooks, using templates, or integrating with CI/CD systems. The **Explanation** section dives deep into Brokkr's architecture, data model, and design decisions.

## Project Structure

Brokkr is implemented as a Rust workspace with multiple crates serving distinct roles. The `brokkr-broker` crate contains the central management service that exposes the REST API and coordinates resource distribution. The `brokkr-agent` crate implements the cluster agent that polls the broker and applies resources to Kubernetes. Shared data models live in `brokkr-models`, while common utilities like configuration parsing and PAK management are in `brokkr-utils`. The `charts/` directory contains Helm charts for deploying both components.

## Development

Common development tasks are managed through Angreal. Run the test suite with `angreal tests unit all` or rebuild a specific service with `angreal local rebuild broker`. See the documentation for complete development workflows.

## License

Brokkr is provided under the Elastic License 2.0. See [LICENSE.txt](./LICENSE.txt) for details.

## Contributing

Pull requests and issues are welcome. For commercial use or implementation assistance, please contact us.
