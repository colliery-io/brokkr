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
//! ## Work orders
//!
//! A pod of a work order needs no opt-in: the owner of a work order reads
//! the whole result (BROKKR-A-0013). A pod belongs to a work order through
//! the mark `brokkr.io/work-order-id` on itself or on an owner
//! ([`crate::work_order_marks`]). Its lines go with the work order id in
//! place of a stack id, and only to a broker that accepts them
//! ([`WsUplink::accepts_work_order_telemetry`]).
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
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use brokkr_wire::{GapReason, LogGap, PodLogLine, WsMessage};
use chrono::Utc;
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use lru::LruCache;
use tokio::io::AsyncBufReadExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::broker_ws::WsUplink;
use crate::k8s::objects::STACK_LABEL;
use crate::work_order_marks::{self, KubeMetaFetcher, MetaFetcher, TelemetryKey};

/// Per-pod (by UID) set of running log-tail tasks.
type ActiveTails = Arc<RwLock<HashMap<String, Vec<JoinHandle<()>>>>>;

/// Annotation that opts a workload into log streaming.
pub const STREAM_LOGS_ANNOTATION: &str = "brokkr.io/stream-logs";

/// Default per-container line-rate ceiling. Drops above this fire LogGap
/// markers; raise only when there's a real operational reason (and even
/// then, prefer shipping to Datadog).
const DEFAULT_LINES_PER_SEC: u64 = 100;

/// How many pods the tailer remembers the work order of (or that they have
/// none). It bounds the owner lookups: one for each pod of a Job or a
/// TaskRun, not one for each update of that pod.
const WORK_ORDER_CACHE_CAP: usize = 10_000;

/// Window for the token-bucket counter.
const RATE_WINDOW: Duration = Duration::from_secs(1);

pub fn spawn(
    client: Client,
    uplink: WsUplink,
    agent_id: Uuid,
    watch_namespace: Option<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let active: ActiveTails = Arc::new(RwLock::new(HashMap::new()));
        // Capped exponential backoff so a persistent failure (e.g. an RBAC
        // denial) doesn't re-dial the API server every 5s forever.
        const MAX_BACKOFF: Duration = Duration::from_secs(60);
        let mut backoff = Duration::from_secs(1);
        loop {
            match watch_pods(
                client.clone(),
                uplink.clone(),
                agent_id,
                active.clone(),
                watch_namespace.as_deref(),
            )
            .await
            {
                Ok(()) => backoff = Duration::from_secs(1),
                Err(e) => warn!(error = %e, "pod-logs watcher fell out; retrying in {:?}", backoff),
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    })
}

