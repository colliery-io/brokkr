// Screenshot every console view for visual / pixel verification. The broker API
// is mocked via Playwright route interception (per-scene fixtures), so views
// render with realistic data without a running broker. The console must be
// served first: `cd crates/brokkr-web && trunk serve --port 9080`.
//   Run: cd crates/brokkr-web/web-e2e && URL=http://127.0.0.1:9080 node shots.mjs
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";

const BASE = process.env.URL || "http://127.0.0.1:9080";
const OUT = "shots";
mkdirSync(OUT, { recursive: true });

// ---- fixtures ------------------------------------------------------------
const FLEET = [
  { agent_id: "1b9d6bcd", name: "prod-agent-01", cluster_name: "prod-us-east-1", status: "ACTIVE", ws_connected: true,
    heartbeat_age_seconds: 3, health_failing: 0, health_degraded: 0,
    pending_object_count: 2, pending_work_orders: 0, claimed_work_orders: 1 },
  { agent_id: "7c9e6679", name: "prod-agent-02", cluster_name: "prod-us-east-1", status: "ACTIVE", ws_connected: false,
    heartbeat_age_seconds: 42, health_failing: 0, health_degraded: 2,
    pending_object_count: 0, pending_work_orders: 1, claimed_work_orders: 0 },
  { agent_id: "a1b2c3d4", name: "staging-agent-01", cluster_name: "staging-eu-west-1", status: "INACTIVE", ws_connected: false,
    heartbeat_age_seconds: 900, health_failing: 1, health_degraded: 0,
    pending_object_count: 0, pending_work_orders: 0, claimed_work_orders: 0 },
];

const ACTIVE_WOS = [
  { id: "9a01ffbe", work_type: "image_build", status: "claimed", retry_count: 0, claimed_by: "1b9d6bcd-bbfd", last_error: null },
  { id: "b2c3d4e5", work_type: "image_build", status: "pending", retry_count: 1, claimed_by: null, last_error: null },
];

const WSCONN = {
  connected_agents: 2,
  live_subscribers: 1,
  connections: [
    { agent_id: "1b9d6bcd-bbfd-4b2d-9b5d-ab8dfbbd4bed", messages_in: 1240, messages_out: 880 },
    { agent_id: "7c9e6679-7425-40de-944b-e07fc1f90ae7", messages_in: 32, messages_out: 18 },
  ],
};

const PROM = `# HELP brokkr_active_agents Active agents
brokkr_active_agents 3
brokkr_ws_connected_agents 2
brokkr_http_requests_total{method="GET",status="200"} 1840
brokkr_http_requests_total{method="POST",status="201"} 95
brokkr_fleet_live_subscribers 1
brokkr_stacks_total 12
brokkr_deployment_objects_total 47
`;

// scene = { name, nav?: sidebar label to click, mocks: { "/path": json } }
const EVENTS = [
  { agent_id: "a1", event_type: "Apply", status: "success", message: "applied Deployment/payments (3 objects)" },
  { agent_id: "a1", event_type: "Heartbeat", status: "success", message: "k8s reachable (12ms)" },
  { agent_id: "a2", event_type: "Reconcile", status: "failure", message: "Service/ingest: port 8080 already allocated" },
];

const JOBS = [
  { id: "7f3a01ab", work_type: "image_build", success: true, retries_attempted: 0, result_message: "pushed ghcr.io/app:sha-7f3a01" },
  { id: "561200cd", work_type: "image_build", success: false, retries_attempted: 3, result_message: "buildah: manifest unknown" },
];
const HOOKS = [
  { id: "h1", name: "prod-alerts", enabled: true, has_url: true, event_types: ["stack.updated", "agent.failed"] },
  { id: "h2", name: "audit-sink", enabled: false, has_url: true, event_types: ["pak.rotated"] },
];
const STACKS = [
  { id: "s1", name: "payments-api", description: "prod payments service", generator_id: "1b9d6bcd-bbfd" },
  { id: "s2", name: "ingest-worker", description: "event ingest", generator_id: "7c9e6679-7425" },
];
// Named PAKs (tenants) for the scope selector (BROKKR-I-0032). IDs line up
// with STACKS.generator_id so scoped mocks stay coherent.
const PAKS = [
  { id: "1b9d6bcd-bbfd", name: "team-payments" },
  { id: "7c9e6679-7425", name: "team-ingest" },
];
// team-payments owns the two prod agents; team-ingest the staging one.
const FLEET_PAYMENTS = FLEET.slice(0, 2);
const TELEM = [
  { agent_id: "a1", event_type: "Apply", status: "success", message: "applied Deployment/payments (3 objects)" },
  { agent_id: "a1", event_type: "Reconcile", status: "success", message: "no drift" },
  { agent_id: "a2", event_type: "Apply", status: "failure", message: "Service/ingest: port 8080 already allocated" },
];

