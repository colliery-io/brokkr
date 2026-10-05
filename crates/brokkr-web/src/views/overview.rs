//! Overview — at-a-glance command view. Five widgets: the KPI row, the fleet
//! by cluster, the broker throughput (sparkline), the live activity feed and
//! the active work orders. Composes /fleet + /metrics + /agent-events +
//! /work-orders. A segmented control arranges the widgets in one of the three
//! layouts of design/README.md (`command`, `grid`, `stream`); the choice is in
//! localStorage. The design's sixth widget (deployment health for each stack)
//! is not here: it needs `/stacks/:id/health` for each stack on each poll, and
//! the Deployments view shows it on demand (BROKKR-T-0328).

use crate::api;
use crate::components::{sev, EmptyNext, DOCS};
use crate::models::FleetAgentRecord;
use aurora_leptos::components::*;
use aurora_leptos::data::{FeedList, FeedRow, Segment, SegmentedBar, Sparkline, StatTile};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use std::collections::BTreeMap;

/// The three arrangements of the widgets, in the order of the control.
pub const LAYOUTS: [&str; 3] = ["command", "grid", "stream"];
const LAYOUT_STORAGE_KEY: &str = "brokkr_overview_layout";

/// A stored layout name, or the default (`command`) for none or an unknown one.
pub fn layout_or_default(stored: Option<&str>) -> &'static str {
    stored
        .and_then(|s| LAYOUTS.iter().find(|l| **l == s))
        .copied()
        .unwrap_or(LAYOUTS[0])
}

fn load_layout() -> &'static str {
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|ls| ls.get_item(LAYOUT_STORAGE_KEY).ok().flatten());
    layout_or_default(stored.as_deref())
}

fn save_layout(layout: &str) {
    if let Some(ls) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = ls.set_item(LAYOUT_STORAGE_KEY, layout);
    }
}

/// The health of one cluster's agents, for the "Fleet by cluster" panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterHealth {
    pub name: String,
    pub up: usize,
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub failing: usize,
    pub offline: usize,
}

impl ClusterHealth {
    /// The hue of the cluster's dot: its worst bucket.
    pub fn hue(&self) -> &'static str {
        if self.failing > 0 {
            token::BAD
        } else if self.degraded > 0 {
            token::GOLD
        } else if self.up == 0 {
            token::MUTED
        } else {
            token::OK
        }
    }
}

/// Group the fleet by `cluster_name` (empty -> "(unknown)"), sorted by name.
/// An agent that is not active is `offline`; an active one is `failing`,
/// `degraded` or `healthy` from its counts, the same buckets as the Fleet view.
pub fn cluster_rollup(agents: &[FleetAgentRecord]) -> Vec<ClusterHealth> {
    let mut by_cluster: BTreeMap<String, ClusterHealth> = BTreeMap::new();
    for a in agents {
        let key = if a.cluster_name.is_empty() {
            "(unknown)".to_string()
        } else {
            a.cluster_name.clone()
        };
        let c = by_cluster
            .entry(key.clone())
            .or_insert_with(|| ClusterHealth {
                name: key,
                up: 0,
                total: 0,
                healthy: 0,
                degraded: 0,
                failing: 0,
                offline: 0,
            });
        c.total += 1;
        // An agent that is not active, or that has never checked in, is offline.
        if !a.is_active() || !a.has_checked_in() {
            c.offline += 1;
            continue;
        }
        c.up += 1;
        match a.health().0 {
            "failing" => c.failing += 1,
            "degraded" => c.degraded += 1,
            _ => c.healthy += 1,
        }
    }
    by_cluster.into_values().collect()
}

/// `created_at` of an event as ms since the epoch, for `FeedRow at`. None for
/// no time, or one the browser cannot read.
pub fn event_at(created_at: Option<&str>) -> Option<f64> {
    let ms = js_sys::Date::parse(created_at?);
    ms.is_finite().then_some(ms)
}

