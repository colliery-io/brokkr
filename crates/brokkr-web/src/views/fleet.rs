//! Fleet view — agents from `GET /api/v1/fleet`, **grouped by cluster** (one panel
//! per `cluster_name`), with a KPI strip and a row per agent (status/health pills,
//! `⇄ ws`, heartbeat "ago"). Clicking a row opens the agent-detail slide-over (a
//! `Drawer`, design/README.md) with
//! the console's writes: **pause/resume** and **run diagnostic**.
//!
//! Diagnostics are deployment-object-scoped in the broker
//! (`POST /deployment-objects/:id/diagnostics`; `deployment_object_id` is NOT
//! NULL), but the fleet rollup carries no deployment-object id — so the drawer
//! fetches the agent's target state (`GET /agents/:id/target-state?mode=full`)
//! and offers a picker. An agent with no deployment objects has nothing to
//! diagnose, and says so instead of showing a dead button (BROKKR-T-0275).
//!
//! The created request's id is kept and polled (`GET /diagnostics/:id`) until the
//! request is terminal or the poll bound is hit, and the outcome — pod statuses,
//! events, log tails — is rendered in the same drawer (BROKKR-T-0301).

use crate::api;
use crate::components::{sev, EmptyNext, LiveDot, Sweep, DOCS, FRESH_BEAT_SECS};
use crate::models::{DiagEvent, DiagnosticData, DiagnosticOutcome, FleetAgentRecord, PodStatus};
use crate::views::ago;
use aurora_leptos::components::*;
use aurora_leptos::data::{CodeBlock, DetailList, KeyValue, SectionLabel, StatTile};
use aurora_leptos::frame::{use_toaster, Drawer};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use std::collections::BTreeMap;
use wasm_bindgen_futures::spawn_local;

/// How often the console re-reads a running diagnostic.
const POLL_EVERY_MS: u64 = 2_000;
/// How many times, at most. The agent picks up pending requests on a 10s timer
/// and then has to talk to the Kubernetes API, so ~90s covers several collection
/// windows.
///
/// The bound is not a nicety. A request the agent claimed and then abandoned
/// stays `claimed` indefinitely (BROKKR-T-0300), and even where the broker does
/// reap it, it does so at `expires_at` — 60 minutes out by default. Polling
/// forever would just spin; the console stops and says so.
const POLL_MAX: u32 = 45;

/// Uppercase section heading, matching the drawer's other headings.
fn heading(text: &'static str) -> AnyView {
    view! { <SectionLabel label=text /> }.into_any()
}

/// A neutral, non-alarming note (used for legitimately empty payloads).
fn note(text: String) -> AnyView {
    view! {
        <span class="brk-note">{text}</span>
    }
    .into_any()
}

/// Parsed pod statuses. An empty list is a legitimate result — a deployment
/// object that applies no workloads has no pods to attribute — so it reads as a
/// note, not as an error.
fn pods_view(pods: Result<Vec<PodStatus>, String>) -> AnyView {
    match pods {
        Err(e) => note(format!("pod statuses unreadable: {e}")),
        Ok(v) if v.is_empty() => note(
            "No pods attributed to this deployment object \u{2014} expected when it applies \
             only non-workload resources, or its workload has not created pods yet."
                .to_string(),
        ),
        Ok(v) => {
            let rows = v
                .into_iter()
                .map(|p| {
                    let ready = p.containers.iter().filter(|c| c.ready).count();
                    let total = p.containers.len();
                    let restarts: i32 = p.containers.iter().map(|c| c.restart_count).sum();
                    let reason = p
                        .containers
                        .iter()
                        .find_map(|c| c.state_reason.clone())
                        .unwrap_or_default();
                    let color = sev(&p.phase);
                    let where_ = format!("{}/{}", p.namespace, p.name);
                    view! {
                        <Group justify="between">
                            <Group gap="sm">
                                <Pill color=color>{p.phase}</Pill>
                                <span class="brk-value">{where_}</span>
                                {(!reason.is_empty()).then(|| view! {
                                    <span class="brk-warn">{reason}</span>
                                })}
                            </Group>
                            <span class="brk-note">
                                {format!("{ready}/{total} ready \u{00b7} {restarts} restarts")}
                            </span>
                        </Group>
                    }
                })
                .collect_view();
            view! { <Stack gap="sm">{rows}</Stack> }.into_any()
        }
    }
}