// Diagnostics (BROKKR-T-0301). The Fleet modal picks a deployment object from
// the agent's target state, POSTs a request, keeps the returned id and polls
// GET /diagnostics/:id. The route mock is method-agnostic (keyed on path), so
// the POST is answered with the 201-shaped body below.
const TARGET_STATE = [
  { id: "d1a2b3c4", stack_id: "s1", sequence_id: 41, is_deletion_marker: false },
];
const DIAG_CREATED = {
  id: "9f10ab22", agent_id: "1b9d6bcd", deployment_object_id: "d1a2b3c4",
  status: "pending", requested_by: "operator-console",
  created_at: "2026-07-27T10:00:00Z", expires_at: "2026-07-27T11:00:00Z",
};
// The result's three payload fields are JSON-encoded *strings*, not nested
// objects — hence the JSON.stringify calls: the console parses them a second time.
const DIAG_DONE = {
  request: { ...DIAG_CREATED, status: "completed", claimed_at: "2026-07-27T10:00:08Z",
    completed_at: "2026-07-27T10:00:14Z" },
  result: {
    request_id: "9f10ab22",
    pod_statuses: JSON.stringify([
      { name: "payments-api-7d9f4-x2k1", namespace: "payments", phase: "Running",
        conditions: [{ condition_type: "Ready", status: "True" }],
        containers: [{ name: "api", ready: true, restart_count: 0, state: "running" }] },
      { name: "payments-api-7d9f4-q8m3", namespace: "payments", phase: "Pending",
        conditions: [{ condition_type: "Ready", status: "False" }],
        containers: [{ name: "api", ready: false, restart_count: 4, state: "waiting",
          state_reason: "ImagePullBackOff" }] },
    ]),
    events: JSON.stringify([
      { event_type: "Warning", reason: "Failed", message: "Failed to pull image \"ghcr.io/app:sha-7f3a01\": not found",
        involved_object: "payments-api-7d9f4-q8m3", involved_object_kind: "Pod", count: 6,
        last_timestamp: "2026-07-27T10:00:12Z" },
      { event_type: "Normal", reason: "Pulled", message: "Successfully pulled image in 1.2s",
        involved_object: "payments-api-7d9f4-x2k1", involved_object_kind: "Pod", count: 1,
        last_timestamp: "2026-07-27T09:59:40Z" },
    ]),
    log_tails: JSON.stringify({
      "payments-api-7d9f4-x2k1/api": "10:00:01 INFO listening on :8080\n10:00:02 INFO ready",
    }),
    collected_at: "2026-07-27T10:00:13Z",
  },
};
// An honest empty success: no pods attributed (legitimate — the object may apply
// no workloads), but the collection itself worked.
const DIAG_EMPTY = {
  request: { ...DIAG_DONE.request },
  result: { request_id: "9f10ab22", pod_statuses: "[]",
    events: JSON.stringify([
      { event_type: "Normal", reason: "Created", message: "Created ConfigMap/payments-config",
        involved_object: "payments-config", involved_object_kind: "ConfigMap", count: 1 },
    ]),
    log_tails: null, collected_at: "2026-07-27T10:00:13Z" },
};
// A FAILED collection: the broker has no `failed` status, so this arrives as
// `completed` with a single `error` entry inside `events`.
const DIAG_ERROR = {
  request: { ...DIAG_DONE.request },
  result: { request_id: "9f10ab22", pod_statuses: "[]",
    events: JSON.stringify([{ error: "Failed to list pods in namespace payments: ApiError: pods is forbidden: User \"system:serviceaccount:brokkr:brokkr-agent\" cannot list resource \"pods\"" }]),
    log_tails: null, collected_at: "2026-07-27T10:00:13Z" },
};
const DIAG_MOCKS = {
  "/fleet": FLEET,
  "/agents/1b9d6bcd/target-state": TARGET_STATE,
  "/deployment-objects/d1a2b3c4/diagnostics": DIAG_CREATED,
};

