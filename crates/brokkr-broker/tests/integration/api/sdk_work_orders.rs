/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! The work order methods of the Rust SDK against a real broker
//! (BROKKR-T-0407).
//!
//! The `brokkr work-order` commands are a thin shell over these methods of
//! `BrokkrClient`. Each test serves the broker's real router on a local port
//! and calls the SDK over HTTP with a real PAK: an admin's, a generator's,
//! and a second generator's.

use brokkr_client::types::WorkOrder;
use brokkr_client::{BrokkrClient, BrokkrError, NewWorkOrderSpec, WorkOrderLogFilter};
use brokkr_models::models::agent_k8s_events::NewAgentK8sEvent;
use brokkr_models::models::agent_pod_logs::NewAgentPodLog;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::fixtures::TestFixture;

const JOB: &str = "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: sdk\n";

/// Serve the v1 routes of the broker on a local port, as the broker does.
async fn serve(fixture: &TestFixture) -> String {
    use brokkr_broker::ws::{ConnectionRegistry, FleetBroadcaster, LiveBroadcaster};
    use brokkr_utils::config::Cors;

    let cors = Cors {
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec!["GET".to_string(), "POST".to_string(), "DELETE".to_string()],
        allowed_headers: vec!["*".to_string()],
        max_age_seconds: 60,
    };
    // The create route pushes the work order to its agents; it reads the
    // registry and the broadcasters from the extensions.
    let app = axum::Router::new()
        .merge(brokkr_broker::api::v1::routes(
            fixture.dal.clone(),
            &cors,
            None,
        ))
        .layer(axum::extract::Extension(ConnectionRegistry::new()))
        .layer(axum::extract::Extension(LiveBroadcaster::new()))
        .layer(axum::extract::Extension(FleetBroadcaster::new()))
        .with_state(fixture.dal.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn sdk(url: &str, pak: &str) -> BrokkrClient {
    BrokkrClient::builder(url)
        .token(pak)
        .max_retries(0)
        .build()
        .expect("the SDK client builds")
}

fn spec(agents: Vec<Uuid>) -> NewWorkOrderSpec {
    NewWorkOrderSpec {
        work_type: "custom".to_string(),
        yaml_content: JOB.to_string(),
        agent_ids: agents,
        ..NewWorkOrderSpec::default()
    }
}

/// The status and the stable code of a refusal.
fn refusal<T: std::fmt::Debug>(result: Result<T, BrokkrError>) -> (u16, String) {
    let err = result.expect_err("the broker must refuse");
    (
        err.status().map(|s| s.as_u16()).unwrap_or(0),
        err.code().unwrap_or("no code").to_string(),
    )
}

/// A broker with two generators, and one agent registered with the first.
struct Scene {
    url: String,
    admin: BrokkrClient,
    owner: BrokkrClient,
    owner_id: Uuid,
    other: BrokkrClient,
    agent_id: Uuid,
}

async fn scene(fixture: &TestFixture, tag: &str) -> Scene {
    let url = serve(fixture).await;
    let (owner, owner_pak) =
        fixture.create_test_generator_with_pak(format!("sdk-wo-owner-{tag}"), None);
    let (_other, other_pak) =
        fixture.create_test_generator_with_pak(format!("sdk-wo-other-{tag}"), None);
    let agent = fixture.create_test_agent(format!("sdk-wo-agent-{tag}"), "c".to_string());
    fixture.register_agent_with_generator(agent.id, owner.id);
    Scene {
        admin: sdk(&url, &fixture.admin_pak),
        owner: sdk(&url, &owner_pak),
        owner_id: owner.id,
        other: sdk(&url, &other_pak),
        agent_id: agent.id,
        url,
    }
}

#[tokio::test]
async fn a_generator_creates_lists_reads_and_deletes_its_work_order() {
    let fixture = TestFixture::new();
    let s = scene(&fixture, "crud").await;

    let created: WorkOrder = s
        .owner
        .create_work_order(NewWorkOrderSpec {
            labels: vec!["env:sdk".to_string()],
            annotations: vec![("tier".to_string(), "gold".to_string())],
            claim_timeout_seconds: Some(120),
            max_retries: Some(1),
            backoff_seconds: Some(5),
            ..spec(vec![s.agent_id])
        })
        .await
        .expect("the generator creates a work order");
    assert_eq!(created.status, "PENDING");
    assert_eq!(created.work_type, "custom");
    assert_eq!(created.owner_generator_id, Some(s.owner_id));
    assert_eq!(
        (
            created.claim_timeout_seconds,
            created.max_retries,
            created.backoff_seconds
        ),
        (120, 1, 5)
    );

    let listed = s.owner.list_work_orders().await.expect("list");
    assert!(listed.iter().any(|w| w.id == created.id));
    let read = s.owner.get_work_order(created.id).await.expect("read");
    assert_eq!(read.id, created.id);

    // An admin sees it; the other generator does not.
    let all = s.admin.list_work_orders().await.expect("admin list");
    assert!(all.iter().any(|w| w.id == created.id));
    let others = s.other.list_work_orders().await.expect("other list");
    assert!(!others.iter().any(|w| w.id == created.id));

    s.owner
        .delete_work_order(created.id)
        .await
        .expect("the owner deletes it");
    assert_eq!(
        refusal(s.owner.get_work_order(created.id).await),
        (404, "work_order_not_found".to_string())
    );
}

#[tokio::test]
async fn an_admin_creates_and_owns_its_work_order() {
    let fixture = TestFixture::new();
    let s = scene(&fixture, "admin").await;

    let created = s
        .admin
        .create_work_order(spec(vec![s.agent_id]))
        .await
        .expect("the admin creates a work order");
    assert_eq!(created.owner_generator_id, None, "no value: an admin's");

    // It targets the generator's agent, and the generator still cannot read it.
    assert_eq!(
        refusal(s.owner.get_work_order(created.id).await),
        (403, "work_order_not_owned".to_string())
    );
    let owned = s.owner.list_work_orders().await.expect("list");
    assert!(!owned.iter().any(|w| w.id == created.id));

    s.admin
        .delete_work_order(created.id)
        .await
        .expect("the admin deletes it");
}

#[tokio::test]
async fn the_three_refusals_have_their_stable_codes() {
    let fixture = TestFixture::new();
    let s = scene(&fixture, "refuse").await;

    // 1. A target agent that is not registered with the generator.
    let stranger = fixture.create_test_agent("sdk-wo-stranger".to_string(), "c".to_string());
    assert_eq!(
        refusal(s.other.create_work_order(spec(vec![stranger.id])).await),
        (403, "agent_not_registered".to_string())
    );
    assert!(
        s.other.list_work_orders().await.expect("list").is_empty(),
        "a refused create leaves no work order"
    );

    // 2. The work order of another owner, on each read and on the delete.
    let created = s
        .owner
        .create_work_order(spec(vec![s.agent_id]))
        .await
        .expect("create");
    let not_owned = (403, "work_order_not_owned".to_string());
    assert_eq!(refusal(s.other.get_work_order(created.id).await), not_owned);
    assert_eq!(
        refusal(s.other.delete_work_order(created.id).await),
        not_owned
    );
    assert_eq!(
        refusal(s.other.work_order_logs(created.id, None).await),
        not_owned
    );
    assert_eq!(
        refusal(s.other.work_order_events(created.id, None).await),
        not_owned
    );
    assert!(
        s.owner.get_work_order(created.id).await.is_ok(),
        "a refused delete leaves the work order"
    );

    // 3. An id that no work order has.
    let unknown = Uuid::new_v4();
    let not_found = (404, "work_order_not_found".to_string());
    assert_eq!(refusal(s.admin.get_work_order(unknown).await), not_found);
    assert_eq!(refusal(s.admin.delete_work_order(unknown).await), not_found);
    assert_eq!(
        refusal(s.admin.work_order_logs(unknown, None).await),
        not_found
    );

    // An agent PAK has no place on these routes.
    let (agent, agent_pak) =
        fixture.create_test_agent_with_pak("sdk-wo-agent-pak".to_string(), "c".to_string());
    let _ = agent;
    let as_agent = sdk(&s.url, &agent_pak);
    assert_eq!(
        refusal(as_agent.list_work_orders().await),
        (403, "work_order_access_denied".to_string())
    );
}

#[tokio::test]
async fn an_ended_work_order_is_read_with_its_result_logs_and_events() {
    let fixture = TestFixture::new();
    let s = scene(&fixture, "ended").await;

    let created = s
        .owner
        .create_work_order(spec(vec![s.agent_id]))
        .await
        .expect("create");

    // The agent's side, at the DAL: claim, send a line and an event, end it.
    fixture
        .dal
        .work_orders()
        .claim(created.id, s.agent_id)
        .expect("claim failed");
    let claimed = s.owner.get_work_order(created.id).await.expect("read");
    assert_eq!(claimed.status, "CLAIMED");
    assert_eq!(claimed.claimed_by, Some(s.agent_id));

    let now = chrono::Utc::now();
    fixture
        .dal
        .agent_pod_logs()
        .create(&NewAgentPodLog {
            agent_id: s.agent_id,
            stack_id: None,
            work_order_id: Some(created.id),
            namespace: "default".to_string(),
            pod: "sdk-abc".to_string(),
            container: "main".to_string(),
            ts: now,
            line: "hello from the job".to_string(),
        })
        .expect("log line");
    fixture
        .dal
        .agent_k8s_events()
        .create(&NewAgentK8sEvent {
            agent_id: s.agent_id,
            stack_id: None,
            work_order_id: Some(created.id),
            observed_at: now,
            reason: "Completed".to_string(),
            message: "Job completed".to_string(),
            event_type: "Normal".to_string(),
            source: None,
            involved_object: serde_json::json!({"kind": "Job", "name": "sdk"}),
        })
        .expect("event");
    fixture
        .dal
        .work_orders()
        .complete_success(created.id, Some("sha256:abc".to_string()))
        .expect("complete_success failed");

    // The read answers from the log, and the log entry has the message.
    for client in [&s.owner, &s.admin] {
        let ended = client.get_work_order(created.id).await.expect("read");
        assert_eq!(ended.status, "COMPLETED");
        let entry = client.get_work_order_log(created.id).await.expect("log");
        assert!(entry.success);
        assert_eq!(entry.result_message.as_deref(), Some("sha256:abc"));
        assert_eq!(entry.owner_generator_id, Some(s.owner_id));

        let logs = client
            .work_order_logs(created.id, None)
            .await
            .expect("logs");
        assert_eq!(logs.lines.len(), 1);
        assert_eq!(logs.lines[0].line, "hello from the job");
        let events = client
            .work_order_events(created.id, Some(10))
            .await
            .expect("events");
        assert_eq!(events.events.len(), 1);
        assert_eq!(events.events[0].reason, "Completed");
    }

    // The other generator reads none of it.
    assert_eq!(
        refusal(s.other.get_work_order_log(created.id).await),
        (403, "work_order_not_owned".to_string())
    );
}

/// End a work order at the DAL, as its agent does: claim, then the result.
fn end(fixture: &TestFixture, work_order: Uuid, agent: Uuid, success: bool, message: &str) {
    fixture
        .dal
        .work_orders()
        .claim(work_order, agent)
        .expect("claim failed");
    if success {
        fixture
            .dal
            .work_orders()
            .complete_success(work_order, Some(message.to_string()))
            .expect("complete_success failed");
    } else {
        fixture
            .dal
            .work_orders()
            .complete_failure(work_order, message.to_string(), false)
            .expect("complete_failure failed");
    }
}

#[tokio::test]
async fn the_log_lists_ended_work_orders_for_their_owner_and_an_admin() {
    let fixture = TestFixture::new();
    let s = scene(&fixture, "history").await;
    let ids = |entries: &[brokkr_client::types::WorkOrderLog]| -> Vec<Uuid> {
        entries.iter().map(|e| e.id).collect()
    };

    // The generator's: one success and one failure. An admin's: one success.
    let good = s
        .owner
        .create_work_order(spec(vec![s.agent_id]))
        .await
        .expect("create");
    let bad = s
        .owner
        .create_work_order(NewWorkOrderSpec {
            work_type: "build".to_string(),
            ..spec(vec![s.agent_id])
        })
        .await
        .expect("create");
    let of_admin = s
        .admin
        .create_work_order(spec(vec![s.agent_id]))
        .await
        .expect("create");
    end(&fixture, good.id, s.agent_id, true, "sha256:abc");
    end(&fixture, bad.id, s.agent_id, false, "job failed");
    end(&fixture, of_admin.id, s.agent_id, true, "ok");

    // The owner gets its two, and not the admin's.
    let own = s
        .owner
        .list_work_order_log(WorkOrderLogFilter::default())
        .await
        .expect("the owner lists its log");
    let own_ids = ids(&own);
    assert!(own_ids.contains(&good.id) && own_ids.contains(&bad.id));
    assert!(!own_ids.contains(&of_admin.id), "an admin's is for admins");
    let entry = own.iter().find(|e| e.id == bad.id).unwrap();
    assert!(!entry.success);
    assert_eq!(entry.result_message.as_deref(), Some("job failed"));
    assert_eq!(entry.claimed_by, Some(s.agent_id));
    assert_eq!(entry.owner_generator_id, Some(s.owner_id));

    // An admin gets all three; the other generator gets none of them.
    let all = ids(&s
        .admin
        .list_work_order_log(WorkOrderLogFilter::default())
        .await
        .expect("the admin lists the log"));
    for id in [good.id, bad.id, of_admin.id] {
        assert!(all.contains(&id));
    }
    let others = ids(&s
        .other
        .list_work_order_log(WorkOrderLogFilter::default())
        .await
        .expect("the other generator lists its log"));
    for id in [good.id, bad.id, of_admin.id] {
        assert!(!others.contains(&id));
    }

    // Each filter narrows the owner's list.
    let failures = ids(&s
        .owner
        .list_work_order_log(WorkOrderLogFilter {
            success: Some(false),
            ..WorkOrderLogFilter::default()
        })
        .await
        .expect("filter by result"));
    assert_eq!(failures, vec![bad.id]);
    let builds = ids(&s
        .owner
        .list_work_order_log(WorkOrderLogFilter {
            work_type: Some("build".to_string()),
            ..WorkOrderLogFilter::default()
        })
        .await
        .expect("filter by type"));
    assert_eq!(builds, vec![bad.id]);
    let by_agent = ids(&s
        .owner
        .list_work_order_log(WorkOrderLogFilter {
            agent_id: Some(s.agent_id),
            limit: Some(1),
            ..WorkOrderLogFilter::default()
        })
        .await
        .expect("filter by agent, with a limit"));
    assert_eq!(by_agent.len(), 1);
    let by_stranger = s
        .owner
        .list_work_order_log(WorkOrderLogFilter {
            agent_id: Some(Uuid::new_v4()),
            ..WorkOrderLogFilter::default()
        })
        .await
        .expect("filter by an agent that ran none");
    assert!(by_stranger.is_empty());

    // An agent PAK has no place on this route.
    let (_agent, agent_pak) =
        fixture.create_test_agent_with_pak("sdk-wo-history-pak".to_string(), "c".to_string());
    assert_eq!(
        refusal(
            sdk(&s.url, &agent_pak)
                .list_work_order_log(WorkOrderLogFilter::default())
                .await
        ),
        (403, "work_order_access_denied".to_string())
    );
}
