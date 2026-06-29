//! Broker health view — Prometheus metric cards (`GET /metrics`) + the internal
//! WS connections panel (`GET /api/v1/admin/ws/connections`).

use crate::api;
use crate::components::DetailRow;
use crate::models::WsConnectionInfo;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.fract() == 0.0 => (x as i64).to_string(),
        Some(x) => format!("{x:.1}"),
        None => "—".into(),
    }
}

#[component]
fn MetricCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] sub: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    view! {
        <div style="background:var(--panel);border:1px solid var(--border);border-radius:10px;\
                    padding:14px 16px;min-width:170px;">
            <div style="font:600 10px var(--font-mono);letter-spacing:.04em;text-transform:uppercase;\
                        color:var(--muted);">{label}</div>
            <div style=format!(
                "font:600 26px var(--font-mono);color:{color};font-variant-numeric:tabular-nums;\
                 margin:4px 0;line-height:1;"
            )>{value}</div>
            <div style="font:10px var(--font-mono);color:var(--faint);">{sub}</div>
        </div>
    }
}

const CARDS: &[(&str, &str, &str)] = &[
    ("Active agents", "brokkr_active_agents", "var(--fg-bright)"),
    ("WS connected", "brokkr_ws_connected_agents", token::TEAL),
    ("HTTP requests", "brokkr_http_requests_total", token::ICE),
    ("Live subscribers", "brokkr_fleet_live_subscribers", token::VIOLET),
    ("Stacks", "brokkr_stacks_total", "var(--fg-bright)"),
    ("Deploy objects", "brokkr_deployment_objects_total", "var(--fg-bright)"),
];

#[component]
pub fn BrokerHealthView() -> impl IntoView {
    let metrics = LocalResource::new(|| api::metrics_text());
    let conns = LocalResource::new(|| api::ws_connections());
    set_interval(
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
                                <MetricCard
                                    label=*label
                                    value=fmt(api::metric_sum(&text, name))
                                    sub=*name
                                    color=*color
                                />
                            }
                        })
                        .collect_view();
                    view! { <Group gap="md" wrap=true>{cards}</Group> }.into_any()
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
                                    <div style="cursor:pointer;" on:click=move |_| {
                                        selected.set(Some(c_sel.clone()));
                                        open.set(true);
                                    }>
                                        <Group justify="between">
                                            <Group gap="sm">
                                                <Dot color=token::TEAL glow=true />
                                                <span style="font:12px var(--font-mono);color:var(--fg);">
                                                    {c.agent_id.clone()}
                                                </span>
                                            </Group>
                                            <span style="font:11px var(--font-mono);color:var(--muted);">
                                                {format!("{}\u{2193} {}\u{2191}", c.messages_in, c.messages_out)}
                                            </span>
                                        </Group>
                                    </div>
                                }
                            })
                            .collect_view();
                        view! { <Stack gap="sm">{rows}</Stack> }.into_any()
                    }
                }}
            </Panel>
        </Stack>

        <Modal open=open title="WS connection">
            {move || match selected.get() {
                None => ().into_any(),
                Some(c) => view! {
                    <Stack gap="md">
                        <span style="font:600 14px var(--font-mono);color:var(--fg-bright);word-break:break-all;">
                            {c.agent_id.clone()}
                        </span>
                        <div>
                            <DetailRow label="messages in">{c.messages_in.to_string()}</DetailRow>
                            <DetailRow label="messages out">{c.messages_out.to_string()}</DetailRow>
                            <DetailRow label="connected since">{c.connected_since.clone().unwrap_or_else(|| "—".into())}</DetailRow>
                        </div>
                    </Stack>
                }
                .into_any(),
            }}
        </Modal>
    }
}
