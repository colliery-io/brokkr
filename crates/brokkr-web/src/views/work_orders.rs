//! Work orders view — completed history from `GET /api/v1/work-order-log`; click a
//! row for detail. NOTE: the broker exposes no "list active work orders" endpoint
//! (only the per-id + the log); active counts live per-agent in the fleet record.
//! So this shows history; an "Active" section needs a broker enhancement (logged on task).

use crate::api;
use crate::components::DetailRow;
use crate::models::WorkOrderLogEntry;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

#[component]
pub fn WorkOrdersView() -> impl IntoView {
    let data = LocalResource::new(|| api::work_order_log());
    set_interval(move || data.refetch(), std::time::Duration::from_secs(5));
    let selected = RwSignal::new(None::<WorkOrderLogEntry>);
    let open = RwSignal::new(false);

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
                        let detail = w.result_message.clone().unwrap_or_default();
                        let w_sel = w.clone();
                        view! {
                            <div style="cursor:pointer;" on:click=move |_| {
                                selected.set(Some(w_sel.clone()));
                                open.set(true);
                            }>
                                <Group justify="between">
                                    <Group gap="sm">
                                        <span style="font:12px var(--font-mono);color:var(--muted);\
                                                     min-width:80px;">{id8}</span>
                                        <Pill color=token::TEAL>{w.work_type.clone()}</Pill>
                                        <Pill color=color>{label}</Pill>
                                    </Group>
                                    <span style="font:11px var(--font-mono);color:var(--faint);\
                                                 max-width:50%;overflow:hidden;text-overflow:ellipsis;\
                                                 white-space:nowrap;">{detail}</span>
                                </Group>
                            </div>
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

        <Modal open=open title="Work order">
            {move || match selected.get() {
                None => ().into_any(),
                Some(w) => {
                    let (label, color) = if w.success { ("completed", token::OK) } else { ("failed", token::BAD) };
                    view! {
                        <Stack gap="md">
                            <Group gap="sm">
                                <Pill color=token::TEAL>{w.work_type.clone()}</Pill>
                                <Pill color=color>{label}</Pill>
                            </Group>
                            <div>
                                <DetailRow label="id">{w.id.clone()}</DetailRow>
                                <DetailRow label="retries">{w.retries_attempted.to_string()}</DetailRow>
                                <DetailRow label="result">{w.result_message.clone().unwrap_or_else(|| "—".into())}</DetailRow>
                            </div>
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Modal>
    }
}