#[component]
pub fn OverviewView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let fleet = LocalResource::new(move || api::fleet(scope.get()));
    let metrics = LocalResource::new(api::metrics_text);
    let events = LocalResource::new(move || api::agent_events(scope.get()));
    let orders = LocalResource::new(api::work_orders);
    let history = RwSignal::new(Vec::<f64>::new());

    let layout = RwSignal::new(load_layout().to_string());
    Effect::new(move |_| save_layout(&layout.get()));

    crate::components::poll(
        move || {
            fleet.refetch();
            metrics.refetch();
            events.refetch();
            orders.refetch();
        },
        std::time::Duration::from_secs(5),
    );

    // Accumulate the http-requests counter into a 44-point ring for the sparkline.
    Effect::new(move |_| {
        if let Some(Ok(text)) = metrics.get() {
            if let Some(v) = api::metric_sum(&text, "brokkr_http_requests_total") {
                history.update(|h| {
                    if h.is_empty() {
                        // seed a short ramp so the sparkline has shape on first paint
                        for k in 0..8 {
                            h.push(v * (0.85 + 0.02 * k as f64));
                        }
                    }
                    h.push(v);
                    while h.len() > 44 {
                        h.remove(0);
                    }
                });
            }
        }
    });

    let options: Vec<String> = LAYOUTS.iter().map(|l| l.to_string()).collect();

    view! {
        <Stack gap="md">
            <div class="brk-topbar" role="group" aria-label="Overview layout">
                <span class="brk-meta">"layout"</span>
                <SegmentedControl options=options value=layout />
            </div>

            <div class=move || format!("brk-overview brk-overview--{}", layout.get())>
                // KPI row
                <div class="brk-ov-kpis">
                    {move || match fleet.get() {
                        Some(Ok(a)) => {
                            let total = a.len();
                            let active = a.iter().filter(|x| x.status.eq_ignore_ascii_case("active")).count();
                            // An agent that never checked in is "unknown": in no bucket.
                            let degraded = a.iter().filter(|x| x.health().0 == "degraded").count();
                            let failing = a.iter().filter(|x| x.health().0 == "failing").count();
                            let healthy = a.iter().filter(|x| x.health().0 == "healthy").count();
                            view! {
                                <div class="brk-kpis">
                                    <StatTile label="active agents" value=format!("{active}/{total}") />
                                    <StatTile label="healthy" value=healthy.to_string() color=token::OK />
                                    <StatTile label="degraded" value=degraded.to_string() color=token::GOLD />
                                    <StatTile label="failing" value=failing.to_string() color=token::BAD />
                                </div>
                            }
                            .into_any()
                        }
                        Some(Err(_)) => view! { <Empty message="fleet unavailable" /> }.into_any(),
                        None => view! { <Loading label="loading" /> }.into_any(),
                    }}
                </div>

                // Fleet by cluster
                <div class="brk-ov-fleet">
                    <Panel title="Fleet by cluster">
                        {move || match fleet.get() {
                            Some(Ok(a)) if a.is_empty() => view! {
                                <EmptyNext
                                    message="No agents yet."
                                    next="Create an agent record, start the agent with its PAK, then activate it from Fleet."
                                    href=format!("{DOCS}/how-to/agent-registration.html")
                                    link="How an agent registers"
                                />
                            }.into_any(),
                            Some(Ok(a)) => {
                                let rows = cluster_rollup(&a).into_iter().map(|c| {
                                    let segments = vec![
                                        Segment::new("healthy", c.healthy as f64, token::OK),
                                        Segment::new("degraded", c.degraded as f64, token::GOLD),
                                        Segment::new("failing", c.failing as f64, token::BAD),
                                        Segment::new("offline", c.offline as f64, token::MUTED),
                                    ];
                                    let label = format!("Health of the agents in {}", c.name);
                                    let caption = format!("{}/{} up", c.up, c.total);
                                    view! {
                                        <Stack gap="xs">
                                            <Group gap="sm">
                                                <Dot color=c.hue() />
                                                <span class="brk-value brk-ellipsis">{c.name.clone()}</span>
                                                <span class="brk-meta brk-grow brk-right">{caption}</span>
                                            </Group>
                                            <SegmentedBar segments=segments label=label height=7 />
                                        </Stack>
                                    }
                                }).collect_view();
                                view! { <Stack gap="sm">{rows}</Stack> }.into_any()
                            }
                            Some(Err(_)) => view! { <Empty message="fleet unavailable" /> }.into_any(),
                            None => view! { <Loading label="" /> }.into_any(),
                        }}
                    </Panel>
                </div>

                // Broker throughput
                <div class="brk-ov-flow">
                    <Panel title="Broker throughput">
                        <Stack gap="sm">
                            <span class="brk-big">
                                {move || {
                                    let last = history.with(|h| h.last().copied().unwrap_or(0.0));
                                    format!("{} req", last as i64)
                                }}
                            </span>
                            <Sparkline
                                values=history
                                fill=true
                                fluid=true
                                height=52.0
                                color=token::ICE
                                label="HTTP requests, recent samples"
                            />
                        </Stack>
                    </Panel>
                </div>

                // Live activity
                <div class="brk-ov-stream">
                    <Panel title="Live activity">
                        {move || match events.get() {
                            None => view! { <Loading label="" /> }.into_any(),
                            Some(Err(e)) => view! { <ErrorState error=e on_retry=Callback::new(move |_| { events.refetch(); }) /> }.into_any(),
                            Some(Ok(evs)) if evs.is_empty() => view! {
                                <EmptyNext
                                    message="No activity yet."
                                    next="An agent reports an event each time it applies, reconciles or heartbeats."
                                />
                            }.into_any(),
                            Some(Ok(evs)) => {
                                let rows = evs.into_iter().take(8).map(|e| {
                                    // Agent-event statuses are Brokkr's (success/failure), not Aurora's.
                                    let sc = sev(&e.status);
                                    let msg = e.message.unwrap_or_default();
                                    let agent: String = e.agent_id.chars().take(8).collect();
                                    // FeedRow strips the Option of `at`: a row with no time passes none.
                                    match event_at(e.created_at.as_deref()) {
                                        Some(t) => view! { <FeedRow at=t dot=sc subject=e.event_type actor=agent>{msg}</FeedRow> }.into_any(),
                                        None => view! { <FeedRow dot=sc subject=e.event_type actor=agent>{msg}</FeedRow> }.into_any(),
                                    }
                                }).collect_view();
                                view! { <FeedList label="Live activity">{rows}</FeedList> }.into_any()
                            }
                        }}
                    </Panel>
                </div>

                // Active work orders
                <div class="brk-ov-jobs">
                    <Panel title="Work orders">
                        {move || match orders.get() {
                            None => view! { <Loading label="" /> }.into_any(),
                            Some(Err(e)) => view! { <ErrorState error=e on_retry=Callback::new(move |_| { orders.refetch(); }) /> }.into_any(),
                            Some(Ok(wos)) => {
                                let act: Vec<_> = wos.into_iter().filter(|w| w.is_active()).collect();
                                if act.is_empty() {
                                    return view! {
                                        <EmptyNext
                                            message="No active work orders."
                                            next="A work order appears when a tenant requests an image build."
                                            href=format!("{DOCS}/reference/work-orders.html")
                                            link="What a work order is"
                                        />
                                    }.into_any();
                                }
                                let rows = act.into_iter().take(8).map(|w| {
                                    let id8: String = w.id.chars().take(8).collect();
                                    view! {
                                        <Group gap="sm">
                                            <span class="brk-value">{id8}</span>
                                            <Pill color=token::TEAL>{w.work_type}</Pill>
                                            <Pill color=sev(&w.status)>{w.status}</Pill>
                                        </Group>
                                    }
                                }).collect_view();
                                view! { <Stack gap="xs">{rows}</Stack> }.into_any()
                            }
                        }}
                    </Panel>
                </div>
            </div>
        </Stack>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An agent the broker has heard from (`seen`), or a bare record.
    fn agent(
        cluster: &str,
        status: &str,
        degraded: i64,
        failing: i64,
        seen: bool,
    ) -> FleetAgentRecord {
        let mut v = serde_json::json!({
            "agent_id": "a", "name": "n", "cluster_name": cluster, "status": status,
            "ws_connected": false, "health_degraded": degraded, "health_failing": failing,
        });
        if seen {
            v["heartbeat_age_seconds"] = serde_json::json!(5);
        }
        serde_json::from_value(v).expect("fleet record")
    }

    #[test]
    fn the_rollup_groups_by_cluster_and_buckets_each_agent() {
        let fleet = [
            agent("prod", "ACTIVE", 0, 0, true),
            agent("prod", "ACTIVE", 1, 0, true),
            agent("prod", "ACTIVE", 1, 2, true),
            agent("prod", "INACTIVE", 0, 5, true),
            agent("prod", "ACTIVE", 0, 0, false),
            agent("staging", "ACTIVE", 0, 0, true),
            agent("", "INACTIVE", 0, 0, false),
        ];
        let got = cluster_rollup(&fleet);
        let names: Vec<_> = got.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["(unknown)", "prod", "staging"]);
        let prod = &got[1];
        assert_eq!((prod.up, prod.total), (3, 5));
        assert_eq!(
            (prod.healthy, prod.degraded, prod.failing, prod.offline),
            (1, 1, 1, 2),
            "an inactive agent is offline, whatever its counts; so is one that never checked in"
        );
        assert_eq!(prod.hue(), token::BAD);
        assert_eq!(got[2].hue(), token::OK);
        assert_eq!(
            got[0].hue(),
            token::MUTED,
            "a cluster with nothing up is muted"
        );
    }

    #[test]
    fn an_agent_that_never_checked_in_has_unknown_health() {
        let new = agent("prod", "INACTIVE", 0, 0, false);
        assert_eq!(new.health().0, "unknown");
        assert!(!new.has_checked_in());
        let seen = agent("prod", "INACTIVE", 0, 0, true);
        assert_eq!(seen.health().0, "healthy");
        assert!(seen.has_checked_in());
    }

    #[test]
    fn the_layout_falls_back_to_command() {
        assert_eq!(layout_or_default(None), "command");
        assert_eq!(layout_or_default(Some("stream")), "stream");
        assert_eq!(layout_or_default(Some("grid")), "grid");
        assert_eq!(layout_or_default(Some("bogus")), "command");
    }
}
