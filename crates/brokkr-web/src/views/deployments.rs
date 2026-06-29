//! Deployments view — stacks from `GET /api/v1/stacks`; click a stack for detail.
//! NOTE: the handoff shows per-stack deployment objects with a per-agent health
//! rollup. That needs the deployment-objects + `/stacks/:id/health` endpoints per
//! stack (N+1); v1 lists the stacks (name + generator) and a detail Modal. Per-object
//! health is a follow-up (logged on the task).

use crate::api;
use crate::components::DetailRow;
use crate::models::Stack;
use aurora_leptos::components::*;
use crate::components::sev;
use leptos::prelude::*;

#[component]
pub fn DeploymentsView() -> impl IntoView {
    let data = LocalResource::new(|| api::stacks());
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
                view! { <Empty message="No stacks." /> }.into_any()
            }
            Some(Ok(stacks)) => {
                let panels = stacks
                    .into_iter()
                    .map(|s| {
                        let gen8: String = s.generator_id.chars().take(8).collect();
                        let desc = s.description.clone().unwrap_or_default();
                        let s_sel = s.clone();
                        view! {
                            <div style="cursor:pointer;" on:click=move |_| {
                                selected.set(Some(s_sel.clone()));
                                open.set(true);
                            }>
                                <Panel title=s.name.clone()>
                                    <Group justify="between">
                                        <span style="font:12px var(--font-sans);color:var(--muted);">
                                            {desc}
                                        </span>
                                        <span style="font:11px var(--font-mono);color:var(--faint);">
                                            {format!("gen · {gen8}")}
                                        </span>
                                    </Group>
                                </Panel>
                            </div>
                        }
                    })
                    .collect_view();
                view! { <Stack gap="md">{panels}</Stack> }.into_any()
            }
        }}

        <Modal open=open title="Stack detail">
            {move || match selected.get() {
                None => ().into_any(),
                Some(s) => view! {
                    <Stack gap="md">
                        <span style="font:600 15px var(--font-mono);color:var(--fg-bright);">{s.name.clone()}</span>
                        <div>
                            <DetailRow label="stack id">{s.id.clone()}</DetailRow>
                            <DetailRow label="generator">{s.generator_id.clone()}</DetailRow>
                            <DetailRow label="description">{s.description.clone().unwrap_or_else(|| "—".into())}</DetailRow>
                        </div>
                        <span style="font:600 10px var(--font-mono);text-transform:uppercase;\
                                     letter-spacing:.05em;color:var(--muted);">"deployment health"</span>
                        {move || match health.get() {
                            None | Some(None) => view! { <Loading label="loading health" /> }.into_any(),
                            Some(Some(Err(_))) => view! {
                                <span style="font:11px var(--font-mono);color:var(--faint);">
                                    "health unavailable"
                                </span>
                            }.into_any(),
                            Some(Some(Ok(h))) => {
                                let oc = sev(&h.overall_status);
                                let rows = h.deployment_objects.into_iter().map(|o| {
                                    let id8: String = o.id.chars().take(8).collect();
                                    view! {
                                        <Group justify="between">
                                            <Group gap="sm">
                                                <Pill color=sev(&o.status)>{o.status}</Pill>
                                                <span style="font:11px var(--font-mono);color:var(--muted);">{id8}</span>
                                            </Group>
                                            <span style="font:10px var(--font-mono);color:var(--faint);">
                                                {format!("{}\u{2713} {}~ {}\u{2717}", o.healthy_agents, o.degraded_agents, o.failing_agents)}
                                            </span>
                                        </Group>
                                    }
                                }).collect_view();
                                view! {
                                    <Stack gap="sm">
                                        <Pill color=oc>{h.overall_status}</Pill>
                                        {rows}
                                    </Stack>
                                }.into_any()
                            }
                        }}
                    </Stack>
                }
                .into_any(),
            }}
        </Modal>
    }
}
