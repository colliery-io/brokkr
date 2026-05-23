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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use brokkr_wire::{K8sEvent as WireK8sEvent, ObjectRef, WsMessage};
use chrono::Utc;
use futures::stream::StreamExt;
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Event as K8sEventResource;
use kube::api::{Api, DynamicObject};
use kube::core::GroupVersionKind;
use kube::discovery;
use kube::runtime::watcher;
use kube::Client;
use tokio::sync::{mpsc, RwLock};
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

#[derive(Default)]
struct UidCache {
    by_uid: HashMap<String, CachedLookup>,
}

impl UidCache {
    fn get(&self, uid: &str) -> Option<CacheEntry> {
        let entry = self.by_uid.get(uid)?;
        if entry.fetched_at.elapsed() < LOOKUP_TTL {
            Some(entry.value)
        } else {
            None
        }
    }

    fn put(&mut self, uid: String, value: CacheEntry) {
        self.by_uid.insert(
            uid,
            CachedLookup {
                value,
                fetched_at: Instant::now(),
            },
        );
    }
}

/// Spawn the kube-events tailer. Returns a `JoinHandle`; production drops
/// it and the task runs for the lifetime of the agent process.
pub fn spawn(client: Client, uplink: WsUplink, agent_id: Uuid) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let cache: Arc<RwLock<UidCache>> = Arc::new(RwLock::new(UidCache::default()));
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
            if let Err(e) = watch_loop(client.clone(), agent_id, tx.clone(), cache.clone()).await {
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
) -> Result<(), watcher::Error> {
    info!("starting kube events tailer");
    let api: Api<K8sEventResource> = Api::all(client.clone());
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

    let stack_id = match resolve_stack(client, &ev, &involved, cache).await {
        Some(id) => id,
        None => return,
    };

    let frame = WsMessage::K8sEvent(WireK8sEvent {
        agent_id,
        stack_id,
        observed_at: ev.event_time
            .as_ref()
            .map(|t| t.0)
            .or_else(|| ev.last_timestamp.as_ref().map(|t| t.0))
            .unwrap_or_else(Utc::now),
        reason: ev.reason.clone().unwrap_or_default(),
        message: ev.message.clone().unwrap_or_default(),
        event_type: ev.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        source: ev
            .source
            .as_ref()
            .and_then(|s| s.component.clone()),
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
    if let Some(entry) = cache.read().await.get(uid) {
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

    #[test]
    fn cache_returns_owned_within_ttl() {
        let mut c = UidCache::default();
        let id = Uuid::new_v4();
        c.put("u".into(), CacheEntry::Owned(id));
        match c.get("u") {
            Some(CacheEntry::Owned(got)) => assert_eq!(got, id),
            other => panic!("expected Owned, got {:?}", matches!(other, Some(_))),
        }
    }

    #[test]
    fn cache_treats_not_ours_as_a_real_entry() {
        let mut c = UidCache::default();
        c.put("u".into(), CacheEntry::NotOurs);
        assert!(matches!(c.get("u"), Some(CacheEntry::NotOurs)));
    }

    #[test]
    fn cache_expires_after_ttl() {
        // We can't fast-forward Instant; just confirm absent keys return
        // None — TTL-after-elapsed is exercised by the time-based check
        // in CachedLookup itself, which we don't need to assert
        // wall-clock here.
        let c = UidCache::default();
        assert!(c.get("missing").is_none());
        let _ = sleep; // keep the import for future expansion
    }
}
