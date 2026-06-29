//! Fleet view — agents from `GET /api/v1/fleet` with a KPI strip and a row per
//! agent (status/health pills, `⇄ ws`, heartbeat "ago"). Clicking a row opens the
//! agent-detail **Modal** with the v1 **run-diagnostic** write (POST /diagnostics).
//! Flat list (no per-cluster grouping — the fleet record lacks cluster_name/labels).

use crate::api;
use crate::components::{toast, ToastBus};
use crate::models::FleetAgentRecord;
use crate::views::{ago, Kpi};
use aurora_leptos::components::*;
use aurora_leptos::tokens::{status_color, token};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn FleetView() -> impl IntoView {
    let data = LocalResource::new(|| api::fleet());
    set_interval(move || data.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<FleetAgentRecord>);
    let open = RwSignal::new(false);
    let bus = use_context::<ToastBus>();

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
                let active = agents.iter().filter(|a| a.status.eq_ignore_ascii_case("active")).count();
                let degraded = agents.iter().filter(|a| a.health_degraded > 0 && a.health_failing == 0).count();
                let failing = agents.iter().filter(|a| a.health_failing > 0).count();

                let rows = agents.iter().cloned().map(|a| {
                    let (h, hc) = a.health();
                    let sc = status_color(&a.status);
                    let live = a.status.eq_ignore_ascii_case("active");
                    let a_sel = a.clone();
                    view! {
                        <div style="cursor:pointer;border-radius:8px;padding:2px 4px;"
                             on:click=move |_| { selected.set(Some(a_sel.clone())); open.set(true); }>
                            <Group justify="between">
                                <Group gap="sm">
                                    <Dot color=sc glow=live />
                                    <span style="font:13px var(--font-mono);color:var(--fg);min-width:150px;">{a.name.clone()}</span>
                                    <Pill color=sc>{a.status.to_lowercase()}</Pill>
                                    <Pill color=hc>{h}</Pill>
                                    {a.ws_connected.then(|| view! {
                                        <span style="font:9.5px var(--font-mono);color:var(--teal);">"\u{21c4} ws"</span>
                                    })}
                                </Group>
                                <span style="font:11px var(--font-mono);color:var(--muted);">{ago(a.heartbeat_age_seconds)}</span>
                            </Group>
                        </div>
                    }
                }).collect_view();

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

        <Modal open=open title="Agent detail">
            {move || match selected.get() {
                None => ().into_any(),
                Some(a) => {
                    let (h, hc) = a.health();
                    let sc = status_color(&a.status);
                    let agent_id = a.agent_id.clone();
                    view! {
                        <Stack gap="md">
                            <span style="font:600 15px var(--font-mono);color:var(--fg-bright);">{a.name.clone()}</span>
                            <span style="font:11px var(--font-mono);color:var(--faint);">{a.agent_id.clone()}</span>
                            <Group gap="sm">
                                <Pill color=sc>{a.status.to_lowercase()}</Pill>
                                <Pill color=hc>{h}</Pill>
                                {a.ws_connected.then(|| view! {
                                    <span style="font:9.5px var(--font-mono);color:var(--teal);">"\u{21c4} ws"</span>
                                })}
                            </Group>
                            <span style="font:11px var(--font-mono);color:var(--muted);">
                                {format!("last heartbeat {}", ago(a.heartbeat_age_seconds))}
                            </span>
                            <Button on_click=Callback::new(move |_| {
                                let id = agent_id.clone();
                                if let Some(b) = bus { toast(b, "diagnostic requested", token::ICE); }
                                spawn_local(async move {
                                    let ok = api::create_diagnostic(&id).await.is_ok();
                                    if let Some(b) = bus {
                                        if ok { toast(b, "diagnostic queued", token::OK); }
                                        else { toast(b, "diagnostic failed", token::BAD); }
                                    }
                                });
                            })>"\u{2315} Run diagnostic"</Button>
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Modal>
    }
}
