/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration coverage for BROKKR-T-0303: a custom work order containing a
//! `batch/v1` Job must be reported against the Job's terminal state, not
//! against the success of the apply.
//!
//! These run against the live k3s cluster the integration harness provides
//! (same kubeconfig the deployment-health suite uses), because the behaviour
//! under test is precisely what the Job controller does over time — pods being
//! scheduled, exiting, and the controller writing `Complete`/`Failed`. The
//! pure interpretation of a `JobStatus` and the derivation of the watch budget
//! from `claim_timeout_seconds` are unit-tested in
//! `src/work_orders/mod.rs`; these tests exist to prove the loop is wired to a
//! real API server correctly.

use brokkr_agent::k8s::api::create_k8s_client;
use brokkr_agent::work_orders::{WorkOrderOutcomeError, execute_custom_work_order};
use brokkr_models::models::agents::Agent;
use brokkr_models::models::work_orders::WorkOrder;
use chrono::Utc;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DynamicObject, Patch, PatchParams};
use kube::{Api, Client as K8sClient};
use std::time::{Duration, Instant};
use uuid::Uuid;

async fn setup() -> K8sClient {
    let client = create_k8s_client(Some("/tmp/brokkr-keys/kubeconfig.yaml"))
        .await
        .unwrap();

    let ns_api = Api::<Namespace>::all(client.clone());
    ns_api
        .list(&Default::default())
        .await
        .expect("Failed to connect to Kubernetes cluster - could not list namespaces");

    client
}

async fn setup_namespace(client: &K8sClient, namespace: &str) {
    let ns_api = Api::<Namespace>::all(client.clone());
    let namespace_obj = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": { "name": namespace }
    });
    let ns: DynamicObject = serde_json::from_value(namespace_obj).unwrap();
    ns_api
        .patch(
            namespace,
            &PatchParams::apply("brokkr-agent-test"),
            &Patch::Apply(serde_json::to_value(&ns).unwrap()),
        )
        .await
        .expect("Failed to create test namespace");
}

async fn cleanup(client: &K8sClient, namespace: &str) {
    let ns_api = Api::<Namespace>::all(client.clone());
    let _ = ns_api.delete(namespace, &Default::default()).await;
}

fn test_agent() -> Agent {
    Agent {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        name: "work-order-job-watch-test".to_string(),
        cluster_name: "test-cluster".to_string(),
        last_heartbeat: None,
        status: "ACTIVE".to_string(),
        pak_hash: String::new(),
        k8s_reachable: Some(true),
        k8s_api_latency_ms: None,
        k8s_reported_at: None,
    }
}

fn custom_work_order(yaml_content: &str) -> WorkOrder {
    WorkOrder {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        work_type: "custom".to_string(),
        yaml_content: yaml_content.to_string(),
        status: "CLAIMED".to_string(),
        claimed_by: None,
        claimed_at: Some(Utc::now()),
        claim_timeout_seconds: 3600,
        max_retries: 3,
        retry_count: 0,
        backoff_seconds: 60,
        next_retry_after: None,
        last_error: None,
        last_error_at: None,
    }
}

/// A Job whose single pod runs `command` and never restarts, so the Job
/// controller reaches a terminal condition on the first attempt.
fn job_yaml(namespace: &str, name: &str, command: &str) -> String {
    format!(
        r#"apiVersion: batch/v1
kind: Job
metadata:
  name: {name}
  namespace: {namespace}
spec:
  backoffLimit: 0
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: work
        image: busybox
        command: ["sh", "-c", "{command}"]
"#
    )
}

/// A Job that exits 0 must be reported as a success, and only after the Job
/// controller has actually said `Complete` — not at apply time.
#[tokio::test]
async fn test_custom_work_order_succeeding_job_is_reported_successful() {
    let namespace = format!("test-wo-job-ok-{}", Uuid::new_v4());
    let client = setup().await;
    setup_namespace(&client, &namespace).await;

    // A ConfigMap rides along so the apply-only fallback for non-Job kinds is
    // exercised on the success path too.
    let yaml = format!(
        "{}---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: alongside\n  namespace: {}\ndata:\n  key: value\n",
        job_yaml(&namespace, "migration-ok", "echo done; exit 0"),
        namespace
    );

    let work_order = custom_work_order(&yaml);
    let deadline = Instant::now() + Duration::from_secs(180);
    let result = execute_custom_work_order(&client, &test_agent(), &work_order, deadline).await;

    let message = result
        .expect("a Job that exits 0 must be reported as a success")
        .expect("the custom work order must report a result message");

    assert!(
        message.contains("completed successfully"),
        "message should state the Job completed: {message}"
    );
    assert!(
        message.contains(&format!("{namespace}/migration-ok")),
        "message should name the watched Job: {message}"
    );
    assert!(
        message.contains("ConfigMap") && message.contains("not monitored"),
        "non-Job kinds must be reported as applied only: {message}"
    );

    cleanup(&client, &namespace).await;
}

