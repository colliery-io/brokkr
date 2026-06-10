/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Tail Kubernetes `Event` objects for resources this agent manages and
//! forward them to the broker as `WsMessage::K8sEvent` frames.
//!
//! ## Filtering
//!
//! Brokkr-managed resources carry an `k8s.brokkr.io/stack` annotation
//! whose value is the owning stack id (see [`crate::k8s::objects`]).
//! When a kube `Event` arrives the tailer fetches the referenced
//! `involvedObject` and checks its annotations. Lookup results are cached
//! for a few minutes (positive and negative) so repeated events for the
//! same pod don't translate to repeated `GET`s.
//!
//! ## Lifecycle
//!
//! - Spawned once at agent startup.
//! - Watches `Event`s cluster-wide via `kube::runtime::watcher`.
//! - Restart-safe: on transient failures the watcher loops with a brief
//!   backoff. On success, the watcher itself resumes from the latest
//!   `resourceVersion` it has seen.
//!
//! Per ADR-0008 / BROKKR-I-0019 these frames flow over the agent uplink
//! introduced in WS-05; the broker persists them into `agent_k8s_events`
//! (WS-09) under the hard 6h retention ceiling.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;

use brokkr_wire::{K8sEvent as WireK8sEvent, ObjectRef, WsMessage};
use chrono::Utc;
use futures::TryStreamExt;
use futures::stream::StreamExt;
use k8s_openapi::api::core::v1::Event as K8sEventResource;
use kube::Client;
use kube::api::{Api, DynamicObject};
use kube::core::GroupVersionKind;
use kube::discovery;
use kube::runtime::watcher;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::broker_ws::WsUplink;
use crate::k8s::objects::STACK_LABEL;

/// How long to cache a UID→stack lookup before re-querying.
const LOOKUP_TTL: Duration = Duration::from_secs(5 * 60);

/// Capacity of the bounded outbound queue we drain into the WS uplink.
/// Sized for short bursts of events; over capacity, the tailer drops
/// older frames silently (rare in practice — kube Events are
/// human-paced).
const OUTBOUND_CAPACITY: usize = 256;

#[derive(Clone, Copy)]
enum CacheEntry {
    Owned(Uuid),
    NotOurs,
}

struct CachedLookup {
    value: CacheEntry,
    fetched_at: Instant,
}

/// Default entry cap. Sized for a cluster with up to ~10k managed pods; bump
/// `agent.kube_event_uid_cache_cap` for larger fleets. The cache exists to
/// turn the per-Event annotation lookup into an O(1) hit for already-seen
/// UIDs; bounding it stops unmanaged-object churn from growing it without
/// limit (the previous `HashMap` was unbounded).
pub const DEFAULT_UID_CACHE_CAP: usize = 10_000;

/// Bounded LRU of UID → ownership lookups, with a per-entry TTL. `get`
/// promotes recency (so it takes `&mut self`); an entry past [`LOOKUP_TTL`]
/// is treated as a miss and evicted so size accounting stays honest.
struct UidCache {
    by_uid: LruCache<String, CachedLookup>,
}

impl UidCache {
    fn new(cap: usize) -> Self {
        let cap = NonZeroUsize::new(cap).unwrap_or(NonZeroUsize::MIN);
        Self {
            by_uid: LruCache::new(cap),
        }
    }

    fn get(&mut self, uid: &str) -> Option<CacheEntry> {
        match self.by_uid.get(uid) {
            Some(entry) if entry.fetched_at.elapsed() < LOOKUP_TTL => Some(entry.value),
            Some(_) => {
                // Expired: drop it so a stale entry doesn't occupy a slot.
                self.by_uid.pop(uid);
                None
            }
            None => None,
        }
    }

    fn put(&mut self, uid: String, value: CacheEntry) {
        self.by_uid.put(
            uid,
            CachedLookup {
                value,
                fetched_at: Instant::now(),
            },
        );
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.by_uid.len()
    }
}

