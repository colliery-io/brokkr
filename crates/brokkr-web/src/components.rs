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
/// and last beat < 8s"). The pulse is Aurora's `cl-pulse` keyframes; it stops
/// when the system asks for less motion (style/brokkr.css).
#[component]
pub fn LiveDot(#[prop(into)] color: String, live: bool) -> impl IntoView {
    if live {
        view! { <span class="brk-pulse"><Dot color=color glow=true /></span> }.into_any()
    } else {
        view! { <Dot color=color /> }.into_any()
    }
}

/// An indeterminate progress bar (the design's `brk-sweep`), for work that has
/// no percentage, such as a diagnostic the agent is collecting.
#[component]
pub fn Sweep(#[prop(into)] label: String) -> impl IntoView {
    let aria = label.clone();
    view! {
        <div role="status" aria-label=aria>
            <div class="brk-sweep" aria-hidden="true"></div>
            <div class="brk-note">{label}</div>
        </div>
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
