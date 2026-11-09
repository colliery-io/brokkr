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
use crate::components::{
    agent_events_href, same_agent, sev, stack_href, LiveDot, PanelError, DOCS, FRESH_BEAT_SECS,
};
use crate::models::{DiagEvent, DiagnosticData, DiagnosticOutcome, FleetAgentRecord, PodStatus};
use crate::views::ago;
use aurora_leptos::components::*;
use aurora_leptos::data::{CodeBlock, DetailList, KeyValue, SectionLabel, StatTile};
use aurora_leptos::frame::{use_toaster, Drawer, Modal, SecretReveal};
use aurora_leptos::tokens::token;
use aurora_leptos::widgets::Meter;
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
        Err(e) => note(format!("The console cannot read the pod statuses: {e}")),
        Ok(v) if v.is_empty() => note(
            "No pods belong to this deployment object. This is normal when the object has no workloads, or when its workloads have no pods yet."
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
        Err(e) => note(format!("The console cannot read the events: {e}")),
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
        Err(e) => note(format!("The console cannot read the log tails: {e}")),
        Ok(v) if v.is_empty() => note("The agent collected no log tails.".to_string()),
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

/// The agent just created, held only while the reveal panel is open
/// (BROKKR-T-0372). Not persisted anywhere.
#[derive(Clone)]
struct CreatedReveal {
    id: String,
    name: String,
    cluster: String,
    pak: String,
    generator_ids: Vec<String>,
}

/// The origin of this page: the broker that serves the console. The agent
/// may reach the broker by another address; the operator edits the command.
fn page_origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://brokkr-broker:3000".to_string())
}

/// The Helm command that installs an agent with these values. The same
/// shape as `brokkr agent create` prints.
fn helm_install_command(
    broker_url: &str,
    pak: &str,
    name: &str,
    cluster: &str,
    generator_ids: &[String],
) -> String {
    let mut lines = vec![
        "helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \\".to_string(),
        format!("  --set broker.url={} \\", broker_url.trim_end_matches('/')),
        format!("  --set broker.pak={pak} \\"),
        format!("  --set broker.agentName={name} \\"),
        format!("  --set broker.clusterName={cluster}"),
    ];
    if !generator_ids.is_empty() {
        let last = lines.len() - 1;
        lines[last].push_str(" \\");
        lines.push(format!(
            "  --set broker.generatorIds=\"{{{}}}\"",
            generator_ids.join(",")
        ));
    }
    lines.join("\n")
}

