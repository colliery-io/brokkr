//! Work orders view — **Active** (live, from `GET /api/v1/work-orders`, admin-gated)
//! over the completed **history** (`GET /api/v1/work-order-log`); click a history row
//! for detail. NOTE: the active list needs an admin PAK; with an operator-scoped PAK
//! that panel renders a note and the history still shows.

use crate::api;
use crate::components::{sev, EmptyNext, DOCS};
use crate::models::WorkOrderLogEntry;
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue};
use aurora_leptos::frame::Drawer;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

fn outcome(success: bool) -> (&'static str, &'static str) {
    if success {
        ("completed", token::OK)
    } else {
        ("failed", token::BAD)
    }
}

#[component]
pub fn WorkOrdersView() -> impl IntoView {
    let active = LocalResource::new(api::work_orders);
    let data = LocalResource::new(api::work_order_log);
    crate::components::poll(
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
                        <span class="brk-note">
                            "unavailable (the active list requires an admin PAK)"
                        </span>
                    </Panel>
                }.into_any(),
                Some(Ok(wos)) => {
                    let act: Vec<_> = wos.into_iter().filter(|w| w.is_active()).collect();
                    if act.is_empty() {
                        view! {
                            <Panel title="Active">
                                <EmptyNext
                                    message="No work orders in flight."
                                    next="A work order appears when a tenant requests an image build."
                                    href=format!("{DOCS}/reference/work-orders.html")
                                    link="What a work order is"
                                />
                            </Panel>
                        }.into_any()
                    } else {
                        let rows = act.into_iter().map(|w| {
                            let id8: String = w.id.chars().take(8).collect();
                            let claimed = w.claimed_by
                                .map(|c| format!("claimed by {}", c.chars().take(8).collect::<String>()))
                                .unwrap_or_else(|| "unclaimed".into());
                            let note = if w.retry_count > 0 { format!("{claimed} · retry {}", w.retry_count) } else { claimed };
                            view! {
                                <TableRow>
                                    <td class="brk-muted">{id8}</td>
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
                                        <th>"ID"</th><th>"Type"</th><th>"Status"</th><th>"Claim"</th>
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
                            let (label, color) = outcome(w.success);
                            let id8: String = w.id.chars().take(8).collect();
                            let detail = w.result_message.clone().unwrap_or_default();
                            let w_sel = w.clone();
                            view! {
                                <TableRow on_click=Callback::new(move |_| {
                                    selected.set(Some(w_sel.clone()));
                                    open.set(true);
                                })>
                                    <td class="brk-muted">{id8}</td>
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
                                widths=vec!["14%".into(), "18%".into(), "14%".into(), "54%".into()]>
                                <thead><tr>
                                    <th>"ID"</th><th>"Type"</th><th>"Outcome"</th><th>"Result"</th>
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
            {move || match selected.get() {
                None => ().into_any(),
                Some(w) => {
                    let (label, color) = outcome(w.success);
                    view! {
                        <Stack gap="md">
                            <Group gap="sm">
                                <Pill color=token::TEAL>{w.work_type.clone()}</Pill>
                                <Pill color=color>{label}</Pill>
                            </Group>
                            <DetailList mono=true>
                                <KeyValue label="id">{w.id.clone()}</KeyValue>
                                <KeyValue label="retries">{w.retries_attempted.to_string()}</KeyValue>
                                <KeyValue label="result">{w.result_message.clone().unwrap_or_else(|| "—".into())}</KeyValue>
                            </DetailList>
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Drawer>
    }
}
