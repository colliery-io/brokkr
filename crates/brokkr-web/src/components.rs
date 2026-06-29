//! App-local components the design handoff needs but `aurora-leptos` does not ship
//! (BROKKR-T-0255): an SVG sparkline, a segmented health bar, a right-anchored
//! slide-over; plus the toast system (BROKKR-T-0256). Built on Aurora tokens.

use leptos::prelude::*;
use std::cell::Cell;
use std::time::Duration;

/// SVG area sparkline over a value series (rendered via `inner_html` to sidestep
/// leptos SVG-attr casing). `color` is any CSS color (a `token::*` or `var(--*)`).
#[component]
pub fn Sparkline(#[prop(into)] values: Vec<f64>, #[prop(into)] color: String) -> impl IntoView {
    let (w, h) = (240.0_f64, 52.0_f64);
    let n = values.len().max(2);
    let max = values.iter().copied().fold(f64::MIN, f64::max).max(1.0);
    let min = values.iter().copied().fold(f64::MAX, f64::min).min(0.0);
    let range = (max - min).max(1.0);
    let pts: Vec<String> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f64 / (n as f64 - 1.0) * w;
            let y = h - ((v - min) / range) * h;
            format!("{x:.1},{y:.1}")
        })
        .collect();
    let line = pts.join(" ");
    let area = format!("0,{h} {line} {w},{h}");
    let svg = format!(
        "<svg viewBox=\"0 0 {w} {h}\" width=\"100%\" height=\"52\" preserveAspectRatio=\"none\">\
         <polygon points=\"{area}\" fill=\"color-mix(in srgb, {color} 12%, transparent)\"/>\
         <polyline points=\"{line}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"1.6\"/></svg>"
    );
    view! { <div inner_html=svg></div> }
}

/// Proportional healthy/degraded/failing/offline bar (handoff fleet-by-cluster).
#[component]
pub fn SegmentedHealthBar(
    #[prop(default = 0)] healthy: usize,
    #[prop(default = 0)] degraded: usize,
    #[prop(default = 0)] failing: usize,
    #[prop(default = 0)] offline: usize,
) -> impl IntoView {
    let total = (healthy + degraded + failing + offline).max(1) as f64;
    let seg = |n: usize, color: &str| {
        format!(
            "<span style=\"width:{:.1}%;background:{color};display:block;\"></span>",
            n as f64 / total * 100.0
        )
    };
    let html = format!(
        "{}{}{}{}",
        seg(healthy, "var(--ok)"),
        seg(degraded, "var(--gold)"),
        seg(failing, "var(--bad)"),
        seg(offline, "var(--border-control)")
    );
    view! {
        <div
            style="display:flex;height:7px;border-radius:4px;overflow:hidden;background:var(--inset);"
            inner_html=html
        ></div>
    }
}

/// Right-anchored slide-over panel + scrim. Always rendered; visibility is toggled
/// by `open` (so `children` runs once and may contain its own reactive closures).
#[component]
pub fn SlideOver(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            style=move || format!(
                "position:fixed;inset:0;background:rgba(6,8,11,.55);z-index:50;{}",
                if open.get() { "" } else { "display:none;" }
            )
            on:click=move |_| open.set(false)
        ></div>
        <div style=move || format!(
            "position:fixed;top:0;right:0;width:430px;max-width:92vw;height:100vh;\
             background:var(--panel);border-left:1px solid var(--border);\
             box-shadow:-24px 0 60px rgba(0,0,0,.5);z-index:51;padding:18px 20px;\
             overflow-y:auto;transition:transform .18s ease;transform:translateX({});",
            if open.get() { "0" } else { "100%" }
        )>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;">
                <span style="font:600 16px var(--font-mono);color:var(--fg-bright);">{title}</span>
                <span style="cursor:pointer;color:var(--muted);font-size:16px;"
                      on:click=move |_| open.set(false)>"\u{2715}"</span>
            </div>
            {children()}
        </div>
    }
}

// ---- toasts --------------------------------------------------------------

#[derive(Clone)]
pub struct Toast {
    pub id: u32,
    pub msg: String,
    pub color: String,
}

#[derive(Clone, Copy)]
pub struct ToastBus(pub RwSignal<Vec<Toast>>);

thread_local! {
    static NEXT_ID: Cell<u32> = const { Cell::new(1) };
}

/// Install the toast bus at the app root. Call once.
pub fn provide_toasts() {
    provide_context(ToastBus(RwSignal::new(Vec::new())));
}

/// Push a toast onto a specific bus (auto-dismisses after 3.4s). Use this from
/// async handlers where `use_context` is unavailable — capture the bus first.
pub fn toast(bus: ToastBus, msg: impl Into<String>, color: &'static str) {
    let id = NEXT_ID.with(|c| {
        let v = c.get();
        c.set(v.wrapping_add(1));
        v
    });
    bus.0.update(|v| {
        v.push(Toast {
            id,
            msg: msg.into(),
            color: color.into(),
        })
    });
    set_timeout(
        move || bus.0.update(|v| v.retain(|t| t.id != id)),
        Duration::from_millis(3400),
    );
}

/// Push a toast via the context bus (call from a reactive scope).
pub fn push_toast(msg: impl Into<String>, color: &'static str) {
    if let Some(bus) = use_context::<ToastBus>() {
        toast(bus, msg, color);
    }
}

/// Bottom-right toast stack. Mount once near the app root.
#[component]
pub fn Toaster() -> impl IntoView {
    let bus = use_context::<ToastBus>().expect("ToastBus provided");
    view! {
        <div style="position:fixed;bottom:18px;right:18px;display:flex;flex-direction:column;\
                    gap:9px;z-index:60;">
            <For each=move || bus.0.get() key=|t| t.id let:t>
                <div style=format!(
                    "background:var(--control);border:1px solid var(--border-control);\
                     border-left:3px solid {};border-radius:9px;padding:10px 14px;min-width:230px;\
                     box-shadow:0 12px 30px rgba(0,0,0,.4);font:12px var(--font-sans);color:var(--fg);",
                    t.color
                )>{t.msg.clone()}</div>
            </For>
        </div>
    }
}