// Tenants view (BROKKR-T-0318). `GET /generators` lists tenants; the mint
// dialog POSTs to the same path and gets back the created generator plus its
// one-time PAK — hence the method-aware mock keys below.
const GENERATORS = [
  { id: "1b9d6bcd-bbfd", name: "team-payments", description: "prod payments service",
    is_active: true, is_system: false, last_active_at: "2026-07-29T09:14:02Z" },
  { id: "7c9e6679-7425", name: "team-ingest", description: "event ingest",
    is_active: true, is_system: false, last_active_at: "2026-07-29T08:51:40Z" },
  { id: "a1b2c3d4-0000", name: "team-sandbox", description: null,
    is_active: false, is_system: false, last_active_at: null },
];
// The one-time secret the reveal panel shows. Distinct from the seeded
// `brokkr_pak` so the persistence assertion below cannot pass by accident.
const MINTED_PAK = "brokkr_MINTED9_Zx7QvT2mKp8sLd4NrB6yCw3EfH5jA1gU";
const CREATED_GENERATOR = {
  generator: { id: "f00dcafe-1234", name: "team-checkout", description: "new tenant",
    is_active: true, is_system: false, last_active_at: null },
  pak: MINTED_PAK,
};
// The admin PAK an operator would paste. Never stored by the console — asserted
// after the mint scene.
const TYPED_ADMIN_PAK = "brokkr_ADMINxx_TypedByOperatorNeverPersisted00";

const SCENES = [
  { name: "overview", mocks: { "/fleet": FLEET, "/agent-events": EVENTS } },
  { name: "fleet", nav: "Fleet", mocks: { "/fleet": FLEET } },
  { name: "fleet-empty", nav: "Fleet", mocks: { "/fleet": [] } },
  { name: "fleet-modal", nav: "Fleet", click: "prod-agent-01", mocks: { "/fleet": FLEET } },
  { name: "health", nav: "Broker health", mocks: { "/admin/ws/connections": WSCONN } },
  { name: "health-modal", nav: "Broker health", click: "1b9d6bcd-bbfd-4b2d-9b5d-ab8dfbbd4bed", mocks: { "/admin/ws/connections": WSCONN } },
  { name: "jobs", nav: "Work orders", mocks: { "/work-order-log": JOBS, "/work-orders": ACTIVE_WOS } },
  { name: "jobs-modal", nav: "Work orders", click: "completed", mocks: { "/work-order-log": JOBS, "/work-orders": ACTIVE_WOS } },
  { name: "webhooks", nav: "Webhooks", mocks: { "/webhooks": HOOKS } },
  { name: "webhooks-modal", nav: "Webhooks", click: "prod-alerts", mocks: { "/webhooks": HOOKS,
    "/webhooks/h1/deliveries": [
      { event_type: "stack.updated", status: "delivered", attempts: 1, last_error: null },
      { event_type: "agent.failed", status: "failed", attempts: 3, last_error: "connect ETIMEDOUT 10.0.0.4:443" },
    ] } },
  { name: "deployments", nav: "Deployments", mocks: { "/stacks": STACKS } },
  { name: "deployments-modal", nav: "Deployments", click: "payments-api", mocks: { "/stacks": STACKS,
    "/stacks/s1/health": { stack_id: "s1", overall_status: "degraded", deployment_objects: [
      { id: "d1a2b3c4", status: "healthy", healthy_agents: 3, degraded_agents: 0, failing_agents: 0 },
      { id: "e5f6a7b8", status: "degraded", healthy_agents: 1, degraded_agents: 2, failing_agents: 0 },
    ] } } },
  { name: "telemetry", nav: "Telemetry", mocks: { "/agent-events": TELEM } },
  { name: "telemetry-modal", nav: "Telemetry", click: "Apply", mocks: { "/agent-events": TELEM } },
  // Diagnostics request -> result (BROKKR-T-0301): open the agent modal, run a
  // diagnostic, and screenshot the polled outcome. Three outcomes that must not
  // look alike: a real collection, an empty-but-successful one, and a failure.
  { name: "fleet-diagnostic", nav: "Fleet", click: "prod-agent-01", then_click: "Run diagnostic",
    mocks: { ...DIAG_MOCKS, "/diagnostics/9f10ab22": DIAG_DONE } },
  { name: "fleet-diagnostic-empty", nav: "Fleet", click: "prod-agent-01", then_click: "Run diagnostic",
    mocks: { ...DIAG_MOCKS, "/diagnostics/9f10ab22": DIAG_EMPTY } },
  { name: "fleet-diagnostic-error", nav: "Fleet", click: "prod-agent-01", then_click: "Run diagnostic",
    mocks: { ...DIAG_MOCKS, "/diagnostics/9f10ab22": DIAG_ERROR } },
  // Scope selector (BROKKR-I-0032): selector visible with named PAKs...
  { name: "scope-selector", nav: "Fleet", mocks: { "/paks": PAKS, "/fleet": FLEET } },
  // ...and selecting a tenant narrows the fleet to its agents.
  { name: "fleet-scoped", nav: "Fleet", select: "team-payments",
    mocks: { "/paks": PAKS, "/fleet": FLEET, "/fleet?pak_id=1b9d6bcd-bbfd": FLEET_PAYMENTS } },
  // Tenants (BROKKR-T-0318): list, empty state, the mint dialog, and the
  // reveal-once panel. The last one is the whole point of the feature, so it is
  // driven end to end rather than screenshotted mid-form.
  { name: "tenants", nav: "Tenants", mocks: { "/generators": GENERATORS } },
  { name: "tenants-empty", nav: "Tenants", mocks: { "/generators": [] } },
  { name: "tenants-new", nav: "Tenants", click: "+ New tenant",
    mocks: { "/generators": GENERATORS } },
  { name: "tenants-minted", nav: "Tenants", click: "+ New tenant",
    fill: [["acme-payments", "team-checkout"], ["brokkr_…", TYPED_ADMIN_PAK]],
    then_click: "Create tenant",
    // Asserts the credential-handling criterion a screenshot cannot: the typed
    // admin PAK must appear nowhere in browser storage afterwards.
    assert_no_stored: TYPED_ADMIN_PAK,
    mocks: { "/generators": GENERATORS, "POST /generators": CREATED_GENERATOR } },
];

