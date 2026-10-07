//! Operator-console app shell: Aurora `AppShell` with a top bar (brand, broker
//! live state, clock, theme toggle) and a `SideNav` sidebar (groups, counts,
//! tenant scope). Styled only via Aurora components and tokens, plus the few
//! `brk-` classes in `style/brokkr.css`.
//!
//! The header once carried a Live/Paused `SegmentedControl`. It drove nothing —
//! no view ever read the signal — and in a deployment tool a global "Paused"
//! reads as *the fleet is paused*, which it never was. Removed in
//! BROKKR-T-0322; pausing is now a real, per-agent, admin-authorized action in
//! the Fleet view's agent drawer, where it has a target and a credential.

use aurora_leptos::components::*;
use aurora_leptos::data::{LiveIndicator, LiveState};
use aurora_leptos::frame::{
    provide_toaster, AppShell, PageHeader, SideNav, SideNavGroup, SideNavLink, ToastStack,
};
use aurora_leptos::theme::{provide_theme, ThemeToggle};
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
    (
        "Operations",
        &[("jobs", "Work orders"), ("webhooks", "Webhooks")],
    ),
    (
        "System",
        &[("system", "Broker health"), ("tenants", "Tenants")],
    ),
];

/// (title, subtitle) for a view id.
fn meta(id: &str) -> (&'static str, &'static str) {
    match id {
        "overview" => ("Overview", "command view"),
        "fleet" => ("Fleet", "agents by cluster"),
        "deployments" => ("Deployments", "per-stack health"),
        "telemetry" => ("Telemetry", "agent events · kube events · pod logs"),
        "jobs" => ("Work orders", "active · history"),
        "webhooks" => ("Webhooks", "subscriptions · deliveries"),
        "system" => ("Broker health", "metrics · connections"),
        "tenants" => ("Tenants", "tenant list · PAK minting"),
        _ => ("Brokkr", ""),
    }
}

/// A selection carried in the URL fragment (`#fleet/agent/<id>`), so a link
/// from another view opens the right drawer and the back button returns
/// (BROKKR-T-0337). `kind` is `agent` or `stack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub kind: String,
    pub id: String,
}

pub type SelectionSignal = RwSignal<Option<Selection>>;

/// Read the app-wide selection signal from context (installed by [`App`]).
pub fn use_selection() -> SelectionSignal {
    use_context::<SelectionSignal>().expect("selection signal provided at app root")
}

/// The view id and the selection for a URL fragment: `#fleet` is ("fleet",
/// None), `#fleet/agent/abc` is ("fleet", agent abc), and an unknown view is
/// the overview with no selection. The nav links are plain `#id` anchors, so
/// a reload or a shared link opens the same view.
pub fn parse_hash(hash: &str) -> (&'static str, Option<Selection>) {
    let want = hash.trim_start_matches('#');
    let mut parts = want.splitn(3, '/');
    let view = parts.next().unwrap_or("");
    let route = NAV
        .iter()
        .flat_map(|(_, items)| items.iter())
        .map(|(id, _)| *id)
        .find(|id| *id == view);
    let Some(route) = route else {
        return ("overview", None);
    };
    let selection = match (parts.next(), parts.next()) {
        (Some(kind), Some(id)) if !kind.is_empty() && !id.is_empty() => Some(Selection {
            kind: kind.to_string(),
            id: id.to_string(),
        }),
        _ => None,
    };
    (route, selection)
}

fn route_for_hash(hash: &str) -> &'static str {
    parse_hash(hash).0
}

fn current_hash() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
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

/// The selected tenant scope (BROKKR-I-0032): `None` = all tenants, `Some(id)`
/// = only resources under that named PAK (generator). Provided as context at
/// the app root; data views read it and pass it to the scoped API fetches.
pub type ScopeSignal = RwSignal<Option<String>>;

/// Read the app-wide scope signal from context (installed by [`App`]).
pub fn use_scope() -> ScopeSignal {
    use_context::<ScopeSignal>().expect("scope signal provided at app root")
}

const SCOPE_STORAGE_KEY: &str = "brokkr_scope";

