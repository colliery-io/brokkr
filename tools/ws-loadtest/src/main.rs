/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! WS channel load-test harness (BROKKR-T-0177 / I-0020 B4).
//!
//! Drives a synthetic agent fleet against a live broker over the internal
//! WebSocket channel to establish a v0.5.0 throughput/footprint baseline.
//! Each synthetic agent is a thin `tokio-tungstenite` client — it does NOT
//! run the real `brokkr-agent` runtime; it just speaks the wire protocol.
//!
//! What it does:
//!   1. Provisions a generator, a pool of stacks, and N agents (each with a
//!      unique PAK) via the REST API using an admin PAK.
//!   2. Opens N agent WS connections; each sends a `heartbeat` every 5s and
//!      telemetry (`k8s_event` / `pod_log_line`, alternating) at a target
//!      per-agent message rate. Telemetry is what exercises the new I-0019
//!      Postgres write path under the 6h retention ceiling.
//!   3. Opens K live-subscriber connections (admin PAK) across the stack pool
//!      to exercise the per-stack broadcast fan-out.
//!   4. Samples broker `/metrics`, `docker stats` (CPU%/RSS), and the
//!      telemetry-table row counts every few seconds, then prints a summary
//!      with the achieved rates and peak footprint.
//!
//! The wire JSON is hand-rolled rather than pulling in `brokkr-wire` (which
//! would drag `brokkr-models` + diesel into this throwaway tool). The format
//! mirrors `brokkr_wire::WsMessage` exactly: external-tagged
//! (`{"type": "...", "body": {...}}`), snake_case variant names. The run
//! validates correctness empirically: if the format were wrong the broker
//! would reject the frames and the telemetry row counts would never grow,
//! which the summary would show as a zero write rate.
//!
//! Config is via env vars (all optional, with baseline defaults):
//!   BROKER_URL          default http://localhost:3000
//!   ADMIN_PAK           default the dev admin PAK
//!   LT_AGENTS           default 500
//!   LT_STACKS           default 50
//!   LT_SUBSCRIBERS      default 50
//!   LT_MSG_RATE         default 10   (telemetry msgs/sec/agent)
//!   LT_DURATION_SECS    default 300
//!   LT_SAMPLE_SECS      default 10
//!   LT_BROKER_CONTAINER default brokkr-dev-broker-1
//!   LT_PG_CONTAINER     default brokkr-dev-postgres-1

use std::process::Command;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;

struct Config {
    broker_url: String,
    admin_pak: String,
    agents: usize,
    stacks: usize,
    subscribers: usize,
    msg_rate: u64,
    duration: Duration,
    sample: Duration,
    broker_container: String,
    pg_container: String,
}

impl Config {
    fn from_env() -> Self {
        let env_or = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        let num = |k: &str, d: u64| -> u64 {
            std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
        };
        Config {
            broker_url: env_or("BROKER_URL", "http://localhost:3000"),
            admin_pak: env_or(
                "ADMIN_PAK",
                "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8",
            ),
            agents: num("LT_AGENTS", 500) as usize,
            stacks: num("LT_STACKS", 50) as usize,
            subscribers: num("LT_SUBSCRIBERS", 50) as usize,
            msg_rate: num("LT_MSG_RATE", 10),
            duration: Duration::from_secs(num("LT_DURATION_SECS", 300)),
            sample: Duration::from_secs(num("LT_SAMPLE_SECS", 10)),
            broker_container: env_or("LT_BROKER_CONTAINER", "brokkr-dev-broker-1"),
            pg_container: env_or("LT_PG_CONTAINER", "brokkr-dev-postgres-1"),
        }
    }
}

/// Shared counters across all synthetic clients.
#[derive(Default)]
struct Stats {
    connected: AtomicU64,
    conn_errors: AtomicU64,
    sent: AtomicU64,
    send_errors: AtomicU64,
    sub_connected: AtomicU64,
    sub_received: AtomicU64,
    sub_errors: AtomicU64,
}

