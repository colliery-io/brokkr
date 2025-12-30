/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Brokkr End-to-End Test Suite
//!
//! Holistic tests that exercise the entire Brokkr system as a user would.
//! These tests mirror the UI walkthrough in examples/ui-slim/DEMO_WALKTHROUGH.md
//!
//! Prerequisites:
//! - Full Brokkr stack running (broker, agent, postgres, k3s)
//! - Admin PAK available via ADMIN_PAK environment variable
//!
//! Run with: angreal tests e2e

mod api;
mod scenarios;

use std::env;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let broker_url = env::var("BROKER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let admin_pak = env::var("ADMIN_PAK").unwrap_or_else(|_| {
        "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8".to_string()
    });
    // Echo server URL for webhook delivery testing (in docker-compose: webhook-echo:8080)
    let echo_server_url = env::var("WEBHOOK_ECHO_URL").ok();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Brokkr End-to-End Test Suite                       ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Broker URL: {:<50} ║", &broker_url);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let client = api::Client::new(&broker_url, &admin_pak);

    // Wait for broker to be ready
    println!("⏳ Waiting for broker to be ready...");
    if let Err(e) = client.wait_for_ready(30).await {
        eprintln!("❌ Broker not ready: {}", e);
        return ExitCode::FAILURE;
    }
    println!("✅ Broker is ready\n");

    let mut passed = 0;
    let mut failed = 0;

    // Macro to run a scenario and track results
    macro_rules! run_scenario {
        ($name:expr, $scenario:expr) => {{
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🧪 {}", $name);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            match $scenario.await {
                Ok(()) => {
                    println!("✅ {} PASSED\n", $name);
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ {} FAILED: {}\n", $name, e);
                    failed += 1;
                }
            }
        }};
    }

    run_scenario!("Part 1: Agent Management", scenarios::test_agent_management(&client));
    run_scenario!("Part 2: Stack Creation & Deployment", scenarios::test_stack_deployment(&client));
    run_scenario!("Part 3: Agent Targeting (Labels & Explicit)", scenarios::test_targeting(&client));
    run_scenario!("Part 4: Templates", scenarios::test_templates(&client));
    run_scenario!("Part 5: Work Orders", scenarios::test_work_orders(&client));
    run_scenario!("Part 6: Health & Diagnostics", scenarios::test_health_diagnostics(&client));
    run_scenario!("Part 7: Webhooks", scenarios::test_webhooks(&client, echo_server_url.as_deref()));
    run_scenario!("Part 8: Audit Logs", scenarios::test_audit_logs(&client));
    run_scenario!("Part 9: Metrics & Observability", scenarios::test_metrics(&client));

    // Summary
    println!("══════════════════════════════════════════════════════════════════");
    println!("📊 Results: {} passed, {} failed", passed, failed);
    println!("══════════════════════════════════════════════════════════════════");

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
