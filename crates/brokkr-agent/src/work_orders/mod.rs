/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Work Orders Module
//!
//! This module handles the work order lifecycle for the Brokkr agent:
//! - Fetching pending work orders from the broker
//! - Claiming work orders for execution
//! - Executing work based on work type (e.g., builds)
//! - Reporting completion (success/failure) to the broker
//!
//! ## Work Order Flow
//!
//! ```text
//! 1. Poll broker for pending work orders
//! 2. Claim a work order
//! 3. Apply YAML content (Build + WorkOrder resources)
//! 4. Execute work type handler (e.g., create BuildRun for builds)
//! 5. Watch for completion
//! 6. Report result to broker
//! ```

pub mod broker;
pub mod build;

use brokkr_client::BrokkrClient;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::work_orders::WorkOrder;
use brokkr_utils::config::Settings;
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use kube::Client as K8sClient;
use kube::api::{Api, DynamicObject};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, warn};

/// `apiVersion` of the only kind whose completion the custom work-order path
/// watches. `batch/v1` `Job` is the one kind with unambiguous terminal
/// semantics (see [`interpret_job_status`]); everything else is applied only.
const JOB_API_VERSION: &str = "batch/v1";
/// `kind` of the watched resource.
const JOB_KIND: &str = "Job";

/// How often the Job watch re-reads `status` from the API server. Matches the
/// build path's poll cadence.
const JOB_STATUS_POLL_INTERVAL_SECS: u64 = 5;

/// Smallest slice of `claim_timeout_seconds` held back as safety margin, in
/// seconds. See [`job_watch_budget`].
const MIN_CLAIM_SAFETY_MARGIN_SECS: u64 = 60;

/// Percentage of `claim_timeout_seconds` held back as safety margin when that
/// is larger than [`MIN_CLAIM_SAFETY_MARGIN_SECS`]. See [`job_watch_budget`].
const CLAIM_SAFETY_MARGIN_PERCENT: u64 = 10;

/// An outcome the agent has classified itself, so that
/// [`process_single_work_order`] does not have to infer retryability from
/// error text via [`is_error_retryable`].
///
/// This exists specifically for Job watch outcomes. `is_error_retryable`
/// classifies any message containing "timeout" as retryable, and a retry of a
/// timed-out work order would re-dispatch the order **while the original Job
/// is still running in the cluster** — the exact double-execution this watch
/// is meant to prevent. Job outcomes therefore carry their disposition
/// explicitly rather than through wording.
#[derive(Debug)]
pub struct WorkOrderOutcomeError {
    message: String,
    retryable: bool,
}

impl WorkOrderOutcomeError {
    /// A terminal outcome the broker must not retry.
    fn non_retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }

    /// Whether the broker should schedule a retry of this work order.
    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}

impl std::fmt::Display for WorkOrderOutcomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorkOrderOutcomeError {}

/// Determines if an error is retryable by inspecting the error message.
///
/// Non-retryable errors include:
/// - 404 NotFound (resource doesn't exist)
/// - 403 Forbidden (permission denied)
/// - 400 BadRequest (malformed request)
/// - Validation errors
///
/// Retryable errors include:
/// - 429 TooManyRequests
/// - 500 InternalServerError
/// - 503 ServiceUnavailable
/// - 504 GatewayTimeout
/// - Network/connectivity errors
fn is_error_retryable(error: &dyn std::error::Error) -> bool {
    let error_str = error.to_string().to_lowercase();

    // Non-retryable patterns (permanent failures)
    let non_retryable_patterns = [
        "notfound",
        "not found",
        "forbidden",
        "unauthorized",
        "badrequest",
        "bad request",
        "invalid",
        "unprocessable",
        "conflict",
    ];

    for pattern in &non_retryable_patterns {
        if error_str.contains(pattern) {
            debug!(
                "Error classified as non-retryable (matched '{}'): {}",
                pattern, error
            );
            return false;
        }
    }

    // Retryable patterns (transient failures)
    let retryable_patterns = [
        "timeout",
        "unavailable",
        "connection",
        "network",
        "internal",
        "too many requests",
        "throttl",
    ];

    for pattern in &retryable_patterns {
        if error_str.contains(pattern) {
            debug!(
                "Error classified as retryable (matched '{}'): {}",
                pattern, error
            );
            return true;
        }
    }

    // Default to non-retryable for unknown errors
    // This prevents infinite retry loops for unhandled cases
    debug!(
        "Error classified as non-retryable (no pattern match): {}",
        error
    );
    false
}

