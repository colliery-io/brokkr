//! Overview — at-a-glance command view: KPI row, fleet health (segmented bar),
//! broker throughput (sparkline), and a live activity feed. Composes fleet +
//! /metrics + /agent-events. NOTE: the handoff's per-cluster fleet panel needs
//! cluster_name on the fleet record (gap); shown as overall fleet health here.
//! The 3 layout variants are deferred — this is the "command" layout.

use crate::api;
use crate::components::sev;
use aurora_leptos::components::*;
use aurora_leptos::data::{FeedList, FeedRow, Segment, SegmentedBar, Sparkline, StatTile};
use aurora_leptos::tokens::token;
use leptos::prelude::*;

#[component]
pub fn OverviewView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let fleet = LocalResource::new(move || api::fleet(scope.get()));
    let metrics = LocalResource::new(api::metrics_text);
    let events = LocalResource::new(move || api::agent_events(scope.get()));
    let history = RwSignal::new(Vec::<f64>::new());

    crate::components::poll(
        move || {
            fleet.refetch();
            metrics.refetch();
            events.refetch();
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

    view! {
        <Stack gap="md">
            // KPI row
            {move || match fleet.get() {
                Some(Ok(a)) => {
                    let total = a.len();
                    let active = a.iter().filter(|x| x.status.eq_ignore_ascii_case("active")).count();
                    let degraded = a.iter().filter(|x| x.health_degraded > 0 && x.health_failing == 0).count();
                    let failing = a.iter().filter(|x| x.health_failing > 0).count();
                    let healthy = total.saturating_sub(degraded + failing);
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

            <div class="brk-grid-2">
                // Fleet health
                <Panel title="Fleet health">
                    {move || match fleet.get() {
                        Some(Ok(a)) => {
                            let degraded = a.iter().filter(|x| x.health_degraded > 0 && x.health_failing == 0).count();
                            let failing = a.iter().filter(|x| x.health_failing > 0).count();
                            let offline = a.iter().filter(|x| !x.status.eq_ignore_ascii_case("active")).count();
                            let healthy = a.len().saturating_sub(degraded + failing + offline);
                            let segments = vec![
                                Segment::new("healthy", healthy as f64, token::OK),
                                Segment::new("degraded", degraded as f64, token::GOLD),
                                Segment::new("failing", failing as f64, token::BAD),
                                Segment::new("offline", offline as f64, token::MUTED),
                            ];
                            view! { <SegmentedBar segments=segments legend=true label="Fleet health" /> }
                                .into_any()
                        }
                        _ => view! { <Loading label="" /> }.into_any(),
                    }}
                </Panel>

                // Broker throughput
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
            <Panel title="Live activity">
                {move || match events.get() {
                    None => view! { <Loading label="" /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e on_retry=Callback::new(move |_| { events.refetch(); }) /> }.into_any(),
                    Some(Ok(evs)) if evs.is_empty() => view! { <Empty message="No recent activity." /> }.into_any(),
                    Some(Ok(evs)) => {
                        let rows = evs.into_iter().take(8).map(|e| {
                            // Agent-event statuses are Brokkr's (success/failure), not Aurora's.
                            let sc = sev(&e.status);
                            let msg = e.message.unwrap_or_default();
                            let agent: String = e.agent_id.chars().take(8).collect();
                            view! { <FeedRow dot=sc subject=e.event_type actor=agent>{msg}</FeedRow> }
                        }).collect_view();
                        view! { <div class="brk-feed-notime"><FeedList label="Live activity">{rows}</FeedList></div> }.into_any()
                    }
                }}
            </Panel>
        </Stack>
    }
}
