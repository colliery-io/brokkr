//! Webhooks view — subscription summaries from `GET /api/v1/webhooks`; click a card
//! for detail and that subscription's deliveries (`/webhooks/:id/deliveries`).
//! Below the cards, "Recent deliveries" shows the newest deliveries across all
//! subscriptions from `GET /api/v1/webhook-deliveries` (BROKKR-T-0264). NOTE: the
//! API redacts the URL (encrypted at rest) to `has_url`; the feed has no URL.

use crate::api;
use crate::components::{sev, PanelError, DOCS};
use crate::models::WebhookSummary;
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, SectionLabel};
use aurora_leptos::frame::{Card, Drawer};
use aurora_leptos::tokens::token;
use leptos::prelude::*;

fn event_chip(e: String) -> impl IntoView {
    view! { <Pill color=token::ICE>{e}</Pill> }
}

fn enabled(on: bool) -> (&'static str, &'static str) {
    if on {
        ("enabled", token::OK)
    } else {
        ("disabled", token::MUTED)
    }
}

/// Seconds since an RFC 3339 time, for `ago`. None for no time, or one the
/// browser cannot read.
fn age_secs(at: Option<&str>) -> Option<i64> {
    let ms = crate::views::overview::event_at(at)?;
    Some(((js_sys::Date::now() - ms) / 1000.0).max(0.0) as i64)
}