// ---- driver --------------------------------------------------------------
const browser = await chromium.launch();
const ctx = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
});
const page = await ctx.newPage();
const errs = [];
page.on("console", (m) => m.type() === "error" && errs.push(`[console] ${m.text()}`));
page.on("pageerror", (e) => errs.push(`[pageerror] ${e.message}`));

// seed a PAK so the fetch layer attaches auth (the mock ignores it).
await page.addInitScript(() => localStorage.setItem("brokkr_pak", "brokkr_BRtest_e2e"));

// /metrics is top-level (not under /api/v1) and Prometheus text.
await page.route("**/metrics", (route) =>
  route.fulfill({ status: 200, contentType: "text/plain", body: PROM })
);

let MOCKS = {};
await page.route("**/api/v1/**", (route) => {
  const url = new URL(route.request().url());
  const suffix = url.pathname.replace(/^\/api\/v1/, "");
  // Query-aware first (scoped fixtures like "/fleet?pak_id=..."), then bare
  // path. Trailing separators are stripped so URL-builder quirks can't dodge
  // a scoped fixture.
  const withQuery = (suffix + url.search).replace(/[&?]+$/, "");
  // Method-aware first (BROKKR-T-0318): `POST /generators` returns the created
  // generator + its one-time PAK, while `GET /generators` returns the list.
  // Keying on path alone cannot express both, and silently answering the POST
  // with the list array made the mint look like it failed.
  const method = route.request().method();
  const key = [`${method} ${withQuery}`, `${method} ${suffix}`, withQuery, suffix].find(
    (k) => k in MOCKS
  ) ?? suffix;
  // The scope selector fetches /paks on every scene; scenes that don't care
  // get an empty tenant list (selector hidden) instead of 404 noise.
  if (!(key in MOCKS) && suffix === "/paks") {
    return route.fulfill({ status: 200, contentType: "application/json", body: "[]" });
  }
  if (key in MOCKS) {
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(MOCKS[key]),
    });
  }
  return route.fulfill({
    status: 404,
    contentType: "application/json",
    body: JSON.stringify({ code: "not_found", message: `no mock for ${suffix}` }),
  });
});

/// The page header's title, or "" if it is not rendered yet.
async function headerTitle() {
  return (
    (await page.locator(".cl-page-header__title").first().textContent().catch(() => "")) ?? ""
  ).trim();
}