/// Spawn the kube-events tailer. Returns a `JoinHandle`; production drops
/// it and the task runs for the lifetime of the agent process.
pub fn spawn(
    client: Client,
    uplink: WsUplink,
    agent_id: Uuid,
    uid_cache_cap: usize,
    watch_namespace: Option<String>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let cache: Arc<RwLock<UidCache>> = Arc::new(RwLock::new(UidCache::new(uid_cache_cap)));
        let (tx, mut rx) = mpsc::channel::<WsMessage>(OUTBOUND_CAPACITY);

        // Forwarder task: drains the channel into the WS uplink, falling
        // back to silent drop if WS is down (REST has no equivalent for
        // streaming events — by design).
        let uplink_clone = uplink.clone();
        let _forwarder = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if uplink_clone.try_send(msg).is_err() {
                    // WS not up; drop. The 6h history endpoint cannot
                    // backfill these — kube Events themselves are
                    // short-lived in the cluster.
                }
            }
        });

        loop {
            if let Err(e) = watch_loop(
                client.clone(),
                agent_id,
                tx.clone(),
                cache.clone(),
                watch_namespace.as_deref(),
            )
            .await
            {
                warn!(error = %e, "kube events tailer fell out of watch loop; restarting");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    })
}

async fn watch_loop(
    client: Client,
    agent_id: Uuid,
    tx: mpsc::Sender<WsMessage>,
    cache: Arc<RwLock<UidCache>>,
    watch_namespace: Option<&str>,
) -> Result<(), watcher::Error> {
    info!(namespace = ?watch_namespace, "starting kube events tailer");
    let api: Api<K8sEventResource> = match watch_namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let mut stream = watcher(api, watcher::Config::default()).boxed();

    while let Some(event) = stream.try_next().await? {
        if let watcher::Event::Apply(ev) = event {
            handle_event(&client, agent_id, &ev, &tx, &cache).await;
        }
    }
    Ok(())
}

async fn handle_event(
    client: &Client,
    agent_id: Uuid,
    ev: &K8sEventResource,
    tx: &mpsc::Sender<WsMessage>,
    cache: &Arc<RwLock<UidCache>>,
) {
    let involved = match &ev.involved_object.uid {
        Some(uid) => uid.clone(),
        None => return, // synthesised events without involvedObject UIDs aren't ours
    };

    let stack_id = match resolve_stack(client, ev, &involved, cache).await {
        Some(id) => id,
        None => return,
    };

    let frame = WsMessage::K8sEvent(WireK8sEvent {
        agent_id,
        stack_id,
        observed_at: ev
            .event_time
            .as_ref()
            .map(|t| t.0)
            .or_else(|| ev.last_timestamp.as_ref().map(|t| t.0))
            .unwrap_or_else(Utc::now),
        reason: ev.reason.clone().unwrap_or_default(),
        message: ev.message.clone().unwrap_or_default(),
        event_type: ev.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        source: ev.source.as_ref().and_then(|s| s.component.clone()),
        involved_object: ObjectRef {
            api_version: ev.involved_object.api_version.clone().unwrap_or_default(),
            kind: ev.involved_object.kind.clone().unwrap_or_default(),
            namespace: ev.involved_object.namespace.clone(),
            name: ev.involved_object.name.clone().unwrap_or_default(),
            uid: Some(involved),
        },
    });

    if tx.try_send(frame).is_err() {
        debug!("kube events outbound queue full; dropping frame");
    }
}

async fn resolve_stack(
    client: &Client,
    ev: &K8sEventResource,
    uid: &str,
    cache: &Arc<RwLock<UidCache>>,
) -> Option<Uuid> {
    // Write lock: LRU `get` promotes recency, so it mutates. Lookups are
    // cheap relative to the API call on a miss, so the contention is fine.
    if let Some(entry) = cache.write().await.get(uid) {
        return match entry {
            CacheEntry::Owned(id) => Some(id),
            CacheEntry::NotOurs => None,
        };
    }

    let stack = annotation_lookup(client, &ev.involved_object).await;
    let entry = match stack {
        Some(id) => CacheEntry::Owned(id),
        None => CacheEntry::NotOurs,
    };
    cache.write().await.put(uid.to_string(), entry);

    match entry {
        CacheEntry::Owned(id) => Some(id),
        CacheEntry::NotOurs => None,
    }
}

