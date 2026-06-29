//! Webhooks view — subscription summaries from `GET /api/v1/webhooks`; click a card
//! for detail. NOTE: the API redacts the URL (encrypted at rest) to `has_url`, and
//! delivery history is per-subscription (`/webhooks/:id/deliveries`), not a global
//! feed — so this shows subscriptions; a global "recent deliveries" panel needs a
//! broker enhancement (logged on the task).

use crate::api;
use crate::components::DetailRow;
use crate::models::WebhookSummary;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use crate::components::sev;
use leptos::prelude::*;

fn event_chip(e: String) -> impl IntoView {
    view! {
        <span style="font:9.5px var(--font-mono);color:var(--ice);background:rgba(127,178,255,.1);\
                     padding:2px 6px;border-radius:6px;">{e}</span>
    }
}

#[component]
pub fn WebhooksView() -> impl IntoView {
    let data = LocalResource::new(|| api::webhooks());
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
                view! { <Empty message="No webhook subscriptions." /> }.into_any()
            }
            Some(Ok(subs)) => {
                let cards = subs
                    .into_iter()
                    .map(|s| {
                        let (label, color) = if s.enabled {
                            ("enabled", token::OK)
                        } else {
                            ("disabled", token::MUTED)
                        };
                        let chips = s.event_types.iter().cloned().map(event_chip).collect_view();
                        let has_url = s.has_url;
                        let s_sel = s.clone();
                        view! {
                            <div style="min-width:260px;flex:1;cursor:pointer;" on:click=move |_| {
                                selected.set(Some(s_sel.clone()));
                                open.set(true);
                            }>
                                <Panel title=s.name.clone()>
                                    <Stack gap="sm">
                                        <Group gap="sm">
                                            <Pill color=color>{label}</Pill>
                                            {(!has_url).then(|| view! {
                                                <span style="font:9.5px var(--font-mono);\
                                                             color:var(--faint);">"url redacted"</span>
                                            })}
                                        </Group>
                                        <Group gap="xs" wrap=true>{chips}</Group>
                                    </Stack>
                                </Panel>
                            </div>
                        }
                    })
                    .collect_view();
                view! { <Group gap="md" wrap=true>{cards}</Group> }.into_any()
            }
        }}

        <Modal open=open title="Webhook subscription">
            {move || match selected.get() {
                None => ().into_any(),
                Some(s) => {
                    let (label, color) = if s.enabled { ("enabled", token::OK) } else { ("disabled", token::MUTED) };
                    let chips = s.event_types.iter().cloned().map(event_chip).collect_view();
                    view! {
                        <Stack gap="md">
                            <span style="font:600 15px var(--font-mono);color:var(--fg-bright);">{s.name.clone()}</span>
                            <div>
                                <DetailRow label="status"><Pill color=color>{label}</Pill></DetailRow>
                                <DetailRow label="id">{s.id.clone()}</DetailRow>
                                <DetailRow label="url">{if s.has_url { "configured (redacted)" } else { "—" }}</DetailRow>
                            </div>
                            <Group gap="xs" wrap=true>{chips}</Group>
                            <span style="font:600 10px var(--font-mono);text-transform:uppercase;\
                                         letter-spacing:.05em;color:var(--muted);">"recent deliveries"</span>
                            {move || match deliveries.get() {
                                None | Some(None) => view! { <Loading label="loading deliveries" /> }.into_any(),
                                Some(Some(Err(_))) => view! {
                                    <span style="font:11px var(--font-mono);color:var(--faint);">"deliveries unavailable"</span>
                                }.into_any(),
                                Some(Some(Ok(ds))) if ds.is_empty() => view! {
                                    <span style="font:11px var(--font-mono);color:var(--faint);">"no deliveries yet"</span>
                                }.into_any(),
                                Some(Some(Ok(ds))) => {
                                    let rows = ds.into_iter().take(8).map(|d| {
                                        let err = d.last_error.unwrap_or_default();
                                        view! {
                                            <Group justify="between">
                                                <Group gap="sm">
                                                    <Pill color=sev(&d.status)>{d.status}</Pill>
                                                    <span style="font:11px var(--font-mono);color:var(--muted);">{d.event_type}</span>
                                                </Group>
                                                <span style="font:10px var(--font-mono);color:var(--faint);\
                                                             max-width:45%;overflow:hidden;text-overflow:ellipsis;\
                                                             white-space:nowrap;">
                                                    {if err.is_empty() { format!("{}\u{00d7}", d.attempts) } else { err }}
                                                </span>
                                            </Group>
                                        }
                                    }).collect_view();
                                    view! { <Stack gap="sm">{rows}</Stack> }.into_any()
                                }
                            }}
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Modal>
    }
}
