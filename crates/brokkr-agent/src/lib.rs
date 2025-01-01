//! # Brokkr Agent
//!
//! Brokkr Agent is a Kubernetes-native component responsible for managing and orchestrating
//! deployments across clusters. It operates as a bridge between the Kubernetes API server
//! and deployment objects, handling resource management and state reconciliation.
//!
//! ## Architecture
//!
//! The agent consists of several core components:
//!
//! ### Broker Module
//! ```rust
//! pub mod broker;
//! ```
//! Handles communication with the Brokkr Broker service:
//! - Deployment object fetching
//! - Event reporting
//! - Agent heartbeat management
//! - PAK verification
//!
//! ### Kubernetes Module
//! ```rust
//! pub mod k8s;
//! ```
//! Manages all Kubernetes interactions:
//! - Resource creation/deletion
//! - State reconciliation
//! - Object validation
//! - Dynamic client management
//!
//! ### CLI Module
//! ```rust
//! pub mod cli;
//! ```
//! Provides command-line interface functionality:
//! - Command parsing
//! - Agent initialization
//! - Runtime control
//!
//! ## Operation Flow
//!
//! ```mermaid
//! sequenceDiagram
//!     participant Agent
//!     participant Broker
//!     participant K8s
//!
//!     Agent->>Broker: Verify PAK
//!     Broker-->>Agent: PAK Valid
//!
//!     loop Every polling interval
//!         Agent->>Broker: Send Heartbeat
//!         Agent->>Broker: Fetch Deployments
//!         Broker-->>Agent: Deployment Objects
//!
//!         loop For each deployment
//!             Agent->>K8s: Validate Objects
//!             Agent->>K8s: Apply Objects
//!             K8s-->>Agent: Apply Result
//!             Agent->>Broker: Report Status
//!         end
//!     end
//! ```
//!
//! ## Configuration
//!
//! The agent is configured through environment variables or a configuration file:
//!
//! ```yaml
//! agent:
//!   name: "my-agent"
//!   cluster_name: "prod-cluster"
//!   polling_interval: 30
//!   pak: "your-pak-here"
//!   broker_url: "http://broker:8080"
//!   kubeconfig_path: "/path/to/kubeconfig"
//! ```

pub mod broker;
pub mod cli;
pub mod k8s;
pub mod utils;
