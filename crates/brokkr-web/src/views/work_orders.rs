//! Work orders view — **Active** (live, from `GET /api/v1/work-orders`) over the
//! completed **history** (`GET /api/v1/work-order-log`). Each row names its
//! owner: the generator that created the work order, or "admin". The two
//! routes take no generator filter, so the view narrows to the generator scope
//! in the client (BROKKR-I-0032).
//!
//! A click on a row opens the work order: its detail, and the pod logs and the
//! Kubernetes events of its job (`/work-orders/:id/{logs,events}`), with the
//! rows of the stack telemetry view. The broker has no live stream for a work
//! order, so the shown tab reads again every 5 s while the work order runs,
//! and once for an ended one.

use crate::api;
use crate::components::{in_scope, sev, work_order_owner, PanelError, DOCS};
use crate::live::LogBuffer;
use crate::models::{WorkOrder, WorkOrderLogEntry};
use crate::views::telemetry::{log_lines, KubeEventFeed, Retention};
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, LogView};
use aurora_leptos::frame::{Drawer, TabItem, TabPanel, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;

/// The work order the drawer shows: one that runs, or one that ended.
#[derive(Debug, Clone, PartialEq)]
enum Picked {
    Active(WorkOrder),
    Ended(WorkOrderLogEntry),
}

impl Picked {
    fn id(&self) -> &str {
        match self {
            Picked::Active(w) => &w.id,
            Picked::Ended(w) => &w.id,
        }
    }

    fn owner(&self) -> Option<&str> {
        match self {
            Picked::Active(w) => w.owner_generator_id.as_deref(),
            Picked::Ended(w) => w.owner_generator_id.as_deref(),
        }
    }

    fn ended(&self) -> bool {
        matches!(self, Picked::Ended(_))
    }

    /// When it ended, for an ended one that says so.
    fn ended_at(&self) -> Option<&str> {
        match self {
            Picked::Active(_) => None,
            Picked::Ended(w) => w.completed_at.as_deref(),
        }
    }
}

/// Why a tab of a work order has no row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quiet {
    /// The work order runs: rows can still come.
    Waiting,
    /// The work order ended inside the retention window with no row.
    Nothing,
    /// The work order ended longer ago than the broker keeps rows.
    Gone,
}

/// Which quiet state applies. The broker answers 200 with no row both when
/// there was none and when the rows are past the retention window, so `Gone`
/// is an inference: the work order ended more than `retention_secs` ago.
/// An ended work order with no end time, or a broker that states no window,
/// reads as `Nothing`.
pub fn quiet(ended: bool, ended_secs_ago: Option<f64>, retention_secs: u64) -> Quiet {
    if !ended {
        return Quiet::Waiting;
    }
    match ended_secs_ago {
        Some(age) if retention_secs > 0 && age > retention_secs as f64 => Quiet::Gone,
        _ => Quiet::Nothing,
    }
}

/// The stream a tab of a work order shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Logs,
    Events,
}

/// The message and the hint of a tab with no row. `window` is the retention
/// the broker stated ("6 h").
pub fn quiet_text(stream: Stream, state: Quiet, window: &str) -> (String, String) {
    let (rows, source, none) = match stream {
        Stream::Logs => (
            "log lines",
            "the log lines of the job's pods",
            "The pods of the job wrote no line, or the agent sent none.",
        ),
        Stream::Events => (
            "Kubernetes events",
            "the events of the job's objects",
            "The objects of the job had no event, or the agent sent none.",
        ),
    };
    match state {
        Quiet::Waiting => (
            format!("No {rows} yet."),
            format!(
                "The agent sends {source} while the work order runs. The console reads them again every 5 seconds."
            ),
        ),
        Quiet::Nothing => (format!("No {rows} for this work order."), none.to_string()),
        Quiet::Gone => (
            format!("The {rows} of this work order are gone."),
            format!("The broker keeps up to {window} of this stream. This work order ended before that."),
        ),
    }
}

/// Seconds since `iso`, by the browser's clock. None for no time, or one the
/// browser cannot read.
fn secs_since(iso: Option<&str>) -> Option<f64> {
    let ms = js_sys::Date::parse(iso?);
    ms.is_finite().then(|| (js_sys::Date::now() - ms) / 1000.0)
}

