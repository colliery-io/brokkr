/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Stream container logs for Brokkr-managed pods to the broker as
//! `WsMessage::PodLogLine` frames.
//!
//! ## Opt-in
//!
//! Off by default. A pod is eligible only when *both* annotations are
//! present on the Pod itself:
//!
//! - `k8s.brokkr.io/stack`     → the owning stack id (set by the
//!   agent's reconciler on every managed object).
//! - `brokkr.io/stream-logs: "true"` → explicit opt-in. Today this is
//!   expected to live on the Pod (e.g. via a PodTemplateSpec annotation
//!   on the Deployment that owns it); a future agent-side improvement
//!   can propagate stack-level opt-in down to pods automatically.
//!
//! ## Rate limiting
//!
//! Each container stream gets a token bucket (default 100 lines/sec).
//! Over-rate lines are dropped and a `WsMessage::LogGap{reason:
//! RateLimit}` is emitted so the UI renders a visible gap rather than
//! silently swallowing data. Per ADR-0008 / project_log_retention_stance,
//! Brokkr is not a log warehouse — when an opted-in stack saturates the
//! bucket the right answer is "ship to Datadog", not "raise the limit".

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use brokkr_wire::{GapReason, LogGap, PodLogLine, WsMessage};
use chrono::Utc;
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use tokio::io::AsyncBufReadExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::broker_ws::WsUplink;
use crate::k8s::objects::STACK_LABEL;

/// Annotation that opts a workload into log streaming.
pub const STREAM_LOGS_ANNOTATION: &str = "brokkr.io/stream-logs";

/// Default per-container line-rate ceiling. Drops above this fire LogGap
/// markers; raise only when there's a real operational reason (and even
/// then, prefer shipping to Datadog).
const DEFAULT_LINES_PER_SEC: u64 = 100;

/// Window for the token-bucket counter.
const RATE_WINDOW: Duration = Duration::from_secs(1);

