//! Work orders view — **Active** (live, from `GET /api/v1/work-orders`) over the
//! completed **history** (`GET /api/v1/work-order-log`); click a history row
//! for detail. Each row names its owner: the generator that created the work
//! order, or "admin". The two routes take no generator filter, so the view
//! narrows to the generator scope in the client (BROKKR-I-0032).

use crate::api;
use crate::components::{in_scope, sev, work_order_owner, PanelError, DOCS};
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
    let scope = crate::app::use_scope();
    let active = LocalResource::new(api::work_orders);
    let data = LocalResource::new(api::work_order_log);
    // The generator list, for the owner's name.
    let generators = LocalResource::new(api::generators);
    let owner_name = move |owner: Option<&str>| -> String {
        let gens = generators.get().and_then(|r| r.ok()).unwrap_or_default();
        work_order_owner(&gens, owner)
    };
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
                Some(Err(e)) => view! {
                    <PanelError error=e on_retry=Callback::new(move |_| { active.refetch(); }) />
                }.into_any(),
                Some(Ok(wos)) => {
                    let sc = scope.get();
                    let act: Vec<_> = wos
                        .into_iter()
                        .filter(|w| w.is_active())
                        .filter(|w| in_scope(w.owner_generator_id.as_deref(), sc.as_deref()))
                        .collect();
                    if act.is_empty() {
                        view! {
                            <Panel title="Active">
                                <Empty
                                    message="No active work orders."
                                    hint="A work order is a job that runs one time. An admin or a generator creates it."
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
                            let owner = owner_name(w.owner_generator_id.as_deref());
                            view! {
                                <TableRow>
                                    <td class="brk-muted">{id8}</td>
                                    <td>{owner}</td>
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
                                        <th>"ID"</th><th>"Owner"</th><th>"Type"</th><th>"Status"</th><th>"Claim"</th>
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
                    <PanelError error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
                }
                .into_any(),
                Some(Ok(log)) => {
                    let sc = scope.get();
                    let log: Vec<_> = log
                        .into_iter()
                        .filter(|w| in_scope(w.owner_generator_id.as_deref(), sc.as_deref()))
                        .collect();
                    if log.is_empty() {
                        return view! { <Panel title="History"><Empty message="No completed work orders yet." /></Panel> }.into_any();
                    }
                    let rows = log
                        .into_iter()
                        .map(|w| {
                            let (label, color) = outcome(w.success);
                            let id8: String = w.id.chars().take(8).collect();
                            let detail = w.result_message.clone().unwrap_or_default();
                            let owner = owner_name(w.owner_generator_id.as_deref());
                            let w_sel = w.clone();
                            view! {
                                <TableRow on_click=Callback::new(move |_| {
                                    selected.set(Some(w_sel.clone()));
                                    open.set(true);
                                })>
                                    <td class="brk-muted">{id8}</td>
                                    <td class="brk-ellipsis">{owner}</td>
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
                                widths=vec!["12%".into(), "16%".into(), "16%".into(), "14%".into(), "42%".into()]>
                                <thead><tr>
                                    <th>"ID"</th><th>"Owner"</th><th>"Type"</th><th>"Outcome"</th><th>"Result"</th>
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
                                <KeyValue label="owner">{owner_name(w.owner_generator_id.as_deref())}</KeyValue>
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
