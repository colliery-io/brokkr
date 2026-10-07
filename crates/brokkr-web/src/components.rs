//! The few app-local pieces the console needs on top of Aurora. The sparkline,
//! the segmented health bar, the detail rows and the toasts that once lived
//! here are Aurora's now (`Sparkline`, `SegmentedBar`, `DetailList`/`KeyValue`,
//! `provide_toaster`/`ToastStack`).

use aurora_leptos::components::Dot;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

/// Map a Brokkr domain status string to a severity color. Covers the statuses
/// `status_color` doesn't (healthy/degraded/failing, delivered, …); falls back to muted.
pub fn sev(status: &str) -> &'static str {
    match status.to_ascii_lowercase().as_str() {
        "healthy" | "delivered" | "active" | "completed" | "success" | "succeeded" | "ok"
        | "ready" => token::OK,
        "degraded" | "pending" | "claimed" | "retrying" | "in_progress" | "warning" | "queued" => {
            token::GOLD
        }
        "failing" | "failed" | "failure" | "error" | "errored" | "unhealthy" | "inactive"
        | "offline" => token::BAD,
        _ => token::MUTED,
    }
}

/// Run `f` every `every` (the same argument order as `set_interval`) while the calling component is mounted.
///
/// `set_interval` alone outlives the view: after a switch to another view the
/// old timer kept firing and read the disposed view's signals, which panics
/// (the WASM `unreachable` seen when leaving the Fleet view). The timer is
/// cleared when the owner is cleaned up.
pub fn poll(f: impl Fn() + 'static, every: std::time::Duration) {
    if let Ok(handle) = set_interval_with_handle(f, every) {
        on_cleanup(move || handle.clear());
    }
}

/// A heartbeat is fresh (its status dot pulses) under this many seconds.
pub const FRESH_BEAT_SECS: i64 = 8;

/// A status dot that pulses while `live` (design/README.md: "pulses if active
/// and last beat < 8s"). The pulse is Aurora's `.cl-pulse`, which stops when
/// the system asks for less motion.
#[component]
pub fn LiveDot(#[prop(into)] color: String, live: bool) -> impl IntoView {
    if live {
        view! { <span class="brk-pulse cl-pulse"><Dot color=color glow=true /></span> }.into_any()
    } else {
        view! { <Dot color=color /> }.into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sev_maps_brokkr_statuses_to_hues() {
        assert_eq!(sev("Healthy"), token::OK);
        assert_eq!(sev("degraded"), token::GOLD);
        assert_eq!(sev("FAILING"), token::BAD);
        assert_eq!(sev("something-new"), token::MUTED);
    }
}

/// The docs site, for the "next step" links of empty states.
pub const DOCS: &str = "https://colliery-io.github.io/brokkr";

/// A link to another view, as a hash route plus a selection
/// (`#fleet/agent/<id>`), so the browser's back button works
/// (BROKKR-T-0337). `crate::app::parse_hash` reads them.
pub fn agent_href(id: &str) -> String {
    format!("#fleet/agent/{id}")
}

pub fn stack_href(id: &str) -> String {
    format!("#deployments/stack/{id}")
}

pub fn agent_events_href(id: &str) -> String {
    format!("#telemetry/agent/{id}")
}

/// Whether `id` names the agent `agent_id`: equal, or one a prefix of the
/// other. The console shows 8-character ids in places, and a link made from
/// one must still find the agent.
pub fn same_agent(agent_id: &str, id: &str) -> bool {
    !id.is_empty() && (agent_id == id || agent_id.starts_with(id) || id.starts_with(agent_id))
}

/// The agent's name from the fleet, or the first 8 characters of its id.
pub fn agent_name(fleet: &[crate::models::FleetAgentRecord], id: &str) -> String {
    fleet
        .iter()
        .find(|a| same_agent(&a.agent_id, id))
        .map(|a| a.name.clone())
        .unwrap_or_else(|| id.chars().take(8).collect())
}
