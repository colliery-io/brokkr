/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Surface tests — confirm the generated client exposes the operations we
//! care about, with the request types we expect. Does not actually talk to a
//! broker (integration tests live in the ergonomic wrapper crate added by
//! task BROKKR-T-0137 and consume a running broker).

use brokkr_client::Client;

#[test]
fn client_constructs() {
    let _c = Client::new("http://localhost:3000");
}

#[test]
fn client_exposes_baseline_operations() {
    // If any of these method references stop compiling, the spec lost a
    // contract that the agent and operators rely on.
    let c = Client::new("http://localhost:3000");
    let _ = c.list_agents();
    let _ = c.create_agent();
    let _ = c.get_agent();
    let _ = c.list_stacks();
    let _ = c.create_stack();
    let _ = c.list_work_orders();
    let _ = c.create_work_order();
    let _ = c.claim_work_order();
    let _ = c.complete_work_order();
    let _ = c.verify_pak();
    let _ = c.update_health_status();
    let _ = c.list_webhooks();
    let _ = c.get_pending_agent_webhooks();
}

#[test]
fn client_surfaces_typed_error_response() {
    use brokkr_client::types::ErrorResponse;
    // Just verifying the type exists and carries the canonical fields.
    let err = ErrorResponse {
        code: "agent_not_found".to_string(),
        message: "agent not found".to_string(),
        details: None,
    };
    assert_eq!(err.code, "agent_not_found");
}
