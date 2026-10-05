//! Deployments view — stacks from `GET /api/v1/stacks`; click a stack for detail.
//! NOTE: the handoff shows per-stack deployment objects with a per-agent health
//! rollup. That needs the deployment-objects + `/stacks/:id/health` endpoints per
//! stack (N+1); v1 lists the stacks (name + generator) and a detail drawer. Per-object
//! health is a follow-up (logged on the task).

use crate::api;
use crate::components::{sev, EmptyNext, DOCS};
use crate::models::Stack;
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, SectionLabel};
use aurora_leptos::frame::{Card, Drawer};
use leptos::prelude::*;

#[component]
pub fn DeploymentsView() -> impl IntoView {
    // Scope-reactive (BROKKR-I-0032): refetches when the tenant selection changes.
    let scope = crate::app::use_scope();
    let data = LocalResource::new(move || api::stacks(scope.get()));
    let selected = RwSignal::new(None::<Stack>);
    let open = RwSignal::new(false);
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
                        let gen8: String = s.generator_id.chars().take(8).collect();
                        let desc = s.description.clone().unwrap_or_default();
                        let s_sel = s.clone();
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
                                <Group justify="between">
                                    <span class="brk-text">{desc}</span>
                                    <span class="brk-note">{format!("gen · {gen8}")}</span>
                                </Group>
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
                            <KeyValue label="generator">{s.generator_id.clone()}</KeyValue>
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
                    </Stack>
                }
                .into_any(),
            }}
        </Drawer>
    }
}
