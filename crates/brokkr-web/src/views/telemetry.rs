//! Telemetry view — Kube events / Pod logs tabs; click an event for detail.
//! NOTE: true kube events + pod logs are per-stack (`/stacks/:id/{events,logs}`),
//! with no global feed; the events tab binds to the global `/agent-events`
//! (Apply/Heartbeat/Reconcile lifecycle events) as the closest global stream, and
//! the logs tab needs a stack selected. REST-poll, 6h retention (logged on task).

use crate::api;
use crate::components::DetailRow;
use crate::models::AgentEventDto;
use aurora_leptos::components::*;
use aurora_leptos::tokens::status_color;
use leptos::prelude::*;

#[component]
pub fn TelemetryView() -> impl IntoView {
    let tab = RwSignal::new(String::from("Kube events"));
    let events = LocalResource::new(|| api::agent_events());
    set_interval(move || events.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<AgentEventDto>);
    let open = RwSignal::new(false);

    view! {
        <Stack gap="md">
            <Group justify="between">
                <SegmentedControl
                    value=tab
                    options=vec![String::from("Kube events"), String::from("Pod logs")]
                />
                <span style="font:11px var(--font-mono);color:var(--gold);">
                    "\u{26a0} 6h retention window \u{b7} ship to Datadog for long-term"
                </span>
            </Group>
            {move || {
                if tab.get() == "Pod logs" {
                    view! {
                        <Panel title="Pod logs">
                            <Empty message="Select a stack to tail its pod logs (per-stack; no global feed)." />
                        </Panel>
                    }
                    .into_any()
                } else {
                    match events.get() {
                        None => view! { <Loading label="loading events" /> }.into_any(),
                        Some(Err(e)) => view! {
                            <ErrorState error=e on_retry=Callback::new(move |_| { events.refetch(); }) />
                        }
                        .into_any(),
                        Some(Ok(evs)) if evs.is_empty() => {
                            view! { <Empty message="No agent events in the retention window." /> }
                                .into_any()
                        }
                        Some(Ok(evs)) => {
                            let rows = evs
                                .into_iter()
                                .map(|e| {
                                    let sc = status_color(&e.status);
                                    let msg = e.message.clone().unwrap_or_default();
                                    let e_sel = e.clone();
                                    view! {
                                        <div style="cursor:pointer;" on:click=move |_| {
                                            selected.set(Some(e_sel.clone()));
                                            open.set(true);
                                        }>
                                            <Group gap="sm">
                                                <Pill color=sc>{e.event_type.clone()}</Pill>
                                                <span style="font:11px var(--font-mono);\
                                                             color:var(--muted);">{e.status.clone()}</span>
                                                <span style="font:12px var(--font-sans);\
                                                             color:var(--fg-2);max-width:600px;\
                                                             overflow:hidden;text-overflow:ellipsis;\
                                                             white-space:nowrap;">{msg}</span>
                                            </Group>
                                        </div>
                                    }
                                })
                                .collect_view();
                            view! {
                                <Panel title="Agent events">
                                    <Stack gap="sm">{rows}</Stack>
                                </Panel>
                            }
                            .into_any()
                        }
                    }
                }
            }}
        </Stack>

        <Modal open=open title="Event detail">
            {move || match selected.get() {
                None => ().into_any(),
                Some(e) => {
                    let sc = status_color(&e.status);
                    view! {
                        <Stack gap="md">
                            <Group gap="sm">
                                <Pill color=sc>{e.event_type.clone()}</Pill>
                                <span style="font:11px var(--font-mono);color:var(--muted);">{e.status.clone()}</span>
                            </Group>
                            <div>
                                <DetailRow label="agent">{e.agent_id.clone()}</DetailRow>
                            </div>
                            <span style="font:12px var(--font-sans);color:var(--fg);line-height:1.5;">
                                {e.message.clone().unwrap_or_else(|| "(no message)".into())}
                            </span>
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Modal>
    }
}
