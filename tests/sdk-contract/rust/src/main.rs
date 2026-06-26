/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Brokkr Rust SDK Contract Test Suite
//!
//! Exercises the generated `brokkr-client` Rust SDK against a running broker.
//! Mirrors the UAT walkthrough using ONLY the SDK's typed API surface — both
//! the ergonomic [`brokkr_client::BrokkrClient`] wrapper and the raw progenitor
//! [`brokkr_client::Client`]. Hand-rolled HTTP is intentionally absent: if
//! the SDK cannot express a step, the test fails by design.
//!
//! This complements `tests/e2e/` (which uses raw reqwest) by surfacing
//! consumer-visible drift that bypasses the spec-drift CI gate.
//!
//! Run with: `angreal tests sdk-contract rust`

use std::env;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use brokkr_client::types::{
    CreateAgentRequest, CreateAgentResponse, CreateDeploymentObjectRequest, ErrorResponse,
    NewAgentTarget, NewGenerator, NewStack, NewStackAnnotation,
};
use brokkr_client::{ApplyOutcome, BrokkrClient, BrokkrError, Client as RawClient};
use uuid::Uuid;

/// Convert a progenitor `Error<ErrorResponse>` into our typed [`BrokkrError`].
/// All scenario calls go through this so we can match on `.code()`.
fn berr(e: progenitor_client::Error<ErrorResponse>) -> BrokkrError {
    BrokkrError::from(e)
}

const DEMO_YAML: &str = r#"apiVersion: v1
kind: Namespace
metadata:
  name: sdk-contract-rust-ns
  labels:
    app: sdk-contract-rust
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: sdk-contract-rust-config
  namespace: sdk-contract-rust-ns
data:
  KEY: "value"
"#;

