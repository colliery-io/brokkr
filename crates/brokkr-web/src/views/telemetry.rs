//! Telemetry view — three tabs. "Agent events" is the global lifecycle stream
//! (`/agent-events`: Apply, Heartbeat, Reconcile). "Kube events" and "Pod
//! logs" are per stack (`/stacks/:id/{events,logs}`): a selector above the
//! tabs picks the stack, and the broker states the retention window on each
//! answer (BROKKR-T-0338). REST poll every 5 s; the Pod logs tab also tails
//! the stack's live stream (`crate::live`, BROKKR-T-0348).

use crate::api;
use crate::components::{agent_href, agent_name, same_agent, sev, LiveDot, PanelError};
use crate::live::{Entry, LiveTail};
use crate::models::{AgentEventDto, RetentionInfo};
use aurora_leptos::components::*;
use aurora_leptos::data::{
    DetailList, FeedList, FeedRow, KeyValue, LogLine, LogView, RelativeTime,
};
use aurora_leptos::frame::{Drawer, TabItem, TabPanel, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;

/// The hue of a Kubernetes event type: `Normal` is fine, `Warning` is not.
fn kube_hue(event_type: &str) -> &'static str {
    if event_type.eq_ignore_ascii_case("normal") {
        token::OK
    } else {
        sev(event_type)
    }
}

/// The retention the broker stated, in the console's voice.
#[component]
fn Retention(info: RetentionInfo) -> impl IntoView {
    let window = info.window();
    let oldest = info.oldest_available_ts.clone();
    view! {
        <span class="brk-meta">
            {format!("The broker keeps up to {window} of this stream. ")}
            {match oldest {
                Some(iso) => view! { "Oldest entry: " <RelativeTime iso=iso /> "." }.into_any(),
                None => ().into_any(),
            }}
        </span>
    }
}

