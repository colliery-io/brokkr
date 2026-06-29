//! Work orders view — completed history from `GET /api/v1/work-order-log`.
//! NOTE: the broker exposes no "list active work orders" endpoint (only the
//! per-id + the log); active counts live per-agent in the fleet record. So this
//! shows history; an "Active" section needs a broker enhancement (logged on task).

use crate::api;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

#[component]
pub fn WorkOrdersView() -> impl IntoView {
    let data = LocalResource::new(|| api::work_order_log());
    set_interval(move || data.refetch(), std::time::Duration::from_secs(5));

    view! {
        {move || match data.get() {
            None => view! { <Loading label="loading work orders" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(log)) if log.is_empty() => {
                view! { <Empty message="No work orders yet." /> }.into_any()
            }
            Some(Ok(log)) => {
                let rows = log
                    .into_iter()
                    .map(|w| {
                        let (label, color) = if w.success {
                            ("completed", token::OK)
                        } else {
                            ("failed", token::BAD)
                        };
                        let id8: String = w.id.chars().take(8).collect();
                        let detail = w.result_message.unwrap_or_default();
                        view! {
                            <Group justify="between">
                                <Group gap="sm">
                                    <span style="font:12px var(--font-mono);color:var(--muted);\
                                                 min-width:80px;">{id8}</span>
                                    <Pill color=token::TEAL>{w.work_type}</Pill>
                                    <Pill color=color>{label}</Pill>
                                </Group>
                                <span style="font:11px var(--font-mono);color:var(--faint);\
                                             max-width:50%;overflow:hidden;text-overflow:ellipsis;\
                                             white-space:nowrap;">{detail}</span>
                            </Group>
                        }
                    })
                    .collect_view();
                view! {
                    <Panel title="Work order history">
                        <Stack gap="sm">{rows}</Stack>
                    </Panel>
                }
                .into_any()
            }
        }}
    }
}
