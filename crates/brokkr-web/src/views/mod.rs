//! Operator-console views. Each is a read-only surface bound to a broker API and
//! wrapped in Aurora `Loading`/`Empty`/`ErrorState`.

pub mod deployments;
pub mod fleet;
pub mod health;
pub mod overview;
pub mod telemetry;
pub mod tenants;
pub mod webhooks;
pub mod work_orders;

use aurora_leptos::data::format_relative;

/// Human "N ago" from a seconds count (Aurora's wording: "just now", "3m ago").
pub fn ago(secs: Option<i64>) -> String {
    match secs {
        None => "\u{2014}".into(),
        Some(s) => format_relative(s as f64 * 1000.0),
    }
}

#[cfg(test)]
mod tests {
    use super::ago;

    #[test]
    fn ago_uses_aurora_wording() {
        assert_eq!(ago(None), "\u{2014}");
        assert_eq!(ago(Some(2)), "just now");
        assert_eq!(ago(Some(42)), "42s ago");
        assert_eq!(ago(Some(900)), "15m ago");
    }
}