#[component]
pub fn WebhooksView() -> impl IntoView {
    let data = LocalResource::new(api::webhooks);
    let selected = RwSignal::new(None::<WebhookSummary>);
    let open = RwSignal::new(false);
    // Recent delivery attempts for the selected subscription.
    let deliveries = LocalResource::new(move || {
        let id = selected.get().map(|s| s.id.clone());
        async move {
            match id {
                Some(id) => Some(api::webhook_deliveries(&id).await),
                None => None,
            }
        }
    });

    // Newest deliveries across all subscriptions, kept fresh like the other
    // live panels.
    let recent = LocalResource::new(api::recent_webhook_deliveries);
    crate::components::poll(move || recent.refetch(), std::time::Duration::from_secs(10));
    // Open the drawer of the subscription a feed row belongs to, when the
    // subscription is still in the list.
    let open_subscription = move |id: &str| {
        if let Some(Ok(subs)) = data.get_untracked() {
            if let Some(s) = subs.into_iter().find(|s| s.id == id) {
                selected.set(Some(s));
                open.set(true);
            }
        }
    };

    view! {
        <Stack gap="md">
        {move || match data.get() {
            None => view! { <Loading label="loading webhooks" /> }.into_any(),
            Some(Err(e)) => view! {
                <PanelError error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(subs)) if subs.is_empty() => {
                view! {
                    <Empty
                        message="No webhook subscriptions."
                        hint="Create a subscription with the API to get events pushed to a URL."
                        href=format!("{DOCS}/how-to/webhooks.html")
                        link="How to configure webhooks"
                    />
                }.into_any()
            }
            Some(Ok(subs)) => {
                let cards = subs
                    .into_iter()
                    .map(|s| {
                        let (label, color) = enabled(s.enabled);
                        let chips = s.event_types.iter().cloned().map(event_chip).collect_view();
                        let has_url = s.has_url;
                        let s_sel = s.clone();
                        view! {
                            <Card
                                title=s.name.clone()
                                on_click=Callback::new(move |_| {
                                    selected.set(Some(s_sel.clone()));
                                    open.set(true);
                                })
                            >
                                <Stack gap="sm">
                                    <Group gap="sm">
                                        <Pill color=color>{label}</Pill>
                                        {(!has_url).then(|| view! {
                                            <span class="brk-note">"url redacted"</span>
                                        })}
                                    </Group>
                                    <Group gap="xs" wrap=true>{chips}</Group>
                                </Stack>
                            </Card>
                        }
                    })
                    .collect_view();
                view! { <div class="brk-cards">{cards}</div> }.into_any()
            }
        }}

        {move || match recent.get() {
            None => view! { <Loading label="loading recent deliveries" /> }.into_any(),
            // In its panel, so that the error names what failed: the
            // subscriptions above can still show.
            Some(Err(e)) => view! {
                <Panel title="Recent deliveries">
                    <PanelError error=e on_retry=Callback::new(move |_| { recent.refetch(); }) />
                </Panel>
            }
            .into_any(),
            Some(Ok(ds)) if ds.is_empty() => view! {
                <Panel title="Recent deliveries">
                    <Empty
                        message="No deliveries yet."
                        hint="A delivery appears when an event matches a subscription."
                    />
                </Panel>
            }
            .into_any(),
            Some(Ok(ds)) => {
                let rows = ds
                    .into_iter()
                    .map(|d| {
                        let note = d.last_error.clone().unwrap_or_default();
                        let age = crate::views::ago(age_secs(d.created_at.as_deref()));
                        let name = d
                            .subscription_name
                            .clone()
                            .unwrap_or_else(|| "(subscription removed)".into());
                        let sub_id = d.subscription_id.clone();
                        let name_title = name.clone();
                        let note_title = note.clone();
                        view! {
                            <TableRow on_click=Callback::new(move |_| open_subscription(&sub_id))>
                                <td class="brk-ellipsis" title=name_title>{name}</td>
                                <td><Pill color=sev(&d.status)>{d.status.clone()}</Pill></td>
                                <td class="brk-muted brk-ellipsis">{d.event_type.clone()}</td>
                                <td class="brk-muted cl-num">{d.attempts.to_string()}</td>
                                <td class="brk-muted cl-num">{age}</td>
                                <td class="brk-faint brk-ellipsis" title=note_title>{note}</td>
                            </TableRow>
                        }
                    })
                    .collect_view();
                view! {
                    <Panel title="Recent deliveries">
                        <Table mono=true fixed=true label="Recent deliveries"
                            widths=vec![
                                "22%".into(), "12%".into(), "22%".into(),
                                "10%".into(), "12%".into(), "22%".into(),
                            ]>
                            <thead><tr>
                                <th>"Subscription"</th><th>"Status"</th><th>"Event"</th>
                                <th class="cl-num">"Attempts"</th><th class="cl-num">"Age"</th><th>"Last error"</th>
                            </tr></thead>
                            <tbody>{rows}</tbody>
                        </Table>
                    </Panel>
                }
                .into_any()
            }
        }}
        </Stack>

        <Drawer open=open title="Webhook subscription" size="lg">
            {move || match selected.get() {
                None => ().into_any(),
                Some(s) => {
                    let (label, color) = enabled(s.enabled);
                    let chips = s.event_types.iter().cloned().map(event_chip).collect_view();
                    view! {
                        <Stack gap="md">
                            <span class="brk-detail-title">{s.name.clone()}</span>
                            <DetailList mono=true>
                                <KeyValue label="status"><Pill color=color>{label}</Pill></KeyValue>
                                <KeyValue label="id">{s.id.clone()}</KeyValue>
                                <KeyValue label="url">{if s.has_url { "configured (redacted)" } else { "—" }}</KeyValue>
                            </DetailList>
                            <Group gap="xs" wrap=true>{chips}</Group>
                            <SectionLabel label="recent deliveries" />
                            {move || match deliveries.get() {
                                None | Some(None) => view! { <Loading label="loading deliveries" /> }.into_any(),
                                Some(Some(Err(_))) => view! {
                                    <span class="brk-note">"deliveries unavailable"</span>
                                }.into_any(),
                                Some(Some(Ok(ds))) if ds.is_empty() => view! {
                                    <span class="brk-note">"no deliveries yet"</span>
                                }.into_any(),
                                Some(Some(Ok(ds))) => {
                                    let rows = ds.into_iter().take(8).map(|d| {
                                        let err = d.last_error.unwrap_or_default();
                                        let note = if err.is_empty() { format!("{}\u{00d7}", d.attempts) } else { err };
                                        view! {
                                            <TableRow>
                                                <td><Pill color=sev(&d.status)>{d.status}</Pill></td>
                                                <td class="brk-muted">{d.event_type}</td>
                                                <td class="brk-faint brk-ellipsis">{note}</td>
                                            </TableRow>
                                        }
                                    }).collect_view();
                                    view! {
                                        <Table mono=true fixed=true label="Recent deliveries"
                                            widths=vec!["28%".into(), "32%".into(), "40%".into()]>
                                            <tbody>{rows}</tbody>
                                        </Table>
                                    }.into_any()
                                }
                            }}
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Drawer>
    }
}