fn load_scope() -> Option<String> {
    let ls = web_sys::window()?.local_storage().ok()??;
    ls.get_item(SCOPE_STORAGE_KEY)
        .ok()?
        .filter(|s| !s.is_empty())
}

fn save_scope(scope: &Option<String>) {
    let Some(ls) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    match scope {
        Some(id) => {
            let _ = ls.set_item(SCOPE_STORAGE_KEY, id);
        }
        None => {
            let _ = ls.remove_item(SCOPE_STORAGE_KEY);
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_theme();
    provide_toaster();

    let route = RwSignal::new(route_for_hash(&current_hash()));
    let selection: SelectionSignal = RwSignal::new(parse_hash(&current_hash()).1);
    provide_context(selection);
    // Back / forward and hand-typed fragments move the view too.
    let _ = window_event_listener(leptos::ev::hashchange, move |_| {
        let (next, sel) = parse_hash(&current_hash());
        selection.set(sel);
        // The link's own click has set it already; setting the same view
        // again would re-mount it.
        if route.get_untracked() != next {
            route.set(next);
        }
    });

    // Wall-clock, ticking each second.
    let clock = RwSignal::new(now_hms());
    crate::components::poll(
        move || clock.set(now_hms()),
        std::time::Duration::from_secs(1),
    );

    // Tenant scope (BROKKR-I-0032): restored from localStorage, persisted on
    // change, available to every data view via context.
    let scope: ScopeSignal = RwSignal::new(load_scope());
    provide_context(scope);
    Effect::new(move |_| save_scope(&scope.get()));

    // Shell-level reads: the nav counts (design/README.md: Fleet = agent
    // count, Work orders = active count) and the broker's live state.
    let fleet = LocalResource::new(move || crate::api::fleet(scope.get()));
    let work_orders = LocalResource::new(crate::api::work_orders);
    crate::components::poll(
        move || {
            fleet.refetch();
            work_orders.refetch();
        },
        std::time::Duration::from_secs(10),
    );
    let counts = NavCounts {
        fleet: Signal::derive(move || fleet.get().and_then(|r| r.ok()).map(|a| a.len())),
        // The active list is admin-gated: with no access there is no badge.
        work_orders: Signal::derive(move || {
            work_orders
                .get()
                .and_then(|r| r.ok())
                .map(|w| w.iter().filter(|w| w.is_active()).count())
        }),
    };
    // Every request with the injected token feeds this, so a refusal after a
    // success (a broker restart, or a different replica) and a token refused
    // from the first request show in the shell.
    let session = RwSignal::new(crate::api::Session::default());
    crate::api::watch_session(session);
    let session: Signal<crate::api::Session> = session.into();
    let live = Signal::derive(move || match fleet.get() {
        _ if session.get().refused() => LiveState::Offline,
        None => LiveState::Connecting,
        Some(Ok(_)) => LiveState::Live,
        Some(Err(_)) => LiveState::Offline,
    });

    view! {
        <AuroraStyles/>
        <AppShell
            brand=std::sync::Arc::new(|| view! { <Brand /> }.into_any())
            header=Box::new(move || view! { <TopBar live=live session=session clock=clock /> }.into_any())
            navbar=Box::new(move || view! { <Sidebar route=route counts=counts /> }.into_any())
        >
            <SessionBanner session=session />
            <Main route=route />
        </AppShell>
        <ToastStack />
    }
}

/// The product mark: a hammer on an ice square, the name, and "control plane".
#[component]
fn Brand() -> impl IntoView {
    view! {
        <div class="brk-brand">
            <div class="brk-brand__mark">
                // hammer glyph, drawn in the mark's text colour (--on-accent)
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none"
                     stroke="currentColor" stroke-width="2.1" stroke-linecap="round"
                     stroke-linejoin="round" aria-hidden="true">
                    <path d="M3 21l8-8" />
                    <path d="M12.5 4.5l7 7-3 3-7-7z" />
                </svg>
            </div>
            <div class="brk-brand__text">
                <span class="brk-brand__name">"Brokkr"</span>
                <span class="brk-brand__sub">"control plane"</span>
            </div>
        </div>
    }
}

/// The right side of the top bar: the broker's live state, the clock and the
/// theme toggle. When the broker refused the token, the broker is reachable,
/// so the indicator says "session expired" or "token refused", not "broker
/// unreachable". A network failure with no answer does not change the session,
/// so it still says "broker unreachable".
#[component]
fn TopBar(
    live: Signal<LiveState>,
    session: Signal<crate::api::Session>,
    clock: RwSignal<String>,
) -> impl IntoView {
    use crate::api::Session;
    view! {
        <div class="brk-topbar">
            {move || {
                let offline = match session.get() {
                    Session::Expired => "session expired",
                    Session::Refused => "token refused",
                    Session::Unknown | Session::Valid => "broker unreachable",
                };
                view! {
                    <LiveIndicator
                        state=live
                        live_label="broker ready"
                        connecting_label="connecting"
                        offline_label=offline
                    />
                }
            }}
            <span class="brk-clock">{move || clock.get()}</span>
            <ThemeToggle />
        </div>
    }
}

/// The text of the session banner.
const SESSION_EXPIRED: &str =
    "The broker restarted, or this page reached a different replica. Reload to get a new session.";

/// The text of the banner when the broker refused the token from the first
/// request. Each broker process makes its own token and puts it in the page
/// that it serves (`brokkr-broker/src/utils/ui_pak.rs`), and the page is not
/// cached (`Cache-Control: no-store`), so a reload gets a token again.
const TOKEN_REFUSED: &str = "The broker refused the token of this console. Reload the page. \
     If this does not help, open the console directly at the address of one broker. \
     Each broker process gives a different token to the page that it serves.";

/// One banner for the whole console when the broker refused the token, above
/// every view, so the views' own "Not authorized" errors have an explanation
/// and a fix. A reload gets the page again, with the token of the broker that
/// serves it.
#[component]
fn SessionBanner(session: Signal<crate::api::Session>) -> impl IntoView {
    use crate::api::Session;
    let reload = Callback::new(|_| {
        if let Some(w) = web_sys::window() {
            let _ = w.location().reload();
        }
    });
    move || {
        let (title, text) = match session.get() {
            Session::Expired => ("Session expired", SESSION_EXPIRED),
            Session::Refused => ("Token refused", TOKEN_REFUSED),
            Session::Unknown | Session::Valid => return None,
        };
        Some(view! {
            <div class="brk-page brk-session">
                <Alert title=title color=token::GOLD>
                    <div class="brk-session__body">
                        <span class="brk-text">{text}</span>
                        <Button on_click=reload>"Reload"</Button>
                    </div>
                </Alert>
            </div>
        })
    }
}

#[derive(Clone, Copy)]
struct NavCounts {
    fleet: Signal<Option<usize>>,
    work_orders: Signal<Option<usize>>,
}

#[component]
fn Sidebar(route: RwSignal<&'static str>, counts: NavCounts) -> impl IntoView {
    let groups = NAV
        .iter()
        .map(|(group, items)| {
            let links = items
                .iter()
                .map(|(id, label)| {
                    let id = *id;
                    let count: Signal<Option<usize>> = match id {
                        "fleet" => counts.fleet,
                        "jobs" => counts.work_orders,
                        _ => Signal::stored(None),
                    };
                    let count_color = if id == "jobs" { token::GOLD } else { "" };
                    view! {
                        <SideNavLink
                            href=format!("#{id}")
                            active=move || route.get() == id
                            count=count
                            count_color=count_color
                            on_click=Callback::new(move |_| {
                                if route.get_untracked() != id {
                                    route.set(id);
                                }
                            })
                        >
                            {*label}
                        </SideNavLink>
                    }
                })
                .collect_view();
            view! { <SideNavGroup label=*group>{links}</SideNavGroup> }
        })
        .collect_view();

    view! {
        <SideNav footer=Box::new(|| view! {
            <div class="brk-sidefoot">
                // Tenant scope selector (BROKKR-I-0032)
                <ScopeSelector />
                <div class="brk-sidefoot__line">
                    <span>"tenant · public"</span>
                    <span>"wasm"</span>
                </div>
            </div>
        }.into_any())>
            {groups}
        </SideNav>
    }
}

/// Tenant scope selector (BROKKR-I-0032): "All" + one entry per named PAK from
/// `GET /api/v1/paks`. Hidden entirely on single-tenant installs (empty list)
/// or when the listing fails (the views still work unscoped). A `Select`
/// rather than a `SegmentedControl`: the sidebar can't fit segments for
/// arbitrary tenant names, and `value=id` keeps duplicate names unambiguous.
#[component]
fn ScopeSelector() -> impl IntoView {
    let scope = use_scope();
    let paks = LocalResource::new(crate::api::paks);
    // Aurora's `Select` binds a string; "" is "All".
    let choice = RwSignal::new(scope.get_untracked().unwrap_or_default());
    Effect::new(move |_| choice.set(scope.get().unwrap_or_default()));

    view! {
        {move || match paks.get() {
            Some(Ok(list)) if !list.is_empty() => {
                // A stored scope whose tenant no longer exists falls back to All.
                if let Some(current) = scope.get_untracked() {
                    if !list.iter().any(|p| p.id == current) {
                        scope.set(None);
                    }
                }
                let pairs = std::iter::once((String::new(), "All".to_string()))
                    .chain(list.iter().map(|p| (p.id.clone(), p.name.clone())))
                    .collect::<Vec<_>>();
                view! {
                    <Select
                        label="Tenant"
                        option_pairs=pairs
                        value=choice
                        on_change=Callback::new(move |v: String| {
                            scope.set((!v.is_empty()).then_some(v));
                        })
                    />
                }
                .into_any()
            }
            _ => ().into_any(),
        }}
    }
}

#[component]
fn Main(route: RwSignal<&'static str>) -> impl IntoView {
    view! {
        <div class="brk-page">
            {move || {
                let (title, sub) = meta(route.get());
                view! { <PageHeader title=title sub=sub /> }
            }}
            {move || match route.get() {
                "overview" => view! { <crate::views::overview::OverviewView /> }.into_any(),
                "fleet" => view! { <crate::views::fleet::FleetView /> }.into_any(),
                "system" => view! { <crate::views::health::BrokerHealthView /> }.into_any(),
                "jobs" => view! { <crate::views::work_orders::WorkOrdersView /> }.into_any(),
                "webhooks" => view! { <crate::views::webhooks::WebhooksView /> }.into_any(),
                "deployments" => view! { <crate::views::deployments::DeploymentsView /> }.into_any(),
                "telemetry" => view! { <crate::views::telemetry::TelemetryView /> }.into_any(),
                "tenants" => view! { <crate::views::tenants::TenantsView /> }.into_any(),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fragment_selects_its_view() {
        assert_eq!(route_for_hash("#fleet"), "fleet");
        assert_eq!(route_for_hash("system"), "system");
        assert_eq!(route_for_hash(""), "overview");
        assert_eq!(route_for_hash("#nope"), "overview");
    }

    #[test]
    fn a_fragment_can_carry_a_selection() {
        let (route, sel) = parse_hash("#fleet/agent/1b9d6bcd");
        assert_eq!(route, "fleet");
        assert_eq!(
            sel,
            Some(Selection {
                kind: "agent".into(),
                id: "1b9d6bcd".into()
            })
        );
        assert_eq!(
            parse_hash("#deployments/stack/s1").1.map(|s| s.kind),
            Some("stack".into())
        );
        assert_eq!(
            parse_hash("#fleet/agent/").1,
            None,
            "an empty id is no selection"
        );
        assert_eq!(parse_hash("#fleet/agent").1, None);
        assert_eq!(parse_hash("#nope/agent/x"), ("overview", None));
    }

    #[test]
    fn every_nav_view_has_a_title() {
        for (_, items) in NAV {
            for (id, label) in *items {
                assert_eq!(meta(id).0, *label, "title of {id}");
            }
        }
    }
}
