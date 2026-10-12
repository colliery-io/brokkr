//! Broker health view — Prometheus metric tiles (`GET /metrics`) + the internal
//! WS connections panel (`GET /api/v1/admin/ws/connections`). The database
//! tiles (queries per minute, p95 query latency, pool in use) degrade to "—"
//! on a broker that does not export the database metrics.

use crate::api;
use crate::components::{agent_href, agent_name, PanelError};
use crate::models::WsConnectionInfo;
use crate::views::overview::CounterSamples;
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

const DB_QUERIES: &str = "brokkr_database_queries_total";
const DB_LATENCY: &str = "brokkr_database_query_duration_seconds";
const DB_POOL: &str = "brokkr_database_pool_connections";
const DB_POOL_MAX: &str = "brokkr_database_pool_max_connections";
/// Shown under a database tile when the broker does not export the metric
/// (a broker from before the metric existed).
const NOT_EXPORTED: &str = "not sent by this broker";

/// Snapshots of a Prometheus histogram's cumulative buckets, for a quantile
/// over a recent window (BROKKR-T-0265).
///
/// The p95 is taken from the bucket deltas between the oldest and the newest
/// snapshot, so it describes the queries of about the last minute, not the
/// whole life of the broker. When the window holds no queries, or the counts
/// went down (the broker restarted), it falls back to the newest snapshot
/// alone: the queries since the broker started.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HistogramSamples {
    /// `(time in ms since the epoch, buckets)`, oldest first.
    points: Vec<(f64, Vec<(f64, f64)>)>,
}

/// Which queries a quantile describes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantileWindow {
    /// The queries in the last `n` seconds.
    Recent(u32),
    /// All queries since the broker started.
    SinceStart,
}

impl HistogramSamples {
    /// 13 samples at one every 5 s span one minute.
    pub const KEEP: usize = 13;

    pub fn push(&mut self, at_ms: f64, buckets: Vec<(f64, f64)>) {
        self.points.push((at_ms, buckets));
        while self.points.len() > Self::KEEP {
            self.points.remove(0);
        }
    }

    /// The `q` quantile in seconds and the window it covers, or None while no
    /// query has been recorded.
    pub fn quantile(&self, q: f64) -> Option<(f64, QuantileWindow)> {
        let (t1, newest) = self.points.last()?;
        let (t0, oldest) = self.points.first()?;
        let same_bounds =
            oldest.len() == newest.len() && oldest.iter().zip(newest).all(|(a, b)| a.0 == b.0);
        if same_bounds && t1 > t0 {
            let delta: Option<Vec<(f64, f64)>> = oldest
                .iter()
                .zip(newest)
                .map(|(a, b)| (b.1 >= a.1).then_some((b.0, b.1 - a.1)))
                .collect();
            if let Some(v) = delta.and_then(|d| api::histogram_quantile(&d, q)) {
                let secs = ((t1 - t0) / 1000.0).round() as u32;
                return Some((v, QuantileWindow::Recent(secs)));
            }
        }
        api::histogram_quantile(newest, q).map(|v| (v, QuantileWindow::SinceStart))
    }
}

/// A latency in seconds as a short reading: `0.8 ms`, `12 ms`, `1.3 s`.
pub fn fmt_latency(secs: f64) -> String {
    let ms = secs * 1000.0;
    if ms < 10.0 {
        format!("{ms:.1} ms")
    } else if ms < 1000.0 {
        format!("{} ms", ms.round() as i64)
    } else {
        format!("{secs:.1} s")
    }
}