#[tokio::main]
async fn main() {
    let cfg = Config::from_env();
    let stats = Arc::new(Stats::default());

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Brokkr WS channel load-test (BROKKR-T-0177)          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ broker        : {:<46}║", cfg.broker_url);
    println!("║ agents        : {:<46}║", cfg.agents);
    println!("║ stacks        : {:<46}║", cfg.stacks);
    println!("║ subscribers   : {:<46}║", cfg.subscribers);
    println!("║ msg rate/agent: {:<46}║", format!("{}/s", cfg.msg_rate));
    println!("║ duration      : {:<46}║", format!("{}s", cfg.duration.as_secs()));
    println!("╚══════════════════════════════════════════════════════════════╝");

    let http = reqwest::Client::new();

    // Unique per-run suffix so generator/stack/agent names don't collide with
    // a previous run's rows (names are unique-constrained).
    let run = chrono::Utc::now().format("%H%M%S").to_string();

    // ---- 1. Provision generator + stack pool + agent fleet ---------------
    println!("\n⏳ Provisioning generator + {} stacks...", cfg.stacks);
    let generator_id = create_generator(&http, &cfg, &run).await;
    let stacks = create_stacks(&http, &cfg, &generator_id, &run).await;
    if stacks.is_empty() {
        eprintln!("❌ no stacks created; aborting");
        return;
    }
    println!("   created {} stacks", stacks.len());

    println!("⏳ Provisioning {} agents (this can take a moment)...", cfg.agents);
    let agents = create_agents(&http, &cfg, &run).await;
    println!("   provisioned {} agents", agents.len());
    if agents.is_empty() {
        eprintln!("❌ no agents created; aborting");
        return;
    }

    let ws_base = ws_url(&cfg.broker_url);
    let deadline = Instant::now() + cfg.duration;
    let stacks = Arc::new(stacks);

    // ---- 2. Spawn agent senders ------------------------------------------
    println!("\n🚀 Opening {} agent WS connections...", agents.len());
    let stack_rr = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    for (idx, (id, pak)) in agents.into_iter().enumerate() {
        let url = format!("{ws_base}/internal/ws/agent");
        let stats = stats.clone();
        let stacks = stacks.clone();
        let stack_rr = stack_rr.clone();
        let rate = cfg.msg_rate;
        tasks.push(tokio::spawn(async move {
            agent_loop(url, pak, id, idx, stats, stacks, stack_rr, rate, deadline).await;
        }));
    }

    // ---- 3. Spawn live subscribers ---------------------------------------
    println!("📡 Opening {} live-subscriber connections...", cfg.subscribers);
    for i in 0..cfg.subscribers {
        let stack_id = stacks[i % stacks.len()].clone();
        let url = format!("{ws_base}/api/v1/stacks/{stack_id}/live");
        let pak = cfg.admin_pak.clone();
        let stats = stats.clone();
        tasks.push(tokio::spawn(async move {
            subscriber_loop(url, pak, stats, deadline).await;
        }));
    }

    // ---- 4. Sampler ------------------------------------------------------
    println!("\n📊 Sampling every {}s for {}s...\n", cfg.sample.as_secs(), cfg.duration.as_secs());
    let samples = sample_loop(&http, &cfg, &stats, deadline).await;

    // Let client tasks wind down.
    for t in tasks {
        let _ = t.await;
    }

    print_summary(&cfg, &stats, &samples);
}

