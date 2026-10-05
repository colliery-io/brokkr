//! Deployments view — stacks from `GET /api/v1/stacks`, each card with its
//! tenant and its overall health (`/stacks/:id/health`, one call per stack on
//! load); click a stack for its deployment objects and the agents that have
//! them (BROKKR-T-0337). The agents come from each agent's target state,
//! fetched when the drawer opens, not on a poll.

use crate::api;
use crate::components::{agent_href, sev, EmptyNext, DOCS};
use crate::models::{FleetAgentRecord, Stack};
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, SectionLabel};
use aurora_leptos::frame::{Card, Drawer};
use leptos::prelude::*;

#[component]
pub fn DeploymentsView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let data = LocalResource::new(move || api::stacks(scope.get()));
    let generators = LocalResource::new(api::generators);
    let selected = RwSignal::new(None::<Stack>);
    let open = RwSignal::new(false);
    // Overall health of each stack in view, for the card pills.
    let healths = LocalResource::new(move || {
        let stacks = data.get();
        async move {
            let mut out = Vec::new();
            if let Some(Ok(list)) = stacks {
                for s in list {
                    if let Ok(h) = api::stack_health(&s.id).await {
                        out.push((s.id.clone(), h.overall_status));
                    }
                }
            }
            out
        }
    });
    // Per-stack deployment-object health, refetched when the selection changes.
    let health = LocalResource::new(move || {
        let id = selected.get().map(|s| s.id.clone());
        async move {
            match id {
                Some(id) => Some(api::stack_health(&id).await),
                None => None,
            }
        }
    });
    // The agents that have an object of the selected stack: each agent's
    // target state, read once per open drawer.
    let agents = LocalResource::new(move || {
        let id = selected.get().map(|s| s.id.clone());
        let sc = scope.get();
        async move {
            let id = id?;
            let fleet = api::fleet(sc).await.ok()?;
            let mut out: Vec<(FleetAgentRecord, usize)> = Vec::new();
            for a in fleet {
                if let Ok(objs) = api::agent_target_state(&a.agent_id).await {
                    let n = objs.iter().filter(|o| o.stack_id == id).count();
                    if n > 0 {
                        out.push((a, n));
                    }
                }
            }
            Some(out)
        }
    });
    // A link from another view (`#deployments/stack/<id>`) opens that stack's
    // drawer once (BROKKR-T-0337).
    let selection = crate::app::use_selection();
    let applied = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        let Some(sel) = selection.get() else {
            return;
        };
        if sel.kind != "stack" || applied.get_untracked().as_deref() == Some(sel.id.as_str()) {
            return;
        }
        if let Some(Ok(stacks)) = data.get() {
            if let Some(s) = stacks
                .iter()
                .find(|s| s.id == sel.id || s.id.starts_with(&sel.id))
            {
                applied.set(Some(sel.id.clone()));
                selected.set(Some(s.clone()));
                open.set(true);
            }
        }
    });
    // The tenant's name for a generator id, or the first 8 characters of the id.
    let tenant_name = move |gid: &str| -> String {
        generators
            .get()
            .and_then(|r| r.ok())
            .and_then(|gs| gs.into_iter().find(|g| g.id == gid).map(|g| g.name))
            .unwrap_or_else(|| gid.chars().take(8).collect())
    };

    view! {
        {move || match data.get() {
            None => view! { <Loading label="loading stacks" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(stacks)) if stacks.is_empty() => {
                view! {
                    <EmptyNext
                        message="No stacks yet."
                        next="A tenant creates a stack with its own PAK, then pushes manifests to it."
                        href=format!("{DOCS}/tutorials/first-deployment.html")
                        link="Deploy a first application"
                    />
                }.into_any()
            }
            Some(Ok(stacks)) => {
                let cards = stacks
                    .into_iter()
                    .map(|s| {
                        let tenant = tenant_name(&s.generator_id);
                        let desc = s.description.clone().unwrap_or_default();
                        let s_sel = s.clone();
                        let s_id = s.id.clone();
                        view! {
                            <Card
                                title=s.name.clone()
                                selected=Signal::derive({
                                    let id = s.id.clone();
                                    move || open.get() && selected.with(|x| x.as_ref().is_some_and(|x| x.id == id))
                                })
                                on_click=Callback::new(move |_| {
                                    selected.set(Some(s_sel.clone()));
                                    open.set(true);
                                })
                            >
                                <Stack gap="xs">
                                    <span class="brk-text">{desc}</span>
                                    <Group gap="sm">
                                        {move || healths.get().and_then(|v| {
                                            v.into_iter().find(|(id, _)| *id == s_id).map(|(_, st)| st)
                                        }).map(|st| view! { <Pill color=sev(&st)>{st}</Pill> })}
                                        <span class="brk-note">{format!("tenant · {tenant}")}</span>
                                    </Group>
                                </Stack>
                            </Card>
                        }
                    })
                    .collect_view();
                view! { <div class="brk-cards">{cards}</div> }.into_any()
            }
        }}

        <Drawer open=open title="Stack detail">
            {move || match selected.get() {
                None => ().into_any(),
                Some(s) => view! {
                    <Stack gap="md">
                        <span class="brk-detail-title">{s.name.clone()}</span>
                        <DetailList mono=true>
                            <KeyValue label="stack id">{s.id.clone()}</KeyValue>
                            <KeyValue label="tenant">{let gid = s.generator_id.clone(); move || tenant_name(&gid)}</KeyValue>
                            <KeyValue label="description">{s.description.clone().unwrap_or_else(|| "—".into())}</KeyValue>
                        </DetailList>
                        <SectionLabel label="deployment health" />
                        {move || match health.get() {
                            None | Some(None) => view! { <Loading label="loading health" /> }.into_any(),
                            Some(Some(Err(_))) => view! {
                                <span class="brk-note">"health unavailable"</span>
                            }.into_any(),
                            Some(Some(Ok(h))) => {
                                let oc = sev(&h.overall_status);
                                let rows = h.deployment_objects.into_iter().map(|o| {
                                    let id8: String = o.id.chars().take(8).collect();
                                    view! {
                                        <Group justify="between">
                                            <Group gap="sm">
                                                <Pill color=sev(&o.status)>{o.status}</Pill>
                                                <span class="brk-meta">{id8}</span>
                                            </Group>
                                            <span class="brk-note">
                                                {format!("{}\u{2713} {}~ {}\u{2717}", o.healthy_agents, o.degraded_agents, o.failing_agents)}
                                            </span>
                                        </Group>
                                    }
                                }).collect_view();
                                view! {
                                    <Stack gap="sm">
                                        <Group gap="sm">
                                            <span class="brk-meta">"overall"</span>
                                            <Pill color=oc>{h.overall_status}</Pill>
                                        </Group>
                                        {rows}
                                    </Stack>
                                }.into_any()
                            }
                        }}
                        <SectionLabel label="agents" />
                        {move || match agents.get() {
                            None | Some(None) => view! { <Loading label="loading agents" /> }.into_any(),
                            Some(Some(list)) if list.is_empty() => view! {
                                <span class="brk-note">"No agent has this stack yet. An admin targets an agent, or a label matches one."</span>
                            }.into_any(),
                            Some(Some(list)) => {
                                let rows = list.into_iter().map(|(a, n)| {
                                    let (h, hc) = a.health();
                                    view! {
                                        <Group gap="sm">
                                            <Dot color=hc />
                                            <Anchor href=agent_href(&a.agent_id)>{a.name.clone()}</Anchor>
                                            <Pill color=hc>{h}</Pill>
                                            <span class="brk-meta">{format!("{n} object{}", if n == 1 { "" } else { "s" })}</span>
                                        </Group>
                                    }
                                }).collect_view();
                                view! { <Stack gap="xs">{rows}</Stack> }.into_any()
                            }
                        }}
                    </Stack>
                }
                .into_any(),
            }}
        </Drawer>
    }
}
