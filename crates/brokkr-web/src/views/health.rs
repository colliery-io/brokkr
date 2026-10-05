//! Broker health view — Prometheus metric tiles (`GET /metrics`) + the internal
//! WS connections panel (`GET /api/v1/admin/ws/connections`).

use crate::api;
use crate::models::WsConnectionInfo;
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, StatTile};
use aurora_leptos::frame::Drawer;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.fract() == 0.0 => (x as i64).to_string(),
        Some(x) => format!("{x:.1}"),
        None => "—".into(),
    }
}

/// (label, metric, hue). An empty hue is the tile's default bright text.
const CARDS: &[(&str, &str, &str)] = &[
    ("Active agents", "brokkr_active_agents", ""),
    ("WS connected", "brokkr_ws_connected_agents", token::TEAL),
    ("HTTP requests", "brokkr_http_requests_total", token::ICE),
    (
        "Live subscribers",
        "brokkr_fleet_live_subscribers",
        token::VIOLET,
    ),
    ("Stacks", "brokkr_stacks_total", ""),
    ("Deploy objects", "brokkr_deployment_objects_total", ""),
];

#[component]
pub fn BrokerHealthView() -> impl IntoView {
    let metrics = LocalResource::new(api::metrics_text);
    let conns = LocalResource::new(api::ws_connections);
    crate::components::poll(
        move || {
            metrics.refetch();
            conns.refetch();
        },
        std::time::Duration::from_secs(5),
    );
    let selected = RwSignal::new(None::<WsConnectionInfo>);
    let open = RwSignal::new(false);

    view! {
        <Stack gap="md">
            {move || match metrics.get() {
                None => view! { <Loading label="loading metrics" /> }.into_any(),
                Some(Err(e)) => view! {
                    <ErrorState error=e on_retry=Callback::new(move |_| { metrics.refetch(); }) />
                }
                .into_any(),
                Some(Ok(text)) => {
                    let cards = CARDS
                        .iter()
                        .map(|(label, name, color)| {
                            view! {
                                <StatTile
                                    label=*label
                                    value=fmt(api::metric_sum(&text, name))
                                    sub=name.to_string()
                                    color=*color
                                />
                            }
                        })
                        .collect_view();
                    view! { <div class="brk-kpis brk-kpis--wide">{cards}</div> }.into_any()
                }
            }}
            <Panel title="Internal WS connections">
                {move || match conns.get() {
                    None => view! { <Loading label="loading connections" /> }.into_any(),
                    Some(Err(e)) => view! {
                        <ErrorState error=e on_retry=Callback::new(move |_| { conns.refetch(); }) />
                    }
                    .into_any(),
                    Some(Ok(r)) if r.connections.is_empty() => view! {
                        <Empty message="No agents connected on the internal WS channel." />
                    }
                    .into_any(),
                    Some(Ok(r)) => {
                        let rows = r
                            .connections
                            .into_iter()
                            .map(|c| {
                                let c_sel = c.clone();
                                view! {
                                    <TableRow on_click=Callback::new(move |_| {
                                        selected.set(Some(c_sel.clone()));
                                        open.set(true);
                                    })>
                                        <td>
                                            <Group gap="sm">
                                                <Dot color=token::TEAL glow=true />
                                                <span>{c.agent_id.clone()}</span>
                                            </Group>
                                        </td>
                                        <td class="cl-num">{c.messages_in.to_string()}</td>
                                        <td class="cl-num">{c.messages_out.to_string()}</td>
                                    </TableRow>
                                }
                            })
                            .collect_view();
                        view! {
                            <Table mono=true label="Internal WS connections">
                                <thead>
                                    <tr>
                                        <th>"Agent"</th>
                                        <th class="cl-num">"Messages in \u{2193}"</th>
                                        <th class="cl-num">"Messages out \u{2191}"</th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                            </Table>
                        }
                        .into_any()
                    }
                }}
            </Panel>
        </Stack>

        <Drawer open=open title="WS connection">
            {move || match selected.get() {
                None => ().into_any(),
                Some(c) => view! {
                    <Stack gap="md">
                        <span class="brk-detail-title">{c.agent_id.clone()}</span>
                        <DetailList mono=true>
                            <KeyValue label="messages in">{c.messages_in.to_string()}</KeyValue>
                            <KeyValue label="messages out">{c.messages_out.to_string()}</KeyValue>
                            <KeyValue label="connected since">{c.connected_since.clone().unwrap_or_else(|| "—".into())}</KeyValue>
                        </DetailList>
                    </Stack>
                }
                .into_any(),
            }}
        </Drawer>
    }
}
