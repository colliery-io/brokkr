/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Broker-side push helpers for the internal WebSocket channel.
//!
//! Call sites are post-commit hooks in the v1 API handlers. Each helper is
//! fire-and-forget: pushes that fail (agent not connected, lane full,
//! transient error) are logged and dropped. The agent's REST polling
//! fallback is the source of truth — WS is an optimisation that lowers
//! perceived latency in the common case.
//!
//! See [[BROKKR-A-0008]] for the "WS is a hint, REST is the source of truth"
//! invariant and the post-commit ordering requirement.

use std::sync::Arc;

use brokkr_wire::{AgentTarget, Stack, WorkOrder, WsMessage};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::dal::DAL;

use super::registry::{ConnectionRegistry, SendError};

/// Push a freshly-created [`WorkOrder`] to every agent that it reaches: by
/// a target, a label or an annotation, and for a work order that a generator
/// owns, only the agents registered with that generator
/// (`WorkOrdersDAL::agents_reached`, BROKKR-T-0412). The set comes from the
/// rule that the pending list and the claim use, so an agent gets the push
/// only for a work order that it can claim.
///
/// An agent that is not connected gets nothing here and finds the work order
/// on its next poll. An error on the read of the set is logged and swallowed
/// for the same reason.
pub fn push_work_order(registry: &Arc<ConnectionRegistry>, dal: &DAL, work_order: &WorkOrder) {
    let agents = match dal.work_orders().agents_reached(work_order.id) {
        Ok(agents) => agents,
        Err(e) => {
            warn!(work_order_id = %work_order.id, error = %e, "failed to fetch the agents of the work order for the push; relying on REST fallback");
            return;
        }
    };
    if agents.is_empty() {
        debug!(work_order_id = %work_order.id, "no agent has this work order; nothing to push");
        return;
    }
    for agent_id in agents {
        deliver(
            registry,
            agent_id,
            WsMessage::WorkOrder(work_order.clone()),
            "work_order",
        );
    }
}

/// Push a [`AgentTarget`] change to the affected agent. Used by
/// `add_target`. Remove-target is intentionally not pushed in v1: REST
/// polling surfaces the deletion on the next tick and the message body
/// for the wire is a created-target shape; signalling "your target X was
/// removed" cleanly is a v2 wire change.
pub fn push_target_changed(registry: &Arc<ConnectionRegistry>, target: &AgentTarget) {
    deliver(
        registry,
        target.agent_id,
        WsMessage::TargetChanged(target.clone()),
        "target_changed",
    );
}

/// Push a [`Stack`] change to every agent that serves it: by a target, a
/// label or an annotation (`StacksDAL::agents_serving`, BROKKR-T-0395). This
/// is the v1 mechanism for "your stack just got a new deployment object,
/// reconcile now" — handlers should call it after committing the change
/// that the agent cares about (new deployment object, label/annotation
/// edit, stack metadata update).
///
/// Errors fetching the agent list are logged and swallowed; the REST
/// polling fallback will catch up the affected agents.
pub fn push_stack_changed_to_targets(registry: &Arc<ConnectionRegistry>, dal: &DAL, stack: &Stack) {
    let agents = match dal.stacks().agents_serving(stack.id) {
        Ok(agents) => agents,
        Err(e) => {
            warn!(stack_id = %stack.id, error = %e, "failed to fetch the agents of the stack for the push; relying on REST fallback");
            return;
        }
    };
    if agents.is_empty() {
        debug!(stack_id = %stack.id, "no agent serves this stack; nothing to push");
        return;
    }
    for agent_id in agents {
        deliver(
            registry,
            agent_id,
            WsMessage::StackChanged(stack.clone()),
            "stack_changed",
        );
    }
}

fn deliver(registry: &Arc<ConnectionRegistry>, agent_id: Uuid, msg: WsMessage, kind: &'static str) {
    match registry.send_control(agent_id, msg) {
        Ok(()) => debug!(%agent_id, kind, "pushed WS message"),
        Err(SendError::NotConnected(_)) => {
            debug!(%agent_id, kind, "agent not connected; REST polling will pick this up");
        }
        Err(SendError::LaneUnavailable(_)) => {
            warn!(%agent_id, kind, "WS control lane full or closed; REST polling will pick this up");
        }
    }
}