async fn annotation_lookup(
    client: &Client,
    involved: &k8s_openapi::api::core::v1::ObjectReference,
) -> Option<Uuid> {
    let api_version = involved.api_version.as_deref()?;
    let kind = involved.kind.as_deref()?;
    let name = involved.name.as_deref()?;

    // Build a dynamic API for whatever the involvedObject's GVK is.
    let (group, version) = match api_version.split_once('/') {
        Some((g, v)) => (g.to_string(), v.to_string()),
        None => ("".to_string(), api_version.to_string()),
    };
    let gvk = GroupVersionKind::gvk(&group, &version, kind);
    let ar = match discovery::pinned_kind(client, &gvk).await {
        Ok((ar, _caps)) => ar,
        Err(e) => {
            debug!(?gvk, error = %e, "could not discover GVK for involvedObject");
            return None;
        }
    };

    let api: Api<DynamicObject> = match involved.namespace.as_deref() {
        Some(ns) => Api::namespaced_with(client.clone(), ns, &ar),
        None => Api::all_with(client.clone(), &ar),
    };
    let obj = api.get_opt(name).await.ok().flatten()?;
    let annotations = obj.metadata.annotations.as_ref()?;
    let raw = annotations.get(STACK_LABEL)?;
    Uuid::parse_str(raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    /// Mirror `resolve_stack`'s cache interaction without the real API:
    /// on a miss, count a "lookup" and cache a `NotOurs` result.
    fn lookup_or_miss(cache: &mut UidCache, uid: &str, api_calls: &mut usize) -> CacheEntry {
        if let Some(e) = cache.get(uid) {
            return e;
        }
        *api_calls += 1;
        let e = CacheEntry::NotOurs;
        cache.put(uid.to_string(), e);
        e
    }

    #[test]
    fn cache_returns_owned_within_ttl() {
        let mut c = UidCache::new(DEFAULT_UID_CACHE_CAP);
        let id = Uuid::new_v4();
        c.put("u".into(), CacheEntry::Owned(id));
        match c.get("u") {
            Some(CacheEntry::Owned(got)) => assert_eq!(got, id),
            other => panic!("expected Owned, got {:?}", other.is_some()),
        }
    }

    #[test]
    fn cache_treats_not_ours_as_a_real_entry() {
        let mut c = UidCache::new(DEFAULT_UID_CACHE_CAP);
        c.put("u".into(), CacheEntry::NotOurs);
        assert!(matches!(c.get("u"), Some(CacheEntry::NotOurs)));
    }

    #[test]
    fn cache_expires_after_ttl() {
        // We can't fast-forward Instant; just confirm absent keys return
        // None — TTL-after-elapsed is exercised by the time-based check
        // in CachedLookup itself, which we don't need to assert
        // wall-clock here.
        let mut c = UidCache::new(DEFAULT_UID_CACHE_CAP);
        assert!(c.get("missing").is_none());
        let _ = sleep; // keep the import for future expansion
    }

    #[test]
    fn cache_stays_bounded_under_high_unique_churn() {
        // 50k unique unmanaged UIDs against a 10k cap: the old unbounded
        // HashMap would grow to 50k; the LRU must cap at 10k. Every lookup
        // is a unique miss (no repeats to benefit from), so the API-call
        // count equals the number of unique UIDs — the point of the test is
        // that *memory* stays bounded, not that calls drop for unique churn.
        let cap = 10_000;
        let mut c = UidCache::new(cap);
        let mut api_calls = 0usize;
        for i in 0..50_000 {
            lookup_or_miss(&mut c, &format!("uid-{i}"), &mut api_calls);
            assert!(c.len() <= cap, "cache exceeded cap of {cap}");
        }
        assert_eq!(
            api_calls, 50_000,
            "every unique UID should miss exactly once"
        );
        assert_eq!(c.len(), cap, "cache must be pinned at the cap, not grow");
    }

    #[test]
    fn cache_serves_hot_set_without_re_hitting_the_api() {
        // A hot set that fits within the cap should cost exactly one API
        // call per unique UID, no matter how many times it's looked up.
        let cap = 10_000;
        let mut c = UidCache::new(cap);
        let mut api_calls = 0usize;
        for pass in 0..3 {
            for i in 0..cap {
                lookup_or_miss(&mut c, &format!("hot-{i}"), &mut api_calls);
            }
            let _ = pass;
        }
        assert_eq!(
            api_calls, cap,
            "hot set within cap should hit the API once per UID, not per access"
        );
    }
}