#[tokio::main]
async fn main() -> ExitCode {
    let broker_url = env::var("BROKER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let admin_pak = env::var("ADMIN_PAK")
        .unwrap_or_else(|_| "brokkr_BR3rVsDa_GK3QN7CDUzYc6iKgMkJ98M2WSimM5t6U8".to_string());

    let base_url = format!("{}/api/v1", broker_url.trim_end_matches('/'));

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Brokkr Rust SDK Contract Test Suite                    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Broker base: {:<49} ║", &base_url);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    println!("⏳ Waiting for broker to be ready...");
    if let Err(e) = wait_for_ready(&broker_url, 30).await {
        eprintln!("❌ Broker not ready: {e}");
        return ExitCode::FAILURE;
    }
    println!("✅ Broker is ready\n");

    let mut passed = 0u32;
    let mut failed = 0u32;

    macro_rules! run {
        ($name:expr, $fut:expr) => {{
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🧪 {}", $name);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            match $fut.await {
                Ok(()) => {
                    println!("✅ {} PASSED\n", $name);
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ {} FAILED: {:#}\n", $name, e);
                    failed += 1;
                }
            }
        }};
    }

    run!(
        "UAT walkthrough via generator PAK",
        scenario_uat_walkthrough(&base_url, &admin_pak)
    );
    run!(
        "Negative path: generator targets a stack it does not own",
        scenario_target_mismatch(&base_url, &admin_pak)
    );
    run!(
        "Raw progenitor Client surface compiles & accepts builders",
        scenario_raw_progenitor_surface(&base_url, &admin_pak)
    );
    run!(
        "WS-10/13: telemetry history + ws connections via wrapper",
        scenario_telemetry_and_ws_diagnostics(&base_url, &admin_pak)
    );
    run!(
        "I-0021: submit_manifests + idempotent apply (folder helpers)",
        scenario_manifest_apply(&base_url, &admin_pak)
    );

    println!("══════════════════════════════════════════════════════════════════");
    println!("📊 Results: {} passed, {} failed", passed, failed);
    println!("══════════════════════════════════════════════════════════════════");

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn wait_for_ready(broker_url: &str, timeout_secs: u64) -> Result<()> {
    let http = reqwest::Client::new();
    let start = std::time::Instant::now();
    loop {
        let r = http
            .get(format!("{}/healthz", broker_url.trim_end_matches('/')))
            .send()
            .await;
        if let Ok(resp) = r {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        if start.elapsed() > Duration::from_secs(timeout_secs) {
            return Err(anyhow!("broker /healthz never returned 2xx"));
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Build a [`BrokkrClient`] for a given PAK.
fn client(base_url: &str, pak: &str) -> Result<BrokkrClient> {
    BrokkrClient::builder(base_url)
        .token(pak)
        .build()
        .map_err(|e| anyhow!("failed to build client: {e}"))
}

/// Suffix used to keep names unique across reruns.
fn unique(prefix: &str) -> String {
    let suffix = Uuid::new_v4().simple().to_string();
    format!("{}-{}", prefix, &suffix[..8])
}

/// Full UAT walkthrough using a generator PAK after admin bootstrap.
async fn scenario_uat_walkthrough(base_url: &str, admin_pak: &str) -> Result<()> {
    let admin = client(base_url, admin_pak)?;

    // Step 1: admin creates generator → capture generator PAK
    let gen_name = unique("sdk-contract-rust-gen");
    println!("  → [admin] create generator '{}'", gen_name);
    let gen_resp = admin
        .api()
        .create_generator()
        .body(
            NewGenerator::builder()
                .name(gen_name.clone())
                .description(Some("rust sdk contract".to_string())),
        )
        .send()
        .await
        .map_err(berr)
        .context("create_generator")?
        .into_inner();
    let generator_id = gen_resp.generator.id;
    let generator_pak = gen_resp.pak.clone();
    println!("    generator_id={} pak=…{}", generator_id, last4(&generator_pak));

    // Step 2: admin creates agent → capture agent id + agent PAK
    let agent_name = unique("sdk-contract-rust-agent");
    println!("  → [admin] create agent '{}'", agent_name);
    let agent_resp: CreateAgentResponse = admin
        .api()
        .create_agent()
        .body(
            CreateAgentRequest::builder()
                .name(agent_name.clone())
                .cluster_name("sdk-contract-rust-cluster".to_string()),
        )
        .send()
        .await
        .map_err(berr)
        .context("create_agent")?
        .into_inner();
    let agent_id: Uuid = agent_resp.agent.id;
    if agent_resp.initial_pak.is_empty() {
        return Err(anyhow!("create_agent returned empty initial_pak"));
    }
    println!("    agent_id={}", agent_id);

    // Build a generator-PAK client for the next steps.
    let gen = client(base_url, &generator_pak)?;

    // Step 3: generator creates a stack
    let stack_name = unique("sdk-contract-rust-stack");
    println!("  → [generator] create stack '{}'", stack_name);
    let stack = gen
        .api()
        .create_stack()
        .body(
            NewStack::builder()
                .name(stack_name.clone())
                .generator_id(generator_id)
                .description(Some("rust sdk contract".to_string())),
        )
        .send()
        .await
        .map_err(berr)
        .context("create_stack")?
        .into_inner();
    let stack_id = stack.id;
    println!("    stack_id={}", stack_id);

    // Step 4: generator adds stack label (BROKKR-T-0152 — application/json JSON-string body)
    println!("  → [generator] add stack label 'contract-test'");
    let label = gen
        .api()
        .stacks_add_label()
        .id(stack_id)
        .body("contract-test")
        .send()
        .await
        .map_err(berr)
        .context("stacks_add_label")?
        .into_inner();
    println!("    label_id={} label='{}'", label.id, label.label);

    // Step 5: generator adds stack annotation
    println!("  → [generator] add stack annotation purpose=sdk-contract");
    let ann = gen
        .api()
        .stacks_add_annotation()
        .id(stack_id)
        .body(
            NewStackAnnotation::builder()
                .stack_id(stack_id)
                .key("purpose".to_string())
                .value("sdk-contract".to_string()),
        )
        .send()
        .await
        .map_err(berr)
        .context("stacks_add_annotation")?
        .into_inner();
    println!("    annotation_id={}", ann.id);

    // Step 6: generator creates a deployment object on the stack
    println!("  → [generator] create deployment object");
    let dep = gen
        .api()
        .create_deployment_object()
        .id(stack_id)
        .body(
            CreateDeploymentObjectRequest::builder()
                .yaml_content(DEMO_YAML.to_string())
                .is_deletion_marker(false),
        )
        .send()
        .await
        .map_err(berr)
        .context("create_deployment_object")?
        .into_inner();
    let deployment_id = dep.id;
    println!("    deployment_id={}", deployment_id);

    // Step 6.5: register agent with generator before targeting
    println!("  → [admin] register agent with generator");
    admin
        .api()
        .register_agent()
        .id(generator_id)
        .body(brokkr_client::types::AgentRegistrationBody::builder().agent_id(agent_id))
        .send()
        .await
        .map_err(berr)
        .context("register_agent")?;

    // Step 7: generator targets the stack to the agent (BROKKR-T-0153 — generator PAK allowed for own stacks)
    println!("  → [generator] add agent target");
    let target = gen
        .api()
        .add_target()
        .id(agent_id)
        .body(
            NewAgentTarget::builder()
                .agent_id(agent_id)
                .stack_id(stack_id),
        )
        .send()
        .await
        .map_err(berr)
        .context("add_target")?
        .into_inner();
    println!("    target_id={}", target.id);

    // Step 7.5: list_stacks as the generator (BROKKR-T-0155 — must filter to own).
    println!("  → [generator] list_stacks (expect filtered to own)");
    let listed = gen
        .api()
        .list_stacks()
        .send()
        .await
        .map_err(berr)
        .context("list_stacks")?
        .into_inner();
    if !listed.iter().any(|s| s.id == stack_id) {
        return Err(anyhow!(
            "list_stacks (as generator) did not include this generator's stack {stack_id}; got {} stack(s)",
            listed.len()
        ));
    }
    if listed.iter().any(|s| s.generator_id != generator_id) {
        return Err(anyhow!(
            "list_stacks (as generator) leaked stacks from another generator"
        ));
    }
    println!("    saw {} stack(s), all owned by this generator", listed.len());

    // Step 8: GET the stack and verify shape
    println!("  → [generator] get stack");
    let fetched = gen
        .api()
        .get_stack()
        .id(stack_id)
        .send()
        .await
        .map_err(berr)
        .context("get_stack")?
        .into_inner();
    if fetched.id != stack_id {
        return Err(anyhow!(
            "stack id mismatch: requested {stack_id} got {}",
            fetched.id
        ));
    }
    if fetched.name != stack_name {
        return Err(anyhow!(
            "stack name mismatch: expected {stack_name} got {}",
            fetched.name
        ));
    }
    if fetched.generator_id != generator_id {
        return Err(anyhow!("stack generator_id mismatch"));
    }
    println!("    stack name+id+generator_id verified");

    // Cleanup (best-effort).
    println!("  → cleanup (best-effort)");
    let _ = admin.api().remove_target().id(agent_id).stack_id(stack_id).send().await;
    let _ = admin.api().delete_stack().id(stack_id).send().await;
    let _ = admin.api().delete_agent().id(agent_id).send().await;
    let _ = admin.api().delete_generator().id(generator_id).send().await;

    Ok(())
}

/// A generator must not be able to target a stack it does not own — the
/// broker should return a 403 with `code = "target_generator_mismatch"`.
async fn scenario_target_mismatch(base_url: &str, admin_pak: &str) -> Result<()> {
    let admin = client(base_url, admin_pak)?;

    // Two generators, each with its own stack.
    let gen_a_name = unique("sdk-contract-rust-gen-a");
    let gen_a = admin
        .api()
        .create_generator()
        .body(NewGenerator::builder().name(gen_a_name.clone()))
        .send()
        .await
        .map_err(berr)?
        .into_inner();
    let gen_b_name = unique("sdk-contract-rust-gen-b");
    let gen_b = admin
        .api()
        .create_generator()
        .body(NewGenerator::builder().name(gen_b_name.clone()))
        .send()
        .await
        .map_err(berr)?
        .into_inner();
    let gen_a_client = client(base_url, &gen_a.pak)?;

    // Stack B is owned by generator B.
    let stack_b = admin
        .api()
        .create_stack()
        .body(
            NewStack::builder()
                .name(unique("sdk-contract-rust-stack-b"))
                .generator_id(gen_b.generator.id),
        )
        .send()
        .await
        .map_err(berr)?
        .into_inner();

    // Agent (admin-created).
    let agent_resp: CreateAgentResponse = admin
        .api()
        .create_agent()
        .body(
            CreateAgentRequest::builder()
                .name(unique("sdk-contract-rust-agent-x"))
                .cluster_name("sdk-contract-rust-cluster".to_string()),
        )
        .send()
        .await
        .map_err(berr)?
        .into_inner();
    let agent_id: Uuid = agent_resp.agent.id;

    // Register agent with Gen A so the mismatch check fires (not the registration check).
    admin
        .api()
        .register_agent()
        .id(gen_a.generator.id)
        .body(brokkr_client::types::AgentRegistrationBody::builder().agent_id(agent_id))
        .send()
        .await
        .map_err(berr)
        .context("register_agent for mismatch test")?;

    // Generator A tries to target Generator B's stack.
    println!("  → [generator A] add target for stack owned by generator B (expect 403)");
    let result = gen_a_client
        .api()
        .add_target()
        .id(agent_id)
        .body(
            NewAgentTarget::builder()
                .agent_id(agent_id)
                .stack_id(stack_b.id),
        )
        .send()
        .await;

    let err = match result {
        Ok(_) => {
            // Cleanup, then fail.
            let _ = admin.api().remove_target().id(agent_id).stack_id(stack_b.id).send().await;
            let _ = admin.api().delete_stack().id(stack_b.id).send().await;
            let _ = admin.api().delete_agent().id(agent_id).send().await;
            let _ = admin.api().delete_generator().id(gen_a.generator.id).send().await;
            let _ = admin.api().delete_generator().id(gen_b.generator.id).send().await;
            return Err(anyhow!(
                "expected 403 target_generator_mismatch but call succeeded"
            ));
        }
        Err(e) => BrokkrError::from(e),
    };

    let status = err.status();
    let code = err.code();
    println!("    got status={:?} code={:?}", status, code);

    if status != Some(reqwest::StatusCode::FORBIDDEN) {
        return Err(anyhow!("expected 403 FORBIDDEN, got {status:?}"));
    }
    if code != Some("target_generator_mismatch") {
        return Err(anyhow!(
            "expected ErrorResponse.code='target_generator_mismatch', got {code:?}"
        ));
    }
    println!("    ✓ typed ErrorResponse with code='target_generator_mismatch'");

    // Cleanup (best-effort).
    let _ = admin.api().delete_stack().id(stack_b.id).send().await;
    let _ = admin.api().delete_agent().id(agent_id).send().await;
    let _ = admin.api().delete_generator().id(gen_a.generator.id).send().await;
    let _ = admin.api().delete_generator().id(gen_b.generator.id).send().await;

    Ok(())
}

/// Smoke-check the raw progenitor [`brokkr_client::Client`] surface. This
/// catches generated-API regressions even when the ergonomic wrapper would
/// have papered them over.
async fn scenario_raw_progenitor_surface(base_url: &str, admin_pak: &str) -> Result<()> {
    let http = reqwest::Client::builder()
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(admin_pak)?,
            );
            h
        })
        .build()?;
    let raw = RawClient::new_with_client(base_url, http);

    println!("  → [raw] list_agents");
    let agents = raw.list_agents().send().await.map_err(berr)?.into_inner();
    println!("    agents count = {}", agents.len());

    println!("  → [raw] list_stacks");
    let stacks = raw.list_stacks().send().await.map_err(berr)?.into_inner();
    println!("    stacks count = {}", stacks.len());

    println!("  → [raw] list_generators");
    let generators = raw
        .list_generators()
        .send()
        .await
        .map_err(berr)?
        .into_inner();
    println!("    generators count = {}", generators.len());

    Ok(())
}

/// WS-10 + WS-13 surface: ergonomic-wrapper methods for the telemetry
/// history endpoints and the admin ws/connections snapshot. We don't
/// generate telemetry rows here — that needs a running agent + kube
/// cluster — so we assert the response *shape* (retention metadata
/// present and within spec, empty `events`/`lines` arrays accepted).
async fn scenario_telemetry_and_ws_diagnostics(base_url: &str, admin_pak: &str) -> Result<()> {
    let admin = client(base_url, admin_pak)?;

    // Need a stack id for the history endpoints. Create one through
    // the existing UAT-style bootstrap path (admin → generator → stack).
    let gen_name = unique("sdk-contract-rust-tel-gen");
    let gen_resp = admin
        .api()
        .create_generator()
        .body(NewGenerator::builder().name(gen_name).description(None))
        .send()
        .await
        .map_err(berr)
        .context("create_generator")?
        .into_inner();
    let generator_id = gen_resp.generator.id;
    let generator_pak = gen_resp.pak.clone();

    let stack_name = unique("sdk-contract-rust-tel-stack");
    let gen_client = client(base_url, &generator_pak)?;
    let stack = gen_client
        .api()
        .create_stack()
        .body(
            NewStack::builder()
                .name(stack_name)
                .generator_id(generator_id)
                .description(None),
        )
        .send()
        .await
        .map_err(berr)
        .context("create_stack")?
        .into_inner();
    println!("  → seeded stack {} for telemetry queries", stack.id);

    // 1. list_telemetry_events
    println!("  → list_telemetry_events(stack_id=..., since=None, limit=Some(10))");
    let events = admin
        .list_telemetry_events(stack.id, None, Some(10))
        .await
        .context("list_telemetry_events via wrapper")?;
    if events.retention.retention_ceiling_seconds != 21600 {
        return Err(anyhow!(
            "expected retention_ceiling_seconds=21600 (6h), got {}",
            events.retention.retention_ceiling_seconds
        ));
    }
    if !events.retention.long_term_sink_hint.contains("Datadog") {
        return Err(anyhow!(
            "expected long_term_sink_hint to mention Datadog, got {:?}",
            events.retention.long_term_sink_hint
        ));
    }
    println!("    retention_ceiling_seconds=21600 ✓ ; long_term_sink_hint mentions Datadog ✓");

    // 2. list_telemetry_logs
    println!("  → list_telemetry_logs(stack_id=..., since=None, limit=None)");
    let logs = admin
        .list_telemetry_logs(stack.id, None, None)
        .await
        .context("list_telemetry_logs via wrapper")?;
    if logs.retention.retention_ceiling_seconds != 21600 {
        return Err(anyhow!(
            "expected retention_ceiling_seconds=21600 on /logs, got {}",
            logs.retention.retention_ceiling_seconds
        ));
    }
    println!("    /logs retention_ceiling_seconds=21600 ✓");

    // 3. list_ws_connections (admin-only)
    println!("  → list_ws_connections()");
    let snapshot = admin
        .list_ws_connections()
        .await
        .context("list_ws_connections via wrapper")?;
    // The contract harness doesn't bring up an agent; expect 0 connected
    // agents and 0 live subscribers, but the response *shape* is the
    // proof under test.
    println!(
        "    connected_agents={} live_subscribers={}",
        snapshot.connected_agents, snapshot.live_subscribers
    );

    Ok(())
}

fn last4(s: &str) -> String {
    let len = s.len();
    if len <= 4 {
        s.to_string()
    } else {
        s[len - 4..].to_string()
    }
}


/// BROKKR-T-0195: the manifest folder helpers — `submit_manifests` on an
/// existing stack, and idempotent `apply` (create → unchanged → updated).
async fn scenario_manifest_apply(base_url: &str, admin_pak: &str) -> Result<()> {
    let admin = client(base_url, admin_pak)?;

    // admin creates a generator → generator PAK (apply needs a generator).
    let gen_name = unique("sdk-contract-rust-apply-gen");
    let gen_resp = admin
        .api()
        .create_generator()
        .body(NewGenerator::builder().name(gen_name).description(Some("apply contract".to_string())))
        .send()
        .await
        .map_err(berr)
        .context("create_generator")?
        .into_inner();
    let generator_id = gen_resp.generator.id;
    let gen = client(base_url, &gen_resp.pak)?;

    // A temp folder of manifests, intentionally unsorted on disk.
    let dir = tempfile::tempdir().context("tempdir")?;
    std::fs::write(
        dir.path().join("02-cm.yaml"),
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: apply-cm\n",
    )?;
    std::fs::write(
        dir.path().join("01-ns.yaml"),
        "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: apply-ns\n",
    )?;

    let stack_name = unique("sdk-contract-rust-apply-stack");

    // First apply → Created, stack auto-created, label set.
    println!("  → [generator] apply folder (expect Created)");
    match gen.apply(&stack_name, dir.path(), &["env:contract".to_string()]).await? {
        ApplyOutcome::Created(obj) => {
            if obj.is_deletion_marker {
                return Err(anyhow!("first apply should not be a deletion marker"));
            }
        }
        other => return Err(anyhow!("expected Created, got {other:?}")),
    }

    // Second apply, unchanged folder → Unchanged (no new revision).
    println!("  → [generator] apply same folder (expect Unchanged)");
    match gen.apply(&stack_name, dir.path(), &["env:contract".to_string()]).await? {
        ApplyOutcome::Unchanged => {}
        other => return Err(anyhow!("expected Unchanged, got {other:?}")),
    }

    // Mutate the folder → Updated.
    std::fs::write(
        dir.path().join("03-svc.yaml"),
        "apiVersion: v1\nkind: Service\nmetadata:\n  name: apply-svc\nspec:\n  selector:\n    app: x\n  ports:\n  - port: 80\n",
    )?;
    println!("  → [generator] apply changed folder (expect Updated)");
    match gen.apply(&stack_name, dir.path(), &["env:contract".to_string()]).await? {
        ApplyOutcome::Updated(_) => {}
        other => return Err(anyhow!("expected Updated, got {other:?}")),
    }

    // The stack now exists and carries the targeting label.
    let stacks = gen.api().list_stacks().send().await.map_err(berr)?.into_inner();
    let stack = stacks
        .iter()
        .find(|s| s.name == stack_name)
        .ok_or_else(|| anyhow!("apply did not create the named stack"))?;
    let labels = gen
        .api()
        .stacks_list_labels()
        .id(stack.id)
        .send()
        .await
        .map_err(berr)?
        .into_inner();
    if !labels.iter().any(|l| l.label == "env:contract") {
        return Err(anyhow!("targeting label was not applied"));
    }

    // submit_manifests against the existing stack id returns a new object.
    println!("  → [generator] submit_manifests on stack id");
    let obj = gen.submit_manifests(stack.id, dir.path()).await?;
    if obj.stack_id != stack.id {
        return Err(anyhow!("submit_manifests returned object for the wrong stack"));
    }

    Ok(())
}
