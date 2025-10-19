/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Brokkr Agent
//!
//! Brokkr Agent is a Kubernetes-native component responsible for managing and orchestrating
//! deployments across clusters.
//!
//! For detailed documentation, including architecture, components, and implementation details,
//! see the [Brokkr Documentation](https://brokkr.io/explanation/architecture).

pub mod broker;
pub mod cli;
pub mod health;
pub mod k8s;
pub mod utils;