/// The three database tiles as `(label, value, sub, hue)`, from the newest
/// metrics text and the samples so far.
fn db_tiles(
    text: &str,
    rate: Option<f64>,
    p95: Option<(f64, QuantileWindow)>,
) -> [(&'static str, String, String, &'static str); 3] {
    let has_queries = api::metric_sum(text, DB_QUERIES).is_some();
    let has_latency = !api::histogram_buckets(text, DB_LATENCY).is_empty();

    let queries = match (has_queries, rate) {
        (false, _) => ("—".to_string(), NOT_EXPORTED.to_string()),
        (true, None) => (
            "collecting".to_string(),
            "the rate needs two samples".to_string(),
        ),
        (true, Some(r)) => (fmt(Some(r.round())), DB_QUERIES.to_string()),
    };
    let latency = match (has_latency, p95) {
        (false, _) => ("—".to_string(), NOT_EXPORTED.to_string()),
        (true, None) => ("collecting".to_string(), "no queries yet".to_string()),
        (true, Some((v, QuantileWindow::Recent(s)))) => (fmt_latency(v), format!("last {s} s")),
        (true, Some((v, QuantileWindow::SinceStart))) => {
            (fmt_latency(v), "since broker start".to_string())
        }
    };
    let in_use = api::metric_sum(text, &format!("{DB_POOL}{{state=\"in_use\"}}"));
    let max = api::metric_sum(text, DB_POOL_MAX);
    let pool = match (in_use, max) {
        (Some(u), Some(m)) => (format!("{}/{}", u as i64, m as i64), DB_POOL.to_string()),
        _ => ("—".to_string(), NOT_EXPORTED.to_string()),
    };
    [
        ("DB queries / min", queries.0, queries.1, token::ICE),
        ("DB query p95", latency.0, latency.1, token::GOLD),
        ("DB pool in use", pool.0, pool.1, ""),
    ]
}

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
    // Sample the database counter and histogram on each poll. The rate and
    // the recent p95 come from the deltas between samples (BROKKR-T-0265).
    let db_queries = RwSignal::new(CounterSamples::default());
    let db_latency = RwSignal::new(HistogramSamples::default());
    Effect::new(move |_| {
        if let Some(Ok(text)) = metrics.get() {
            let now = js_sys::Date::now();
            if let Some(v) = api::metric_sum(&text, DB_QUERIES) {
                db_queries.update(|s| s.push(now, v));
            }
            let buckets = api::histogram_buckets(&text, DB_LATENCY);
            if !buckets.is_empty() {
                db_latency.update(|s| s.push(now, buckets));
            }
        }
    });
    let selected = RwSignal::new(None::<WsConnectionInfo>);
    let open = RwSignal::new(false);
    // Agent names for the connections table (BROKKR-T-0337).
    let fleet = LocalResource::new(move || api::fleet(None));

    view! {
        <Stack gap="md">
            {move || match metrics.get() {
                None => view! { <Loading label="loading metrics" /> }.into_any(),
                Some(Err(e)) => view! {
                    <PanelError error=e on_retry=Callback::new(move |_| { metrics.refetch(); }) />
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
                    let rate = db_queries.with(|s| s.latest());
                    let p95 = db_latency.with(|s| s.quantile(0.95));
                    let db_cards = db_tiles(&text, rate, p95)
                        .into_iter()
                        .map(|(label, value, sub, color)| {
                            view! { <StatTile label=label value=value sub=sub color=color /> }
                        })
                        .collect_view();
                    view! { <div class="brk-kpis brk-kpis--wide">{cards}{db_cards}</div> }
                        .into_any()
                }
            }}
            <Panel title="Internal WS connections">
                {move || match conns.get() {
                    None => view! { <Loading label="loading connections" /> }.into_any(),
                    Some(Err(e)) => view! {
                        <PanelError error=e on_retry=Callback::new(move |_| { conns.refetch(); }) />
                    }
                    .into_any(),
                    Some(Ok(r)) if r.connections.is_empty() => view! {
                        <Empty message="No agents connected on the internal WS channel." />
                    }
                    .into_any(),
                    Some(Ok(r)) => {
                        let names = fleet.get().and_then(|f| f.ok()).unwrap_or_default();
                        let rows = r
                            .connections
                            .into_iter()
                            .map(|c| {
                                let c_sel = c.clone();
                                let name = agent_name(&names, &c.agent_id);
                                view! {
                                    <TableRow on_click=Callback::new(move |_| {
                                        selected.set(Some(c_sel.clone()));
                                        open.set(true);
                                    })>
                                        <td>
                                            <Group gap="sm">
                                                <Dot color=token::TEAL glow=true />
                                                <span>{name}</span>
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
                        <span class="brk-detail-title">
                            {agent_name(&fleet.get().and_then(|f| f.ok()).unwrap_or_default(), &c.agent_id)}
                        </span>
                        <DetailList mono=true>
                            <KeyValue label="agent"><Anchor href=agent_href(&c.agent_id)>{c.agent_id.clone()}</Anchor></KeyValue>
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

#[cfg(test)]
mod tests {
    use super::*;

    fn b(counts: [f64; 3]) -> Vec<(f64, f64)> {
        vec![
            (0.001, counts[0]),
            (0.01, counts[1]),
            (f64::INFINITY, counts[2]),
        ]
    }

    #[test]
    fn the_p95_uses_the_window_deltas() {
        let mut s = HistogramSamples::default();
        assert_eq!(s.quantile(0.95), None);
        // Lifetime: all 100 queries under 1 ms.
        s.push(0.0, b([100.0, 100.0, 100.0]));
        let (v, w) = s.quantile(0.95).unwrap();
        assert_eq!(w, QuantileWindow::SinceStart);
        assert!(v <= 0.001);
        // In the next 60 s, 10 queries, all between 1 and 10 ms.
        s.push(60_000.0, b([100.0, 110.0, 110.0]));
        let (v, w) = s.quantile(0.95).unwrap();
        assert_eq!(w, QuantileWindow::Recent(60));
        assert!(v > 0.001 && v <= 0.01, "{v}");
    }

    #[test]
    fn an_idle_window_or_a_restart_falls_back_to_since_start() {
        let mut s = HistogramSamples::default();
        s.push(0.0, b([10.0, 20.0, 20.0]));
        s.push(5_000.0, b([10.0, 20.0, 20.0]));
        assert_eq!(s.quantile(0.95).unwrap().1, QuantileWindow::SinceStart);
        // The counts went down: the broker restarted.
        s.push(10_000.0, b([1.0, 2.0, 2.0]));
        assert_eq!(s.quantile(0.95).unwrap().1, QuantileWindow::SinceStart);
    }

    #[test]
    fn the_ring_keeps_one_minute() {
        let mut s = HistogramSamples::default();
        for i in 0..(HistogramSamples::KEEP + 5) {
            s.push(i as f64 * 5_000.0, b([i as f64, i as f64, i as f64]));
        }
        assert_eq!(s.quantile(0.95).unwrap().1, QuantileWindow::Recent(60));
    }

    #[test]
    fn latency_reads_in_ms_or_s() {
        assert_eq!(fmt_latency(0.0008), "0.8 ms");
        assert_eq!(fmt_latency(0.012), "12 ms");
        assert_eq!(fmt_latency(1.25), "1.2 s");
    }

    #[test]
    fn an_older_broker_shows_a_dash_not_zero() {
        let tiles = db_tiles("brokkr_active_agents 1\n", None, None);
        for (_, value, sub, _) in tiles {
            assert_eq!(value, "—");
            assert_eq!(sub, NOT_EXPORTED);
        }
    }

    #[test]
    fn a_new_broker_collects_then_shows_values() {
        let text = "\
brokkr_database_queries_total{result=\"ok\"} 10
brokkr_database_queries_total{result=\"error\"} 0
brokkr_database_query_duration_seconds_bucket{le=\"+Inf\"} 0
brokkr_database_pool_connections{state=\"idle\"} 3
brokkr_database_pool_connections{state=\"in_use\"} 2
brokkr_database_pool_max_connections 10
";
        let tiles = db_tiles(text, None, None);
        assert_eq!(tiles[0].1, "collecting");
        assert_eq!(tiles[1].1, "collecting");
        assert_eq!(tiles[2].1, "2/10");
        let tiles = db_tiles(text, Some(42.4), Some((0.0031, QuantileWindow::Recent(60))));
        assert_eq!(tiles[0].1, "42");
        assert_eq!(tiles[1].1, "3.1 ms");
        assert_eq!(tiles[1].2, "last 60 s");
    }
}