/// A Job whose pod exits non-zero is the regression this ticket exists for:
/// before the watch it was recorded as a success at apply time.
#[tokio::test]
async fn test_custom_work_order_failing_job_is_reported_failed() {
    let namespace = format!("test-wo-job-fail-{}", Uuid::new_v4());
    let client = setup().await;
    setup_namespace(&client, &namespace).await;

    let yaml = job_yaml(&namespace, "migration-fail", "echo boom; exit 1");
    let work_order = custom_work_order(&yaml);
    let deadline = Instant::now() + Duration::from_secs(180);
    let result = execute_custom_work_order(&client, &test_agent(), &work_order, deadline).await;

    let error = result.expect_err("a Job that exits non-zero must not be reported as a success");
    let message = error.to_string();

    assert!(
        message.contains("ran and failed"),
        "a failed Job must be reported as having run and failed: {message}"
    );
    assert!(
        !message.contains("did not finish within"),
        "a failed Job must not be conflated with a watch timeout: {message}"
    );
    assert!(
        message.contains(&format!("{namespace}/migration-fail")),
        "message should name the failed Job: {message}"
    );
    assert_eq!(
        error
            .downcast_ref::<WorkOrderOutcomeError>()
            .map(|o| o.is_retryable()),
        Some(false),
        "a Job that ran and failed must not be re-dispatched"
    );

    cleanup(&client, &namespace).await;
}

/// A Job still running when the watch budget expires must be reported
/// distinguishably from a Job that ran and failed, and must not be marked
/// retryable — a retry would re-dispatch the order while this Job is still
/// running in the cluster.
#[tokio::test]
async fn test_custom_work_order_unfinished_job_is_reported_as_timeout_not_failure() {
    let namespace = format!("test-wo-job-timeout-{}", Uuid::new_v4());
    let client = setup().await;
    setup_namespace(&client, &namespace).await;

    // Outlives the deadline by a wide margin, so the watch is guaranteed to
    // give up while the Job is genuinely still running.
    let yaml = job_yaml(&namespace, "migration-slow", "sleep 600");
    let work_order = custom_work_order(&yaml);
    let deadline = Instant::now() + Duration::from_secs(15);
    let result = execute_custom_work_order(&client, &test_agent(), &work_order, deadline).await;

    let error = result.expect_err("an unfinished Job must not be reported as a success");
    let message = error.to_string();

    assert!(
        message.contains("did not finish within the watch window"),
        "an unfinished Job must be reported as not finished: {message}"
    );
    assert!(
        !message.contains("ran and failed"),
        "a watch timeout must not be conflated with a Job failure: {message}"
    );
    assert!(
        message.contains("NOT cancelled"),
        "the operator must be told the Job is still running: {message}"
    );
    assert_eq!(
        error
            .downcast_ref::<WorkOrderOutcomeError>()
            .map(|o| o.is_retryable()),
        Some(false),
        "a timed-out watch must never be retried: the original Job is still running"
    );

    cleanup(&client, &namespace).await;
}

/// Work orders with no Job stay apply-only, and say so.
#[tokio::test]
async fn test_custom_work_order_without_a_job_is_apply_only() {
    let namespace = format!("test-wo-no-job-{}", Uuid::new_v4());
    let client = setup().await;
    setup_namespace(&client, &namespace).await;

    let yaml = format!(
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: only-config\n  namespace: {namespace}\ndata:\n  key: value\n"
    );

    let work_order = custom_work_order(&yaml);
    let deadline = Instant::now() + Duration::from_secs(180);
    let result = execute_custom_work_order(&client, &test_agent(), &work_order, deadline).await;

    let message = result
        .expect("applying a ConfigMap should succeed")
        .expect("the custom work order must report a result message");

    assert!(
        message.contains("No batch/v1 Job present"),
        "an apply-only work order must say nothing was watched: {message}"
    );
    assert!(
        message.contains("not monitored"),
        "an apply-only work order must say the resources are unmonitored: {message}"
    );

    cleanup(&client, &namespace).await;
}
