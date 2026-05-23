/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Continuous retention enforcement for the agent telemetry tables
//! (`agent_k8s_events`, `agent_pod_logs`).
//!
//! ## Stance
//!
//! Brokkr's telemetry buffers exist for **immediate operational support**,
//! not long-term retention. The hard ceiling is **6 hours**. See the
//! `project_log_retention_stance` memory and [[BROKKR-A-0008]]. Callers
//! can construct a worker with a shorter retention but never longer —
//! [`RetentionConfig::new`] clamps anything past 6h back down to 6h.
//!
//! Eviction runs continuously on a tokio interval (default every 60s).
//! `created_at` (server-side) is the eviction key — a misbehaving agent
//! sending past-dated timestamps cannot keep ancient rows alive past the
//! ceiling. The worker logs row counts at each tick for observability.

use std::time::Duration;

use chrono::Utc;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use crate::dal::DAL;

/// Hard cap on retained telemetry — never configurable upward.
pub const HARD_RETENTION_CEILING: Duration = Duration::from_secs(6 * 60 * 60);

/// Default eviction tick interval.
pub const DEFAULT_EVICTION_TICK: Duration = Duration::from_secs(60);

/// Retention policy for the agent telemetry buffers.
#[derive(Debug, Clone, Copy)]
pub struct RetentionConfig {
    /// Effective retention window — guaranteed `<= HARD_RETENTION_CEILING`.
    pub retention: Duration,
    /// How often the eviction worker runs.
    pub tick_interval: Duration,
}

impl RetentionConfig {
    /// Construct a policy, clamping `retention` to the hard ceiling. Any
    /// caller asking for more than 6h silently gets 6h — this is the
    /// product invariant, not a configuration error.
    pub fn new(retention: Duration, tick_interval: Duration) -> Self {
        let clamped = if retention > HARD_RETENTION_CEILING {
            HARD_RETENTION_CEILING
        } else {
            retention
        };
        Self {
            retention: clamped,
            tick_interval,
        }
    }

    /// Default policy: 6h retention, 60s tick.
    pub fn default_policy() -> Self {
        Self::new(HARD_RETENTION_CEILING, DEFAULT_EVICTION_TICK)
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self::default_policy()
    }
}

/// Spawn the continuous eviction worker. Returns the join handle so the
/// caller can shut it down explicitly during tests; production drops it
/// and the worker runs for the process lifetime.
pub fn spawn(dal: DAL, config: RetentionConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(config.tick_interval);
        // Skip the immediate fire — first eviction is one tick in.
        interval.tick().await;
        loop {
            interval.tick().await;
            run_once(&dal, config);
        }
    })
}

/// Synchronous single eviction pass — exposed for tests so they can call
/// it deterministically without waiting for the interval to fire.
pub fn run_once(dal: &DAL, config: RetentionConfig) {
    let cutoff = Utc::now() - chrono::Duration::from_std(config.retention).unwrap_or_default();
    match dal.agent_k8s_events().evict_older_than(cutoff) {
        Ok(n) if n > 0 => debug!(rows = n, "evicted agent_k8s_events older than retention"),
        Ok(_) => {}
        Err(e) => warn!(error = %e, "agent_k8s_events eviction failed"),
    }
    match dal.agent_pod_logs().evict_older_than(cutoff) {
        Ok(n) if n > 0 => debug!(rows = n, "evicted agent_pod_logs older than retention"),
        Ok(_) => {}
        Err(e) => warn!(error = %e, "agent_pod_logs eviction failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_above_ceiling_is_clamped() {
        let cfg = RetentionConfig::new(Duration::from_secs(24 * 60 * 60), Duration::from_secs(1));
        assert_eq!(cfg.retention, HARD_RETENTION_CEILING);
    }

    #[test]
    fn retention_below_ceiling_is_preserved() {
        let cfg = RetentionConfig::new(Duration::from_secs(60), Duration::from_secs(1));
        assert_eq!(cfg.retention, Duration::from_secs(60));
    }

    #[test]
    fn default_policy_uses_ceiling_and_one_minute_tick() {
        let cfg = RetentionConfig::default_policy();
        assert_eq!(cfg.retention, HARD_RETENTION_CEILING);
        assert_eq!(cfg.tick_interval, DEFAULT_EVICTION_TICK);
    }
}
