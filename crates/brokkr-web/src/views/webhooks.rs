//! Webhooks view — subscription summaries from `GET /api/v1/webhooks`; click a card
//! for detail. NOTE: the API redacts the URL (encrypted at rest) to `has_url`, and
//! delivery history is per-subscription (`/webhooks/:id/deliveries`), not a global
//! feed — so this shows subscriptions; a global "recent deliveries" panel needs a
//! broker enhancement (logged on the task).

use crate::api;
use crate::components::{sev, DOCS};
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

    view! {
        {move || match data.get() {
            None => view! { <Loading label="loading webhooks" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
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
