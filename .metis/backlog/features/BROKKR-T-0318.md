---
id: console-admin-panel-pak-generator-minting
level: task
title: "Console: mint a generator tenant from the UI by supplying an admin PAK per action"
short_code: "BROKKR-T-0318"
created_at: 2026-07-29T06:30:00+00:00
updated_at: 2026-07-29T06:45:00+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/backlog"


exit_criteria_met: false
initiative_id: NULL
---

# Console: mint a generator tenant from the UI by supplying an admin PAK per action

## Objective

Make standing up a new **generator tenant** easy from the Operator Console, instead of requiring CLI or curl access.

Flow, as specified by Dylan (2026-07-29):

1. Operator clicks a button in the console.
2. Operator pastes their **admin PAK** into the dialog.
3. The console creates the generator and returns the new generator PAK **in the UI, with a copy button** — shown once.

## Backlog Item Details

### Type
- [x] Feature - New functionality

### Priority
- [x] P2 - Medium (pure operator ergonomics; the capability already exists via API and CLI)

## 2026-07-29 — this needs NO broker changes, and no change to the console's security model

An earlier revision of this ticket framed the console's read-only design as a blocking constraint. **For this ask, it is not** — the per-action admin PAK sidesteps it entirely, and the reason is worth stating because it is the thing that makes this cheap:

The console's ephemeral UI PAK resolves to `admin: true, **readonly: true**`, and the readonly branch in the auth middleware is what rejects writes (`middleware.rs:159`, allowlist at `:216-233`). A request bearing the **operator's admin PAK** is a different credential entirely: `verify_pak` matches the admin role first and returns `admin: true, **readonly: false**` (`middleware.rs:246-259`), so `readonly_request_allowed` is never consulted.

Consequences:

- **`readonly_request_allowed` is not touched.** The UI PAK stays read-only.
- **"Network reach is the authentication boundary" stays true** for everything the console does on its own. The admin panel is gated by a credential only the operator holds, so an unauthenticated network peer gains nothing.
- **No new endpoint.** `POST /api/v1/generators` already exists, already requires admin, already mints the PAK (`pak::create_pak()`, `generators.rs:140`) and already returns the plaintext once in `CreateGeneratorResponse { generator, pak }` (`generators.rs:37-42`).
- **No CORS work.** The console is served by the broker itself, so this is same-origin; the chart's `cors.allowedHeaders` includes `Authorization` in any case.

So the work is entirely in `brokkr-web`: a dialog, a one-shot request with an operator-supplied `Authorization` header, and a reveal-once panel with a copy button.

### The one thing to be careful about: handling the admin PAK in a browser

The console will hold the strongest credential in the system, in a browser, for the duration of one action. That is acceptable and normal, but it constrains the implementation:

- **Memory only.** Never `localStorage`, `sessionStorage`, a cookie, or a URL parameter. It should not survive the dialog closing or a page reload.
- **Never logged.** Not to the console, not into an error message, not into a rendered error body.
- **Not retained after use.** Clear it as soon as the request completes, success or failure — do not keep it around to make the next mint convenient. Re-prompting is the feature, not friction.
- The returned **generator** PAK is shown once and is unrecoverable; the panel must say so plainly next to the copy button, since the operator's only alternative afterwards is rotation.

### Deliberately out of scope

- **Admin PAK minting/rotation.** `generate-pak` and `rotate admin` are CLI-only *by design* — `how-to/pak-management.md` records the rationale: it prevents an attacker with a compromised admin PAK from locking out the real admin. A console path would require inventing an API surface that was deliberately never built, and reverses that decision. Not part of this.
- **Agent creation.** `POST /agents` also mints a PAK and could use the same treatment, but it is a separate surface with its own registration semantics. Worth a follow-up once this pattern is proven, not a widening of this ticket.
- **The broader UI pass** Dylan also called for. Scoped separately so the sweep is not entangled with a privileged surface.

## Acceptance Criteria

- [ ] An operator can create a generator from the console by supplying an admin PAK, and sees the new PAK with a working copy button.
- [ ] The panel states that the generator PAK is shown once and cannot be recovered.
- [ ] The admin PAK is held in memory only, never persisted, never logged, and cleared after the request completes — verified by inspecting storage and the console log after a mint.
- [ ] A wrong or non-admin PAK produces a clear error and does not leave the credential in memory.
- [ ] `readonly_request_allowed` is unchanged, and the UI PAK still cannot perform this action — asserted by a test that attempts the mint with only the console's own credential and expects 403.
- [ ] No change is needed to the "network reach is the authentication boundary" statements in `security-model.md`; if that turns out to be false during implementation, stop and reopen the design question.

## Status Updates

*To be added during implementation*
