/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Correctness-anchor tests for the BROKKR-T-0226 fleet rollup DAL methods.
//!
//! These tests prove that the grouped, set-based queries used by the fleet
//! surface (`pending_counts_by_agent`, work-order `pending_counts_by_agent`,
//! `claimed_counts_by_agent`, `status_counts_by_agent`, `last_event_at_by_agent`)
//! produce the same per-agent values as the existing per-agent ground-truth
//! functions (`get_target_state_for_agent`, `list_pending_for_agent`, ...).

use crate::fixtures::TestFixture;
use std::collections::HashMap;
use uuid::Uuid;

/// Seeds a deliberately heterogeneous fleet (agents matched via targets,
/// labels, and annotations; some acknowledged objects; pending + claimed work
/// orders; health records) and asserts the grouped DAL methods agree with the
/// per-agent ground truth for EVERY agent.
#[test]
fn test_fleet_grouped_methods_match_per_agent_ground_truth() {
    let fixture = TestFixture::new();
    let gen_id = fixture.admin_generator.id;

    // --- Stacks ---------------------------------------------------------
    // s_target: reached only via a hard agent_target.
    // s_label:  reached only via a shared label.
    // s_annot:  reached only via a shared annotation.
    // s_orphan: reached by nobody.
    let s_target = fixture.create_test_stack("s_target".to_string(), None, gen_id);
    let s_label = fixture.create_test_stack("s_label".to_string(), None, gen_id);
    let s_annot = fixture.create_test_stack("s_annot".to_string(), None, gen_id);
    let s_orphan = fixture.create_test_stack("s_orphan".to_string(), None, gen_id);

    fixture.create_test_stack_label(s_label.id, "team-a".to_string());
    fixture.create_test_stack_annotation(s_annot.id, "tier", "gold");

    // Deployment objects: give each stack a couple of versions so the "latest
    // per stack" reduction is actually exercised.
    fixture.create_test_deployment_object(s_target.id, "t-v1".to_string(), false);
    let s_target_latest =
        fixture.create_test_deployment_object(s_target.id, "t-v2".to_string(), false);
    let s_label_latest =
        fixture.create_test_deployment_object(s_label.id, "l-v1".to_string(), false);
    let s_annot_latest =
        fixture.create_test_deployment_object(s_annot.id, "a-v1".to_string(), false);
    fixture.create_test_deployment_object(s_orphan.id, "o-v1".to_string(), false);

    // --- Agents ---------------------------------------------------------
    // a_target: hard-targets s_target.
    // a_label:  shares label team-a -> s_label.
    // a_annot:  shares annotation tier=gold -> s_annot.
    // a_multi:  targets s_target AND shares the label AND the annotation
    //           (matched via multiple mechanisms; must be counted once/stack).
    // a_none:   matches nothing.
    let a_target = fixture.create_test_agent("a_target".to_string(), "c".to_string());
    let a_label = fixture.create_test_agent("a_label".to_string(), "c".to_string());
    let a_annot = fixture.create_test_agent("a_annot".to_string(), "c".to_string());
    let a_multi = fixture.create_test_agent("a_multi".to_string(), "c".to_string());
    let a_none = fixture.create_test_agent("a_none".to_string(), "c".to_string());

    fixture.create_test_agent_target(a_target.id, s_target.id);
    fixture.create_test_agent_label(a_label.id, "team-a".to_string());
    fixture.create_test_agent_annotation(a_annot.id, "tier".to_string(), "gold".to_string());
    // a_multi matches s_target (target), s_label (label), s_annot (annotation).
    fixture.create_test_agent_target(a_multi.id, s_target.id);
    fixture.create_test_agent_label(a_multi.id, "team-a".to_string());
    fixture.create_test_agent_annotation(a_multi.id, "tier".to_string(), "gold".to_string());

    // --- Acknowledgements (agent_events) --------------------------------
    // a_label acknowledges its latest object -> its pending count should drop.
    fixture.create_test_agent_event(&a_label, &s_label_latest, "DEPLOY", "SUCCESS", None);
    // a_multi acknowledges the s_target latest object only -> still pending on
    // s_label and s_annot.
    fixture.create_test_agent_event(&a_multi, &s_target_latest, "DEPLOY", "SUCCESS", None);
    // touch the other "latest" handles so they are not flagged unused.
    let _ = (&s_annot_latest,);

    let all_agents = [
        a_target.id,
        a_label.id,
        a_annot.id,
        a_multi.id,
        a_none.id,
    ];

    // ====================================================================
    // 1. pending_object_count: grouped == per-agent ground truth
    // ====================================================================
    let grouped_objects: HashMap<Uuid, i64> = fixture
        .dal
        .deployment_objects()
        .pending_counts_by_agent()
        .expect("pending_counts_by_agent failed")
        .into_iter()
        .collect();

    for agent_id in all_agents {
        let ground_truth = fixture
            .dal
            .deployment_objects()
            .get_target_state_for_agent(agent_id, false)
            .expect("get_target_state_for_agent failed")
            .len() as i64;
        let grouped = grouped_objects.get(&agent_id).copied().unwrap_or(0);
        assert_eq!(
            grouped, ground_truth,
            "pending_object_count mismatch for agent {agent_id}: grouped={grouped} ground_truth={ground_truth}"
        );
    }

    // Sanity: a_multi acknowledged s_target's latest, so it should be pending
    // on exactly s_label + s_annot = 2.
    assert_eq!(grouped_objects.get(&a_multi.id).copied().unwrap_or(0), 2);
    // a_label acknowledged its only object, so pending == 0 (absent from map).
    assert_eq!(grouped_objects.get(&a_label.id).copied().unwrap_or(0), 0);
    // a_none matches no stack.
    assert_eq!(grouped_objects.get(&a_none.id).copied().unwrap_or(0), 0);

    // ====================================================================
    // 2. pending_work_orders: grouped == per-agent ground truth
    // ====================================================================
    // Pending WO via hard target -> a_target.
    let wo_target = fixture.create_test_work_order("build", "wo-target");
    fixture.create_test_work_order_target(wo_target.id, a_target.id);
    fixture.create_test_work_order_target(wo_target.id, a_multi.id);

    // Pending WO via label -> a_label + a_multi.
    let wo_label = fixture.create_test_work_order("build", "wo-label");
    fixture.create_test_work_order_label(wo_label.id, "team-a");

    // Pending WO via annotation -> a_annot + a_multi.
    let wo_annot = fixture.create_test_work_order("build", "wo-annot");
    fixture.create_test_work_order_annotation(wo_annot.id, "tier", "gold");

    let grouped_wo_pending: HashMap<Uuid, i64> = fixture
        .dal
        .work_orders()
        .pending_counts_by_agent()
        .expect("work_orders pending_counts_by_agent failed")
        .into_iter()
        .collect();

    for agent_id in all_agents {
        let ground_truth = fixture
            .dal
            .work_orders()
            .list_pending_for_agent(agent_id, None)
            .expect("list_pending_for_agent failed")
            .len() as i64;
        let grouped = grouped_wo_pending.get(&agent_id).copied().unwrap_or(0);
        assert_eq!(
            grouped, ground_truth,
            "pending_work_orders mismatch for agent {agent_id}: grouped={grouped} ground_truth={ground_truth}"
        );
    }

    // a_multi matches all three pending work orders.
    assert_eq!(grouped_wo_pending.get(&a_multi.id).copied().unwrap_or(0), 3);

    // ====================================================================
    // 3. claimed_work_orders: grouped == manual count
    // ====================================================================
    // Claim wo_target for a_target -> moves it to CLAIMED.
    fixture
        .dal
        .work_orders()
        .claim(wo_target.id, a_target.id)
        .expect("claim failed");

    let grouped_claimed: HashMap<Uuid, i64> = fixture
        .dal
        .work_orders()
        .claimed_counts_by_agent()
        .expect("claimed_counts_by_agent failed")
        .into_iter()
        .collect();
    assert_eq!(grouped_claimed.get(&a_target.id).copied().unwrap_or(0), 1);
    assert_eq!(grouped_claimed.get(&a_multi.id).copied().unwrap_or(0), 0);

    // ====================================================================
    // 4. health status counts: grouped == per-agent list_by_agent counts
    // ====================================================================
    use brokkr_models::models::deployment_health::NewDeploymentHealth;
    for (status, obj) in [
        ("failing", &s_target_latest),
        ("degraded", &s_label_latest),
    ] {
        let h = NewDeploymentHealth::new(
            a_target.id,
            obj.id,
            status.to_string(),
            None,
            chrono::Utc::now(),
        )
        .expect("NewDeploymentHealth");
        fixture
            .dal
            .deployment_health()
            .upsert(&h)
            .expect("health upsert failed");
    }

    let mut grouped_failing: HashMap<Uuid, i64> = HashMap::new();
    let mut grouped_degraded: HashMap<Uuid, i64> = HashMap::new();
    for (agent_id, status, count) in fixture
        .dal
        .deployment_health()
        .status_counts_by_agent()
        .expect("status_counts_by_agent failed")
    {
        if status == "failing" {
            *grouped_failing.entry(agent_id).or_insert(0) += count;
        } else if status == "degraded" {
            *grouped_degraded.entry(agent_id).or_insert(0) += count;
        }
    }
    assert_eq!(grouped_failing.get(&a_target.id).copied().unwrap_or(0), 1);
    assert_eq!(grouped_degraded.get(&a_target.id).copied().unwrap_or(0), 1);

    // ====================================================================
    // 5. last_event_at: grouped present for agents with events
    // ====================================================================
    let last_event: HashMap<Uuid, chrono::DateTime<chrono::Utc>> = fixture
        .dal
        .agent_events()
        .last_event_at_by_agent()
        .expect("last_event_at_by_agent failed")
        .into_iter()
        .collect();
    assert!(last_event.contains_key(&a_label.id));
    assert!(last_event.contains_key(&a_multi.id));
    assert!(!last_event.contains_key(&a_none.id));
}