#[component]
pub fn TelemetryView() -> impl IntoView {
    let tab = RwSignal::new(String::from("agent"));
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let events = LocalResource::new(move || api::agent_events(scope.get()));
    let stacks = LocalResource::new(move || api::stacks(scope.get()));
    // Agent names for the rows, and the agent a link asked for (BROKKR-T-0337).
    let fleet = LocalResource::new(move || api::fleet(scope.get()));
    let selection = crate::app::use_selection();
    let only_agent = move || selection.get().filter(|s| s.kind == "agent").map(|s| s.id);
    // The stack the two per-stack tabs read; "" is none.
    let stack = RwSignal::new(String::new());
    // Each per-stack tab reads the broker only while it is shown, so the poll
    // does not fetch logs nobody is looking at.
    let kube = LocalResource::new(move || {
        let id = stack.get();
        let shown = tab.get() == "kube";
        async move {
            if id.is_empty() || !shown {
                None
            } else {
                Some(api::stack_events(&id).await)
            }
        }
    });
    let logs = LocalResource::new(move || {
        let id = stack.get();
        let shown = tab.get() == "logs";
        async move {
            if id.is_empty() || !shown {
                None
            } else {
                Some(api::stack_logs(&id).await)
            }
        }
    });
    // The live tail of the Pod logs tab: open only while the tab shows a
    // stack; a change of tab or stack, or leaving the view, closes it. Each
    // time the stream opens, the history is read once more, so lines written
    // while it was down are not lost; the buffer drops the duplicates.
    let tail = LiveTail::new(Callback::new(move |_| logs.refetch()));
    Effect::new(move |_| {
        let id = stack.get();
        let shown = tab.get() == "logs";
        tail.follow((shown && !id.is_empty()).then_some(id));
    });
    Effect::new(move |_| {
        if let Some(Some(Ok(h))) = logs.get() {
            tail.lines.update(|b| b.merge_history(&h.lines));
        }
    });
    crate::components::poll(
        move || {
            events.refetch();
            kube.refetch();
            // While the stream is live it brings every line, so the 5 s
            // history read stops; it runs again while the stream is down.
            if !tail.state.get_untracked().is_live() {
                logs.refetch();
            }
        },
        std::time::Duration::from_secs(5),
    );
    let selected = RwSignal::new(None::<AgentEventDto>);
    let open = RwSignal::new(false);

    // The pod log lines (history, then live), as Aurora's LogView wants
    // them. A gap is a row with a "gap" pill.
    let log_lines = Signal::derive(move || {
        tail.lines.with(|b| {
            b.entries()
                .map(|e| {
                    let (line, clock) = match e {
                        Entry::Line(l) => (
                            LogLine::new(format!("{}: {}", l.source(), l.line)),
                            l.clock(),
                        ),
                        Entry::Gap(g) => (
                            LogLine::new(g.text()).level_color("gap", token::GOLD),
                            g.clock(),
                        ),
                    };
                    match clock {
                        Some(t) => line.time(t),
                        None => line,
                    }
                })
                .collect::<Vec<_>>()
        })
    });
    let logs_ready = Memo::new(move |_| matches!(logs.get(), Some(Some(Ok(_)))));
    // The state of the stream, next to the Pod logs title.
    let live_state = move || {
        let st = tail.state.get();
        let hue = match st {
            crate::live::LiveState::Live => token::OK,
            crate::live::LiveState::Reconnecting { .. } | crate::live::LiveState::Polling => {
                token::GOLD
            }
            _ => token::MUTED,
        };
        view! {
            <Group gap="sm">
                <LiveDot color=hue live=st.is_live() />
                <span class="brk-live-state" data-live-state=st.label()>{st.label()}</span>
                <span class="brk-meta">{st.hint()}</span>
            </Group>
        }
    };

    view! {
        <Stack gap="md">
            // Stack selector for the two per-stack tabs.
            <div class="brk-topbar">
                {move || match stacks.get() {
                    Some(Ok(list)) if !list.is_empty() => {
                        let pairs: Vec<(String, String)> =
                            list.iter().map(|s| (s.id.clone(), s.name.clone())).collect();
                        view! {
                            <Select
                                label="Stack"
                                option_pairs=pairs
                                value=stack
                                placeholder="Select a stack"
                            />
                        }
                        .into_any()
                    }
                    _ => ().into_any(),
                }}
            </div>
            <Tabs
                value=tab
                label="Telemetry"
                tabs=vec![
                    TabItem::new("agent", "Agent events"),
                    TabItem::new("kube", "Kube events"),
                    TabItem::new("logs", "Pod logs"),
                ]
            >
                <TabPanel value="agent">
                    {move || match events.get() {
                        None => view! { <Loading label="loading events" /> }.into_any(),
                        Some(Err(e)) => view! {
                            <PanelError error=e on_retry=Callback::new(move |_| { events.refetch(); }) />
                        }
                        .into_any(),
                        Some(Ok(evs)) => {
                            let names = fleet.get().and_then(|r| r.ok()).unwrap_or_default();
                            let only = only_agent();
                            let evs: Vec<AgentEventDto> = match &only {
                                Some(id) => evs.into_iter().filter(|e| same_agent(&e.agent_id, id)).collect(),
                                None => evs,
                            };
                            if evs.is_empty() {
                                return match only {
                                    Some(id) => view! {
                                        <Stack gap="sm">
                                            <Empty
                                                message=format!("No events from {} in the retention window.", agent_name(&names, &id))
                                                hint="An agent reports an event each time it applies, reconciles or heartbeats."
                                                href="#telemetry"
                                                link="Show all agents"
                                            />
                                        </Stack>
                                    }
                                    .into_any(),
                                    None => view! {
                                        <Empty
                                            message="No agent events yet."
                                            hint="An agent reports an event each time it applies, reconciles or heartbeats."
                                        />
                                    }
                                    .into_any(),
                                };
                            }
                            let filter_line = only.as_ref().map(|id| {
                                let name = agent_name(&names, id);
                                view! {
                                    <Group gap="sm">
                                        <span class="brk-meta">{format!("Events of {name}.")}</span>
                                        <Anchor href="#telemetry">"Show all agents"</Anchor>
                                    </Group>
                                }
                            });
                            let rows = evs
                                .into_iter()
                                .map(|e| {
                                    let sc = sev(&e.status);
                                    let msg = e.message.clone().unwrap_or_default();
                                    let subject = e.event_type.clone();
                                    let status = e.status.clone();
                                    let agent = agent_name(&names, &e.agent_id);
                                    let at = crate::views::overview::event_at(e.created_at.as_deref());
                                    let e_sel = e.clone();
                                    let on_click = Callback::new(move |_| {
                                        selected.set(Some(e_sel.clone()));
                                        open.set(true);
                                    });
                                    // FeedRow strips the Option of `at`: a row with no time passes none.
                                    match at {
                                        Some(t) => view! {
                                            <FeedRow at=t dot=sc subject=subject actor=agent status=status status_color=sc on_click=on_click>
                                                {msg}
                                            </FeedRow>
                                        }
                                        .into_any(),
                                        None => view! {
                                            <FeedRow dot=sc subject=subject actor=agent status=status status_color=sc on_click=on_click>
                                                {msg}
                                            </FeedRow>
                                        }
                                        .into_any(),
                                    }
                                })
                                .collect_view();
                            view! {
                                <Panel title="Agent events">
                                    <Stack gap="sm">
                                        {filter_line}
                                        <FeedList label="Agent events">{rows}</FeedList>
                                    </Stack>
                                </Panel>
                            }
                            .into_any()
                        }
                    }}
                </TabPanel>
                <TabPanel value="kube">
                    {move || match kube.get() {
                        None => view! { <Loading label="loading events" /> }.into_any(),
                        Some(None) => view! {
                            <Empty
                                message="Select a stack to see its Kubernetes events."
                                hint="The agent that applies the stack reports the events of the objects it manages."
                            />
                        }
                        .into_any(),
                        Some(Some(Err(e))) => view! {
                            <PanelError error=e on_retry=Callback::new(move |_| { kube.refetch(); }) />
                        }
                        .into_any(),
                        Some(Some(Ok(h))) if h.events.is_empty() => view! {
                            <Stack gap="sm">
                                <Empty
                                    message="No Kubernetes events for this stack in the retention window."
                                    hint="Events appear once the agent applies the stack and its objects change state."
                                />
                                <Retention info=h.retention.clone() />
                            </Stack>
                        }
                        .into_any(),
                        Some(Some(Ok(h))) => {
                            let rows = h.events.iter().map(|e| {
                                let hue = kube_hue(&e.event_type);
                                let object = e.object();
                                let reason = e.reason.clone();
                                let kind = e.event_type.clone();
                                let msg = e.message.clone();
                                match crate::views::overview::event_at(e.observed_at.as_deref()) {
                                    Some(t) => view! {
                                        <FeedRow at=t dot=hue subject=reason actor=object status=kind status_color=hue>
                                            {msg}
                                        </FeedRow>
                                    }
                                    .into_any(),
                                    None => view! {
                                        <FeedRow dot=hue subject=reason actor=object status=kind status_color=hue>
                                            {msg}
                                        </FeedRow>
                                    }
                                    .into_any(),
                                }
                            }).collect_view();
                            view! {
                                <Stack gap="sm">
                                    <Panel title="Kubernetes events">
                                        <FeedList label="Kubernetes events">{rows}</FeedList>
                                    </Panel>
                                    <Retention info=h.retention.clone() />
                                </Stack>
                            }
                            .into_any()
                        }
                    }}
                </TabPanel>
                <TabPanel value="logs">
                    {move || match logs.get() {
                        None => view! { <Loading label="loading logs" /> }.into_any(),
                        Some(None) => view! {
                            <Empty
                                message="Select a stack to tail its pod logs."
                                hint="The agent streams the log lines of the stack's pods to the broker, which keeps a short window of them."
                            />
                        }
                        .into_any(),
                        Some(Some(Err(e))) => view! {
                            <PanelError error=e on_retry=Callback::new(move |_| { logs.refetch(); }) />
                        }
                        .into_any(),
                        // The panel below shows the lines.
                        Some(Some(Ok(_))) => ().into_any(),
                    }}
                    // Outside the match: a history read (each 5 s while the
                    // stream is down) must not make a new LogView, which would
                    // lose the scroll position, and a frame callback of the
                    // old one would then read its disposed node.
                    <Show when=move || logs_ready.get()>
                        <Stack gap="sm">
                            <Panel title="Pod logs">
                                {live_state}
                                <LogView
                                    lines=log_lines
                                    max_height="520px"
                                    empty="No log lines for this stack in the retention window."
                                    label="Pod logs"
                                />
                            </Panel>
                            {move || {
                                logs.get().flatten().and_then(|r| r.ok()).map(|h| {
                                    view! { <Retention info=h.retention /> }
                                })
                            }}
                        </Stack>
                    </Show>
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
                                <KeyValue label="agent">
                                    <Anchor href=agent_href(&e.agent_id)>
                                        {agent_name(&fleet.get().and_then(|r| r.ok()).unwrap_or_default(), &e.agent_id)}
                                    </Anchor>
                                </KeyValue>
                                <KeyValue label="time"><RelativeTime iso=e.created_at.clone().unwrap_or_default() /></KeyValue>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{K8sEventDto, PodLogDto};

    #[test]
    fn a_kube_event_names_its_object() {
        let e: K8sEventDto = serde_json::from_value(serde_json::json!({
            "reason": "BackOff", "message": "m", "event_type": "Warning",
            "involved_object": {"kind": "Pod", "name": "api-1", "namespace": "payments"},
        }))
        .expect("event");
        assert_eq!(e.object(), "Pod/api-1");
        assert_eq!(kube_hue(&e.event_type), token::GOLD);
        assert_eq!(kube_hue("Normal"), token::OK);
    }

    #[test]
    fn a_log_line_has_a_clock_and_a_source() {
        let l: PodLogDto = serde_json::from_value(serde_json::json!({
            "ts": "2026-10-05T12:04:53.120Z", "namespace": "payments", "pod": "api-1",
            "container": "api", "line": "ready",
        }))
        .expect("line");
        assert_eq!(l.clock().as_deref(), Some("12:04:53Z"));
        assert_eq!(l.source(), "payments/api-1/api");
        let none: PodLogDto =
            serde_json::from_value(serde_json::json!({"line": "x"})).expect("line");
        assert_eq!(none.clock(), None);
    }

    #[test]
    fn retention_reads_as_hours_or_minutes() {
        let r = RetentionInfo {
            retention_ceiling_seconds: 21600,
            effective_retention_seconds: 21600,
            oldest_available_ts: None,
        };
        assert_eq!(r.window(), "6 h");
        let r = RetentionInfo {
            retention_ceiling_seconds: 21600,
            effective_retention_seconds: 1800,
            oldest_available_ts: None,
        };
        assert_eq!(r.window(), "30 min");
    }
}