/// Processes pending work orders for the agent.
///
/// This function:
/// 1. Fetches pending work orders from the broker
/// 2. Claims the first available work order
/// 3. Executes the work based on work type
/// 4. Reports completion to the broker
///
/// # Arguments
/// * `config` - Application settings
/// * `http_client` - HTTP client for broker communication
/// * `k8s_client` - Kubernetes client for resource operations
/// * `agent` - Agent details
///
/// # Returns
/// Number of work orders processed
pub async fn process_pending_work_orders(
    config: &Settings,
    http_client: &BrokkrClient,
    k8s_client: &K8sClient,
    agent: &Agent,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Fetch pending work orders
    let pending = broker::fetch_pending_work_orders(config, http_client, agent, None).await?;

    if pending.is_empty() {
        trace!("No pending work orders for agent {}", agent.name);
        return Ok(0);
    }

    info!(
        "Found {} pending work orders for agent {}",
        pending.len(),
        agent.name
    );

    let mut processed = 0;

    // Process one work order at a time
    // In the future, we could parallelize this based on work type
    for work_order in pending.iter().take(1) {
        match process_single_work_order(config, http_client, k8s_client, agent, work_order).await {
            Ok(_) => {
                processed += 1;
                info!(
                    "Successfully processed work order {} (type: {})",
                    work_order.id, work_order.work_type
                );
            }
            Err(e) => {
                error!(
                    "Failed to process work order {} (type: {}): {}",
                    work_order.id, work_order.work_type, e
                );
                // Continue with next work order instead of failing completely
            }
        }
    }

    Ok(processed)
}

/// Processes a single work order through its complete lifecycle.
async fn process_single_work_order(
    config: &Settings,
    http_client: &BrokkrClient,
    k8s_client: &K8sClient,
    agent: &Agent,
    work_order: &WorkOrder,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Processing work order {} (type: {}, status: {})",
        work_order.id, work_order.work_type, work_order.status
    );

    // Taken *before* the claim request is sent, so it is guaranteed to be no
    // later than the `claimed_at` the broker stamps while serving it. Any
    // deadline derived from this instant is therefore conservative with
    // respect to the broker's stale-claim reaper, without depending on the
    // two clocks agreeing.
    let claim_requested_at = Instant::now();

    // Claim the work order
    let claimed = broker::claim_work_order(config, http_client, agent, work_order.id).await?;
    info!("Successfully claimed work order {}", claimed.id);

    let watch_deadline = claim_requested_at + job_watch_budget(claimed.claim_timeout_seconds);

    // Execute based on work type
    let result = match work_order.work_type.as_str() {
        "build" => execute_build_work_order(config, http_client, k8s_client, agent, &claimed).await,
        "custom" => execute_custom_work_order(k8s_client, agent, &claimed, watch_deadline).await,
        unknown => Err(format!("Unknown work type: {}", unknown).into()),
    };

    // Report completion
    match result {
        Ok(message) => {
            broker::complete_work_order(config, http_client, work_order.id, true, message, true)
                .await?;
            info!("Work order {} completed successfully", work_order.id);
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Outcomes the agent has already classified (Job watch results)
            // carry their own disposition; everything else is inferred from
            // the error text.
            let retryable = match e.downcast_ref::<WorkOrderOutcomeError>() {
                Some(outcome) => outcome.is_retryable(),
                None => is_error_retryable(e.as_ref()),
            };
            if retryable {
                warn!(
                    "Work order {} failed with retryable error: {}",
                    work_order.id, e
                );
            } else {
                error!(
                    "Work order {} failed with non-retryable error: {}",
                    work_order.id, e
                );
            }
            broker::complete_work_order(
                config,
                http_client,
                work_order.id,
                false,
                Some(error_msg),
                retryable,
            )
            .await?;
            return Err(e);
        }
    }

    Ok(())
}

