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

/// The text of a panel that waits for a new session.
pub const WAITING_FOR_SESSION: &str = "Waiting for a new session.";

/// The failed state of a panel. Every view uses this, not Aurora's
/// `ErrorState` directly.
///
/// While the broker refuses the console's token (session `Expired` or
/// `Refused`), the shell banner says what happened and offers a reload. A 401
/// or 403 in a panel is the same refusal, so the panel shows a short neutral
/// state, not a red "Not authorized" with the raw broker body. Any other error
/// (404, 5xx, network), or a 401/403 while the session works, shows Aurora's
/// `ErrorState` with its retry. The session can change after the error (for
/// example, a refused token that later works), so the choice is reactive.
#[component]
pub fn PanelError(
    error: aurora_leptos::tokens::ApiError,
    #[prop(optional)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    use aurora_leptos::components::{Empty, ErrorState};
    let session = crate::api::session();
    move || {
        if session.get().quiets(&error) {
            view! {
                <div class="brk-panel-wait" role="status">
                    <Empty message=WAITING_FOR_SESSION hint="Reload the page." />
                </div>
            }
            .into_any()
        } else {
            let error = error.clone();
            match on_retry {
                Some(cb) => view! { <ErrorState error=error on_retry=cb /> }.into_any(),
                None => view! { <ErrorState error=error /> }.into_any(),
            }
        }
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