/// Click a sidebar nav item and *verify* the view changed, retrying if it did
/// not.
///
/// A fixed settle delay cannot be made correct here. Leptos renders the sidebar
/// before its click handlers respond, so a click in that window is accepted by
/// the DOM and silently does nothing — and how long the window lasts depends on
/// machine load, which in a 23-scene run with 2x full-page screenshots varies
/// by seconds. The original harness clicked once, swallowed every failure with
/// `.catch(() => {})`, and screenshotted whatever was on screen; most scenes
/// were quietly capturing the default Overview view.
///
/// Clicking until the header actually reads the target is deterministic
/// regardless of load, and fails loudly when the view genuinely does not exist.
async function navigateTo(scene, label) {
  for (let attempt = 1; attempt <= 8; attempt++) {
    await page
      .getByText(label, { exact: true })
      .first()
      .click({ timeout: 5000 })
      .catch(() => {});
    await page.waitForTimeout(300);
    if ((await headerTitle()) === label) {
      // Let the view's resources resolve before the caller screenshots.
      await page.waitForTimeout(700);
      return true;
    }
  }
  errs.push(
    `[nav] ${scene}: clicked "${label}" 8x but the header still reads "${await headerTitle()}" — screenshot would be of the wrong view`
  );
  return false;
}

for (const s of SCENES) {
  MOCKS = s.mocks || {};
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  // Wait for the WASM app to mount before interacting. `domcontentloaded` fires
  // long before Leptos has rendered anything, so clicking straight after it was
  // a race: the nav item did not exist yet, the click was swallowed by the
  // `.catch()` below, and the scene screenshotted whatever view was default.
  // That produced confident-looking screenshots of the wrong page.
  await page
    .getByText("control plane", { exact: true })
    .waitFor({ state: "visible", timeout: 15000 })
    .catch(() => errs.push(`[mount] ${s.name}: app never rendered`));
  if (s.nav) {
    await navigateTo(s.name, s.nav);
  } else {
    await page.waitForTimeout(800);
  }
  if (s.click) {
    await page.getByText(s.click, { exact: true }).first().click().catch(() => {});
    await page.waitForTimeout(500);
  }
  if (s.select) {
    await page.locator("select").last().selectOption({ label: s.select }).catch(() => {});
    await page.waitForTimeout(500);
  }
  // Type into fields by placeholder (BROKKR-T-0318's mint dialog). Aurora's
  // inputs carry no name/id, so the placeholder is the stable handle.
  if (s.fill) {
    for (const [placeholder, value] of s.fill) {
      await page
        .getByPlaceholder(placeholder)
        .first()
        .fill(value)
        .catch(() => {});
    }
    await page.waitForTimeout(200);
  }
  // A second click *inside* whatever the first one opened (the modal's "Run
  // diagnostic" button). Substring match: the button label carries a glyph.
  if (s.then_click) {
    await page.getByText(s.then_click).first().click().catch(() => {});
    await page.waitForTimeout(900);
  }
  await page.waitForTimeout(700);
  await page.screenshot({ path: `${OUT}/${s.name}.png`, fullPage: true });
  console.log(`shot: ${s.name}`);

  // Behavioural check, not a pixel one: a secret typed into the page must not
  // survive in localStorage or sessionStorage. A screenshot can show the reveal
  // panel looking right while the credential is quietly persisted, so this is
  // asserted rather than eyeballed (BROKKR-T-0318).
  if (s.assert_no_stored) {
    const leaked = await page.evaluate((needle) => {
      const hits = [];
      for (const store of [localStorage, sessionStorage]) {
        for (let i = 0; i < store.length; i++) {
          const k = store.key(i);
          if ((store.getItem(k) ?? "").includes(needle)) hits.push(k);
        }
      }
      return hits;
    }, s.assert_no_stored);
    if (leaked.length) {
      errs.push(
        `[assert] ${s.name}: the supplied admin PAK was persisted under ${leaked.join(", ")}`
      );
    } else {
      console.log(`  assert: admin PAK not persisted ✓`);
    }
  }
  // The selected scope persists in localStorage; clear it so scenes stay independent.
  await page.evaluate(() => localStorage.removeItem("brokkr_scope"));
}

console.log(errs.length ? `CONSOLE ERRORS:\n${errs.join("\n")}` : "no console errors");
await browser.close();
