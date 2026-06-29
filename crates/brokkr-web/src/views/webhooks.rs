//! Webhooks view — subscription summaries from `GET /api/v1/webhooks`.
//! NOTE: the API redacts the URL (encrypted at rest) to `has_url`, and delivery
//! history is per-subscription (`/webhooks/:id/deliveries`), not a global feed —
//! so this shows subscriptions; a global "recent deliveries" panel needs a broker
//! enhancement (logged on the task).

use crate::api;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;

#[component]
pub fn WebhooksView() -> impl IntoView {
    let data = LocalResource::new(|| api::webhooks());

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
                        let chips = s
                            .event_types
                            .into_iter()
                            .map(|e| {
                                view! {
                                    <span style="font:9.5px var(--font-mono);color:var(--ice);\
                                                 background:rgba(127,178,255,.1);padding:2px 6px;\
                                                 border-radius:6px;">{e}</span>
                                }
                            })
                            .collect_view();
                        view! {
                            <div style="min-width:260px;flex:1;">
                                <Panel title=s.name.clone()>
                                    <Stack gap="sm">
                                        <Group gap="sm">
                                            <Pill color=color>{label}</Pill>
                                            {(!s.has_url).then(|| view! {
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
    }
}
