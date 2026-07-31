//! Tenants view — the generator list from `GET /api/v1/generators`, plus the
//! console's one privileged action: minting a new generator tenant
//! (BROKKR-T-0318).
//!
//! **Why this view asks for a PAK when nothing else does.** Every other console
//! surface rides the broker-injected UI token, which is a *read-only* admin
//! credential — so reaching the page grants visibility and nothing more, and
//! network reachability is the console's authentication boundary. Minting a
//! credential is the one thing that must not follow from merely opening the
//! URL, so it is gated on an admin PAK the operator supplies per action.
//!
//! The supplied PAK lives in a signal for the duration of one request and is
//! cleared the moment it completes, success or failure. It is never written to
//! storage, never logged, and never rendered. The console has no persistent
//! credential store at all — BROKKR-T-0320 removed the one it used to have.

use crate::api;
use crate::components::{sev, toast, DetailRow, ToastBus};
use crate::models::Generator;
use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// The freshly minted credential, held only long enough for the operator to
/// copy it. Not persisted anywhere — a reload loses it, which is the same
/// guarantee the CLI gives.
#[derive(Clone)]
struct Minted {
    name: String,
    pak: String,
}

#[component]
pub fn TenantsView() -> impl IntoView {
    let data = LocalResource::new(api::generators);

    // Dialog state.
    let open = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    // The admin PAK. Memory only, cleared after every attempt.
    let admin_pak = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let minted = RwSignal::new(None::<Minted>);

    let bus = use_context::<ToastBus>();

    // Resets everything the dialog holds, including the credential fields.
    let reset = move || {
        name.set(String::new());
        description.set(String::new());
        admin_pak.set(String::new());
        error.set(None);
        minted.set(None);
    };

    let submit = move || {
        let n = name.get().trim().to_string();
        if n.is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        let pak = admin_pak.get();
        if pak.trim().is_empty() {
            error.set(Some("An admin PAK is required to mint a tenant.".into()));
            return;
        }
        let d = description.get().trim().to_string();
        let d = (!d.is_empty()).then_some(d);

        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::create_generator(&n, d.as_deref(), &pak).await;
            // Clear the admin credential immediately, on both paths, before
            // anything else can observe it. Re-prompting next time is the
            // intent, not an oversight.
            admin_pak.set(String::new());
            busy.set(false);
            match result {
                Ok(resp) => {
                    minted.set(Some(Minted {
                        name: resp.generator.name.clone(),
                        pak: resp.pak,
                    }));
                    name.set(String::new());
                    description.set(String::new());
                    data.refetch();
                    if let Some(b) = bus {
                        toast(b, "tenant created", token::OK);
                    }
                }
                Err(e) => {
                    // Surface the broker's own message. `ErrorResponse` never
                    // echoes the Authorization header, so this cannot leak the
                    // PAK that was just used.
                    let msg = match e {
                        aurora_leptos::tokens::ApiError::Http { status: 403, .. } => {
                            "Rejected: that PAK is not an admin credential.".to_string()
                        }
                        aurora_leptos::tokens::ApiError::Http { status: 409, .. } => {
                            "A generator with that name already exists.".to_string()
                        }
                        aurora_leptos::tokens::ApiError::Http { status, .. } => {
                            format!("Broker rejected the request (HTTP {status}).")
                        }
                        aurora_leptos::tokens::ApiError::Network => {
                            "Could not reach the broker.".to_string()
                        }
                        // Never interpolate the raw error: it is the one place a
                        // future change could surface request details.
                        _ => "The request failed.".to_string(),
                    };
                    error.set(Some(msg));
                    if let Some(b) = bus {
                        toast(b, "tenant creation failed", token::BAD);
                    }
                }
            }
        });
    };

    view! {
        <div style="display:flex;justify-content:flex-end;margin-bottom:14px;">
            <Button on_click=Callback::new(move |_| {
                reset();
                open.set(true);
            })>"+ New tenant"</Button>
        </div>

        {move || match data.get() {
            None => view! { <Loading label="loading tenants" /> }.into_any(),
            Some(Err(e)) => view! {
                <ErrorState error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
            }
            .into_any(),
            Some(Ok(gens)) if gens.is_empty() => view! {
                <Empty message="No tenants yet. Create one to scope stacks and agents to an application." />
            }
            .into_any(),
            Some(Ok(gens)) => {
                let rows = gens
                    .into_iter()
                    .map(|g: Generator| {
                        let status = if g.is_active { "active" } else { "inactive" };
                        let color = sev(status);
                        view! {
                            <tr>
                                <td>{g.name}</td>
                                <td style="color:var(--muted);">
                                    {g.description.unwrap_or_else(|| "\u{2014}".into())}
                                </td>
                                <td><Pill color=color.to_string()>{status}</Pill></td>
                                <td style="color:var(--faint);">
                                    {g.last_active_at.unwrap_or_else(|| "never".into())}
                                </td>
                                <td style="color:var(--faint);">{g.id}</td>
                            </tr>
                        }
                    })
                    .collect_view();
                view! {
                    <Panel title="Tenants">
                        <Table mono=true>
                            <thead>
                                <tr>
                                    <th>"Name"</th>
                                    <th>"Description"</th>
                                    <th>"Status"</th>
                                    <th>"Last active"</th>
                                    <th>"ID"</th>
                                </tr>
                            </thead>
                            <tbody>{rows}</tbody>
                        </Table>
                    </Panel>
                }
                .into_any()
            }
        }}

        <Modal open=open title="New tenant">
            {move || match minted.get() {
                // ---- reveal-once panel ------------------------------------
                Some(m) => view! {
                    <Alert color=token::GOLD.to_string()>
                        "This PAK is shown once and cannot be recovered. Store it now \u{2014} \
                         the broker keeps only a hash, so the only way back is to rotate."
                    </Alert>
                    <div style="margin-top:12px;">
                        <DetailRow label="Tenant">{m.name.clone()}</DetailRow>
                    </div>
                    <div style="margin-top:12px;display:flex;align-items:center;gap:10px;">
                        <code style="flex:1;font:12px var(--font-mono);color:var(--fg);\
                                     background:var(--inset);border:1px solid var(--border-control);\
                                     border-radius:8px;padding:10px 12px;word-break:break-all;">
                            {m.pak.clone()}
                        </code>
                        <CopyButton value=m.pak.clone() />
                    </div>
                    <div style="margin-top:16px;display:flex;justify-content:flex-end;gap:8px;">
                        <Button on_click=Callback::new(move |_| {
                            reset();
                            open.set(false);
                        })>"Done"</Button>
                    </div>
                }
                .into_any(),

                // ---- the form ---------------------------------------------
                None => view! {
                    <TextInput label="Name" placeholder="acme-payments" value=name />
                    <div style="margin-top:10px;">
                        <TextInput
                            label="Description (optional)"
                            placeholder="What this tenant deploys"
                            value=description
                        />
                    </div>
                    <div style="margin-top:14px;">
                        <PasswordInput
                            label="Admin PAK"
                            placeholder="brokkr_\u{2026}"
                            value=admin_pak
                        />
                        <div style="font:10px var(--font-mono);color:var(--faint);margin-top:6px;\
                                    line-height:1.5;">
                            "Required: the console's own credential is read-only, so it cannot \
                             mint tenants. Held in memory for this request only \u{2014} never \
                             stored, and cleared as soon as it completes."
                        </div>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div style="margin-top:12px;">
                            <Alert color=token::BAD.to_string()>{e}</Alert>
                        </div>
                    })}

                    <div style="margin-top:18px;display:flex;justify-content:flex-end;gap:8px;">
                        <Button on_click=Callback::new(move |_| {
                            reset();
                            open.set(false);
                        })>"Cancel"</Button>
                        <Button on_click=Callback::new(move |_| submit())>
                            {move || if busy.get() { "Creating\u{2026}" } else { "Create tenant" }}
                        </Button>
                    </div>
                }
                .into_any(),
            }}
        </Modal>
    }
}
