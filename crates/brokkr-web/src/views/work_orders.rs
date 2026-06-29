//! Work orders view — **Active** (live, from `GET /api/v1/work-orders`, admin-gated)
//! over the completed **history** (`GET /api/v1/work-order-log`); click a history row
//! for detail. NOTE: the active list needs an admin PAK; with an operator-scoped PAK
//! that panel renders an error and the history still shows.

use crate::api;
use crate::components::{sev, DetailRow};
use crate::models::WorkOrderLogEntry;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

#[component]
pub fn WorkOrdersView() -> impl IntoView {
    let active = LocalResource::new(|| api::work_orders());
    let data = LocalResource::new(|| api::work_order_log());
    set_interval(
        move || {
            active.refetch();
            data.refetch();
        },
        std::time::Duration::from_secs(5),
    );
    let selected = RwSignal::new(None::<WorkOrderLogEntry>);
    let open = RwSignal::new(false);

    view! {
        <Stack gap="md">
            // Active (live) work orders
            {move || match active.get() {
                None => view! { <Loading label="loading active" /> }.into_any(),
                Some(Err(_)) => view! {
                    <Panel title="Active">
                        <span style="font:11px var(--font-mono);color:var(--faint);">
                            "unavailable (the active list requires an admin PAK)"
                        </span>
                    </Panel>
                }.into_any(),
                Some(Ok(wos)) => {
                    let act: Vec<_> = wos.into_iter().filter(|w| w.is_active()).collect();
                    if act.is_empty() {
                        view! { <Panel title="Active"><Empty message="No work orders in flight." /></Panel> }.into_any()
                    } else {
                        let rows = act.into_iter().map(|w| {
                            let id8: String = w.id.chars().take(8).collect();
                            let claimed = w.claimed_by
                                .map(|c| format!("claimed by {}", c.chars().take(8).collect::<String>()))
                                .unwrap_or_else(|| "unclaimed".into());
                            view! {
                                <Group justify="between">
                                    <Group gap="sm">
                                        <span style="font:12px var(--font-mono);color:var(--muted);min-width:80px;">{id8}</span>
                                        <Pill color=token::TEAL>{w.work_type}</Pill>
                                        <Pill color=sev(&w.status)>{w.status}</Pill>
                                    </Group>
                                    <span style="font:11px var(--font-mono);color:var(--faint);">
                                        {if w.retry_count > 0 { format!("{claimed} · retry {}", w.retry_count) } else { claimed }}
                                    </span>
                                </Group>
                            }
                        }).collect_view();
                        view! { <Panel title="Active"><Stack gap="sm">{rows}</Stack></Panel> }.into_any()
                    }
                }
            }}

            // Completed history
            {move || match data.get() {
                None => view! { <Loading label="loading history" /> }.into_any(),
                Some(Err(e)) => view! {
                    <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
                }
                .into_any(),
                Some(Ok(log)) if log.is_empty() => {
                    view! { <Panel title="History"><Empty message="No completed work orders yet." /></Panel> }.into_any()
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
                        <Panel title="History">
                            <Stack gap="sm">{rows}</Stack>
                        </Panel>
                    }
                    .into_any()
                }
            }}
        </Stack>

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
