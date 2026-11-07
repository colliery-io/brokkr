//! Tenants view — the generator list from `GET /api/v1/generators`, plus the
//! console's privileged actions on tenants: minting a new generator tenant
//! (BROKKR-T-0318), registering agents with it (BROKKR-T-0368) and rotating
//! its PAK (BROKKR-T-0369).
//!
//! **Why this view asks for a PAK when nothing else does.** Every other console
//! surface rides the broker-injected UI token, which is a *read-only* admin
//! credential — so reaching the page grants visibility and nothing more, and
//! network reachability is the console's authentication boundary. Minting a
//! credential, and deciding which clusters serve a tenant, are things that must
//! not follow from merely opening the URL, so they are gated on an admin PAK
//! the operator supplies.
//!
//! **How long the supplied PAK lives.** In the tenant drawer, one PAK, one
//! request, cleared on both paths (the rule of BROKKR-T-0322). In the New
//! tenant dialog it is typed once and kept in a signal until the dialog
//! closes, so the mint and the registrations that finish onboarding use the
//! same credential (BROKKR-T-0368). It is never written to storage, never
//! logged, and never rendered. The console has no persistent credential store
//! at all — BROKKR-T-0320 removed the one it used to have.

use crate::api;
use crate::components::{agent_name, sev, write_error, PanelError};
use crate::models::{FleetAgentRecord, Generator, Registration};
use aurora_leptos::components::*;
use aurora_leptos::data::{DetailList, KeyValue, SectionLabel};
use aurora_leptos::frame::{use_toaster, ConfirmDialog, Drawer, Modal, SecretReveal};
use aurora_leptos::tokens::{token, ApiError};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// The freshly minted tenant, held only long enough for the operator to copy
/// its PAK and register its agents. Not persisted anywhere — a reload loses
/// it, which is the same guarantee the CLI gives.
#[derive(Clone)]
struct Minted {
    id: String,
    name: String,
    pak: String,
}

/// The fleet, as the registration controls read it: the whole list, or
/// nothing while it loads or when it failed.
type Fleet = LocalResource<Result<Vec<FleetAgentRecord>, ApiError>>;