/// One synthetic agent: connect, then heartbeat every 5s + telemetry at the
/// target rate until the deadline.
#[allow(clippy::too_many_arguments)]
async fn agent_loop(
    url: String,
    pak: String,
    agent_id: String,
    idx: usize,
    stats: Arc<Stats>,
    stacks: Arc<Vec<String>>,
    stack_rr: Arc<AtomicUsize>,
    msg_rate: u64,
    deadline: Instant,
) {
    let req = match auth_request(&url, &pak) {
        Some(r) => r,
        None => {
            stats.conn_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    let mut socket = match tokio_tungstenite::connect_async(req).await {
        Ok((s, _)) => s,
        Err(_) => {
            stats.conn_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    stats.connected.fetch_add(1, Ordering::Relaxed);

    // Telemetry body agent_id MUST equal the authenticated agent (the broker
    // drops mismatches before persist/broadcast — see handler.rs), and the
    // telemetry tables FK on it. So we use the provisioned agent id, not a
    // synthetic one.
    let agent_id = agent_id.as_str();
    let interval = Duration::from_secs_f64(1.0 / msg_rate.max(1) as f64);
    let mut last_heartbeat = Instant::now();
    let mut tick = 0u64;

    while Instant::now() < deadline {
        // Heartbeat every 5s.
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            let hb = heartbeat_json(agent_id);
            if socket.send(Message::Text(hb)).await.is_err() {
                stats.send_errors.fetch_add(1, Ordering::Relaxed);
                return;
            }
            stats.sent.fetch_add(1, Ordering::Relaxed);
            last_heartbeat = Instant::now();
        }

        // Telemetry — round-robin a stack, alternate event/log so both
        // telemetry tables grow.
        let stack = {
            let n = stack_rr.fetch_add(1, Ordering::Relaxed);
            &stacks[n % stacks.len()]
        };
        let msg = if tick % 2 == 0 {
            k8s_event_json(agent_id, stack, idx, tick)
        } else {
            pod_log_json(agent_id, stack, idx, tick)
        };
        if socket.send(Message::Text(msg)).await.is_err() {
            stats.send_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
        stats.sent.fetch_add(1, Ordering::Relaxed);
        tick += 1;

        tokio::time::sleep(interval).await;
    }
    let _ = socket.close(None).await;
}

/// One live subscriber: drain frames until the deadline, counting receipts.
async fn subscriber_loop(url: String, pak: String, stats: Arc<Stats>, deadline: Instant) {
    let req = match auth_request(&url, &pak) {
        Some(r) => r,
        None => {
            stats.sub_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    let mut socket = match tokio_tungstenite::connect_async(req).await {
        Ok((s, _)) => s,
        Err(_) => {
            stats.sub_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    stats.sub_connected.fetch_add(1, Ordering::Relaxed);

    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_secs(2), socket.next()).await {
            Ok(Some(Ok(_frame))) => {
                stats.sub_received.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Some(Err(_))) | Ok(None) => {
                stats.sub_errors.fetch_add(1, Ordering::Relaxed);
                return;
            }
            Err(_) => {} // idle window; keep going
        }
    }
    let _ = socket.close(None).await;
}

struct Sample {
    at: Instant,
    connected_gauge: Option<f64>,
    cpu_pct: Option<f64>,
    rss_mib: Option<f64>,
    k8s_rows: Option<u64>,
    log_rows: Option<u64>,
}

async fn sample_loop(
    http: &reqwest::Client,
    cfg: &Config,
    stats: &Arc<Stats>,
    deadline: Instant,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    while Instant::now() < deadline {
        tokio::time::sleep(cfg.sample).await;
        let connected_gauge = scrape_gauge(http, cfg, "brokkr_ws_connected_agents").await;
        let (cpu_pct, rss_mib) = docker_stats(&cfg.broker_container);
        let (k8s_rows, log_rows) = pg_counts(&cfg.pg_container);

        let elapsed = cfg.duration.saturating_sub(deadline.saturating_duration_since(Instant::now()));
        println!(
            "  [{:>4}s] connected≈{:<5} sent={:<9} send_err={:<4} sub_rx={:<9} | broker cpu={} rss={} | k8s_rows={} log_rows={}",
            elapsed.as_secs(),
            connected_gauge.map(|v| v as u64).map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
            stats.sent.load(Ordering::Relaxed),
            stats.send_errors.load(Ordering::Relaxed),
            stats.sub_received.load(Ordering::Relaxed),
            cpu_pct.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "?".into()),
            rss_mib.map(|v| format!("{v:.0}MiB")).unwrap_or_else(|| "?".into()),
            k8s_rows.map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
            log_rows.map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
        );

        samples.push(Sample {
            at: Instant::now(),
            connected_gauge,
            cpu_pct,
            rss_mib,
            k8s_rows,
            log_rows,
        });
    }
    samples
}

fn print_summary(cfg: &Config, stats: &Stats, samples: &[Sample]) {
    let sent = stats.sent.load(Ordering::Relaxed);
    let secs = cfg.duration.as_secs().max(1);
    let peak_connected = samples
        .iter()
        .filter_map(|s| s.connected_gauge)
        .fold(0.0_f64, f64::max);
    let cpu_vals: Vec<f64> = samples.iter().filter_map(|s| s.cpu_pct).collect();
    let cpu_peak = cpu_vals.iter().cloned().fold(0.0_f64, f64::max);
    let cpu_avg = if cpu_vals.is_empty() { 0.0 } else { cpu_vals.iter().sum::<f64>() / cpu_vals.len() as f64 };
    let rss_peak = samples.iter().filter_map(|s| s.rss_mib).fold(0.0_f64, f64::max);

    // Postgres write rate from first→last row-count sample.
    let pg_rate = |pick: fn(&Sample) -> Option<u64>| -> Option<f64> {
        let pts: Vec<(&Sample, u64)> = samples.iter().filter_map(|s| pick(s).map(|v| (s, v))).collect();
        if pts.len() < 2 {
            return None;
        }
        let (first_s, first_v) = pts.first().unwrap();
        let (last_s, last_v) = pts.last().unwrap();
        let dt = last_s.at.duration_since(first_s.at).as_secs_f64();
        if dt <= 0.0 {
            return None;
        }
        Some((last_v.saturating_sub(*first_v)) as f64 / dt)
    };
    let k8s_rate = pg_rate(|s| s.k8s_rows);
    let log_rate = pg_rate(|s| s.log_rows);

    println!("\n══════════════════════════════════════════════════════════════════");
    println!("📊 v0.5.0 WS load-test baseline");
    println!("══════════════════════════════════════════════════════════════════");
    println!("  agents requested   : {}", cfg.agents);
    println!("  agents connected   : {} (peak gauge {:.0}, conn errors {})",
        stats.connected.load(Ordering::Relaxed), peak_connected, stats.conn_errors.load(Ordering::Relaxed));
    println!("  subscribers        : {} connected, {} errors, {} frames received",
        stats.sub_connected.load(Ordering::Relaxed), stats.sub_errors.load(Ordering::Relaxed), stats.sub_received.load(Ordering::Relaxed));
    println!("  messages sent      : {} total → {:.0} msg/s achieved",
        sent, sent as f64 / secs as f64);
    println!("  send errors        : {}", stats.send_errors.load(Ordering::Relaxed));
    println!("  broker CPU         : avg {:.0}%  peak {:.0}%", cpu_avg, cpu_peak);
    println!("  broker RSS         : peak {:.0} MiB", rss_peak);
    println!("  pg write rate      : agent_k8s_events {}  agent_pod_logs {}",
        k8s_rate.map(|v| format!("{v:.0} rows/s")).unwrap_or_else(|| "?".into()),
        log_rate.map(|v| format!("{v:.0} rows/s")).unwrap_or_else(|| "?".into()));
    println!("══════════════════════════════════════════════════════════════════");
    println!("\nPaste this block into BROKKR-T-0177 Status Updates as the v0.5.0 baseline.");
}

// ---- provisioning ------------------------------------------------------

async fn create_generator(http: &reqwest::Client, cfg: &Config, run: &str) -> String {
    let resp = http
        .post(format!("{}/api/v1/generators", cfg.broker_url))
        .header("Authorization", format!("Bearer {}", cfg.admin_pak))
        .json(&serde_json::json!({ "name": format!("ws-loadtest-gen-{run}"), "description": "BROKKR-T-0177 load test" }))
        .send()
        .await
        .expect("create generator request");
    let v: serde_json::Value = resp.json().await.expect("generator json");
    v["generator"]["id"].as_str().expect("generator id").to_string()
}

async fn create_stacks(http: &reqwest::Client, cfg: &Config, generator_id: &str, run: &str) -> Vec<String> {
    let mut out = Vec::new();
    for i in 0..cfg.stacks {
        let resp = http
            .post(format!("{}/api/v1/stacks", cfg.broker_url))
            .header("Authorization", format!("Bearer {}", cfg.admin_pak))
            .json(&serde_json::json!({
                "name": format!("ws-loadtest-stack-{run}-{i}"),
                "description": "BROKKR-T-0177",
                "generator_id": generator_id,
            }))
            .send()
            .await;
        if let Ok(r) = resp {
            if let Ok(v) = r.json::<serde_json::Value>().await {
                if let Some(id) = v["id"].as_str() {
                    out.push(id.to_string());
                }
            }
        }
    }
    out
}

async fn create_agents(http: &reqwest::Client, cfg: &Config, run: &str) -> Vec<(String, String)> {
    // Provision concurrently in bounded batches so we don't open thousands of
    // sockets at once but still finish 500 creates quickly.
    use futures_util::stream::{self, StreamExt};
    let results = stream::iter(0..cfg.agents)
        .map(|i| {
            let http = http.clone();
            let url = format!("{}/api/v1/agents", cfg.broker_url);
            let auth = format!("Bearer {}", cfg.admin_pak);
            async move {
                let resp = http
                    .post(url)
                    .header("Authorization", auth)
                    .json(&serde_json::json!({
                        "name": format!("ws-loadtest-agent-{run}-{i}"),
                        "cluster_name": "ws-loadtest",
                    }))
                    .send()
                    .await
                    .ok()?;
                let v: serde_json::Value = resp.json().await.ok()?;
                let id = v["agent"]["id"].as_str()?.to_string();
                let pak = v["initial_pak"].as_str()?.to_string();
                Some((id, pak))
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<_>>()
        .await;
    results.into_iter().flatten().collect()
}

// ---- metrics / docker / postgres scraping ------------------------------

async fn scrape_gauge(http: &reqwest::Client, cfg: &Config, name: &str) -> Option<f64> {
    let body = http
        .get(format!("{}/metrics", cfg.broker_url))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    for line in body.lines() {
        if line.starts_with('#') {
            continue;
        }
        // gauge has no labels: "brokkr_ws_connected_agents 42"
        if let Some(rest) = line.strip_prefix(name) {
            let rest = rest.trim();
            if let Ok(v) = rest.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// `docker stats --no-stream` for one container → (cpu%, rss MiB).
fn docker_stats(container: &str) -> (Option<f64>, Option<f64>) {
    let out = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.CPUPerc}}|{{.MemUsage}}",
            container,
        ])
        .output();
    let Ok(out) = out else { return (None, None) };
    if !out.status.success() {
        return (None, None);
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let line = s.trim();
    let mut parts = line.split('|');
    let cpu = parts
        .next()
        .and_then(|c| c.trim().trim_end_matches('%').parse::<f64>().ok());
    // MemUsage looks like "123.4MiB / 7.6GiB"; take the used side and convert to MiB.
    let rss = parts.next().and_then(|m| {
        let used = m.split('/').next()?.trim();
        parse_mem_mib(used)
    });
    (cpu, rss)
}

fn parse_mem_mib(s: &str) -> Option<f64> {
    let s = s.trim();
    let (num, unit): (String, String) = s.chars().partition(|c| c.is_ascii_digit() || *c == '.');
    let v: f64 = num.parse().ok()?;
    Some(match unit.trim() {
        "GiB" => v * 1024.0,
        "MiB" => v,
        "KiB" => v / 1024.0,
        "B" => v / (1024.0 * 1024.0),
        "GB" => v * 953.674,
        "MB" => v * 0.953674,
        "kB" => v / 1048.576,
        _ => v,
    })
}

/// Two `select count(*)` via `docker exec ... psql` → (k8s_events, pod_logs).
fn pg_counts(container: &str) -> (Option<u64>, Option<u64>) {
    let out = Command::new("docker")
        .args([
            "exec",
            container,
            "psql",
            "-U",
            "brokkr",
            "-d",
            "brokkr",
            "-tA",
            "-c",
            "select count(*) from agent_k8s_events; select count(*) from agent_pod_logs;",
        ])
        .output();
    let Ok(out) = out else { return (None, None) };
    if !out.status.success() {
        return (None, None);
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let nums: Vec<u64> = s
        .lines()
        .filter_map(|l| l.trim().parse::<u64>().ok())
        .collect();
    (nums.first().copied(), nums.get(1).copied())
}

// ---- wire-format helpers (mirror brokkr_wire::WsMessage) ---------------

fn auth_request(
    url: &str,
    pak: &str,
) -> Option<tokio_tungstenite::tungstenite::handshake::client::Request> {
    let mut req = url.into_client_request().ok()?;
    req.headers_mut()
        .insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {pak}")).ok()?);
    Some(req)
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn heartbeat_json(agent_id: &str) -> String {
    serde_json::json!({
        "type": "heartbeat",
        "body": { "agent_id": agent_id, "sent_at": now_rfc3339() }
    })
    .to_string()
}

fn k8s_event_json(agent_id: &str, stack_id: &str, idx: usize, tick: u64) -> String {
    serde_json::json!({
        "type": "k8s_event",
        "body": {
            "agent_id": agent_id,
            "stack_id": stack_id,
            "observed_at": now_rfc3339(),
            "reason": "LoadTest",
            "message": format!("synthetic event a{idx} t{tick}"),
            "event_type": "Normal",
            "source": "ws-loadtest",
            "involved_object": {
                "api_version": "v1",
                "kind": "Pod",
                "namespace": "loadtest",
                "name": format!("lt-pod-{idx}"),
                "uid": null
            }
        }
    })
    .to_string()
}

fn pod_log_json(agent_id: &str, stack_id: &str, idx: usize, tick: u64) -> String {
    serde_json::json!({
        "type": "pod_log_line",
        "body": {
            "agent_id": agent_id,
            "stack_id": stack_id,
            "namespace": "loadtest",
            "pod": format!("lt-pod-{idx}"),
            "container": "c",
            "ts": now_rfc3339(),
            "line": format!("synthetic log a{idx} t{tick}")
        }
    })
    .to_string()
}

fn ws_url(broker_url: &str) -> String {
    if let Some(rest) = broker_url.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = broker_url.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        broker_url.to_string()
    }
}