/// Executes a build work order using Shipwright.
async fn execute_build_work_order(
    _config: &Settings,
    _http_client: &BrokkrClient,
    k8s_client: &K8sClient,
    agent: &Agent,
    work_order: &WorkOrder,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Executing build work order {} for agent {}",
        work_order.id, agent.name
    );

    // Parse the YAML content to extract Build and WorkOrder resources
    let yaml_docs = crate::utils::multidoc_deserialize(&work_order.yaml_content)
        .map_err(|e| format!("failed to parse work order yaml: {e}"))?;

    if yaml_docs.is_empty() {
        return Err("Work order YAML content is empty".into());
    }

    // Apply all K8s resources from the YAML
    // The YAML should contain Shipwright Build + brokkr WorkOrder CRD
    for _doc in &yaml_docs {
        debug!("Applying K8s resource from work order YAML");
        // We'll implement the actual application in the build module
    }

    // Execute the build using the build handler
    let result = build::execute_build(
        k8s_client,
        &work_order.yaml_content,
        &work_order.id.to_string(),
    )
    .await?;

    Ok(result)
}

/// How long the agent may watch an applied `batch/v1` Job before it must stop
/// and report, derived from the work order's own `claim_timeout_seconds`.
///
/// The broker's maintenance task reclaims `CLAIMED` work orders once
/// `claimed_at + claim_timeout_seconds` has passed, returning them to
/// `PENDING` for any agent to pick up. A watch that outlives the claim would
/// therefore get the order re-dispatched to a second agent *while the first
/// Job is still running* — a migration executed twice. The budget is the claim
/// timeout minus a safety margin covering the apply that precedes the watch,
/// one poll interval of granularity, the completion report that follows it,
/// and broker/agent clock skew.
///
/// The margin is [`CLAIM_SAFETY_MARGIN_PERCENT`] of the claim timeout, floored
/// at [`MIN_CLAIM_SAFETY_MARGIN_SECS`]: for the 3600s default that is 360s,
/// leaving a 3240s watch. Claim timeouts at or below the floor yield a zero
/// budget, which still permits a single status read (see
/// [`watch_job_completion`]) before the order is reported as not finished —
/// deliberately, because silently reporting an unwatched Job as succeeded is
/// the bug this ticket exists to fix.
pub(crate) fn job_watch_budget(claim_timeout_seconds: i32) -> Duration {
    let claim_timeout = u64::try_from(claim_timeout_seconds).unwrap_or(0);
    let margin =
        (claim_timeout * CLAIM_SAFETY_MARGIN_PERCENT / 100).max(MIN_CLAIM_SAFETY_MARGIN_SECS);
    Duration::from_secs(claim_timeout.saturating_sub(margin))
}

/// Whether an applied object is a `batch/v1` Job, the one kind this path
/// watches to completion.
fn is_batch_v1_job(object: &DynamicObject) -> bool {
    object
        .types
        .as_ref()
        .is_some_and(|t| t.api_version == JOB_API_VERSION && t.kind == JOB_KIND)
}

/// Terminal (or not-yet-terminal) reading of a `batch/v1` Job's status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobOutcome {
    /// The Job reported the `Complete` condition.
    Succeeded,
    /// The Job reported the `Failed` condition, with the reason/message it gave.
    Failed(String),
    /// No terminal condition yet — keep watching.
    Running,
}

/// Interprets a `batch/v1` Job status.
///
/// A Job is finished only when it carries a terminal condition — `Complete`
/// or `Failed` — with status `True`; the API guarantees it cannot carry both.
/// The `succeeded` / `failed` pod counters are deliberately *not* used as the
/// signal: `failed` increments on every pod attempt, so a Job with a non-zero
/// `backoffLimit` can show `failed >= 1` while it is still retrying and may
/// yet succeed. Non-terminal conditions the controller also publishes
/// (`Suspended`, `SuccessCriteriaMet`, `FailureTarget`) are ignored.
pub(crate) fn interpret_job_status(status: &JobStatus) -> JobOutcome {
    let Some(conditions) = status.conditions.as_ref() else {
        return JobOutcome::Running;
    };

    for condition in conditions {
        if condition.status != "True" {
            continue;
        }
        match condition.type_.as_str() {
            "Complete" => return JobOutcome::Succeeded,
            "Failed" => {
                let detail = match (condition.reason.as_deref(), condition.message.as_deref()) {
                    (Some(reason), Some(message)) => format!("{reason}: {message}"),
                    (Some(reason), None) => reason.to_string(),
                    (None, Some(message)) => message.to_string(),
                    (None, None) => "no reason reported".to_string(),
                };
                return JobOutcome::Failed(detail);
            }
            _ => {}
        }
    }

    JobOutcome::Running
}

