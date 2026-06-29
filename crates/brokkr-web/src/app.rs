//! Operator-console app shell (slice 1a): Aurora `AppShell` with the fixed
//! sidebar (brand + live status + nav + footer) and a per-view header carrying
//! a live clock + Live/Paused toggle. Views are placeholders; live data lands in
//! the later slices. Styled only via Aurora tokens (`var(--*)` / `token::*`).

use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use aurora_leptos::AuroraStyles;
use leptos::prelude::*;

/// Sidebar nav: (group label, [(view id, label)]). View ids are `&'static str`
/// so the route signal stays `Copy` (no clones in the click/style closures).
const NAV: &[(&str, &[(&str, &str)])] = &[
    (
        "Monitor",
        &[
            ("overview", "Overview"),
            ("fleet", "Fleet"),
            ("deployments", "Deployments"),
            ("telemetry", "Telemetry"),
        ],
    ),
    ("Operations", &[("jobs", "Work orders"), ("webhooks", "Webhooks")]),
    ("System", &[("system", "Broker health")]),
];

/// (title, subtitle) for a view id.
fn meta(id: &str) -> (&'static str, &'static str) {
    match id {
        "overview" => ("Overview", "command view"),
        "fleet" => ("Fleet", "agents by cluster"),
        "deployments" => ("Deployments", "per-stack health"),
        "telemetry" => ("Telemetry", "kube events · pod logs"),
        "jobs" => ("Work orders", "active · history"),
        "webhooks" => ("Webhooks", "subscriptions · deliveries"),
        "system" => ("Broker health", "metrics · connections"),
        _ => ("Brokkr", ""),
    }
}

fn now_hms() -> String {
    let d = js_sys::Date::new_0();
    format!(
        "{:02}:{:02}:{:02}",
        d.get_hours(),
        d.get_minutes(),
        d.get_seconds()
    )
}

#[component]
pub fn App() -> impl IntoView {
    let route = RwSignal::new("overview");
    // Live/Paused toggle (drives the live engine in a later slice).
    let live = RwSignal::new(String::from("Live"));
    // Wall-clock, ticking each second.
    let clock = RwSignal::new(now_hms());
    set_interval(
        move || clock.set(now_hms()),
        std::time::Duration::from_secs(1),
    );
    crate::components::provide_toasts();

    view! {
        <AuroraStyles/>
        <AppShell navbar=Box::new(move || view! { <Sidebar route=route /> }.into_any())>
            <Main route=route live=live clock=clock />
        </AppShell>
        <crate::components::Toaster/>
    }
}

#[component]
fn Sidebar(route: RwSignal<&'static str>) -> impl IntoView {
    let groups = NAV
        .iter()
        .map(|(group, items)| {
            let links = items
                .iter()
                .map(|(id, label)| {
                    let id = *id;
                    let style = move || {
                        let base = "padding:8px 10px;border-radius:8px;cursor:pointer;\
                                    font-size:13px;line-height:1;";
                        if route.get() == id {
                            format!(
                                "{base}color:var(--fg-bright);background:rgba(127,178,255,.10);\
                                 box-shadow:inset 2px 0 0 var(--ice);"
                            )
                        } else {
                            format!("{base}color:var(--muted);")
                        }
                    };
                    view! { <div style=style on:click=move |_| route.set(id)>{*label}</div> }
                })
                .collect_view();
            view! {
                <div style="display:flex;flex-direction:column;gap:2px;">
                    <div style="font:600 9.5px/1 var(--font-mono);letter-spacing:.12em;\
                                text-transform:uppercase;color:var(--faint);padding:0 10px 4px;">
                        {*group}
                    </div>
                    {links}
                </div>
            }
        })
        .collect_view();

    view! {
        <div style="display:flex;flex-direction:column;gap:14px;height:100%;">
            // Brand block
            <div style="display:flex;align-items:center;gap:9px;">
                <div style="width:26px;height:26px;border-radius:7px;background:var(--ice);\
                            display:flex;align-items:center;justify-content:center;flex:none;">
                    // hammer glyph
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none"
                         stroke="#0b0d10" stroke-width="2.1" stroke-linecap="round"
                         stroke-linejoin="round" aria-hidden="true">
                        <path d="M3 21l8-8" />
                        <path d="M12.5 4.5l7 7-3 3-7-7z" />
                    </svg>
                </div>
                <div style="display:flex;flex-direction:column;">
                    <span style="font:600 16px/1 var(--font-sans);color:var(--fg-bright);">
                        "Brokkr"
                    </span>
                    <span style="font:9.5px/1.4 var(--font-mono);letter-spacing:.13em;\
                                 text-transform:uppercase;color:var(--faint);">
                        "control plane"
                    </span>
                </div>
            </div>
            // Live status line
            <div style="display:flex;align-items:center;gap:7px;padding:0 2px;">
                <Dot color=token::OK glow=true />
                <span style="font:10.5px var(--font-mono);color:var(--muted);">
                    "broker ready"
                </span>
            </div>
            // Nav
            <div style="display:flex;flex-direction:column;gap:14px;">{groups}</div>
            // Footer
            <div style="margin-top:auto;border-top:1px solid var(--border-fainter,#15191f);\
                        padding-top:8px;display:flex;justify-content:space-between;\
                        font:10px var(--font-mono);color:var(--faint);">
                <span>"tenant · public"</span>
                <span>"wasm"</span>
            </div>
        </div>
    }
}

#[component]
fn Main(
    route: RwSignal<&'static str>,
    live: RwSignal<String>,
    clock: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div style="padding:20px 26px;max-width:1500px;margin:0 auto;">
            {move || {
                let (title, sub) = meta(route.get());
                view! {
                    <PageHeader
                        title=title
                        sub=sub
                        right=Box::new(move || {
                            view! {
                                <div style="display:flex;align-items:center;gap:12px;">
                                    <SegmentedControl
                                        value=live
                                        options=vec![String::from("Live"), String::from("Paused")]
                                    />
                                    <span style="font:12px var(--font-mono);color:var(--muted);\
                                                 font-variant-numeric:tabular-nums;">
                                        {move || clock.get()}
                                    </span>
                                </div>
                            }
                            .into_any()
                        })
                    />
                }
            }}
            <div style="margin-top:16px;">
                {move || match route.get() {
                    "overview" => view! { <crate::views::overview::OverviewView /> }.into_any(),
                    "fleet" => view! { <crate::views::fleet::FleetView /> }.into_any(),
                    "system" => view! { <crate::views::health::BrokerHealthView /> }.into_any(),
                    "jobs" => view! { <crate::views::work_orders::WorkOrdersView /> }.into_any(),
                    "webhooks" => view! { <crate::views::webhooks::WebhooksView /> }.into_any(),
                    "deployments" => view! { <crate::views::deployments::DeploymentsView /> }.into_any(),
                    "telemetry" => view! { <crate::views::telemetry::TelemetryView /> }.into_any(),
                    other => {
                        let (title, _) = meta(other);
                        view! {
                            <Panel title="Coming soon">
                                <Text dimmed=true>{format!("{title} — not yet implemented.")}</Text>
                            </Panel>
                        }
                        .into_any()
                    }
                }}
            </div>
        </div>
    }
}