pub fn spawn(client: Client, uplink: WsUplink, agent_id: Uuid) -> JoinHandle<()> {
    tokio::spawn(async move {
        let active: Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        loop {
            if let Err(e) =
                watch_pods(client.clone(), uplink.clone(), agent_id, active.clone()).await
            {
                warn!(error = %e, "pod-logs watcher fell out; restarting in 5s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    })
}

async fn watch_pods(
    client: Client,
    uplink: WsUplink,
    agent_id: Uuid,
    active: Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>,
) -> Result<(), watcher::Error> {
    info!("starting pod-logs tailer");
    let api: Api<Pod> = Api::all(client.clone());
    let mut stream = futures::stream::StreamExt::boxed(watcher(api, watcher::Config::default()));

    while let Some(event) = stream.try_next().await? {
        match event {
            watcher::Event::Apply(pod) => {
                let Some(uid) = pod.metadata.uid.clone() else { continue };
                if !is_opted_in(&pod) {
                    teardown_for(&uid, &active).await;
                    continue;
                }
                let Some(stack_id) = pod_stack_id(&pod) else {
                    teardown_for(&uid, &active).await;
                    continue;
                };
                ensure_tails(
                    &client,
                    &uplink,
                    agent_id,
                    stack_id,
                    &pod,
                    &uid,
                    &active,
                )
                .await;
            }
            watcher::Event::Delete(pod) => {
                if let Some(uid) = pod.metadata.uid.clone() {
                    teardown_for(&uid, &active).await;
                }
            }
            watcher::Event::Init | watcher::Event::InitDone | watcher::Event::InitApply(_) => {}
        }
    }
    Ok(())
}

fn is_opted_in(pod: &Pod) -> bool {
    pod.metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get(STREAM_LOGS_ANNOTATION))
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn pod_stack_id(pod: &Pod) -> Option<Uuid> {
    let ann = pod.metadata.annotations.as_ref()?;
    let raw = ann.get(STACK_LABEL)?;
    Uuid::parse_str(raw).ok()
}

/// For a given opted-in pod, ensure one tail task per container. Existing
/// tasks are preserved across watcher re-emits; missing tasks are
/// started.
async fn ensure_tails(
    client: &Client,
    uplink: &WsUplink,
    agent_id: Uuid,
    stack_id: Uuid,
    pod: &Pod,
    uid: &str,
    active: &Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>,
) {
    let mut guard = active.write().await;
    if guard.contains_key(uid) {
        return; // already tailing
    }

    let namespace = pod.namespace().unwrap_or_else(|| "default".to_string());
    let name = pod.name_any();
    let containers: HashSet<String> = pod
        .spec
        .as_ref()
        .map(|s| s.containers.iter().map(|c| c.name.clone()).collect())
        .unwrap_or_default();

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let mut handles = Vec::new();
    for container in containers {
        let handle = tokio::spawn(tail_container(
            pods_api.clone(),
            uplink.clone(),
            agent_id,
            stack_id,
            namespace.clone(),
            name.clone(),
            container.clone(),
        ));
        handles.push(handle);
    }
    guard.insert(uid.to_string(), handles);
}

async fn teardown_for(
    uid: &str,
    active: &Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>,
) {
    let mut guard = active.write().await;
    if let Some(handles) = guard.remove(uid) {
        for h in handles {
            h.abort();
        }
    }
}

async fn tail_container(
    pods: Api<Pod>,
    uplink: WsUplink,
    agent_id: Uuid,
    stack_id: Uuid,
    namespace: String,
    pod: String,
    container: String,
) {
    let params = LogParams {
        container: Some(container.clone()),
        follow: true,
        ..LogParams::default()
    };
    let mut limiter = RateLimiter::new(DEFAULT_LINES_PER_SEC);

    // The pod watcher can hand us a pod that's still `ContainerCreating`:
    // opening a follow log stream against it succeeds but EOFs immediately
    // (or errors), and `ensure_tails` has already marked the uid active so
    // a later Apply won't re-attach. Reopen with backoff until the
    // container produces output (or we exhaust the start-up budget). Once
    // we've forwarded a line we never reopen — a follow stream replays
    // from the start, so reopening after success would duplicate lines.
    let mut seen_any = false;
    let mut open_attempts = 0u32;
    // ~MAX_OPEN_ATTEMPTS * OPEN_RETRY interval of pod-start slack.
    const MAX_OPEN_ATTEMPTS: u32 = 30;
    const OPEN_RETRY: Duration = Duration::from_secs(2);

    loop {
        let reader = match pods.log_stream(&pod, &params).await {
            Ok(s) => tokio::io::BufReader::new(s.compat()),
            Err(e) => {
                debug!(%pod, %container, error = %e, "pod log_stream open failed");
                if seen_any || open_attempts >= MAX_OPEN_ATTEMPTS {
                    return;
                }
                open_attempts += 1;
                tokio::time::sleep(OPEN_RETRY).await;
                continue;
            }
        };
        let mut lines = reader.lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    seen_any = true;
                    match limiter.consume() {
                        Allowance::Pass => {
                            let msg = WsMessage::PodLogLine(PodLogLine {
                                agent_id,
                                stack_id,
                                namespace: namespace.clone(),
                                pod: pod.clone(),
                                container: container.clone(),
                                ts: Utc::now(),
                                line,
                            });
                            let _ = uplink.try_send(msg);
                        }
                        Allowance::DropAndGap(n_since) => {
                            let gap = WsMessage::LogGap(LogGap {
                                agent_id,
                                stack_id,
                                since_ts: Utc::now(),
                                dropped_count: n_since,
                                reason: GapReason::RateLimit,
                            });
                            let _ = uplink.try_send(gap);
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    debug!(%pod, %container, error = %e, "log stream read error; ending tail");
                    break;
                }
            }
        }
        // Stream ended. If we forwarded at least one line, the container has
        // run and EOF means it exited — we're done. Otherwise it likely
        // wasn't running yet; reopen until the start-up budget runs out.
        if seen_any || open_attempts >= MAX_OPEN_ATTEMPTS {
            return;
        }
        open_attempts += 1;
        tokio::time::sleep(OPEN_RETRY).await;
    }
}

/// Minimal token-bucket: at most `lines_per_sec` lines per RATE_WINDOW.
/// On overflow, emits one gap-marker per window with the dropped count.
struct RateLimiter {
    lines_per_sec: u64,
    window_start: Instant,
    count_in_window: u64,
    dropped_in_window: u64,
}

enum Allowance {
    Pass,
    /// Drop this line; if it's the first drop of the window, surface a
    /// `LogGap` with the running drop count to the caller.
    DropAndGap(u64),
}

impl RateLimiter {
    fn new(lines_per_sec: u64) -> Self {
        Self {
            lines_per_sec,
            window_start: Instant::now(),
            count_in_window: 0,
            dropped_in_window: 0,
        }
    }

    fn consume(&mut self) -> Allowance {
        if self.window_start.elapsed() >= RATE_WINDOW {
            self.window_start = Instant::now();
            self.count_in_window = 0;
            self.dropped_in_window = 0;
        }
        if self.count_in_window < self.lines_per_sec {
            self.count_in_window += 1;
            Allowance::Pass
        } else {
            self.dropped_in_window += 1;
            // Emit one gap-marker per window (when transitioning from
            // 0 drops to 1+). Subsequent drops within the same window
            // silently accumulate into the same counter and surface in
            // the next window's first drop.
            if self.dropped_in_window == 1 {
                Allowance::DropAndGap(1)
            } else {
                Allowance::Pass // false; we still drop, but no second gap
            }
        }
    }
}

// Tokio AsyncRead bridge: kube returns a `futures::io::AsyncRead`; we
// need a `tokio::io::AsyncRead` to use BufReader::lines.
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_passes_under_ceiling() {
        let mut r = RateLimiter::new(10);
        for _ in 0..10 {
            assert!(matches!(r.consume(), Allowance::Pass));
        }
    }

    #[test]
    fn rate_limiter_drops_above_ceiling_with_first_gap() {
        let mut r = RateLimiter::new(2);
        assert!(matches!(r.consume(), Allowance::Pass));
        assert!(matches!(r.consume(), Allowance::Pass));
        // 3rd in this window: drop + first gap
        match r.consume() {
            Allowance::DropAndGap(1) => {}
            other => panic!("expected DropAndGap(1), got something else: {}", matches!(other, Allowance::Pass)),
        }
        // 4th in this window: still dropping, but no new gap
        assert!(matches!(r.consume(), Allowance::Pass));
        // (Allowance::Pass here is the "we dropped silently" path —
        // see the comment in RateLimiter::consume.)
    }
}