/// Watches an applied `batch/v1` Job until it reaches a terminal state or the
/// watch budget runs out.
///
/// `deadline` is the absolute instant derived from the work order's claim
/// timeout by [`job_watch_budget`]. Status is read *before* the deadline is
/// checked, so a Job that is already finished is reported correctly even when
/// the budget is zero.
///
/// Transient API read failures do not end the watch — the agent keeps polling
/// and, if the deadline arrives with the last read still failing, says so in
/// the message rather than presenting it as a plain timeout.
///
/// Returns a [`WorkOrderOutcomeError`] whose message distinguishes the three
/// non-success cases: the Job ran and failed, the Job was still running when
/// the budget ran out, or the Job vanished.
pub async fn watch_job_completion(
    k8s_client: &K8sClient,
    namespace: &str,
    name: &str,
    deadline: Instant,
) -> Result<(), WorkOrderOutcomeError> {
    let api: Api<Job> = Api::namespaced(k8s_client.clone(), namespace);
    let poll_interval = Duration::from_secs(JOB_STATUS_POLL_INTERVAL_SECS);

    loop {
        // `None` once the read succeeded, so a timeout report only mentions an
        // API problem when the *most recent* read is the one that failed.
        let last_read_error: Option<String> = match api.get_opt(name).await {
            Ok(Some(job)) => {
                let status = job.status.unwrap_or_default();
                match interpret_job_status(&status) {
                    JobOutcome::Succeeded => {
                        info!("Job {}/{} completed successfully", namespace, name);
                        return Ok(());
                    }
                    JobOutcome::Failed(detail) => {
                        error!("Job {}/{} failed: {}", namespace, name, detail);
                        return Err(WorkOrderOutcomeError::non_retryable(format!(
                            "Job {namespace}/{name} ran and failed: {detail}"
                        )));
                    }
                    JobOutcome::Running => {
                        debug!(
                            "Job {}/{} still running (active={:?}, succeeded={:?}, failed={:?})",
                            namespace, name, status.active, status.succeeded, status.failed
                        );
                        None
                    }
                }
            }
            Ok(None) => {
                return Err(WorkOrderOutcomeError::non_retryable(format!(
                    "Job {namespace}/{name} was applied but no longer exists; it was deleted \
                     before reaching a terminal state, so its outcome is unknown"
                )));
            }
            Err(e) => {
                warn!(
                    "Failed to read status of Job {}/{}, will retry: {}",
                    namespace, name, e
                );
                Some(e.to_string())
            }
        };

        let now = Instant::now();
        if now >= deadline {
            let suffix = match &last_read_error {
                Some(e) => format!(" The last status read also failed: {e}."),
                None => String::new(),
            };
            return Err(WorkOrderOutcomeError::non_retryable(format!(
                "Job {namespace}/{name} did not finish within the watch window and was still \
                 running when the agent stopped watching. The window is bounded below the work \
                 order's claim_timeout_seconds so the order cannot be reclaimed and re-run while \
                 this Job is still going. The Job was NOT cancelled and its final outcome is \
                 unknown to Brokkr — this is not a report that the Job failed.{suffix}"
            )));
        }

        // Never sleep past the deadline.
        let remaining = deadline.saturating_duration_since(now);
        tokio::time::sleep(poll_interval.min(remaining)).await;
    }
}

