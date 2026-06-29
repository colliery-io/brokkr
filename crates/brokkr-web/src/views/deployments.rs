//! Deployments view — stacks from `GET /api/v1/stacks`.
//! NOTE: the handoff shows per-stack deployment objects with a per-agent health
//! rollup. That needs the deployment-objects + `/stacks/:id/health` endpoints per
//! stack (N+1); v1 lists the stacks (name + generator). Per-object health is a
//! follow-up (logged on the task).

use crate::api;
use aurora_leptos::components::*;
use leptos::prelude::*;

#[component]
pub fn DeploymentsView() -> impl IntoView {
    let data = LocalResource::new(|| api::stacks());

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
                        let desc = s.description.unwrap_or_default();
                        view! {
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
                        }
                    })
                    .collect_view();
                view! { <Stack gap="md">{panels}</Stack> }.into_any()
            }
        }}
    }
}