#[component]
pub fn TenantsView() -> impl IntoView {
    let data = LocalResource::new(api::generators);
    // Agent names for the registration lists and the picker. Unscoped: a
    // tenant's agents can be anywhere in the fleet.
    let fleet: Fleet = LocalResource::new(|| api::fleet(None));

    // ---- the tenant drawer (BROKKR-T-0368) ---------------------------------
    let selected = RwSignal::new(None::<Generator>);
    let drawer_open = RwSignal::new(false);
    // The drawer's admin PAK: one request, then cleared (BROKKR-T-0322).
    let drawer_pak = RwSignal::new(String::new());
    // PAK rotation (BROKKR-T-0369): the confirmation, the request, and the
    // new PAK, shown once in its own locked dialog.
    let rotate_confirm = RwSignal::new(false);
    let rotate_busy = RwSignal::new(false);
    let rotate_error = RwSignal::new(None::<String>);
    let rotated = RwSignal::new(None::<Minted>);
    let rotated_open = RwSignal::new(false);
    let rotate = move |id: String, name: String| {
        let pak = drawer_pak.get();
        rotate_busy.set(true);
        rotate_error.set(None);
        spawn_local(async move {
            let result = api::rotate_generator_pak(&id, &pak).await;
            // One request, then cleared, on both paths.
            drawer_pak.set(String::new());
            rotate_busy.set(false);
            rotate_confirm.set(false);
            match result {
                Ok(resp) => {
                    rotated.set(Some(Minted {
                        id: resp.generator.id,
                        name,
                        pak: resp.pak,
                    }));
                    rotated_open.set(true);
                }
                Err(e) => rotate_error.set(Some(write_error(e))),
            }
        });
    };

    // ---- the New tenant dialog (BROKKR-T-0318) -----------------------------
    let open = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    // The admin PAK. Memory only; kept while the dialog is open, cleared when
    // it closes and after a failed mint.
    let admin_pak = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let minted = RwSignal::new(None::<Minted>);
    // While the mint runs, and while the one-time PAK is on screen, Escape and
    // × must not close the dialog: the PAK would be gone with no way back but
    // rotation (BROKKR-T-0367). Only Done closes it then.
    let locked = Signal::derive(move || busy.get() || minted.get().is_some());

    let toaster = use_toaster();

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
            error.set(Some("Enter a name.".into()));
            return;
        }
        let pak = admin_pak.get();
        if pak.trim().is_empty() {
            error.set(Some("Enter an admin PAK to create a tenant.".into()));
            return;
        }
        let d = description.get().trim().to_string();
        let d = (!d.is_empty()).then_some(d);

        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::create_generator(&n, d.as_deref(), &pak).await;
            busy.set(false);
            match result {
                Ok(resp) => {
                    // The PAK stays for the registrations that follow; the
                    // dialog clears it when it closes.
                    minted.set(Some(Minted {
                        id: resp.generator.id.clone(),
                        name: resp.generator.name.clone(),
                        pak: resp.pak,
                    }));
                    name.set(String::new());
                    description.set(String::new());
                    data.refetch();
                }
                Err(e) => {
                    // A failed mint clears the credential at once.
                    admin_pak.set(String::new());
                    let msg = match e {
                        ApiError::Http { status: 409, .. } => {
                            "A tenant with this name already exists.".to_string()
                        }
                        other => write_error(other),
                    };
                    error.set(Some(msg));
                    toaster.error("The tenant was not created.");
                }
            }
        });
    };

    view! {
        <Stack gap="md">
            <Group justify="end">
                <Button on_click=Callback::new(move |_| {
                    reset();
                    open.set(true);
                })>"+ New tenant"</Button>
            </Group>

            {move || match data.get() {
                None => view! { <Loading label="loading tenants" /> }.into_any(),
                Some(Err(e)) => view! {
                    <PanelError error=e on_retry=Callback::new(move |_| { data.refetch(); }) />
                }
                .into_any(),
                Some(Ok(gens)) if gens.is_empty() => view! {
                    <Empty message="No tenants yet. Create a tenant to give an application its own stacks and agents." />
                }
                .into_any(),
                Some(Ok(gens)) => {
                    let rows = gens
                        .into_iter()
                        .map(|g: Generator| {
                            let status = if g.is_active { "active" } else { "inactive" };
                            let color = sev(status);
                            let g_sel = g.clone();
                            let g_id = g.id.clone();
                            view! {
                                <TableRow
                                    selected=Signal::derive(move || {
                                        drawer_open.get()
                                            && selected.with(|x| x.as_ref().is_some_and(|x| x.id == g_id))
                                    })
                                    on_click=Callback::new(move |_| {
                                        drawer_pak.set(String::new());
                                        rotate_error.set(None);
                                        selected.set(Some(g_sel.clone()));
                                        drawer_open.set(true);
                                    })
                                >
                                    <td>{g.name}</td>
                                    <td class="brk-muted">
                                        {g.description.unwrap_or_else(|| "\u{2014}".into())}
                                    </td>
                                    <td><Pill color=color>{status}</Pill></td>
                                    <td class="brk-faint">
                                        {g.last_active_at.unwrap_or_else(|| "never".into())}
                                    </td>
                                    <td class="brk-faint">{g.id}</td>
                                </TableRow>
                            }
                        })
                        .collect_view();
                    view! {
                        <Panel title="Tenants" caption="Select a tenant to see its agents">
                            <Table mono=true label="Tenants">
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
        </Stack>

        // ---- tenant detail (BROKKR-T-0368) ------------------------------
        <Drawer open=drawer_open title="Tenant detail" size="lg"
            on_close=Callback::new(move |_| drawer_pak.set(String::new()))>
            {move || match selected.get() {
                None => ().into_any(),
                Some(g) => {
                    let status = if g.is_active { "active" } else { "inactive" };
                    let color = sev(status);
                    let id = g.id.clone();
                    let id_text = g.id.clone();
                    let reg_id = g.id.clone();
                    view! {
                        <Stack gap="md">
                            <span class="brk-detail-title">{g.name.clone()}</span>
                            <DetailList mono=true>
                                <KeyValue label="tenant id">
                                    <Group gap="sm">
                                        <span>{id_text}</span>
                                        <CopyButton value=id.clone() icon=true label="Copy tenant id" />
                                    </Group>
                                </KeyValue>
                                <KeyValue label="description">
                                    {g.description.clone().unwrap_or_else(|| "\u{2014}".into())}
                                </KeyValue>
                                <KeyValue label="last active">
                                    {g.last_active_at.clone().unwrap_or_else(|| "never".into())}
                                </KeyValue>
                                <KeyValue label="created">
                                    {g.created_at.clone().unwrap_or_else(|| "\u{2014}".into())}
                                </KeyValue>
                            </DetailList>
                            <Group gap="sm">
                                <Pill color=color>{status}</Pill>
                            </Group>
                            {if g.is_system {
                                view! {
                                    <SectionLabel label="registered agents" />
                                    <span class="brk-note">
                                        "Every agent is registered with the system generator."
                                    </span>
                                }.into_any()
                            } else {
                                let rot_id = g.id.clone();
                                let rot_name = g.name.clone();
                                let confirm_name = g.name.clone();
                                view! {
                                    <Stack gap="xs">
                                        <PasswordInput
                                            label="Admin PAK"
                                            placeholder="brokkr_\u{2026}"
                                            value=drawer_pak
                                            autocomplete="off"
                                        />
                                    </Stack>
                                    <Registrations
                                        generator_id=reg_id
                                        pak=drawer_pak
                                        clear_pak=true
                                        fleet=fleet
                                    />
                                    // ---- PAK rotation (BROKKR-T-0369) ----------------
                                    <SectionLabel label="credential" />
                                    <Group gap="sm">
                                        <Button variant="default" disabled=rotate_busy
                                            on_click=Callback::new(move |_| {
                                                if drawer_pak.get().trim().is_empty() {
                                                    rotate_error.set(Some(
                                                        "Enter an admin PAK to rotate the PAK.".into(),
                                                    ));
                                                    return;
                                                }
                                                rotate_error.set(None);
                                                rotate_confirm.set(true);
                                            })>"Rotate PAK"</Button>
                                    </Group>
                                    {move || rotate_error.get().map(|e| view! {
                                        <Alert color=token::BAD>{e}</Alert>
                                    })}
                                    <ConfirmDialog
                                        open=rotate_confirm
                                        title=format!("Rotate the PAK of {confirm_name}?")
                                        message="The console shows the new PAK once, in the next dialog."
                                        impacts=vec![
                                            "The current PAK stops at once.".to_string(),
                                            "Every pipeline that uses it fails until it gets the new PAK.".to_string(),
                                        ]
                                        confirm_label="Rotate"
                                        busy=rotate_busy
                                        on_confirm=Callback::new(move |_| rotate(rot_id.clone(), rot_name.clone()))
                                    />
                                }.into_any()
                            }}
                        </Stack>
                    }
                    .into_any()
                }
            }}
        </Drawer>

        // The rotated PAK (BROKKR-T-0369): shown once, in a dialog that only
        // Done closes, the same as a mint.
        <Modal open=rotated_open title="PAK rotated" close_on_scrim=false
            locked=Signal::derive(move || rotated.get().is_some())
            on_close=Callback::new(move |_| rotated.set(None))>
            {move || rotated.get().map(|m| view! {
                <Stack gap="md">
                    <DetailList mono=true>
                        <KeyValue label="Tenant">{m.name.clone()}</KeyValue>
                        <KeyValue label="Tenant id">{m.id.clone()}</KeyValue>
                    </DetailList>
                    <SecretReveal
                        secret=m.pak.clone()
                        label="New PAK"
                        done_label="Done"
                        on_done=Callback::new(move |_| {
                            rotated.set(None);
                            rotated_open.set(false);
                        })
                    />
                </Stack>
            })}
        </Modal>

        // The dialog stays open on a stray scrim click: it may hold a typed
        // admin PAK or a one-time secret. While the mint runs or the secret is
        // shown it is locked: only Done closes it (BROKKR-T-0367).
        <Modal open=open title="New tenant" close_on_scrim=false locked=locked
            on_close=Callback::new(move |_| reset())>
            {move || match minted.get() {
                // ---- reveal-once panel, then the agents -------------------
                Some(m) => {
                    let id = m.id.clone();
                    let id_text = m.id.clone();
                    let reg_id = m.id.clone();
                    view! {
                    <Stack gap="md">
                        <DetailList mono=true>
                            <KeyValue label="Tenant">{m.name.clone()}</KeyValue>
                            <KeyValue label="Tenant id">
                                <Group gap="sm">
                                    <span>{id_text}</span>
                                    <CopyButton value=id.clone() icon=true label="Copy tenant id" />
                                </Group>
                            </KeyValue>
                        </DetailList>
                        <Registrations
                            generator_id=reg_id
                            pak=admin_pak
                            clear_pak=false
                            fleet=fleet
                        />
                        <SecretReveal
                            secret=m.pak.clone()
                            label="PAK"
                            done_label="Done"
                            on_done=Callback::new(move |_| {
                                reset();
                                open.set(false);
                            })
                        />
                    </Stack>
                    }
                    .into_any()
                }

                // ---- the form ---------------------------------------------
                None => view! {
                    <Stack gap="md">
                        <TextInput label="Name" placeholder="acme-payments" value=name />
                        <TextInput
                            label="Description (optional)"
                            placeholder="What this tenant deploys"
                            value=description
                        />
                        <Stack gap="xs">
                            <PasswordInput
                                label="Admin PAK"
                                placeholder="brokkr_\u{2026}"
                                value=admin_pak
                                autocomplete="off"
                            />
                            <span class="brk-note">
                                "Not stored."
                            </span>
                        </Stack>

                        {move || error.get().map(|e| view! {
                            <Alert color=token::BAD>{e}</Alert>
                        })}

                        <Group justify="end" gap="sm">
                            <Button variant="default" on_click=Callback::new(move |_| {
                                reset();
                                open.set(false);
                            })>"Cancel"</Button>
                            <Button
                                loading=busy
                                loading_label="Creating\u{2026}"
                                on_click=Callback::new(move |_| submit())
                            >
                                "Create tenant"
                            </Button>
                        </Group>
                    </Stack>
                }
                .into_any(),
            }}
        </Modal>
    }
}

/// The agents registered with one tenant, with Register and Deregister
/// (BROKKR-T-0368). Registration is the agent's consent to receive the
/// tenant's stacks: without it, a target for the agent is refused and a label
/// match is ignored.
///
/// `pak` is the admin PAK the writes carry; the caller shows the field. With
/// `clear_pak` each request clears it, success or failure: the tenant drawer.
/// Without, the caller holds it: the New tenant dialog, where it was typed
/// for the mint and lives until the dialog closes.
#[component]
fn Registrations(
    generator_id: String,
    pak: RwSignal<String>,
    clear_pak: bool,
    fleet: Fleet,
) -> impl IntoView {
    let toaster = use_toaster();
    let gid = StoredValue::new(generator_id);
    let regs = LocalResource::new(move || {
        let id = gid.get_value();
        async move { api::registered_agents(&id).await }
    });
    // The picker's choice, as the agent's name. Empty means "not chosen yet",
    // which resolves to the first option: what an untouched `<select>` shows.
    let pick = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // The agent the operator asked to deregister: `(id, name)`, while the
    // confirmation is open.
    let removing = RwSignal::new(None::<(String, String)>);
    let confirm_open = RwSignal::new(false);

    // The names of the agents not yet registered, for the picker.
    let unregistered = Signal::derive(move || {
        let taken: Vec<String> = regs
            .get()
            .and_then(|r| r.ok())
            .map(|list| list.into_iter().map(|r| r.agent_id).collect())
            .unwrap_or_default();
        fleet
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|a| {
                !taken
                    .iter()
                    .any(|t| crate::components::same_agent(&a.agent_id, t))
            })
            .map(|a| a.name)
            .collect::<Vec<String>>()
    });

    // The typed PAK, or the error that says it is missing.
    let need_pak = move || -> Option<String> {
        let p = pak.get();
        if p.trim().is_empty() {
            error.set(Some(
                "Enter an admin PAK to change the registrations.".into(),
            ));
            None
        } else {
            Some(p)
        }
    };

    let register = move || {
        let Some(p) = need_pak() else { return };
        let options = unregistered.get();
        let want = pick.get();
        let Some(chosen) = options
            .iter()
            .find(|n| **n == want)
            .or_else(|| options.first())
            .cloned()
        else {
            error.set(Some(
                "Every agent is registered with this tenant already.".into(),
            ));
            return;
        };
        let Some(agent_id) = fleet.get().and_then(|r| r.ok()).and_then(|list| {
            list.into_iter()
                .find(|a| a.name == chosen)
                .map(|a| a.agent_id)
        }) else {
            return;
        };
        busy.set(true);
        error.set(None);
        let id = gid.get_value();
        spawn_local(async move {
            let result = api::register_agent(&id, &agent_id, &p).await;
            if clear_pak {
                pak.set(String::new());
            }
            busy.set(false);
            match result {
                Ok(_) => {
                    pick.set(String::new());
                    regs.refetch();
                    toaster.success("Agent registered");
                }
                Err(e) => {
                    error.set(Some(write_error(e)));
                    toaster.error("The agent was not registered.");
                }
            }
        });
    };

    let deregister = move |agent_id: String| {
        let Some(p) = need_pak() else { return };
        busy.set(true);
        error.set(None);
        let id = gid.get_value();
        spawn_local(async move {
            let result = api::deregister_agent(&id, &agent_id, &p).await;
            if clear_pak {
                pak.set(String::new());
            }
            busy.set(false);
            removing.set(None);
            match result {
                Ok(()) => {
                    regs.refetch();
                    toaster.success("Agent deregistered");
                }
                Err(e) => {
                    error.set(Some(write_error(e)));
                    toaster.error("The agent was not deregistered.");
                }
            }
        });
    };

    view! {
        <SectionLabel label="registered agents" />
        {move || match regs.get() {
            None => view! { <Loading label="loading registrations" /> }.into_any(),
            Some(Err(_)) => view! {
                <span class="brk-note">"The registrations are not available."</span>
            }.into_any(),
            Some(Ok(list)) if list.is_empty() => view! {
                <span class="brk-note">
                    "No agent is registered with this tenant. Its stacks reach no cluster until an agent is registered."
                </span>
            }.into_any(),
            Some(Ok(list)) => {
                let names = fleet.get().and_then(|r| r.ok()).unwrap_or_default();
                let rows = list.into_iter().map(|r: Registration| {
                    let n = agent_name(&names, &r.agent_id);
                    let n_confirm = n.clone();
                    let aid = r.agent_id.clone();
                    view! {
                        <Group gap="sm">
                            <Anchor href=crate::components::agent_href(&r.agent_id)>{n}</Anchor>
                            <span class="brk-grow"></span>
                            <Button variant="default" disabled=busy on_click=Callback::new(move |_| {
                                if need_pak().is_none() {
                                    return;
                                }
                                removing.set(Some((aid.clone(), n_confirm.clone())));
                                confirm_open.set(true);
                            })>"Deregister"</Button>
                        </Group>
                    }
                }).collect_view();
                view! { <Stack gap="xs">{rows}</Stack> }.into_any()
            }
        }}
        {move || {
            let options = unregistered.get();
            if options.is_empty() {
                return view! {
                    <span class="brk-note">
                        "Every agent in the fleet is registered with this tenant."
                    </span>
                }.into_any();
            }
            view! {
                <Group gap="sm">
                    <Select label="agent" options=options value=pick />
                    <Button loading=busy loading_label="Working\u{2026}"
                        on_click=Callback::new(move |_| register())>
                        "Register"
                    </Button>
                </Group>
            }.into_any()
        }}
        {move || error.get().map(|e| view! {
            <Alert color=token::BAD>{e}</Alert>
        })}
        {move || removing.get().map(|(aid, n)| view! {
            <ConfirmDialog
                open=confirm_open
                title=format!("Deregister {n}?")
                message="The agent does not receive this tenant's stacks after this."
                impacts=vec![
                    "The broker removes the agent's targets for this tenant's stacks.".to_string(),
                    "The agent removes those resources from its cluster on its next poll.".to_string(),
                ]
                confirm_label="Deregister"
                busy=busy
                on_confirm=Callback::new(move |_| deregister(aid.clone()))
                on_cancel=Callback::new(move |_| removing.set(None))
            />
        })}
    }
}