/// Executes a custom work order by applying YAML resources to the cluster.
///
/// After the apply, every `batch/v1` Job among the applied objects is watched
/// to a terminal state (see [`watch_job_completion`]); the work order succeeds
/// only if all of them report `Complete`. Objects of any other kind are
/// applied only — they are not monitored for completion, and the result
/// message says so explicitly. Ongoing health of non-Job resources is the
/// deployment-object reconciler's concern, not the work-order path's.
///
/// `watch_deadline` bounds the total time spent watching, derived from the
/// work order's `claim_timeout_seconds` by [`job_watch_budget`].
pub async fn execute_custom_work_order(
    k8s_client: &K8sClient,
    agent: &Agent,
    work_order: &WorkOrder,
    watch_deadline: Instant,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    use kube::api::PatchParams;

    info!(
        "Executing custom work order {} for agent {}",
        work_order.id, agent.name
    );

    // Parse the YAML content
    let yaml_docs = crate::utils::multidoc_deserialize(&work_order.yaml_content)
        .map_err(|e| format!("failed to parse work order yaml: {e}"))?;

    if yaml_docs.is_empty() {
        return Err("Work order YAML content is empty".into());
    }

    // Convert YAML docs to DynamicObjects
    let mut objects: Vec<DynamicObject> = Vec::new();
    for yaml_doc in &yaml_docs {
        // Skip null documents
        if yaml_doc.is_null() {
            continue;
        }

        let object: DynamicObject = serde_yaml::from_value(yaml_doc.clone())?;
        let gvk = object
            .types
            .as_ref()
            .ok_or("Object missing type metadata")?;
        debug!(
            "Parsed {} {}/{}",
            gvk.kind,
            object.metadata.namespace.as_deref().unwrap_or("cluster"),
            object.metadata.name.as_deref().unwrap_or("unnamed")
        );
        objects.push(object);
    }

    if objects.is_empty() {
        return Err("No valid Kubernetes objects found in YAML".into());
    }

    info!(
        "Applying {} resource(s) from custom work order {}",
        objects.len(),
        work_order.id
    );

    // Apply all resources using server-side apply
    let patch_params = PatchParams::apply("brokkr-agent").force();
    crate::k8s::api::apply_k8s_objects(&objects, k8s_client.clone(), patch_params)
        .await
        .map_err(|e| format!("failed to apply work order objects: {e}"))?;

    // Applying is not finishing. Watch every batch/v1 Job to a terminal state;
    // every other kind stays apply-only.
    let (jobs, others): (Vec<&DynamicObject>, Vec<&DynamicObject>) =
        objects.iter().partition(|o| is_batch_v1_job(o));

    let unmonitored = describe_unmonitored_kinds(&others);

    if jobs.is_empty() {
        return Ok(Some(format!(
            "Applied {} resource(s). No batch/v1 Job present, so nothing was watched to \
             completion: {} applied only, not monitored. Only batch/v1 Jobs have a terminal \
             state the agent waits on.",
            objects.len(),
            unmonitored
        )));
    }

    let mut watched: Vec<String> = Vec::with_capacity(jobs.len());
    for job in &jobs {
        let namespace = job.metadata.namespace.as_deref().unwrap_or("default");
        let Some(name) = job.metadata.name.as_deref() else {
            return Err(Box::new(WorkOrderOutcomeError::non_retryable(
                "A batch/v1 Job in this work order has no metadata.name, so it cannot be watched \
                 to completion. Give the Job an explicit name (generateName is not supported \
                 here)."
                    .to_string(),
            )));
        };

        info!(
            "Watching Job {}/{} from custom work order {} until it reaches a terminal state",
            namespace, name, work_order.id
        );
        watch_job_completion(k8s_client, namespace, name, watch_deadline)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        watched.push(format!("{namespace}/{name}"));
    }

    let others_note = if others.is_empty() {
        String::new()
    } else {
        format!(" {unmonitored} applied only, not monitored.")
    };

    Ok(Some(format!(
        "Applied {} resource(s). Job(s) {} completed successfully.{}",
        objects.len(),
        watched.join(", "),
        others_note
    )))
}

