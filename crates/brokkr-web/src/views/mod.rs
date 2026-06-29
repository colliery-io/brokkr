//! Operator-console views. Each is a read-only surface bound to a broker API and
//! wrapped in Aurora `Loading`/`Empty`/`ErrorState`.

pub mod deployments;
pub mod fleet;
pub mod health;
pub mod telemetry;
pub mod webhooks;
pub mod work_orders;

use leptos::prelude::*;

/// Human "N ago" from a seconds count.
pub fn ago(secs: Option<i64>) -> String {
    match secs {
        None => "—".into(),
        Some(s) if s < 5 => "now".into(),
        Some(s) if s < 60 => format!("{s}s ago"),
        Some(s) if s < 3600 => format!("{}m ago", s / 60),
        Some(s) if s < 86400 => format!("{}h ago", s / 3600),
        Some(s) => format!("{}d ago", s / 86400),
    }
}

/// A KPI card: mono uppercase label + big tabular value colored by meaning.
/// `color` is any CSS color (a `token::*` hex or a `var(--*)`).
#[component]
pub fn Kpi(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    view! {
        <div style="background:var(--panel);border:1px solid var(--border);border-radius:10px;\
                    padding:13px 15px;min-width:120px;">
            <div style="font:600 10px var(--font-mono);letter-spacing:.04em;text-transform:uppercase;\
                        color:var(--muted);margin-bottom:4px;">{label}</div>
            <div style=format!(
                "font:600 26px var(--font-mono);color:{color};\
                 font-variant-numeric:tabular-nums;line-height:1;"
            )>{value}</div>
        </div>
    }
}
