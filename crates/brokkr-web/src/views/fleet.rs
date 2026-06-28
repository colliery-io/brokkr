//! Fleet view — agents from `GET /api/v1/fleet`, with a KPI strip and a row per
//! agent (status/health pills, `⇄ ws`, heartbeat "ago"). The fleet record has no
//! `cluster_name`/`labels`, so this is a flat list rather than the handoff's
//! per-cluster grouping (that needs a broker enhancement — logged on the task).

use crate::api;
use crate::views::{ago, Kpi};
use aurora_leptos::components::*;
use aurora_leptos::tokens::{status_color, token};
use leptos::prelude::*;

#[component]
pub fn FleetView() -> impl IntoView {
    let data = LocalResource::new(|| api::fleet());
    // Interim live refresh: re-poll every 5s. The richer `/fleet/live` WS push +
    // Live/Paused gating is folded into the live-engine task (BROKKR-T-0256).
    set_interval(
        move || data.refetch(),
        std::time::Duration::from_secs(5),
    );

    view! {
        {move || match data.get() {
            None => view! { <Loading label="loading fleet" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(agents)) if agents.is_empty() => {
                view! { <Empty message="No agents registered with this broker." /> }.into_any()
            }
            Some(Ok(agents)) => {
                let total = agents.len();
                let active = agents
                    .iter()
                    .filter(|a| a.status.eq_ignore_ascii_case("active"))
                    .count();
                let degraded = agents
                    .iter()
                    .filter(|a| a.health_degraded > 0 && a.health_failing == 0)
                    .count();
                let failing = agents.iter().filter(|a| a.health_failing > 0).count();

                let rows = agents
                    .iter()
                    .cloned()
                    .map(|a| {
                        let (h, hc) = a.health();
                        let sc = status_color(&a.status);
                        let live = a.status.eq_ignore_ascii_case("active");
                        view! {
                            <Group justify="between">
                                <Group gap="sm">
                                    <Dot color=sc glow=live />
                                    <span style="font:13px var(--font-mono);color:var(--fg);\
                                                 min-width:150px;">{a.name.clone()}</span>
                                    <Pill color=sc>{a.status.to_lowercase()}</Pill>
                                    <Pill color=hc>{h}</Pill>
                                    {a.ws_connected.then(|| view! {
                                        <span style="font:9.5px var(--font-mono);color:var(--teal);">
                                            "⇄ ws"
                                        </span>
                                    })}
                                </Group>
                                <span style="font:11px var(--font-mono);color:var(--muted);">
                                    {ago(a.heartbeat_age_seconds)}
                                </span>
                            </Group>
                        }
                    })
                    .collect_view();

                view! {
                    <Stack gap="md">
                        <Group gap="md" wrap=true>
                            <Kpi label="agents" value=total.to_string() color="var(--fg-bright)" />
                            <Kpi label="active" value=active.to_string() color=token::OK />
                            <Kpi label="degraded" value=degraded.to_string() color=token::GOLD />
                            <Kpi label="failing" value=failing.to_string() color=token::BAD />
                        </Group>
                        <Panel title="Agents">
                            <Stack gap="sm">{rows}</Stack>
                        </Panel>
                    </Stack>
                }
                .into_any()
            }
        }}
    }
}