/// Parsed events. These are namespace-scoped (every recent event in the searched
/// namespaces, not just this object's), so the list is capped and says so.
fn events_view(events: Result<Vec<DiagEvent>, String>) -> AnyView {
    const SHOW: usize = 25;
    match events {
        Err(e) => note(format!("events unreadable: {e}")),
        Ok(v) if v.is_empty() => note("No events in the searched namespaces.".to_string()),
        Ok(v) => {
            let total = v.len();
            let rows = v
                .into_iter()
                .take(SHOW)
                .map(|e| {
                    let kind = e.event_type.unwrap_or_else(|| "\u{2014}".into());
                    let reason = e.reason.unwrap_or_default();
                    let obj = match (e.involved_object_kind, e.involved_object) {
                        (Some(k), Some(o)) => format!("{k}/{o}"),
                        (None, Some(o)) => o,
                        _ => String::new(),
                    };
                    let msg = e.message.unwrap_or_default();
                    let count = e.count.filter(|c| *c > 1).map(|c| format!(" \u{00d7}{c}"));
                    let color = sev(&kind);
                    view! {
                        <Stack gap="xs">
                            <Group gap="sm">
                                <Pill color=color>{kind}</Pill>
                                <span class="brk-value">{reason}</span>
                                <span class="brk-note">{obj}{count}</span>
                            </Group>
                            <span class="brk-text">{msg}</span>
                        </Stack>
                    }
                })
                .collect_view();
            let more = (total > SHOW).then(|| note(format!("showing {SHOW} of {total} events")));
            view! { <Stack gap="sm">{rows}{more}</Stack> }.into_any()
        }
    }
}

/// Parsed log tails (`pod/container` -> last lines).
fn logs_view(tails: Result<Vec<(String, String)>, String>) -> AnyView {
    match tails {
        Err(e) => note(format!("log tails unreadable: {e}")),
        Ok(v) if v.is_empty() => note("No log tails collected.".to_string()),
        Ok(v) => {
            let blocks = v
                .into_iter()
                .map(|(key, text)| {
                    let label = format!("Log tail {key}");
                    view! {
                        <Stack gap="xs">
                            <span class="brk-logkey">{key}</span>
                            <CodeBlock code=text max_height="160px" wrap=true label=label />
                        </Stack>
                    }
                })
                .collect_view();
            view! { <Stack gap="sm">{blocks}</Stack> }.into_any()
        }
    }
}

/// A completed collection's payload.
fn result_view(data: DiagnosticData) -> AnyView {
    let collected = data.collected_at.map(|t| note(format!("collected at {t}")));
    view! {
        <Stack gap="sm">
            {collected}
            {heading("pod statuses")}
            {pods_view(data.pods)}
            {heading("events")}
            {events_view(data.events)}
            {heading("log tails")}
            {logs_view(data.log_tails)}
        </Stack>
    }
    .into_any()
}

