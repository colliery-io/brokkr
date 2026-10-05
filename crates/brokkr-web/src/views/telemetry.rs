//! Telemetry view — Kube events / Pod logs tabs; click an event for detail.
//! NOTE: true kube events + pod logs are per-stack (`/stacks/:id/{events,logs}`),
//! with no global feed; the events tab binds to the global `/agent-events`
//! (Apply/Heartbeat/Reconcile lifecycle events) as the closest global stream, and
//! the logs tab needs a stack selected. REST-poll, 6h retention (logged on task).

use crate::api;
use crate::components::sev;
use crate::models::AgentEventDto;
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, FeedList, FeedRow, KeyValue};
use aurora_leptos::frame::{Drawer, TabItem, TabPanel, Tabs};
use leptos::prelude::*;

#[component]
pub fn TelemetryView() -> impl IntoView {
    let tab = RwSignal::new(String::from("events"));
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let events = LocalResource::new(move || api::agent_events(scope.get()));
    crate::components::poll(move || events.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<AgentEventDto>);
    let open = RwSignal::new(false);

    view! {
        <Stack gap="md">
            <span class="brk-warn">
                "\u{26a0} 6h retention window \u{b7} ship to Datadog for long-term"
            </span>
            <Tabs
                value=tab
                label="Telemetry"
                tabs=vec![TabItem::new("events", "Kube events"), TabItem::new("logs", "Pod logs")]
            >
                <TabPanel value="events">
                    {move || match events.get() {
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
                                    let sc = sev(&e.status);
                                    let msg = e.message.clone().unwrap_or_default();
                                    let subject = e.event_type.clone();
                                    let status = e.status.clone();
                                    let agent: String = e.agent_id.chars().take(8).collect();
                                    let e_sel = e.clone();
                                    view! {
                                        <FeedRow
                                            dot=sc
                                            subject=subject
                                            actor=agent
                                            status=status
                                            status_color=sc
                                            on_click=Callback::new(move |_| {
                                                selected.set(Some(e_sel.clone()));
                                                open.set(true);
                                            })
                                        >
                                            {msg}
                                        </FeedRow>
                                    }
                                })
                                .collect_view();
                            view! {
                                <Panel title="Agent events">
                                    <div class="brk-feed-notime"><FeedList label="Agent events">{rows}</FeedList></div>
                                </Panel>
                            }
                            .into_any()
                        }
                    }}
                </TabPanel>
                <TabPanel value="logs">
                    <Panel title="Pod logs">
                        <Empty message="Select a stack to tail its pod logs (per-stack; no global feed)." />
                    </Panel>
                </TabPanel>
            </Tabs>
        </Stack>

        <Drawer open=open title="Event detail">
            {move || match selected.get() {
                None => ().into_any(),
                Some(e) => {
                    let sc = sev(&e.status);
                    view! {
                        <Stack gap="md">
                            <Group gap="sm">
                                <Pill color=sc>{e.event_type.clone()}</Pill>
                                <span class="brk-meta">{e.status.clone()}</span>
                            </Group>
                            <DetailList mono=true>
                                <KeyValue label="agent">{e.agent_id.clone()}</KeyValue>
                            </DetailList>
                            <span class="brk-text">
                                {e.message.clone().unwrap_or_else(|| "(no message)".into())}
                            </span>
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Drawer>
    }
}