async fn watch_pods(
    client: Client,
    uplink: WsUplink,
    agent_id: Uuid,
    active: ActiveTails,

    watch_namespace: Option<&str>,
) -> Result<(), watcher::Error> {
    info!(namespace = ?watch_namespace, "starting pod-logs tailer");
    let api: Api<Pod> = match watch_namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let mut stream = futures::stream::StreamExt::boxed(watcher(api, watcher::Config::default()));
    let mut work_orders: LruCache<String, Option<Uuid>> =
        LruCache::new(NonZeroUsize::new(WORK_ORDER_CACHE_CAP).unwrap_or(NonZeroUsize::MIN));
    let mut fetcher = KubeMetaFetcher::new(client.clone());

    while let Some(event) = stream.try_next().await? {
        match event {
            watcher::Event::Apply(pod) => {
                let Some(uid) = pod.metadata.uid.clone() else {
                    continue;
                };
                let key = match stack_key(&pod) {
                    Some(key) => Some(key),
                    // Only for a broker that accepts the frames. The answer
                    // is not kept while it does not, so a pod is looked at
                    // again after the broker is upgraded.
                    None if uplink.accepts_work_order_telemetry() => {
                        work_order_key(&mut fetcher, &mut work_orders, &uid, &pod).await
                    }
                    None => None,
                };
                let Some(key) = key else {
                    teardown_for(&uid, &active).await;
                    continue;
                };
                if !may_attach(key, &pod) {
                    continue;
                }
                ensure_tails(&client, &uplink, agent_id, key, &pod, &uid, &active).await;
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

/// The stack a pod streams its logs for: the pod opted in and names its stack.
fn stack_key(pod: &Pod) -> Option<TelemetryKey> {
    if !is_opted_in(pod) {
        return None;
    }
    pod_stack_id(pod).map(TelemetryKey::Stack)
}

/// The work order a pod belongs to, from the cache or from its owner chain.
async fn work_order_key<F: MetaFetcher>(
    fetcher: &mut F,
    cache: &mut LruCache<String, Option<Uuid>>,
    uid: &str,
    pod: &Pod,
) -> Option<TelemetryKey> {
    let work_order = match cache.get(uid) {
        Some(known) => *known,
        None => {
            let found = work_order_marks::resolve(
                fetcher,
                pod.metadata.namespace.as_deref(),
                &pod.metadata,
            )
            .await;
            cache.put(uid.to_string(), found);
            found
        }
    };
    work_order.map(TelemetryKey::WorkOrder)
}

/// Whether to attach tails to this pod now. A follow stream replays the log
/// from its start, so the pod of a work order that has ended is not tailed
/// again: its lines went up while it ran. A stack pod is as before.
fn may_attach(key: TelemetryKey, pod: &Pod) -> bool {
    match key {
        TelemetryKey::Stack(_) => true,
        TelemetryKey::WorkOrder(_) => !matches!(
            pod.status.as_ref().and_then(|s| s.phase.as_deref()),
            Some("Succeeded") | Some("Failed")
        ),
    }
}

/// For a given opted-in pod, ensure one tail task per container. While any
/// tail for the pod is still running it is preserved across watcher re-emits;
/// once every tail for the pod has finished (EOF or gave up), the next re-emit
/// re-attaches fresh tails so a pod that becomes loggable later is picked up.
/// Decide whether `ensure_tails` should (re)attach tails for `uid`.
///
/// Returns `true` when there is no entry, or when every existing tail for the
/// uid has finished — in the finished case the stale entry is removed so the
/// caller re-attaches. Returns `false` (leaving the entry intact) while any
/// tail is still running. Pulling this out of `ensure_tails` keeps the
/// re-attach decision unit-testable without a live cluster.
fn take_if_attachable(map: &mut HashMap<String, Vec<JoinHandle<()>>>, uid: &str) -> bool {
    match map.get(uid) {
        // At least one tail still running — keep it, do not re-attach.
        Some(existing) if existing.iter().any(|h| !h.is_finished()) => false,
        // All tails finished (EOF, or gave up before the container produced
        // logs). Drop the stale entry so the caller re-attaches; a pod that
        // becomes loggable later is otherwise never tailed again. Race-free
        // against teardown_for — both run under the `active` write lock.
        Some(_) => {
            map.remove(uid);
            true
        }
        None => true,
    }
}

async fn ensure_tails(
    client: &Client,
    uplink: &WsUplink,
    agent_id: Uuid,
    key: TelemetryKey,
    pod: &Pod,
    uid: &str,
    active: &ActiveTails,
) {
    let mut guard = active.write().await;
    if !take_if_attachable(&mut guard, uid) {
        return; // still actively tailing
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
            key,
            namespace.clone(),
            name.clone(),
            container.clone(),
        ));
        handles.push(handle);
    }
    guard.insert(uid.to_string(), handles);
}

async fn teardown_for(uid: &str, active: &ActiveTails) {
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
    key: TelemetryKey,
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
                        Allowance::Allow => {
                            let msg =
                                log_line_frame(agent_id, key, &namespace, &pod, &container, line);
                            let _ = uplink.try_send(msg);
                        }
                        // Over budget after this window already emitted its
                        // gap marker — drop silently, no second gap.
                        Allowance::Drop => {}
                        Allowance::DropAndGap(n_since) => {
                            // A gap frame names a stack. For a work order
                            // the lines over the limit are dropped with no
                            // marker.
                            let TelemetryKey::Stack(stack_id) = key else {
                                continue;
                            };
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

/// One line of a container, for the stack or the work order of its pod.
fn log_line_frame(
    agent_id: Uuid,
    key: TelemetryKey,
    namespace: &str,
    pod: &str,
    container: &str,
    line: String,
) -> WsMessage {
    let (stack_id, work_order_id) = key.ids();
    WsMessage::PodLogLine(PodLogLine {
        agent_id,
        stack_id,
        work_order_id,
        namespace: namespace.to_string(),
        pod: pod.to_string(),
        container: container.to_string(),
        ts: Utc::now(),
        line,
    })
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
    /// Within budget — ship the line.
    Allow,
    /// Over budget — silently drop this line. A `LogGap` was already
    /// emitted earlier this window, so no second marker.
    Drop,
    /// Over budget AND the first drop of this window — drop the line and
    /// surface one `LogGap` carrying the running drop count.
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
            Allowance::Allow
        } else {
            self.dropped_in_window += 1;
            // Emit one gap-marker per window, on the transition from 0 drops
            // to 1+. Subsequent over-budget lines this window are dropped
            // (`Drop`) without a second marker.
            if self.dropped_in_window == 1 {
                Allowance::DropAndGap(1)
            } else {
                Allowance::Drop
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

    fn pod(annotations: &[(&str, &str)], phase: Option<&str>) -> Pod {
        let mut pod = Pod::default();
        pod.metadata.annotations = Some(
            annotations
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        pod.status = phase.map(|p| k8s_openapi::api::core::v1::PodStatus {
            phase: Some(p.to_string()),
            ..Default::default()
        });
        pod
    }

    // --- BROKKR-T-0406: the key of a pod's lines ----------------------------

    #[test]
    fn a_stack_pod_streams_only_when_it_opted_in() {
        let stack = Uuid::new_v4().to_string();
        let opted = pod(
            &[(STACK_LABEL, &stack), (STREAM_LOGS_ANNOTATION, "true")],
            None,
        );
        assert_eq!(
            stack_key(&opted),
            Some(TelemetryKey::Stack(Uuid::parse_str(&stack).unwrap()))
        );
        assert_eq!(stack_key(&pod(&[(STACK_LABEL, &stack)], None)), None);
        assert_eq!(
            stack_key(&pod(&[(STREAM_LOGS_ANNOTATION, "true")], None)),
            None
        );
    }

    #[test]
    fn a_line_goes_with_one_id() {
        let agent = Uuid::new_v4();
        let id = Uuid::new_v4();
        let frame = |key| match log_line_frame(agent, key, "ns", "p", "c", "hello".into()) {
            WsMessage::PodLogLine(line) => (line.stack_id, line.work_order_id, line.line),
            other => panic!("not a log line: {other:?}"),
        };
        assert_eq!(
            frame(TelemetryKey::Stack(id)),
            (Some(id), None, "hello".to_string())
        );
        assert_eq!(
            frame(TelemetryKey::WorkOrder(id)),
            (None, Some(id), "hello".to_string())
        );
    }

    #[test]
    fn the_pod_of_an_ended_work_order_is_not_tailed_again() {
        let work_order = TelemetryKey::WorkOrder(Uuid::new_v4());
        assert!(may_attach(work_order, &pod(&[], Some("Pending"))));
        assert!(may_attach(work_order, &pod(&[], Some("Running"))));
        assert!(!may_attach(work_order, &pod(&[], Some("Succeeded"))));
        assert!(!may_attach(work_order, &pod(&[], Some("Failed"))));
        // A stack pod is as before.
        let stack = TelemetryKey::Stack(Uuid::new_v4());
        assert!(may_attach(stack, &pod(&[], Some("Succeeded"))));
    }

    /// The Job `migrate` has the mark; each read of an owner is counted.
    struct OneJob {
        work_order: Uuid,
        reads: usize,
    }

    impl MetaFetcher for OneJob {
        async fn fetch(
            &mut self,
            _namespace: &str,
            owner: &k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference,
        ) -> Option<k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta> {
            self.reads += 1;
            (owner.kind == "Job" && owner.name == "migrate").then(|| {
                k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    annotations: Some(
                        [(
                            work_order_marks::WORK_ORDER_ID_KEY.to_string(),
                            self.work_order.to_string(),
                        )]
                        .into(),
                    ),
                    ..Default::default()
                }
            })
        }
    }

    #[tokio::test]
    async fn the_work_order_of_a_jobs_pod_is_looked_up_once() {
        let id = Uuid::new_v4();
        let mut job_pod = pod(&[], Some("Pending"));
        job_pod.metadata.namespace = Some("default".into());
        job_pod.metadata.owner_references = Some(vec![
            k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference {
                api_version: "batch/v1".into(),
                kind: "Job".into(),
                name: "migrate".into(),
                uid: "j".into(),
                controller: Some(true),
                block_owner_deletion: None,
            },
        ]);
        let mut fetcher = OneJob {
            work_order: id,
            reads: 0,
        };
        let mut cache: LruCache<String, Option<Uuid>> =
            LruCache::new(NonZeroUsize::new(8).unwrap());

        // The pod has no mark and no opt-in: it is a work order's through its Job.
        assert_eq!(stack_key(&job_pod), None);
        for _ in 0..3 {
            let key = work_order_key(&mut fetcher, &mut cache, "uid", &job_pod).await;
            assert_eq!(key, Some(TelemetryKey::WorkOrder(id)));
        }
        assert_eq!(
            fetcher.reads, 1,
            "each update of the pod must not read the Job again"
        );

        // A pod with no owner: no read, and "none" is remembered too.
        let bare = pod(&[], None);
        assert_eq!(
            work_order_key(&mut fetcher, &mut cache, "bare", &bare).await,
            None
        );
        assert_eq!(cache.peek("bare"), Some(&None));
        assert_eq!(fetcher.reads, 1);
    }

    #[test]
    fn rate_limiter_allows_under_ceiling() {
        let mut r = RateLimiter::new(10);
        for _ in 0..10 {
            assert!(matches!(r.consume(), Allowance::Allow));
        }
    }

    #[test]
    fn rate_limiter_drops_above_ceiling_with_first_gap() {
        let mut r = RateLimiter::new(2);
        assert!(matches!(r.consume(), Allowance::Allow));
        assert!(matches!(r.consume(), Allowance::Allow));
        // 3rd in this window: dropped, and it's the first drop → emit a gap.
        assert!(matches!(r.consume(), Allowance::DropAndGap(1)));
        // 4th in this window: still over budget → dropped silently, no gap.
        assert!(matches!(r.consume(), Allowance::Drop));
    }

    // BROKKR-T-0221: a uid is only re-attachable once all its tails finish, and
    // the stale entry is dropped at that point so ensure_tails re-tails on the
    // next watcher Apply (a pod that becomes loggable later is otherwise never
    // tailed again).
    #[tokio::test]
    async fn take_if_attachable_reattaches_only_after_all_tails_finish() {
        let mut map: HashMap<String, Vec<JoinHandle<()>>> = HashMap::new();

        // No entry yet -> attach.
        assert!(take_if_attachable(&mut map, "u"));

        // A still-running tail -> not attachable, entry preserved.
        let running = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });
        map.insert("u".to_string(), vec![running]);
        assert!(!take_if_attachable(&mut map, "u"));
        assert!(map.contains_key("u"));

        // Replace with a finished tail (abort the long-running one first).
        map.get("u").unwrap()[0].abort();
        let finished = tokio::spawn(async {});
        while !finished.is_finished() {
            tokio::task::yield_now().await;
        }
        map.insert("u".to_string(), vec![finished]);

        // All tails finished -> attach, and the stale entry is removed.
        assert!(take_if_attachable(&mut map, "u"));
        assert!(!map.contains_key("u"));
    }
}
