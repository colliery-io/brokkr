---
id: console-mint-a-generator-tenant
level: task
title: "Console: mint a generator tenant from the UI by supplying an admin PAK per action"
short_code: "BROKKR-T-0318"
created_at: 2026-07-29T06:30:00+00:00
updated_at: 2026-07-29T16:20:42.616334+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


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

## Acceptance Criteria

## Acceptance Criteria

- [ ] An operator can create a generator from the console by supplying an admin PAK, and sees the new PAK with a working copy button.
- [ ] The panel states that the generator PAK is shown once and cannot be recovered.
- [ ] The admin PAK is held in memory only, never persisted, never logged, and cleared after the request completes — verified by inspecting storage and the console log after a mint.
- [ ] A wrong or non-admin PAK produces a clear error and does not leave the credential in memory.
- [ ] `readonly_request_allowed` is unchanged, and the UI PAK still cannot perform this action — asserted by a test that attempts the mint with only the console's own credential and expects 403.
- [ ] No change is needed to the "network reach is the authentication boundary" statements in `security-model.md`; if that turns out to be false during implementation, stop and reopen the design question.

## Status Updates

**2026-07-29 — IN PROGRESS.** `cargo clippy --target wasm32-unknown-unknown` and `trunk build` both pass; broker integration suite running for the new guard test.

### Built

New **Tenants** view (`crates/brokkr-web/src/views/tenants.rs`), under the System nav group: lists generators from `GET /api/v1/generators`, with a "+ New tenant" button opening an Aurora `Modal`. Form is name + optional description + `PasswordInput` for the admin PAK. On success the modal swaps to a reveal-once panel: a gold `Alert` stating the PAK cannot be recovered, the PAK in a `<code>` block, and Aurora's `CopyButton`.

Aurora already shipped every piece — `Modal`, `PasswordInput`, `CopyButton`, `Alert`, `Table`, `Panel` — so nothing bespoke was needed.

**The confirmation that this needed no broker change held up.** `POST /generators` was used exactly as it stands, and `readonly_request_allowed` is untouched.

### The one thing the ticket got wrong about the existing code

The ticket said the admin PAK must be memory-only, never `localStorage`. Reading `api.rs` revealed the console **already has** an operator-pasted PAK override — `api::pak()`, read from `localStorage["brokkr_pak"]`, documented as "the write-capable override" and the only auth path under `trunk serve`.

Reusing it would have been the smaller diff and the wrong call: it persists across reloads and is readable by any script on the origin, so a one-off tenant mint would leave the strongest credential in the system sitting in browser storage indefinitely.

So this adds `api::post_json_with_token`, which takes the bearer explicitly and deliberately does **not** consult `token()`/`pak()`. The credential is borrowed for one request and never captured, stored, or logged. The long-lived `brokkr_pak` override is left exactly as it was — it is a separate, pre-existing mechanism, and **worth its own look**: nothing in this ticket examined whether persisting an admin PAK there is appropriate.

### Credential handling

`admin_pak` is an `RwSignal<String>` cleared immediately after the request resolves, on **both** the success and error paths, before anything else observes it. Error messages are mapped from status codes (403 → "not an admin credential", 409 → "name already exists") rather than interpolating the raw `ApiError`, so no future change can surface request details into the UI; the catch-all arm is deliberately generic for the same reason.

### Guard test

`test_ui_pak_cannot_mint_a_generator` in `tests/integration/api/ui_pak.rs` asserts the console's *own* read-only credential gets 403 on `POST /generators`, **and** that no generator row was written — a 403 that still created the tenant would be worse than no check. That is what makes the per-action design safe, so it is pinned rather than assumed.

### Verified with the existing Playwright harness, which turned out to be broken

Dylan (2026-07-29): *"shouldn't we use playwright and golden image tests instead of you driving chrome?"* — correct, and better than that: `crates/brokkr-web/web-e2e/shots.mjs` already existed, mocking `/api/v1/**` via route interception. Four scenes added (`tenants`, `tenants-empty`, `tenants-new`, `tenants-minted`). The mint scene is driven end to end — open dialog, fill name + admin PAK, submit — and the reveal panel screenshot confirms the alert, the PAK, and the copy button render.

**The harness was silently screenshotting the wrong view for most scenes, and had been.** Every interaction ended in `.catch(() => {})`, so a click that did nothing still printed `shot: <name>` and produced a confident-looking PNG of the default Overview view. Three things had to be fixed before the new scenes proved anything:

1. **`trunk serve` watches `web-e2e/`.** That directory holds the harness *and its PNG output*, so **every screenshot triggered a rebuild, and the rebuild live-reloaded the page mid-scene, resetting the route.** One run logged **310 rebuilds**. This is the actual root cause, and it presented as a click race — it survived several rounds of settle-delay tuning before the rebuild log gave it away. Fixed with a new `Trunk.toml` (`[watch] ignore = ["web-e2e"]`); a run now logs 1.
2. **Rendered ≠ interactive.** Leptos renders the sidebar ~500ms before its click handlers respond, and a click in that window is accepted by the DOM and does nothing. Replaced the single click with `navigateTo()`, which clicks and re-checks `.cl-page-header__title` until it matches, up to 8 attempts — deterministic under any load, where a fixed delay is not.
3. **Mocks were keyed on path only**, so `/generators` could not serve both the GET list and the POST mint response. Keys are now method-aware (`POST /generators`).

Also added a `fill` primitive (Aurora inputs have no name/id, so placeholder is the stable handle).

**A caution worth keeping: my own assertion passed vacuously at first.** The storage check reported `admin PAK not persisted ✓` on runs where the nav had failed and no mint ever occurred — it was confirming the absence of a secret that was never typed. An assertion that can pass because the thing under test did not happen is worse than no assertion, because it reads as evidence. It only became meaningful once the nav was verified.

**Consequence beyond this ticket:** the committed screenshots under `web-e2e/shots/` cannot be trusted as a baseline for anything before this change, and neither could a golden-image diff built on them. Flagged into BROKKR-T-0319, which owns the UI sweep.

### Note for the UI pass (BROKKR-T-0319)

`brokkr-web` carries 7 pre-existing `clippy::redundant_closure` warnings across the other views (`LocalResource::new(|| api::foo())`). This view avoids adding an eighth, but the existing ones are untouched and belong to that sweep.