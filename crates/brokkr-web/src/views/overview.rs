//! Overview — at-a-glance command view: KPI row, fleet health (segmented bar),
//! broker throughput (sparkline), and a live activity feed. Composes fleet +
//! /metrics + /agent-events. NOTE: the handoff's per-cluster fleet panel needs
//! cluster_name on the fleet record (gap); shown as overall fleet health here.
//! The 3 layout variants are deferred — this is the "command" layout.

use crate::api;
use crate::components::{SegmentedHealthBar, Sparkline};
use crate::views::Kpi;
use aurora_leptos::components::*;
use aurora_leptos::tokens::{status_color, token};
use leptos::prelude::*;

#[component]
pub fn OverviewView() -> impl IntoView {
    let fleet = LocalResource::new(|| api::fleet());
    let metrics = LocalResource::new(|| api::metrics_text());
    let events = LocalResource::new(|| api::agent_events());
    let history = RwSignal::new(Vec::<f64>::new());

    set_interval(
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
                        <Group gap="md" wrap=true>
                            <Kpi label="active agents" value=format!("{active}/{total}") color="var(--fg-bright)" />
                            <Kpi label="healthy" value=healthy.to_string() color=token::OK />
                            <Kpi label="degraded" value=degraded.to_string() color=token::GOLD />
                            <Kpi label="failing" value=failing.to_string() color=token::BAD />
                        </Group>
                    }
                    .into_any()
                }
                Some(Err(_)) => view! { <Empty message="fleet unavailable" /> }.into_any(),
                None => view! { <Loading label="loading" /> }.into_any(),
            }}

            <div style="display:grid;grid-template-columns:1fr 1fr;gap:13px;">
                // Fleet health
                <Panel title="Fleet health">
                    {move || match fleet.get() {
                        Some(Ok(a)) => {
                            let degraded = a.iter().filter(|x| x.health_degraded > 0 && x.health_failing == 0).count();
                            let failing = a.iter().filter(|x| x.health_failing > 0).count();
                            let offline = a.iter().filter(|x| !x.status.eq_ignore_ascii_case("active")).count();
                            let healthy = a.len().saturating_sub(degraded + failing + offline);
                            view! {
                                <Stack gap="sm">
                                    <SegmentedHealthBar healthy=healthy degraded=degraded failing=failing offline=offline />
                                    <Group gap="md">
                                        <span style="font:11px var(--font-mono);color:var(--ok);">{format!("{healthy} healthy")}</span>
                                        <span style="font:11px var(--font-mono);color:var(--gold);">{format!("{degraded} degraded")}</span>
                                        <span style="font:11px var(--font-mono);color:var(--bad);">{format!("{failing} failing")}</span>
                                    </Group>
                                </Stack>
                            }.into_any()
                        }
                        _ => view! { <Loading label="" /> }.into_any(),
                    }}
                </Panel>

                // Broker throughput
                <Panel title="Broker throughput">
                    {move || {
                        let h = history.get();
                        let last = h.last().copied().unwrap_or(0.0);
                        view! {
                            <Stack gap="sm">
                                <span style="font:600 22px var(--font-mono);color:var(--ice);font-variant-numeric:tabular-nums;">
                                    {format!("{} req", last as i64)}
                                </span>
                                <Sparkline values=h color=token::ICE />
                            </Stack>
                        }
                    }}
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
                            let sc = status_color(&e.status);
                            let msg = e.message.unwrap_or_default();
                            view! {
                                <Group gap="sm">
                                    <Dot color=sc />
                                    <span style="font:11px var(--font-mono);color:var(--muted);min-width:90px;">{e.event_type}</span>
                                    <span style="font:12px var(--font-sans);color:var(--fg-2);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{msg}</span>
                                </Group>
                            }
                        }).collect_view();
                        view! { <Stack gap="sm">{rows}</Stack> }.into_any()
                    }
                }}
            </Panel>
        </Stack>
    }
}