/// The claim of an active work order, as the Active table and the drawer say it.
fn claim_note(w: &WorkOrder) -> String {
    let claimed = w
        .claimed_by
        .as_ref()
        .map(|c| format!("claimed by {}", c.chars().take(8).collect::<String>()))
        .unwrap_or_else(|| "unclaimed".into());
    if w.retry_count > 0 {
        format!("{claimed} · retry {}", w.retry_count)
    } else {
        claimed
    }
}

fn outcome(success: bool) -> (&'static str, &'static str) {
    if success {
        ("completed", token::OK)
    } else {
        ("failed", token::BAD)
    }
}

#[component]
pub fn WorkOrdersView() -> impl IntoView {
    let scope = crate::app::use_scope();
    let active = LocalResource::new(api::work_orders);
    let data = LocalResource::new(api::work_order_log);
    // The generator list, for the owner's name.
    let generators = LocalResource::new(api::generators);
    let owner_name = move |owner: Option<&str>| -> String {
        let gens = generators.get().and_then(|r| r.ok()).unwrap_or_default();
        work_order_owner(&gens, owner)
    };
    crate::components::poll(
        move || {
            active.refetch();
            data.refetch();
        },
        std::time::Duration::from_secs(5),
    );
    let picked = RwSignal::new(None::<Picked>);
    let open = RwSignal::new(false);
    let tab = RwSignal::new(String::from("logs"));
    let pick = move |p: Picked| {
        picked.set(Some(p));
        open.set(true);
    };
    // The id the drawer shows, while it is open.
    let shown_id = Memo::new(move |_| {
        open.get()
            .then(|| picked.with(|p| p.as_ref().map(|p| p.id().to_string())))
            .flatten()
    });
    // Each tab reads the broker only while it is shown.
    let wo_logs = LocalResource::new(move || {
        let id = shown_id.get().filter(|_| tab.get() == "logs");
        async move {
            match id {
                Some(id) => Some(api::work_order_logs(&id).await),
                None => None,
            }
        }
    });
    let wo_events = LocalResource::new(move || {
        let id = shown_id.get().filter(|_| tab.get() == "events");
        async move {
            match id {
                Some(id) => Some(api::work_order_events(&id).await),
                None => None,
            }
        }
    });
    // No live stream for a work order: while it runs, the shown tab reads
    // again with the lists. An ended one was read when it opened.
    crate::components::poll(
        move || {
            let runs = open.get_untracked()
                && picked.with_untracked(|p| p.as_ref().is_some_and(|p| !p.ended()));
            if runs {
                if tab.get_untracked() == "logs" {
                    wo_logs.refetch();
                } else {
                    wo_events.refetch();
                }
            }
        },
        std::time::Duration::from_secs(5),
    );
    // The drawer follows its work order: a new status while it runs, and its
    // history entry when it ends.
    Effect::new(move |_| {
        let Some(Ok(list)) = active.get() else { return };
        let Some(Picked::Active(old)) = picked.get_untracked() else {
            return;
        };
        if let Some(now) = list.into_iter().find(|w| w.id == old.id) {
            if now != old {
                picked.set(Some(Picked::Active(now)));
            }
        }
    });
    Effect::new(move |_| {
        let Some(Ok(log)) = data.get() else { return };
        let Some(Picked::Active(old)) = picked.get_untracked() else {
            return;
        };
        if let Some(entry) = log.into_iter().find(|w| w.id == old.id) {
            picked.set(Some(Picked::Ended(entry)));
        }
    });
    // A work order outside the selected scope is not shown, so its drawer closes.
    Effect::new(move |_| {
        let sc = scope.get();
        let outside = picked.with_untracked(|p| {
            p.as_ref()
                .is_some_and(|p| !in_scope(p.owner(), sc.as_deref()))
        });
        if outside {
            open.set(false);
        }
    });
    // The rows of the Pod logs tab, oldest first.
    let wo_lines = Signal::derive(move || match wo_logs.get() {
        Some(Some(Ok(h))) => {
            let mut buffer = LogBuffer::default();
            buffer.merge_history(&h.lines);
            log_lines(&buffer)
        }
        _ => Vec::new(),
    });
    let has_lines =
        Memo::new(move |_| matches!(wo_logs.get(), Some(Some(Ok(h))) if !h.lines.is_empty()));
    // The text of a tab with no row, from the work order and the window the
    // broker stated on the answer.
    let quiet_of = move |stream: Stream, info: &crate::models::RetentionInfo| {
        let (ended, at) = picked.with(|p| match p {
            Some(p) => (p.ended(), secs_since(p.ended_at())),
            None => (false, None),
        });
        let state = quiet(ended, at, info.effective_retention_seconds);
        quiet_text(stream, state, &info.window())
    };

    view! {
        <Stack gap="md">
            // Active (live) work orders
            {move || match active.get() {
                None => view! { <Loading label="loading active" /> }.into_any(),
                Some(Err(e)) => view! {
                    <PanelError error=e on_retry=Callback::new(move |_| { active.refetch(); }) />
                }.into_any(),
                Some(Ok(wos)) => {
                    let sc = scope.get();
                    let act: Vec<_> = wos
                        .into_iter()
                        .filter(|w| w.is_active())
                        .filter(|w| in_scope(w.owner_generator_id.as_deref(), sc.as_deref()))
                        .collect();
                    if act.is_empty() {
                        view! {
                            <Panel title="Active">
                                <Empty
                                    message="No active work orders."
                                    hint="A work order is a job that runs one time. An admin or a generator creates it."
                                    href=format!("{DOCS}/reference/work-orders.html")
                                    link="What a work order is"
                                />
                            </Panel>
                        }.into_any()
                    } else {
                        let rows = act.into_iter().map(|w| {
                            let id8: String = w.id.chars().take(8).collect();
                            let note = claim_note(&w);
                            let owner = owner_name(w.owner_generator_id.as_deref());
                            let w_sel = w.clone();
                            view! {
                                <TableRow on_click=Callback::new(move |_| pick(Picked::Active(w_sel.clone())))>
                                    <td class="brk-muted">{id8}</td>
                                    <td>{owner}</td>
                                    <td><Pill color=token::TEAL>{w.work_type}</Pill></td>
                                    <td><Pill color=sev(&w.status)>{w.status}</Pill></td>
                                    <td class="brk-faint">{note}</td>
                                </TableRow>
                            }
                        }).collect_view();
                        view! {
                            <Panel title="Active">
                                <Table mono=true label="Active work orders">
                                    <thead><tr>
                                        <th>"ID"</th><th>"Owner"</th><th>"Type"</th><th>"Status"</th><th>"Claim"</th>
                                    </tr></thead>
                                    <tbody>{rows}</tbody>
                                </Table>
                            </Panel>
                        }.into_any()
                    }
                }
            }}

            // Completed history
            {move || match data.get() {
                None => view! { <Loading label="loading history" /> }.into_any(),
                Some(Err(e)) => view! {
                    <PanelError error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
                }
                .into_any(),
                Some(Ok(log)) => {
                    let sc = scope.get();
                    let log: Vec<_> = log
                        .into_iter()
                        .filter(|w| in_scope(w.owner_generator_id.as_deref(), sc.as_deref()))
                        .collect();
                    if log.is_empty() {
                        return view! { <Panel title="History"><Empty message="No completed work orders yet." /></Panel> }.into_any();
                    }
                    let rows = log
                        .into_iter()
                        .map(|w| {
                            let (label, color) = outcome(w.success);
                            let id8: String = w.id.chars().take(8).collect();
                            let detail = w.result_message.clone().unwrap_or_default();
                            let owner = owner_name(w.owner_generator_id.as_deref());
                            let w_sel = w.clone();
                            view! {
                                <TableRow on_click=Callback::new(move |_| pick(Picked::Ended(w_sel.clone())))>
                                    <td class="brk-muted">{id8}</td>
                                    <td class="brk-ellipsis">{owner}</td>
                                    <td><Pill color=token::TEAL>{w.work_type.clone()}</Pill></td>
                                    <td><Pill color=color>{label}</Pill></td>
                                    <td class="brk-faint brk-ellipsis">{detail}</td>
                                </TableRow>
                            }
                        })
                        .collect_view();
                    view! {
                        <Panel title="History">
                            <Table mono=true fixed=true label="Work order history"
                                widths=vec!["12%".into(), "16%".into(), "16%".into(), "14%".into(), "42%".into()]>
                                <thead><tr>
                                    <th>"ID"</th><th>"Owner"</th><th>"Type"</th><th>"Outcome"</th><th>"Result"</th>
                                </tr></thead>
                                <tbody>{rows}</tbody>
                            </Table>
                        </Panel>
                    }
                    .into_any()
                }
            }}
        </Stack>

        <Drawer open=open title="Work order">
            <Stack gap="md">
                {move || match picked.get() {
                    None => ().into_any(),
                    Some(Picked::Active(w)) => {
                        let status_color = sev(&w.status);
                        let owner = owner_name(w.owner_generator_id.as_deref());
                        let claim = claim_note(&w);
                        let last_error = w.last_error.clone().unwrap_or_else(|| "—".into());
                        view! {
                            <Stack gap="md">
                                <Group gap="sm">
                                    <Pill color=token::TEAL>{w.work_type.clone()}</Pill>
                                    <Pill color=status_color>{w.status.clone()}</Pill>
                                </Group>
                                <DetailList mono=true>
                                    <KeyValue label="id">{w.id.clone()}</KeyValue>
                                    <KeyValue label="owner">{owner}</KeyValue>
                                    <KeyValue label="claim">{claim}</KeyValue>
                                    <KeyValue label="last error">{last_error}</KeyValue>
                                </DetailList>
                            </Stack>
                        }
                        .into_any()
                    }
                    Some(Picked::Ended(w)) => {
                        let (label, color) = outcome(w.success);
                        view! {
                            <Stack gap="md">
                                <Group gap="sm">
                                    <Pill color=token::TEAL>{w.work_type.clone()}</Pill>
                                    <Pill color=color>{label}</Pill>
                                </Group>
                                <DetailList mono=true>
                                    <KeyValue label="id">{w.id.clone()}</KeyValue>
                                    <KeyValue label="owner">{owner_name(w.owner_generator_id.as_deref())}</KeyValue>
                                    <KeyValue label="retries">{w.retries_attempted.to_string()}</KeyValue>
                                    <KeyValue label="result">{w.result_message.clone().unwrap_or_else(|| "—".into())}</KeyValue>
                                </DetailList>
                            </Stack>
                        }
                        .into_any()
                    }
                }}
                <Tabs
                    value=tab
                    label="Telemetry of the work order"
                    tabs=vec![TabItem::new("logs", "Pod logs"), TabItem::new("events", "Kube events")]
                >
                    <TabPanel value="logs">
                        {move || match wo_logs.get() {
                            None => view! { <Loading label="loading logs" /> }.into_any(),
                            Some(None) => ().into_any(),
                            Some(Some(Err(e))) => view! {
                                <PanelError error=e on_retry=Callback::new(move |_| { wo_logs.refetch(); }) />
                            }
                            .into_any(),
                            Some(Some(Ok(h))) if h.lines.is_empty() => {
                                let (message, hint) = quiet_of(Stream::Logs, &h.retention);
                                view! { <Empty message=message hint=hint /> }.into_any()
                            }
                            // The log view below shows the lines.
                            Some(Some(Ok(_))) => ().into_any(),
                        }}
                        // Outside the match: a new read must not make a new
                        // LogView, which would lose the scroll position.
                        <Show when=move || has_lines.get()>
                            <Stack gap="sm">
                                <LogView
                                    lines=wo_lines
                                    max_height="360px"
                                    empty="No log lines for this work order."
                                    label="Pod logs of the work order"
                                />
                                {move || {
                                    wo_logs.get().flatten().and_then(|r| r.ok()).map(|h| {
                                        view! { <Retention info=h.retention /> }
                                    })
                                }}
                            </Stack>
                        </Show>
                    </TabPanel>
                    <TabPanel value="events">
                        {move || match wo_events.get() {
                            None => view! { <Loading label="loading events" /> }.into_any(),
                            Some(None) => ().into_any(),
                            Some(Some(Err(e))) => view! {
                                <PanelError error=e on_retry=Callback::new(move |_| { wo_events.refetch(); }) />
                            }
                            .into_any(),
                            Some(Some(Ok(h))) if h.events.is_empty() => {
                                let (message, hint) = quiet_of(Stream::Events, &h.retention);
                                view! { <Empty message=message hint=hint /> }.into_any()
                            }
                            Some(Some(Ok(h))) => view! {
                                <Stack gap="sm">
                                    <KubeEventFeed events=h.events.clone() />
                                    <Retention info=h.retention.clone() />
                                </Stack>
                            }
                            .into_any(),
                        }}
                    </TabPanel>
                </Tabs>
            </Stack>
        </Drawer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIX_HOURS: u64 = 6 * 3600;

    #[test]
    fn a_tab_with_no_row_says_why() {
        // The work order runs: rows can still come.
        assert_eq!(quiet(false, None, SIX_HOURS), Quiet::Waiting);
        // It ended a minute ago with no row: there was none.
        assert_eq!(quiet(true, Some(60.0), SIX_HOURS), Quiet::Nothing);
        // It ended at the edge of the window: still in it.
        assert_eq!(
            quiet(true, Some(SIX_HOURS as f64), SIX_HOURS),
            Quiet::Nothing
        );
        // It ended seven hours ago: the broker does not keep rows that long.
        assert_eq!(quiet(true, Some(7.0 * 3600.0), SIX_HOURS), Quiet::Gone);
        // No end time, or no stated window: do not claim the rows are gone.
        assert_eq!(quiet(true, None, SIX_HOURS), Quiet::Nothing);
        assert_eq!(quiet(true, Some(7.0 * 3600.0), 0), Quiet::Nothing);
    }

    #[test]
    fn each_quiet_state_has_its_own_text() {
        let mut seen = std::collections::HashSet::new();
        for stream in [Stream::Logs, Stream::Events] {
            for state in [Quiet::Waiting, Quiet::Nothing, Quiet::Gone] {
                let (message, hint) = quiet_text(stream, state, "6 h");
                assert!(
                    !message.is_empty() && !hint.is_empty(),
                    "{stream:?} {state:?}"
                );
                assert!(
                    message.ends_with('.') && hint.ends_with('.'),
                    "{message} / {hint}"
                );
                assert!(
                    seen.insert(message.clone()),
                    "a second state says: {message}"
                );
            }
        }
        let (message, hint) = quiet_text(Stream::Logs, Quiet::Waiting, "6 h");
        assert_eq!(message, "No log lines yet.");
        assert!(hint.contains("every 5 seconds"), "{hint}");
        let (message, hint) = quiet_text(Stream::Logs, Quiet::Gone, "6 h");
        assert_eq!(message, "The log lines of this work order are gone.");
        assert!(hint.contains("6 h"), "the hint names the window: {hint}");
        let (message, _) = quiet_text(Stream::Events, Quiet::Gone, "30 min");
        assert_eq!(
            message,
            "The Kubernetes events of this work order are gone."
        );
    }

    #[test]
    fn the_drawer_knows_its_work_order() {
        let running: WorkOrder = serde_json::from_value(serde_json::json!({
            "id": "9a01ffbe", "work_type": "custom", "status": "CLAIMED",
            "claimed_by": "1b9d6bcd-bbfd", "retry_count": 2, "owner_generator_id": "g1",
        }))
        .expect("work order");
        assert_eq!(claim_note(&running), "claimed by 1b9d6bcd · retry 2");
        let p = Picked::Active(running);
        assert_eq!(
            (p.id(), p.owner(), p.ended(), p.ended_at()),
            ("9a01ffbe", Some("g1"), false, None)
        );

        let ended: WorkOrderLogEntry = serde_json::from_value(serde_json::json!({
            "id": "7f3a01ab", "work_type": "custom", "success": true,
            "completed_at": "2026-11-13T10:00:00Z",
        }))
        .expect("log entry");
        let p = Picked::Ended(ended);
        assert_eq!(
            (p.id(), p.owner(), p.ended(), p.ended_at()),
            ("7f3a01ab", None, true, Some("2026-11-13T10:00:00Z"))
        );
    }
}