/// Renders the kinds that were applied without being watched, for the result
/// message. Constraint: a custom work order must never leave an operator
/// guessing which of its resources the success signal actually covers.
fn describe_unmonitored_kinds(others: &[&DynamicObject]) -> String {
    if others.is_empty() {
        return "no other resources".to_string();
    }

    let mut kinds: Vec<String> = others
        .iter()
        .map(|o| {
            o.types
                .as_ref()
                .map(|t| t.kind.clone())
                .unwrap_or_else(|| "unknown kind".to_string())
        })
        .collect();
    kinds.sort();
    kinds.dedup();

    format!(
        "{} resource(s) of kind(s) {}",
        others.len(),
        kinds.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::batch::v1::JobCondition;

    fn condition(type_: &str, status: &str) -> JobCondition {
        JobCondition {
            type_: type_.to_string(),
            status: status.to_string(),
            ..Default::default()
        }
    }

    // ==================== Claim-timeout-derived watch budget ====================

    #[test]
    fn test_job_watch_budget_default_claim_timeout_leaves_headroom() {
        // The 3600s default: 10% margin -> 3240s of watching, comfortably
        // below the point at which the broker reclaims the claim.
        let budget = job_watch_budget(3600);
        assert_eq!(budget, Duration::from_secs(3240));
        assert!(budget < Duration::from_secs(3600));
    }

    #[test]
    fn test_job_watch_budget_is_always_strictly_below_claim_timeout() {
        for claim_timeout in [1, 30, 60, 61, 120, 600, 3600, 86_400] {
            let budget = job_watch_budget(claim_timeout);
            assert!(
                budget < Duration::from_secs(claim_timeout as u64),
                "budget {:?} must stay under the {}s claim timeout so the stale-claim reaper \
                 cannot re-dispatch the order mid-Job",
                budget,
                claim_timeout
            );
        }
    }

    #[test]
    fn test_job_watch_budget_uses_percentage_margin_above_the_floor() {
        // Below the crossover the flat floor dominates...
        assert_eq!(job_watch_budget(600), Duration::from_secs(540));
        // ...above it the percentage does.
        assert_eq!(job_watch_budget(7200), Duration::from_secs(6480));
    }

    #[test]
    fn test_job_watch_budget_saturates_at_zero_for_short_or_invalid_timeouts() {
        // A claim timeout at or under the safety floor leaves no room to
        // watch. Zero is correct: the watch still performs one status read
        // and then reports "did not finish" rather than a false success.
        assert_eq!(job_watch_budget(60), Duration::ZERO);
        assert_eq!(job_watch_budget(30), Duration::ZERO);
        assert_eq!(job_watch_budget(0), Duration::ZERO);
        assert_eq!(job_watch_budget(-1), Duration::ZERO);
    }

    // ==================== Job terminal-state interpretation ====================

    #[test]
    fn test_interpret_job_status_complete_is_success() {
        let status = JobStatus {
            conditions: Some(vec![condition("Complete", "True")]),
            succeeded: Some(1),
            ..Default::default()
        };
        assert_eq!(interpret_job_status(&status), JobOutcome::Succeeded);
    }

    #[test]
    fn test_interpret_job_status_failed_carries_reason_and_message() {
        let status = JobStatus {
            conditions: Some(vec![JobCondition {
                type_: "Failed".to_string(),
                status: "True".to_string(),
                reason: Some("BackoffLimitExceeded".to_string()),
                message: Some("Job has reached the specified backoff limit".to_string()),
                ..Default::default()
            }]),
            failed: Some(3),
            ..Default::default()
        };
        match interpret_job_status(&status) {
            JobOutcome::Failed(detail) => {
                assert!(detail.contains("BackoffLimitExceeded"));
                assert!(detail.contains("backoff limit"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn test_interpret_job_status_failed_without_reason_or_message() {
        let status = JobStatus {
            conditions: Some(vec![condition("Failed", "True")]),
            ..Default::default()
        };
        assert_eq!(
            interpret_job_status(&status),
            JobOutcome::Failed("no reason reported".to_string())
        );
    }

    #[test]
    fn test_interpret_job_status_no_status_or_conditions_is_running() {
        assert_eq!(
            interpret_job_status(&JobStatus::default()),
            JobOutcome::Running
        );
        assert_eq!(
            interpret_job_status(&JobStatus {
                conditions: Some(vec![]),
                ..Default::default()
            }),
            JobOutcome::Running
        );
    }

    #[test]
    fn test_interpret_job_status_running_pods_are_not_terminal() {
        let status = JobStatus {
            active: Some(1),
            start_time: None,
            ..Default::default()
        };
        assert_eq!(interpret_job_status(&status), JobOutcome::Running);
    }

    #[test]
    fn test_interpret_job_status_failed_pod_within_backoff_limit_is_not_terminal() {
        // The regression this guards: a Job with backoffLimit > 0 shows
        // `failed >= 1` while it is still retrying and may yet succeed.
        // Only the terminal condition may decide the outcome.
        let status = JobStatus {
            active: Some(1),
            failed: Some(2),
            ..Default::default()
        };
        assert_eq!(interpret_job_status(&status), JobOutcome::Running);
    }

    #[test]
    fn test_interpret_job_status_ignores_false_and_non_terminal_conditions() {
        let status = JobStatus {
            conditions: Some(vec![
                condition("Suspended", "False"),
                condition("Complete", "False"),
                condition("Failed", "False"),
                // Pre-terminal signals in newer Kubernetes: the Job is on its
                // way to a terminal state but is not there yet.
                condition("SuccessCriteriaMet", "True"),
                condition("FailureTarget", "True"),
            ]),
            ..Default::default()
        };
        assert_eq!(interpret_job_status(&status), JobOutcome::Running);
    }

    #[test]
    fn test_interpret_job_status_terminal_condition_wins_over_earlier_ones() {
        let status = JobStatus {
            conditions: Some(vec![
                condition("Suspended", "False"),
                condition("Complete", "True"),
            ]),
            ..Default::default()
        };
        assert_eq!(interpret_job_status(&status), JobOutcome::Succeeded);
    }

    // ==================== Kind routing ====================

    fn object(api_version: &str, kind: &str) -> DynamicObject {
        DynamicObject {
            types: Some(kube::core::TypeMeta {
                api_version: api_version.to_string(),
                kind: kind.to_string(),
            }),
            metadata: Default::default(),
            data: serde_json::json!({}),
        }
    }

    #[test]
    fn test_only_batch_v1_jobs_are_watched() {
        assert!(is_batch_v1_job(&object("batch/v1", "Job")));

        // Everything else is apply-only, including the kinds most easily
        // mistaken for a Job.
        assert!(!is_batch_v1_job(&object("batch/v1", "CronJob")));
        assert!(!is_batch_v1_job(&object("batch/v1beta1", "Job")));
        assert!(!is_batch_v1_job(&object("apps/v1", "Deployment")));
        assert!(!is_batch_v1_job(&object("v1", "ConfigMap")));

        let mut untyped = object("batch/v1", "Job");
        untyped.types = None;
        assert!(!is_batch_v1_job(&untyped));
    }

    #[test]
    fn test_describe_unmonitored_kinds_lists_distinct_sorted_kinds() {
        let a = object("v1", "ConfigMap");
        let b = object("v1", "Secret");
        let c = object("v1", "ConfigMap");
        let others = vec![&a, &b, &c];

        let described = describe_unmonitored_kinds(&others);
        assert!(described.contains("3 resource(s)"));
        assert!(described.contains("ConfigMap, Secret"));
        assert_eq!(described.matches("ConfigMap").count(), 1);

        assert_eq!(describe_unmonitored_kinds(&[]), "no other resources");
    }

    // ==================== Retry disposition ====================

    #[test]
    fn test_job_outcomes_are_never_retryable() {
        // Retrying a work order whose Job timed out would re-dispatch it while
        // the original Job is still running. `is_error_retryable` would say
        // otherwise for both of these messages, which is exactly why the
        // outcome carries its own disposition.
        let timed_out = WorkOrderOutcomeError::non_retryable(
            "Job default/db-migration did not finish within the watch window",
        );
        assert!(!timed_out.retryable);

        let failed = WorkOrderOutcomeError::non_retryable(
            "Job default/db-migration ran and failed: BackoffLimitExceeded",
        );
        assert!(!failed.retryable);

        // The text-sniffing classifier disagrees on messages like these, so
        // the downcast in `process_single_work_order` is load-bearing.
        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(
            WorkOrderOutcomeError::non_retryable("watch window timeout: connection"),
        );
        assert!(
            is_error_retryable(boxed.as_ref()),
            "sanity: the text classifier would call this retryable"
        );
        assert_eq!(
            boxed
                .downcast_ref::<WorkOrderOutcomeError>()
                .map(|o| o.retryable),
            Some(false),
            "the classified outcome must win over the text heuristic"
        );
    }
}