#[component]
pub fn FleetView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): reading the scope signal inside the
    // fetcher makes the resource refetch when the generator selection changes.
    let scope = crate::app::use_scope();
    let data = LocalResource::new(move || api::fleet(scope.get()));
    crate::components::poll(move || data.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<FleetAgentRecord>);
    let open = RwSignal::new(false);
    let toaster = use_toaster();
    // Stack names for the drawer's links (BROKKR-T-0337).
    let stacks = LocalResource::new(move || api::stacks(scope.get()));
    // The generators the selected agent is registered with (BROKKR-T-0368), and
    // the generator list for their names. The system generator is left out: every
    // agent is registered with it, so it says nothing about this one.
    let generators = LocalResource::new(api::generators);
    let registrations = LocalResource::new(move || {
        let id = selected.get().map(|a: FleetAgentRecord| a.agent_id.clone());
        async move {
            match id {
                Some(id) => Some(api::agent_registrations(&id).await),
                None => None,
            }
        }
    });
    // A link from another view (`#fleet/agent/<id>`) opens that agent's drawer
    // once; a drawer the operator closed stays closed through the next poll.
    let selection = crate::app::use_selection();
    let applied = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        let Some(sel) = selection.get() else {
            return;
        };
        if sel.kind != "agent" || applied.get_untracked().as_deref() == Some(sel.id.as_str()) {
            return;
        }
        if let Some(Ok(agents)) = data.get() {
            if let Some(a) = agents.iter().find(|a| same_agent(&a.agent_id, &sel.id)) {
                applied.set(Some(sel.id.clone()));
                selected.set(Some(a.clone()));
                open.set(true);
            }
        }
    });
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
    // shape as generator minting. Memory only, cleared as soon as the request
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
    // ---- New agent (BROKKR-T-0372) ---------------------------------------
    // The shape of New generator: name, cluster, the generators to register with,
    // an admin PAK held while the dialog is open. The reveal panel shows the
    // PAK once and the Helm command with the values filled in.
    let new_open = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_cluster = RwSignal::new(String::new());
    let new_pak = RwSignal::new(String::new());
    let new_busy = RwSignal::new(false);
    let new_error = RwSignal::new(None::<String>);
    // The generators to pick from: `(id, name, picked)`, set when the dialog opens.
    let new_generators = RwSignal::new(Vec::<(String, String, RwSignal<bool>)>::new());
    let created = RwSignal::new(None::<CreatedReveal>);
    let new_locked = Signal::derive(move || new_busy.get() || created.get().is_some());
    let reset_new = move || {
        new_name.set(String::new());
        new_cluster.set(String::new());
        new_pak.set(String::new());
        new_error.set(None);
        new_generators.set(Vec::new());
        created.set(None);
    };
    let submit_new = move || {
        let name = new_name.get().trim().to_string();
        let cluster = new_cluster.get().trim().to_string();
        if name.is_empty() || cluster.is_empty() {
            new_error.set(Some("Enter a name and a cluster.".into()));
            return;
        }
        let pak = new_pak.get();
        if pak.trim().is_empty() {
            new_error.set(Some("Enter an admin PAK to create an agent.".into()));
            return;
        }
        let ids: Vec<String> = new_generators
            .get()
            .into_iter()
            .filter(|(_, _, on)| on.get())
            .map(|(id, _, _)| id)
            .collect();
        new_busy.set(true);
        new_error.set(None);
        spawn_local(async move {
            let result = api::create_agent(&name, &cluster, &ids, &pak).await;
            // One request, then cleared, on both paths.
            new_pak.set(String::new());
            new_busy.set(false);
            match result {
                Ok(resp) => {
                    created.set(Some(CreatedReveal {
                        id: resp.agent.id,
                        name: resp.agent.name,
                        cluster: resp.agent.cluster_name,
                        pak: resp.initial_pak,
                        generator_ids: ids,
                    }));
                    data.refetch();
                }
                Err(e) => {
                    new_error.set(Some(crate::components::write_error(e)));
                    toaster.error("The agent was not created.");
                }
            }
        });
    };

    // Drop any previous run's state (used on row click and on a fresh request).
    let reset_diagnostic = move || {
        diag_id.set(None);
        polls.set(0);
        settled.set(false);
    };

    view! {
        <Group justify="end">
            <Button on_click=Callback::new(move |_| {
                reset_new();
                let generators = generators
                    .get()
                    .and_then(|r| r.ok())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|g| !g.is_system)
                    .map(|g| (g.id, g.name, RwSignal::new(false)))
                    .collect();
                new_generators.set(generators);
                new_open.set(true);
            })>"+ New agent"</Button>
        </Group>
        {move || match data.get() {
            None => view! { <Loading label="loading fleet" /> }.into_any(),
            Some(Err(e)) => view! {
                <PanelError error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(agents)) if agents.is_empty() => {
                view! {
                    <Empty
                        message="No agents are registered with this broker."
                        hint="Create an agent record. Start the agent with its PAK, the same name and the same cluster. Then open the agent here and activate it."
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

        // Locked while the request runs and while the PAK is on screen: only
        // Done closes it then (the rule of BROKKR-T-0367).
        <Modal open=new_open title="New agent" close_on_scrim=false locked=new_locked
            on_close=Callback::new(move |_| reset_new())>
            {move || match created.get() {
                Some(c) => {
                    let id = c.id.clone();
                    let id_text = c.id.clone();
                    let helm = helm_install_command(
                        &page_origin(),
                        &c.pak,
                        &c.name,
                        &c.cluster,
                        &c.generator_ids,
                    );
                    view! {
                        <Stack gap="md">
                            <DetailList mono=true>
                                <KeyValue label="Agent">{c.name.clone()}</KeyValue>
                                <KeyValue label="Cluster">{c.cluster.clone()}</KeyValue>
                                <KeyValue label="Agent id">
                                    <Group gap="sm">
                                        <span>{id_text}</span>
                                        <CopyButton value=id.clone() icon=true label="Copy agent id" />
                                    </Group>
                                </KeyValue>
                            </DetailList>
                            <CodeBlock code=helm label="Helm command" wrap=true max_height="240px" />
                            <SecretReveal
                                secret=c.pak.clone()
                                label="PAK"
                                done_label="Done"
                                on_done=Callback::new(move |_| {
                                    reset_new();
                                    new_open.set(false);
                                })
                            />
                            <span class="brk-note">
                                "After the agent checks in, open it here and choose Activate."
                            </span>
                        </Stack>
                    }
                    .into_any()
                }
                None => view! {
                    <Stack gap="md">
                        <TextInput label="Name" placeholder="prod-1" value=new_name />
                        <TextInput label="Cluster" placeholder="us-east-1" value=new_cluster />
                        {move || {
                            let generators = new_generators.get();
                            if generators.is_empty() {
                                return ().into_any();
                            }
                            let rows = generators.into_iter().map(|(_, name, on)| view! {
                                <Switch checked=on label=name />
                            }).collect_view();
                            view! {
                                <SectionLabel label="generators" />
                                <Stack gap="xs">{rows}</Stack>
                            }.into_any()
                        }}
                        <Stack gap="xs">
                            <PasswordInput
                                label="Admin PAK"
                                placeholder="brokkr_\u{2026}"
                                value=new_pak
                                autocomplete="off"
                            />
                            <span class="brk-note">"Not stored."</span>
                        </Stack>
                        {move || new_error.get().map(|e| view! {
                            <Alert color=token::BAD>{e}</Alert>
                        })}
                        <Group justify="end" gap="sm">
                            <Button variant="default" on_click=Callback::new(move |_| {
                                reset_new();
                                new_open.set(false);
                            })>"Cancel"</Button>
                            <Button loading=new_busy loading_label="Creating\u{2026}"
                                on_click=Callback::new(move |_| submit_new())>
                                "Create agent"
                            </Button>
                        </Group>
                    </Stack>
                }
                .into_any(),
            }}
        </Modal>

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
                    let a_id_links = a.agent_id.clone();
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
                                            "Active: the agent fetches and applies the deployment objects that target it."
                                        } else if !a_checked_in {
                                            "Not activated: the agent has not checked in yet. Start it with its PAK, then choose Activate here."
                                        } else {
                                            "Inactive: the agent sends heartbeats, but it fetches no deployment objects and no work orders. A new agent starts inactive. A paused agent stays inactive until an admin activates it. Resources that the agent applied stay in the cluster."
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
                                                    "Enter an admin PAK to change the agent state.".into(),
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
                                                            "Agent activated"
                                                        } else {
                                                            "Agent paused"
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
                                                        toaster.error("The agent state did not change.");
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
                            // ---- links to the stacks and the events (BROKKR-T-0337) ----
                            <SectionLabel label="stacks" />
                            {move || match objects.get() {
                                None | Some(None) => view! { <Loading label="loading stacks" /> }.into_any(),
                                Some(Some(Err(_))) => view! { <span class="brk-note">"stacks unavailable"</span> }.into_any(),
                                Some(Some(Ok(objs))) => {
                                    let mut ids: Vec<String> = objs.iter().map(|o| o.stack_id.clone()).collect();
                                    ids.sort();
                                    ids.dedup();
                                    if ids.is_empty() {
                                        return view! { <span class="brk-note">"No stack targets this agent yet."</span> }.into_any();
                                    }
                                    let names = stacks.get().and_then(|r| r.ok()).unwrap_or_default();
                                    let items = ids.into_iter().map(|id| {
                                        let name = names.iter().find(|s| s.id == id).map(|s| s.name.clone())
                                            .unwrap_or_else(|| id.chars().take(8).collect());
                                        let n = objs.iter().filter(|o| o.stack_id == id).count();
                                        view! {
                                            <Group gap="sm">
                                                <Anchor href=stack_href(&id)>{name}</Anchor>
                                                <span class="brk-meta">{format!("{n} deployment object{}", if n == 1 { "" } else { "s" })}</span>
                                            </Group>
                                        }
                                    }).collect_view();
                                    view! { <Stack gap="xs">{items}</Stack> }.into_any()
                                }
                            }}
                            <Anchor href=agent_events_href(&a_id_links)>"Events of this agent"</Anchor>
                            // ---- the generators this agent serves (BROKKR-T-0368) ----
                            <SectionLabel label="generators" />
                            {move || match registrations.get() {
                                None | Some(None) => view! { <Loading label="loading generators" /> }.into_any(),
                                Some(Some(Err(_))) => view! { <span class="brk-note">"generators unavailable"</span> }.into_any(),
                                Some(Some(Ok(regs))) => {
                                    let gens = generators.get().and_then(|r| r.ok()).unwrap_or_default();
                                    let names: Vec<String> = regs
                                        .iter()
                                        .filter(|r| !gens.iter().any(|g| g.id == r.generator_id && g.is_system))
                                        .map(|r| crate::components::generator_name(&gens, &r.generator_id))
                                        .collect();
                                    if names.is_empty() {
                                        return view! {
                                            <span class="brk-note">
                                                "No generator has registered this agent. Only fleet-wide stacks reach it. To register it, open the generator under Generators."
                                            </span>
                                        }.into_any();
                                    }
                                    let items = names.into_iter().map(|n| view! { <span>{n}</span> }).collect_view();
                                    view! { <Stack gap="xs">{items}</Stack> }.into_any()
                                }
                            }}
                            <SectionLabel label="diagnostics" />
                            {move || match objects.get() {
                                None | Some(None) => {
                                    view! { <Loading label="loading deployment objects" /> }.into_any()
                                }
                                Some(Some(Err(_))) => view! {
                                    <span class="brk-note">"The deployment objects are not available."</span>
                                }.into_any(),
                                Some(Some(Ok(objs))) if objs.is_empty() => view! {
                                    <span class="brk-note">
                                        "No deployment objects target this agent. There is nothing to diagnose."
                                    </span>
                                }.into_any(),
                                Some(Some(Ok(objs))) => {
                                    let options: Vec<String> = objs.iter().map(|o| o.label()).collect();
                                    let agent_id = agent_id.clone();
                                    view! {
                                        <Select label="deployment object" options=options value=chosen />
                                        <Group><Button on_click=Callback::new(move |_| {
                                            // An empty or stale selection means the first
                                            // object: Aurora's Select shows the first option
                                            // and does not write the value.
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
                                                        toaster.success("Diagnostic requested. The agent collects the data.");
                                                    }
                                                    Err(_) => {
                                                        toaster.error("The diagnostic request failed.");
                                                    }
                                                }
                                            });
                                        })>"\u{2315} Run diagnostic"</Button></Group>
                                        <span class="brk-note">
                                            "The agent collects the pod statuses, the events and the log tails of this deployment object."
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
                                            <PanelError error=e on_retry=recheck />
                                        </Stack>
                                    }.into_any(),
                                    Some(Some(Ok(d))) => {
                                        let status = d.request.status.clone();
                                        let id8: String = d.request.id.chars().take(8).collect();
                                        let stalled_msg = format!(
                                            "The console waited {}s. The request is still {status}. A request stays claimed if the agent does not submit a result. This does not show that the collection failed.",
                                            POLL_MAX as u64 * POLL_EVERY_MS / 1000,
                                        );
                                        let body = match d.outcome() {
                                            // Bounded, and honest about what the bound does and
                                            // does not prove.
                                            DiagnosticOutcome::InFlight if exhausted => view! {
                                                <Alert title="No result yet" color=token::GOLD>
                                                    <Stack gap="xs">
                                                        <span class="brk-text">{stalled_msg}</span>
                                                        <Button on_click=recheck>"\u{21bb} Check again"</Button>
                                                    </Stack>
                                                </Alert>
                                            }.into_any(),
                                            DiagnosticOutcome::InFlight => view! {
                                                <Meter indeterminate=true label="The agent collects the pod statuses, the events and the log tails" />
                                                <span class="brk-note">"The agent collects the pod statuses, the events and the log tails…"</span>
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
                                                                "The agent could not read the cluster. The broker shows the request as completed, and the result holds the error."
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
                                                        "No agent claimed the request before it expired. The agent is probably offline.".to_string(),
                                                    ),
                                                    "failed" => (
                                                        "collection failed",
                                                        token::BAD,
                                                        "An agent claimed the request but submitted no result."
                                                            .to_string(),
                                                    ),
                                                    _ => (
                                                        "No result stored",
                                                        token::GOLD,
                                                        format!("The request is {s}, but the broker has no result for it."),
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