#[component]
pub fn FleetView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): reading the scope signal inside the
    // fetcher makes the resource refetch when the tenant selection changes.
    let scope = crate::app::use_scope();
    let data = LocalResource::new(move || api::fleet(scope.get()));
    crate::components::poll(move || data.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<FleetAgentRecord>);
    let open = RwSignal::new(false);
    let toaster = use_toaster();
    // Deployment objects targeted at the selected agent — refetched when the
    // selection changes (same idiom as the stack-health / deliveries panels).
    let objects = LocalResource::new(move || {
        let id = selected.get().map(|a: FleetAgentRecord| a.agent_id.clone());
        async move {
            match id {
                Some(id) => Some(api::agent_target_state(&id).await),
                None => None,
            }
        }
    });
    // Picker selection, held as the `Select` option label. Empty means "not
    // chosen yet", which resolves to the first object — matching what an
    // untouched `<select>` displays. Reset on every row click so a stale label
    // from a previous agent can never leak across.
    let chosen = RwSignal::new(String::new());

    // Pause/resume (BROKKR-T-0322). Changing an agent's status is a privileged
    // write, so it takes an admin PAK supplied for that one request — the same
    // shape as tenant minting. Memory only, cleared as soon as the request
    // resolves on either path.
    let pause_pak = RwSignal::new(String::new());
    let pause_busy = RwSignal::new(false);
    let pause_error = RwSignal::new(None::<String>);
    // Set once the agent's status has been changed here, so the modal can show
    // the new state without waiting for the 5s fleet refetch to come round.
    let pause_status = RwSignal::new(None::<String>);
    let reset_pause = move || {
        pause_pak.set(String::new());
        pause_error.set(None);
        pause_status.set(None);
    };

    // --- diagnostic result polling (BROKKR-T-0301) ------------------------
    // `diag_id` is the id returned by the create call; `polls` counts attempts
    // (bumping it is what re-runs the fetch, since the id itself doesn't change
    // between polls); `settled` stops the poller once the request is terminal or
    // the read failed.
    let diag_id = RwSignal::new(None::<String>);
    let polls = RwSignal::new(0u32);
    let settled = RwSignal::new(false);
    let diagnostic = LocalResource::new(move || {
        let _attempt = polls.get();
        let id = diag_id.get();
        async move {
            match id {
                Some(id) => {
                    let r = api::diagnostic(&id).await;
                    // A terminal status is as far as this request will ever get;
                    // an error means polling can't make progress either.
                    if r.as_ref().map(|d| d.is_terminal()).unwrap_or(true) {
                        settled.set(true);
                    }
                    Some(r)
                }
                None => None,
            }
        }
    });
    crate::components::poll(
        move || {
            if diag_id.get_untracked().is_none() || settled.get_untracked() {
                return;
            }
            let n = polls.get_untracked();
            if n < POLL_MAX {
                polls.set(n + 1);
            }
        },
        std::time::Duration::from_millis(POLL_EVERY_MS),
    );
    // Drop any previous run's state (used on row click and on a fresh request).
    let reset_diagnostic = move || {
        diag_id.set(None);
        polls.set(0);
        settled.set(false);
    };

    view! {
        {move || match data.get() {
            None => view! { <Loading label="loading fleet" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(agents)) if agents.is_empty() => {
                view! {
                    <EmptyNext
                        message="No agents registered with this broker."
                        next="Create an agent record, start the agent with its PAK and a matching name and cluster, then open it here and activate it."
                        href=format!("{DOCS}/how-to/agent-registration.html")
                        link="How an agent registers"
                    />
                }.into_any()
            }
            Some(Ok(agents)) => {
                let total = agents.len();
                let active = agents.iter().filter(|a| a.status.eq_ignore_ascii_case("active")).count();
                let degraded = agents.iter().filter(|a| a.health().0 == "degraded").count();
                let failing = agents.iter().filter(|a| a.health().0 == "failing").count();

                // Group agents by cluster (empty cluster_name -> "(unknown)").
                let mut by_cluster: BTreeMap<String, Vec<FleetAgentRecord>> = BTreeMap::new();
                for a in agents {
                    let key = if a.cluster_name.is_empty() {
                        "(unknown)".to_string()
                    } else {
                        a.cluster_name.clone()
                    };
                    by_cluster.entry(key).or_default().push(a);
                }

                let panels = by_cluster
                    .into_iter()
                    .map(|(cluster, ags)| {
                        let count = ags.len();
                        let active_here = ags.iter().filter(|a| a.status.eq_ignore_ascii_case("active")).count();
                        let rows = ags.into_iter().map(|a| {
                            let (h, hc) = a.health();
                            // Brokkr agent statuses (ACTIVE/INACTIVE) are not Aurora's
                            // run statuses; the dot shows health (design/README.md).
                            let sc = sev(&a.status);
                            // Pulse while the agent is active and its last beat is fresh.
                            let live = a.status.eq_ignore_ascii_case("active")
                                && a.heartbeat_age_seconds.is_some_and(|s| s < FRESH_BEAT_SECS);
                            let a_sel = a.clone();
                            let a_id = a.agent_id.clone();
                            view! {
                                <TableRow
                                    selected=Signal::derive(move || {
                                        open.get() && selected.with(|x| x.as_ref().is_some_and(|x| x.agent_id == a_id))
                                    })
                                    on_click=Callback::new(move |_| {
                                        chosen.set(String::new());
                                        reset_diagnostic();
                                        reset_pause();
                                        selected.set(Some(a_sel.clone()));
                                        open.set(true);
                                    })
                                >
                                    <td>
                                        <Group gap="sm">
                                            <LiveDot color=hc live=live />
                                            <span>{a.name.clone()}</span>
                                        </Group>
                                    </td>
                                    <td><Pill color=sc>{a.status.to_lowercase()}</Pill></td>
                                    <td><Pill color=hc>{h}</Pill></td>
                                    <td>
                                        {a.ws_connected.then(|| view! {
                                            <span class="brk-ws" title="connected on the WebSocket channel">"\u{21c4} ws"</span>
                                        })}
                                    </td>
                                    <td class="brk-muted cl-num">{ago(a.heartbeat_age_seconds)}</td>
                                </TableRow>
                            }
                        }).collect_view();
                        let caption = format!("{active_here}/{count} active");
                        let label = format!("Agents in {cluster}");
                        view! {
                            <Panel title=cluster caption=caption>
                                <Table mono=true fixed=true label=label
                                    widths=vec!["34%".into(), "16%".into(), "16%".into(), "14%".into(), "20%".into()]>
                                    <thead><tr>
                                        <th>"Agent"</th><th>"Status"</th><th>"Health"</th>
                                        <th>"Channel"</th><th class="cl-num">"Heartbeat"</th>
                                    </tr></thead>
                                    <tbody>{rows}</tbody>
                                </Table>
                            </Panel>
                        }
                    })
                    .collect_view();

                view! {
                    <Stack gap="md">
                        <div class="brk-kpis">
                            <StatTile label="agents" value=total.to_string() />
                            <StatTile label="active" value=active.to_string() color=token::OK />
                            <StatTile label="degraded" value=degraded.to_string() color=token::GOLD />
                            <StatTile label="failing" value=failing.to_string() color=token::BAD />
                        </div>
                        {panels}
                    </Stack>
                }
                .into_any()
            }
        }}

        <Drawer open=open title="Agent detail" size="lg">
            {move || match selected.get() {
                None => ().into_any(),
                Some(a) => {
                    let (h, hc) = a.health();
                    let agent_id = a.agent_id.clone();
                    // Owned copies for the pause/resume closures, which outlive `a`.
                    // Two status clones: the pill and the control each move one.
                    let a_status_pill = a.status.clone();
                    let a_status_ctl = a.status.clone();
                    let a_id = a.agent_id.clone();
                    // A new agent and a paused one both read INACTIVE; the heartbeat
                    // tells them apart (BROKKR-T-0336).
                    let a_checked_in = a.has_checked_in();
                    view! {
                        <Stack gap="md">
                            <span class="brk-detail-title">{a.name.clone()}</span>
                            <DetailList mono=true>
                                <KeyValue label="agent id">{a.agent_id.clone()}</KeyValue>
                                <KeyValue label="cluster">{if a.cluster_name.is_empty() { "(unknown)".to_string() } else { a.cluster_name.clone() }}</KeyValue>
                                <KeyValue label="last heartbeat">{ago(a.heartbeat_age_seconds)}</KeyValue>
                            </DetailList>
                            <Group gap="sm">
                                // Reflects a pause/resume done here immediately; otherwise
                                // the agent's status as of the last fleet refetch.
                                {move || {
                                    let shown = pause_status
                                        .get()
                                        .unwrap_or_else(|| a_status_pill.clone());
                                    let c = sev(&shown);
                                    view! { <Pill color=c>{shown.to_lowercase()}</Pill> }
                                }}
                                <Pill color=hc>{h}</Pill>
                                {a.ws_connected.then(|| view! {
                                    <span class="brk-ws">"\u{21c4} ws"</span>
                                })}
                            </Group>

                            // ---- pause / resume (BROKKR-T-0322) ----------------
                            <SectionLabel label="agent state" />
                            {move || {
                                let current =
                                    pause_status.get().unwrap_or_else(|| a_status_ctl.clone());
                                let active = current.eq_ignore_ascii_case("ACTIVE");
                                let (target, verb) = if active {
                                    ("INACTIVE", "Pause")
                                } else {
                                    ("ACTIVE", "Activate")
                                };
                                let id_for_click = a_id.clone();
                                view! {
                                    <span class="brk-text">
                                        {if active {
                                            "Active: this agent fetches and applies the deployment \
                                             objects targeted at it."
                                        } else if !a_checked_in {
                                            "Not yet activated: this agent has never checked in. Start \
                                             the agent with its PAK and a matching name and cluster, \
                                             then activate it here. A new agent applies nothing until \
                                             an admin activates it."
                                        } else {
                                            "Inactive: this agent checks in, but it fetches no \
                                             deployment objects and no work orders. A new agent starts \
                                             inactive, and a paused agent stays inactive, until an \
                                             admin activates it. Already-applied resources stay in the \
                                             cluster."
                                        }}
                                    </span>
                                    <PasswordInput
                                        label="Admin PAK"
                                        placeholder="brokkr_\u{2026}"
                                        value=pause_pak
                                        autocomplete="off"
                                    />
                                    <Group gap="sm">
                                        <Button
                                            loading=pause_busy
                                            loading_label="Working\u{2026}"
                                            on_click=Callback::new(move |_| {
                                            let pak = pause_pak.get();
                                            if pak.trim().is_empty() {
                                                pause_error.set(Some(
                                                    "An admin PAK is required to change agent state.".into(),
                                                ));
                                                return;
                                            }
                                            let id = id_for_click.clone();
                                            pause_busy.set(true);
                                            pause_error.set(None);
                                            spawn_local(async move {
                                                let result =
                                                    api::set_agent_status(&id, target, &pak).await;
                                                // Clear the credential first, on both paths.
                                                pause_pak.set(String::new());
                                                pause_busy.set(false);
                                                match result {
                                                    Ok(updated) => {
                                                        pause_status.set(Some(updated.status.clone()));
                                                        data.refetch();
                                                        toaster.success(if target == "ACTIVE" {
                                                            "agent activated"
                                                        } else {
                                                            "agent paused"
                                                        });
                                                    }
                                                    Err(e) => {
                                                        pause_error.set(Some(match e {
                                                            aurora_leptos::tokens::ApiError::Http {
                                                                status: 403,
                                                                ..
                                                            } => "Rejected: that PAK is not an admin \
                                                                  credential."
                                                                .to_string(),
                                                            aurora_leptos::tokens::ApiError::Http {
                                                                status,
                                                                ..
                                                            } => format!(
                                                                "Broker rejected the request (HTTP {status})."
                                                            ),
                                                            aurora_leptos::tokens::ApiError::Network => {
                                                                "Could not reach the broker.".to_string()
                                                            }
                                                            _ => "The request failed.".to_string(),
                                                        }));
                                                        toaster.error("agent state change failed");
                                                    }
                                                }
                                            });
                                        })>
                                            {verb}
                                        </Button>
                                    </Group>
                                    {move || pause_error.get().map(|e| view! {
                                        <Alert color=token::BAD>{e}</Alert>
                                    })}
                                }
                            }}
                            <SectionLabel label="diagnostics" />
                            {move || match objects.get() {
                                None | Some(None) => {
                                    view! { <Loading label="loading deployment objects" /> }.into_any()
                                }
                                Some(Some(Err(_))) => view! {
                                    <span class="brk-note">"deployment objects unavailable"</span>
                                }.into_any(),
                                Some(Some(Ok(objs))) if objs.is_empty() => view! {
                                    <span class="brk-note">
                                        "No deployment objects target this agent \u{2014} nothing to diagnose."
                                    </span>
                                }.into_any(),
                                Some(Some(Ok(objs))) => {
                                    let options: Vec<String> = objs.iter().map(|o| o.label()).collect();
                                    // Aurora's Select shows its value: start on the first object.
                                    if chosen.get_untracked().is_empty() {
                                        if let Some(first) = options.first() {
                                            chosen.set(first.clone());
                                        }
                                    }
                                    let agent_id = agent_id.clone();
                                    view! {
                                        <Select label="deployment object" options=options value=chosen />
                                        <Group><Button on_click=Callback::new(move |_| {
                                            // Empty/stale selection falls back to the first
                                            // option, which is what the <select> is showing.
                                            let want = chosen.get();
                                            let Some(obj) = objs
                                                .iter()
                                                .find(|o| o.label() == want)
                                                .or_else(|| objs.first())
                                            else {
                                                return;
                                            };
                                            let do_id = obj.id.clone();
                                            let id = agent_id.clone();
                                            // Drop the previous run's result before starting a new one.
                                            reset_diagnostic();
                                            spawn_local(async move {
                                                match api::create_diagnostic(&do_id, &id).await {
                                                    // Keep the id: it's what makes the result readable.
                                                    Ok(req) => {
                                                        diag_id.set(Some(req.id));
                                                        toaster.success("diagnostic requested \u{2014} collecting\u{2026}");
                                                    }
                                                    Err(_) => {
                                                        toaster.error("diagnostic request failed");
                                                    }
                                                }
                                            });
                                        })>"\u{2315} Run diagnostic"</Button></Group>
                                        <span class="brk-note">
                                            "Asks the agent to collect pod status, events and log tails \
                                             for this deployment object."
                                        </span>
                                    }.into_any()
                                }
                            }}
                            // Outcome of the request started above (BROKKR-T-0301).
                            {move || {
                                if diag_id.get().is_none() {
                                    return ().into_any();
                                }
                                // Bound reached without a terminal status.
                                let exhausted = polls.get() >= POLL_MAX && !settled.get();
                                let recheck = Callback::new(move |_| {
                                    polls.set(0);
                                    settled.set(false);
                                    diagnostic.refetch();
                                });
                                match diagnostic.get() {
                                    None | Some(None) => {
                                        view! { <Loading label="reading diagnostic" /> }.into_any()
                                    }
                                    Some(Some(Err(e))) => view! {
                                        <Stack gap="sm">
                                            {heading("diagnostic result")}
                                            <ErrorState error=e on_retry=recheck />
                                        </Stack>
                                    }.into_any(),
                                    Some(Some(Ok(d))) => {
                                        let status = d.request.status.clone();
                                        let id8: String = d.request.id.chars().take(8).collect();
                                        let stalled_msg = format!(
                                            "Stopped polling after {}s; the request is still {status}. \
                                             A claimed request can stay claimed indefinitely if the \
                                             agent never submits (BROKKR-T-0300), so this is not \
                                             proof that collection failed.",
                                            POLL_MAX as u64 * POLL_EVERY_MS / 1000,
                                        );
                                        let body = match d.outcome() {
                                            // Bounded, and honest about what the bound does and
                                            // does not prove.
                                            DiagnosticOutcome::InFlight if exhausted => view! {
                                                <Alert title="no result yet" color=token::GOLD>
                                                    <Stack gap="xs">
                                                        <span class="brk-text">{stalled_msg}</span>
                                                        <Button on_click=recheck>"\u{21bb} Check again"</Button>
                                                    </Stack>
                                                </Alert>
                                            }.into_any(),
                                            DiagnosticOutcome::InFlight => view! {
                                                <Sweep label="waiting for the agent to collect pod statuses, events, log tails\u{2026}" />
                                            }.into_any(),
                                            // `completed` + an `error` entry in `events` is a
                                            // FAILED collection, not an empty one.
                                            DiagnosticOutcome::CollectionFailed(errs) => {
                                                let lines = errs.into_iter().map(|e| view! {
                                                    <span class="brk-value">{e}</span>
                                                }).collect_view();
                                                view! {
                                                    <Alert title="collection failed" color=token::BAD>
                                                        <Stack gap="xs">
                                                            <span class="brk-text">
                                                                "The agent could not read the cluster. The broker still \
                                                                 reports the request as completed \u{2014} the error is \
                                                                 carried inside the result."
                                                            </span>
                                                            {lines}
                                                        </Stack>
                                                    </Alert>
                                                }.into_any()
                                            }
                                            DiagnosticOutcome::Collected(data) => result_view(*data),
                                            DiagnosticOutcome::NoResult(s) => {
                                                let (title, color, text) = match s.to_ascii_lowercase().as_str() {
                                                    "expired" => (
                                                        "request expired",
                                                        token::BAD,
                                                        "No agent claimed the request before its retention window \
                                                         closed \u{2014} the agent is likely offline.".to_string(),
                                                    ),
                                                    "failed" => (
                                                        "collection failed",
                                                        token::BAD,
                                                        "The request was claimed but no result was ever submitted."
                                                            .to_string(),
                                                    ),
                                                    _ => (
                                                        "no result stored",
                                                        token::GOLD,
                                                        format!("The request is {s}, but the broker holds no result \
                                                                 payload for it."),
                                                    ),
                                                };
                                                view! {
                                                    <Alert title=title color=color>
                                                        <span class="brk-text">{text}</span>
                                                    </Alert>
                                                }.into_any()
                                            }
                                        };
                                        view! {
                                            <Stack gap="sm">
                                                {heading("diagnostic result")}
                                                <Group gap="sm">
                                                    <Pill color=sev(&status)>{status.clone()}</Pill>
                                                    <span class="brk-note">{format!("request {id8}")}</span>
                                                </Group>
                                                {body}
                                            </Stack>
                                        }.into_any()
                                    }
                                }
                            }}
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Drawer>
    }
}
