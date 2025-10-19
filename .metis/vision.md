---
id: brokkr-environment-aware
level: vision
title: "Brokkr: Environment-Aware Kubernetes Control Plane"
short_code: "BROKKR-V-0001"
created_at: 2025-10-08T14:48:36.749341+00:00
updated_at: 2025-10-16T02:35:59.976922+00:00
archived: false

tags:
  - "#vision"
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Brokkr: Environment-Aware Kubernetes Control Plane

## Purpose

Brokkr exists to enable dynamic, runtime-driven management of Kubernetes resources across multiple clusters. It provides an intelligent control plane specifically designed for scenarios where services and workloads must be created, configured, and managed dynamically based on runtime contexts and configurations - complementing existing GitOps solutions rather than replacing them.

## Product/Solution Overview

Brokkr is a Rust-based, environment-aware control plane for Kubernetes that consists of:

- **Central Broker Service**: A PostgreSQL-backed API server that maintains the desired state of stacks (collections of Kubernetes resources) and coordinates with agents
- **Lightweight Agents**: Cluster-resident processes that poll the broker, apply resources locally, and report status back
- **Stack Management**: Versioned, immutable deployment objects that capture resource snapshots for audit trails and rollbacks
- **Dynamic Distribution**: Policy-based routing of workloads to appropriate clusters based on environment capabilities and constraints

**Target Audience**: Platform engineers and organizations building control planes that need to dynamically manage services across multiple data plane clusters, particularly for configurable applications with variable lifecycles.

**Key Benefits**: Enables dynamic service lifecycle management, provides runtime-driven resource distribution, and offers programmable multi-cluster orchestration that works alongside existing GitOps workflows.

**Complementary Role**: Brokkr is designed to work with, not replace, GitOps solutions like FluxCD and ArgoCD. While GitOps excels at declarative, repository-driven deployments, Brokkr specializes in runtime-driven, dynamic service management scenarios.

## Current State

Today, organizations building control planes that need to dynamically manage services face significant gaps in available tooling. While GitOps solutions like FluxCD and ArgoCD excel at static, repository-driven deployments, they struggle with dynamic, runtime-driven service creation and configuration. Current tools lack native support for services that need to be created, modified, and destroyed based on runtime events and user requests.

Building custom control planes that manage multiple data plane clusters requires extensive infrastructure and orchestration logic. Managing services with different configurations, lifecycles, and requirements across clusters based on runtime parameters becomes increasingly complex as organizations scale. Most existing solutions require extensive custom development for dynamic, API-driven multi-cluster management scenarios, forcing teams to reinvent orchestration logic repeatedly.

## Future State

With Brokkr, organizations building dynamic control planes will have a foundation that truly enables runtime-driven service management. Rather than being limited to static repository changes, teams can create, configure, and manage services dynamically based on API requests and runtime contexts. The API-first design enables custom control planes to orchestrate services across data plane clusters programmatically, providing native support for services with variable lifecycles - from ephemeral workloads to long-running services with evolving configurations.

Brokkr automatically distributes workloads based on real-time cluster capabilities, policies, and runtime requirements, removing the complexity of manual placement decisions. It works alongside existing GitOps solutions rather than replacing them, handling dynamic scenarios while GitOps continues to manage static infrastructure. This complementary approach means organizations can extend their current investments rather than starting over.

The platform provides the essential building blocks for custom control planes without requiring teams to build extensive multi-cluster orchestration infrastructure from scratch. Built-in event tracking and audit capabilities support both compliance requirements and debugging for dynamic service management scenarios.

## Major Features

- **Dynamic Stack Management**: Create and version collections of Kubernetes resources that can be generated and modified at runtime based on configurations and context
- **Agent-Based Distribution**: Lightweight agents poll the broker and apply resources locally, enabling scalable management across data plane clusters
- **Runtime-Aware Placement**: Intelligent distribution of workloads based on cluster capabilities, policies, and runtime-provided requirements
- **Control Plane API**: RESTful API designed for building custom control planes that need to orchestrate services dynamically
- **Generator Framework**: Extensible system for creating deployment objects programmatically based on runtime parameters and configurations
- **Event & Audit System**: Comprehensive tracking of all dynamic service operations for compliance and debugging
- **Flexible Targeting**: Label and annotation-based targeting system supporting complex, runtime-driven placement decisions
- **Runtime Configuration**: Support for services with variable configurations, lifecycles, and contexts determined at deployment time

## Business Requirements Overview

Brokkr enables new business capabilities that require runtime-driven service creation and management across multiple clusters. By providing foundational multi-cluster orchestration capabilities, it significantly reduces time-to-market for custom control planes. Organizations can support business models that require dynamic, configurable services with varying lifecycles and requirements, while platform teams gain the ability to build self-service capabilities for dynamic service deployment and management.

The system maintains comprehensive audit trails for dynamically created and managed services to meet governance requirements. Most importantly, Brokkr complements existing GitOps investments rather than requiring replacement, extending current capabilities to handle dynamic scenarios that traditional GitOps workflows cannot address effectively.

## Success Criteria

- **Dynamic Service Management**: Successfully enable runtime creation, configuration, and lifecycle management of services across 10+ data plane clusters
- **Control Plane Development**: Reduce time-to-build for custom control planes by 60% compared to building multi-cluster orchestration from scratch
- **Runtime Responsiveness**: Support sub-minute service creation and configuration changes based on runtime API requests
- **Audit Compliance**: 100% of dynamic service operations have complete audit trails with versioning and event tracking
- **System Stability**: Demonstrate stable operation under realistic load conditions with graceful failure handling and recovery
- **Platform Adoption**: Enable platform teams to build self-service capabilities that complement their existing GitOps workflows
- **API Performance**: Support 1000+ concurrent API requests for dynamic service operations and manage 100+ data plane clusters efficiently

## Principles

Brokkr's architecture is built on a pull-based model where agents pull state from the broker rather than receiving pushes, ensuring both scalability and resilience. Deployment objects remain immutable after creation, providing complete audit trails and reliable rollback capabilities. The system makes deployment decisions based on actual cluster capabilities and real-time status rather than static configuration, enabling truly environment-aware orchestration.

Everything in Brokkr is API-first, with all functionality exposed through well-documented RESTful APIs that enable seamless automation and integration. Security is built into the core architecture by default, with strong authentication, authorization, and audit capabilities. Despite its powerful functionality, the system prioritizes operational simplicity to reduce cognitive load and complexity.

The platform provides an extensible framework that supports custom resource generators and integration patterns without requiring modifications to core systems. The entire architecture is designed to handle enterprise-scale deployments efficiently, growing with organizational needs.

## Constraints

Brokkr is designed specifically for Kubernetes clusters and does not support other container orchestration platforms. The broker service requires PostgreSQL for persistence, which adds operational complexity for teams unfamiliar with PostgreSQL management. Agents must maintain reliable network connectivity to the broker service, meaning network partitions can impact deployment capabilities.

Being built in Rust may limit the contributor pool compared to more mainstream languages like Go or Python, though this choice provides performance and safety benefits. The platform introduces new concepts like stacks, deployment objects, and agent targets that require team training and comprehensive documentation.

Each managed cluster requires a dedicated agent, adding some resource consumption to target clusters. The pull-based architecture means deployment updates are eventually consistent rather than immediately consistent, which may not suit all use cases requiring instant propagation.
