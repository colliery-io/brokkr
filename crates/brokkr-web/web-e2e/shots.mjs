// Screenshot every console view for visual / pixel verification. The broker API
// is mocked via Playwright route interception (per-scene fixtures), so views
// render with realistic data without a running broker. The console must be
// served first: `cd crates/brokkr-web && trunk serve --port 9080`.
//   Run: cd crates/brokkr-web/web-e2e && URL=http://127.0.0.1:9080 node shots.mjs
// Aurora has a light and a dark theme that follow the OS. THEME=light or
// THEME=dark emulates that OS setting (default dark); OUT sets the folder.
//   THEME=light OUT=shots/light node shots.mjs
// ONLY=<regex> runs only the scenes whose names match: ONLY=^session- node shots.mjs
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";

const BASE = process.env.URL || "http://127.0.0.1:9080";
const THEME = process.env.THEME === "light" ? "light" : "dark";
const OUT = process.env.OUT || "shots";
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
// A time `s` seconds ago, as the broker sends it (RFC 3339). The feed shows it
// as "3m ago" (BROKKR-T-0328).
const ago = (s) => new Date(Date.now() - s * 1000).toISOString();

const EVENTS = [
  { agent_id: "a1", event_type: "Apply", status: "success", message: "applied Deployment/payments (3 objects)", created_at: ago(42) },
  { agent_id: "a1", event_type: "Heartbeat", status: "success", message: "k8s reachable (12ms)", created_at: ago(190) },
  { agent_id: "a2", event_type: "Reconcile", status: "failure", message: "Service/ingest: port 8080 already allocated", created_at: ago(3700) },
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
// Per-stack telemetry (BROKKR-T-0338): the Kubernetes events and the pod log
// lines the agent reported for payments-api, with the retention the broker
// states on each answer.
const RETENTION = { retention_ceiling_seconds: 21600, effective_retention_seconds: 21600,
  oldest_available_ts: ago(5400), long_term_sink_hint: "" };
const K8S_EVENTS = { retention: RETENTION, events: [
  { id: "e1", agent_id: "a1", stack_id: "s1", observed_at: ago(35), reason: "BackOff", event_type: "Warning",
    message: "Back-off pulling image \"ghcr.io/app:sha-7f3a01\"", source: "kubelet",
    involved_object: { kind: "Pod", name: "payments-api-7d9f4-q8m3", namespace: "payments" } },
  { id: "e2", agent_id: "a1", stack_id: "s1", observed_at: ago(140), reason: "Scheduled", event_type: "Normal",
    message: "Successfully assigned payments/payments-api-7d9f4-x2k1 to node-1", source: "default-scheduler",
    involved_object: { kind: "Pod", name: "payments-api-7d9f4-x2k1", namespace: "payments" } },
  { id: "e3", agent_id: "a1", stack_id: "s1", observed_at: ago(141), reason: "ScalingReplicaSet", event_type: "Normal",
    message: "Scaled up replica set payments-api-7d9f4 to 2", source: "deployment-controller",
    involved_object: { kind: "Deployment", name: "payments-api", namespace: "payments" } },
] };
const POD_LOGS = { retention: RETENTION, lines: [
  { ts: ago(95), namespace: "payments", pod: "payments-api-7d9f4-x2k1", container: "api", line: "listening on :8080" },
  { ts: ago(60), namespace: "payments", pod: "payments-api-7d9f4-x2k1", container: "api", line: "GET /healthz 200 1ms" },
  { ts: ago(31), namespace: "payments", pod: "payments-api-7d9f4-q8m3", container: "api", line: "pulling image ghcr.io/app:sha-7f3a01" },
  { ts: ago(12), namespace: "payments", pod: "payments-api-7d9f4-x2k1", container: "api", line: "POST /charge 201 48ms" },
] };
// Named PAKs (tenants) for the scope selector (BROKKR-I-0032). IDs line up
// with STACKS.generator_id so scoped mocks stay coherent.
const PAKS = [
  { id: "1b9d6bcd-bbfd", name: "team-payments" },
  { id: "7c9e6679-7425", name: "team-ingest" },
];
// team-payments owns the two prod agents; team-ingest the staging one.
const FLEET_PAYMENTS = FLEET.slice(0, 2);
const TELEM = [
  { agent_id: "a1", event_type: "Apply", status: "success", message: "applied Deployment/payments (3 objects)", created_at: ago(42) },
  { agent_id: "a1", event_type: "Reconcile", status: "success", message: "no drift", created_at: ago(900) },
  // No created_at: an older broker. The row shows with no time.
  { agent_id: "a2", event_type: "Apply", status: "failure", message: "Service/ingest: port 8080 already allocated" },
];

// Diagnostics (BROKKR-T-0301). The Fleet modal picks a deployment object from
// the agent's target state, POSTs a request, keeps the returned id and polls
// GET /diagnostics/:id. The route mock is method-agnostic (keyed on path), so
// the POST is answered with the 201-shaped body below.
const TARGET_STATE = [
  { id: "d1a2b3c4", stack_id: "s1", sequence_id: 41, is_deletion_marker: false },
];
// Stack health for the Deployments cards (BROKKR-T-0337).
const S1_HEALTH = { stack_id: "s1", overall_status: "degraded", deployment_objects: [
  { id: "d1a2b3c4", status: "healthy", healthy_agents: 3, degraded_agents: 0, failing_agents: 0 },
  { id: "e5f6a7b8", status: "degraded", healthy_agents: 1, degraded_agents: 2, failing_agents: 0 },
] };
const S2_HEALTH = { stack_id: "s2", overall_status: "healthy", deployment_objects: [
  { id: "f9e8d7c6", status: "healthy", healthy_agents: 2, degraded_agents: 0, failing_agents: 0 },
] };

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
// any real credential, so the persistence assertion below cannot pass by accident.
const MINTED_PAK = "brokkr_MINTED9_Zx7QvT2mKp8sLd4NrB6yCw3EfH5jA1gU";
const CREATED_GENERATOR = {
  generator: { id: "f00dcafe-1234", name: "team-checkout", description: "new tenant",
    is_active: true, is_system: false, last_active_at: null },
  pak: MINTED_PAK,
};
// The admin PAK an operator would paste. Never stored by the console — asserted
// after the mint scene.
const TYPED_ADMIN_PAK = "brokkr_ADMINxx_TypedByOperatorNeverPersisted00";

// Pause/resume (BROKKR-T-0322). PUT /agents/:id answers with the updated agent;
// the modal reads back `status` so the pill flips without waiting for the 5s
// fleet refetch. Method-aware key so it does not collide with any GET.
const AGENT_PAUSED = { id: "1b9d6bcd", name: "prod-agent-01", status: "INACTIVE" };
const PAUSE_MOCKS = {
  "/fleet": FLEET,
  "/agents/1b9d6bcd/target-state": TARGET_STATE,
  "PUT /agents/1b9d6bcd": AGENT_PAUSED,
};

// Every mock the Deployments view and its drawer read: the stacks, their
// health, the tenants, and each agent's target state (BROKKR-T-0337).
const DEPLOY_MOCKS = {
  "/stacks": STACKS, "/stacks/s1/health": S1_HEALTH, "/stacks/s2/health": S2_HEALTH,
  "/generators": GENERATORS, "/fleet": FLEET,
  "/agents/1b9d6bcd/target-state": TARGET_STATE, "/agents/7c9e6679/target-state": TARGET_STATE,
  "/agents/a1b2c3d4/target-state": [],
};
// The fleet with the short ids the event fixtures use, so names resolve.
const FLEET_A = [{ ...FLEET[0], agent_id: "a1" }, { ...FLEET[1], agent_id: "a2" }];

// Day zero (BROKKR-T-0336): an agent record that no process has started yet,
// and the tenant list with one minted tenant.
const NEW_AGENT = { agent_id: "5e7d2c11-9a0b-4c3d-8e2f-1a2b3c4d5e6f", name: "checkout-agent-01", cluster_name: "prod-us-east-1",
  status: "INACTIVE", ws_connected: false, last_heartbeat: null, heartbeat_age_seconds: null,
  health_failing: 0, health_degraded: 0, pending_object_count: 0, pending_work_orders: 0, claimed_work_orders: 0 };
const EMPTY_SHELL = { "/fleet": [], "/agent-events": [], "/work-orders": [], "/stacks": [] };

// One sentence of each shell banner (src/app.rs), to find and count them.
const EXPIRED_BANNER = "Reload to get a new session.";
const REFUSED_BANNER = "The broker refused the token of this console.";
const BANNERS = [EXPIRED_BANNER, REFUSED_BANNER];
// The neutral state of a panel under a session banner (src/components.rs).
const WAITING = "Waiting for a new session.";

const SCENES = [
  // `settle` waits for a second /metrics poll, so the throughput shows a rate.
  { name: "overview", settle: 5500, mocks: { "/fleet": FLEET, "/agent-events": EVENTS } },
  // The two other layouts of design/README.md (BROKKR-T-0328): the segmented
  // control rearranges the five widgets.
  { name: "overview-grid", click: "grid", mocks: { "/fleet": FLEET, "/agent-events": EVENTS } },
  { name: "overview-stream", click: "stream", mocks: { "/fleet": FLEET, "/agent-events": EVENTS } },
  // Day zero (BROKKR-T-0336): every empty state says the next step.
  { name: "overview-empty", mocks: EMPTY_SHELL },
  { name: "deployments-empty", nav: "Deployments", mocks: { ...EMPTY_SHELL, "/stacks": [] } },
  { name: "telemetry-empty", nav: "Telemetry", mocks: { ...EMPTY_SHELL } },
  { name: "webhooks-empty", nav: "Webhooks", mocks: { ...EMPTY_SHELL, "/webhooks": [] } },
  { name: "jobs-empty", nav: "Work orders", mocks: { ...EMPTY_SHELL, "/work-order-log": [] } },
  // A new agent: no heartbeat, so health is unknown and the drawer offers Activate.
  { name: "fleet-new-agent", nav: "Fleet", mocks: { ...EMPTY_SHELL, "/fleet": [NEW_AGENT] } },
  { name: "fleet-new-agent-modal", nav: "Fleet", click: "checkout-agent-01",
    mocks: { ...EMPTY_SHELL, "/fleet": [NEW_AGENT], "/agents/5e7d2c11-9a0b-4c3d-8e2f-1a2b3c4d5e6f/target-state": [] } },
  { name: "fleet", nav: "Fleet", mocks: { "/fleet": FLEET } },
  { name: "fleet-empty", nav: "Fleet", mocks: { "/fleet": [] } },
  { name: "fleet-modal", nav: "Fleet", click: "prod-agent-01",
    mocks: { "/fleet": FLEET, "/stacks": STACKS, "/agents/1b9d6bcd/target-state": TARGET_STATE } },
  // Links between views (BROKKR-T-0337): a hash with a selection opens the
  // drawer, and Telemetry filters to one agent.
  { name: "fleet-by-link", hash: "#fleet/agent/1b9d6bcd", settle: 800,
    mocks: { "/fleet": FLEET, "/stacks": STACKS, "/agents/1b9d6bcd/target-state": TARGET_STATE } },
  { name: "deployments-by-link", hash: "#deployments/stack/s1", settle: 1200, mocks: DEPLOY_MOCKS },
  { name: "telemetry-agent", hash: "#telemetry/agent/a1", settle: 800,
    mocks: { "/agent-events": TELEM, "/stacks": STACKS, "/fleet": FLEET_A } },
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
  { name: "deployments", nav: "Deployments", mocks: DEPLOY_MOCKS },
  { name: "deployments-modal", nav: "Deployments", click: "payments-api", settle: 1200, mocks: DEPLOY_MOCKS },
  { name: "telemetry", nav: "Telemetry", mocks: { "/agent-events": TELEM, "/stacks": STACKS } },
  { name: "telemetry-modal", nav: "Telemetry", click: "Apply", mocks: { "/agent-events": TELEM, "/stacks": STACKS } },
  // Per-stack tabs (BROKKR-T-0338): pick a stack, then its Kubernetes events
  // or its pod logs; and the Pod logs tab with no stack picked.
  { name: "telemetry-kube-events", nav: "Telemetry", select: "payments-api", tab: "Kube events",
    mocks: { "/agent-events": TELEM, "/stacks": STACKS, "/stacks/s1/events": K8S_EVENTS } },
  { name: "telemetry-logs", nav: "Telemetry", select: "payments-api", tab: "Pod logs",
    mocks: { "/agent-events": TELEM, "/stacks": STACKS, "/stacks/s1/logs": POD_LOGS } },
  { name: "telemetry-logs-no-stack", nav: "Telemetry", tab: "Pod logs",
    mocks: { "/agent-events": TELEM, "/stacks": STACKS } },
  // Diagnostics request -> result (BROKKR-T-0301): open the agent modal, run a
  // diagnostic, and screenshot the polled outcome. Three outcomes that must not
  // look alike: a real collection, an empty-but-successful one, and a failure.
  { name: "fleet-diagnostic", nav: "Fleet", click: "prod-agent-01", then_click: "Run diagnostic",
    mocks: { ...DIAG_MOCKS, "/diagnostics/9f10ab22": DIAG_DONE } },
  // Still collecting: the indeterminate progress bar (Aurora's Meter).
  { name: "fleet-diagnostic-pending", nav: "Fleet", click: "prod-agent-01", then_click: "Run diagnostic",
    mocks: { ...DIAG_MOCKS, "/diagnostics/9f10ab22": { request: DIAG_CREATED } } },
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
  // The agent modal's pause control, and the state after pausing.
  { name: "fleet-pause", nav: "Fleet", click: "prod-agent-01", mocks: PAUSE_MOCKS },
  { name: "fleet-paused", nav: "Fleet", click: "prod-agent-01",
    fill: [["brokkr_\u2026", TYPED_ADMIN_PAK]], then_click: "Pause",
    assert_no_stored: TYPED_ADMIN_PAK,
    mocks: PAUSE_MOCKS },
  { name: "tenants", nav: "Tenants", mocks: { "/generators": GENERATORS } },
  { name: "tenants-empty", nav: "Tenants", mocks: { "/generators": [] } },
  // A wrong admin PAK: the dialog says so and keeps the form (BROKKR-T-0336).
  { name: "tenants-rejected", nav: "Tenants", click: "+ New tenant",
    fill: [["acme-payments", "team-checkout"], ["brokkr_\u2026", "brokkr_wrongpak"]], then_click: "Create tenant",
    expect_http: [403],
    mocks: { "/generators": [], "POST /generators": { __status: 403, code: "forbidden", message: "admin required" } } },
  { name: "tenants-new", nav: "Tenants", click: "+ New tenant",
    mocks: { "/generators": GENERATORS } },
  { name: "tenants-minted", nav: "Tenants", click: "+ New tenant",
    fill: [["acme-payments", "team-checkout"], ["brokkr_…", TYPED_ADMIN_PAK]],
    then_click: "Create tenant",
    // Asserts the credential-handling criterion a screenshot cannot: the typed
    // admin PAK must appear nowhere in browser storage afterwards.
    assert_no_stored: TYPED_ADMIN_PAK,
    mocks: { "/generators": GENERATORS, "POST /generators": CREATED_GENERATOR } },
  // A stale session (BROKKR-T-0340): the first reads succeed, then the broker
  // restarts (or a load balancer picks another replica) and every request is
  // refused. `expire` sets the status of every API answer after the Overview
  // has loaded; the navigation then reads with the dead token. The shell must
  // show one banner, and the indicator must say "session expired".
  // `banner` is a sentence of the one banner the scene must show (null: no
  // banner); `indicator` is the text of the top bar indicator.
  // `assert_reload` clicks Reload against a healthy broker and asserts the
  // banner is gone.
  { name: "session-expired-401", nav: "Fleet", expire: 401, expect_http: [401], assert_reload: true,
    banner: EXPIRED_BANNER, indicator: "session expired" },
  { name: "session-expired-403", nav: "Deployments", expire: 403, expect_http: [403],
    banner: EXPIRED_BANNER, indicator: "session expired" },
  // A token refused from the first request (BROKKR-T-0345): the broker
  // answered, so the indicator says "token refused" and one banner says what
  // to do.
  { name: "session-refused-at-load", expire: 401, expire_at_load: true, expect_http: [401], assert_reload: true,
    banner: REFUSED_BANNER, indicator: "token refused" },
  { name: "session-refused-at-load-403", expire: 403, expire_at_load: true, expect_http: [403],
    banner: REFUSED_BANNER, indicator: "token refused" },
  // No answer at all (the connection is refused): the broker is unreachable,
  // and there is no banner.
  { name: "session-broker-unreachable", expire: "abort", expire_at_load: true, expect_net: true,
    banner: null, indicator: "broker unreachable" },
  // An ordinary failure (BROKKR-T-0347): a 500 is not a refused token, so the
  // panel keeps Aurora's error state with its Retry button. `assert_error` is
  // the title of that error state.
  { name: "panel-server-error", nav: "Webhooks", expect_http: [500], assert_error: "Something went wrong",
    mocks: { "/webhooks": { __status: 500, code: "internal", message: "database unavailable" } } },
];

// ---- driver --------------------------------------------------------------
const browser = await chromium.launch();
const ctx = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
  colorScheme: THEME,
});
const page = await ctx.newPage();
const errs = [];
// A scene that mocks an HTTP error on purpose (`expect_http: [403]`) is not a
// console error: the browser logs the failed load, and this filters it.
let EXPECT_HTTP = new Set();
// A scene that drops the connection on purpose (`expect_net: true`) expects
// the browser's network errors too.
let EXPECT_NET = false;
page.on("console", (m) => {
  if (m.type() !== "error") return;
  const t = m.text();
  const hit = /status of (\d+)/.exec(t);
  if (hit && EXPECT_HTTP.has(Number(hit[1]))) return;
  if (EXPECT_NET && /net::ERR_/.test(t)) return;
  errs.push(`[console] ${t}`);
});
page.on("pageerror", (e) => errs.push(`[pageerror] ${e.message}`));

// seed a PAK so the fetch layer attaches auth (the mock ignores it).
// No PAK is seeded: the route mocks below fulfil regardless of headers, so the
// console needs no credential here. The old `localStorage["brokkr_pak"]` seed
// was cosmetic even before that override was removed (BROKKR-T-0320) -- its own
// comment conceded "the mock ignores it".

// /metrics is top-level (not under /api/v1) and Prometheus text.
// The counter grows on each poll, so the Overview can show a rate
// (BROKKR-T-0339): 60 requests per 5 s poll is 720 per minute.
let metricsHits = 0;
await page.route("**/metrics", (route) => {
  const body = PROM.replace(/(brokkr_http_requests_total\{[^}]*\}) (\d+)/, (_, m, n) => `${m} ${Number(n) + 60 * metricsHits++}`);
  return route.fulfill({ status: 200, contentType: "text/plain", body });
});

let MOCKS = {};
// Non-zero: every API request answers with this status (a stale session).
// "abort": every API request fails with no answer (the broker is down).
let EXPIRE = 0;
await page.route("**/api/v1/**", (route) => {
  if (EXPIRE === "abort") return route.abort("connectionrefused");
  if (EXPIRE) {
    return route.fulfill({
      status: EXPIRE,
      contentType: "application/json",
      body: JSON.stringify({ code: EXPIRE === 401 ? "unauthorized" : "forbidden", message: "" }),
    });
  }
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
  // The shell reads the fleet (nav count, broker live state) and the active
  // work orders (nav count) on every view, and every scene opens on the
  // Overview (agent events) before it navigates; scenes that don't mock
  // these get the standard fixtures instead of 404 noise.
  // Fleet reads /stacks and Deployments reads /generators for names
  // (BROKKR-T-0337), so those get fixtures too.
  const SHELL = { "/fleet": FLEET, "/work-orders": ACTIVE_WOS, "/agent-events": EVENTS, "/stacks": STACKS, "/generators": GENERATORS };
  if (!(key in MOCKS) && suffix in SHELL) {
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(SHELL[suffix]),
    });
  }
  if (key in MOCKS) {
    // A fixture with `__status` answers with that HTTP status (BROKKR-T-0336:
    // the rejected-PAK scene needs a 403).
    const body = MOCKS[key];
    const status = body && body.__status ? body.__status : 200;
    // `__status` is for the mock only; the broker body does not carry it.
    const sent = body && body.__status ? { ...body, __status: undefined } : body;
    return route.fulfill({ status, contentType: "application/json", body: JSON.stringify(sent) });
  }
  return route.fulfill({
    status: 404,
    contentType: "application/json",
    body: JSON.stringify({ code: "not_found", message: `no mock for ${suffix}` }),
  });
});

/// How many elements with exactly `text` are visible.
async function visibleCount(text) {
  let n = 0;
  for (const el of await page.getByText(text, { exact: true }).all()) {
    if (await el.isVisible()) n++;
  }
  return n;
}

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

// ONLY=<regex> runs the scenes whose names match (ONLY=^session- for one feature).
const ONLY = process.env.ONLY ? new RegExp(process.env.ONLY) : null;
for (const s of SCENES.filter((x) => !ONLY || ONLY.test(x.name))) {
  MOCKS = s.mocks || {};
  EXPECT_HTTP = new Set(s.expect_http || []);
  EXPECT_NET = !!s.expect_net;
  EXPIRE = s.expire && s.expire_at_load ? s.expire : 0;
  // `hash` opens a view with a selection (BROKKR-T-0337): `#fleet/agent/<id>`.
  await page.goto(BASE + (s.hash || ""), { waitUntil: "domcontentloaded" });
  // Wait for the WASM app to mount before interacting. `domcontentloaded` fires
  // long before Leptos has rendered anything, so clicking straight after it was
  // a race: the nav item did not exist yet, the click was swallowed by the
  // `.catch()` below, and the scene screenshotted whatever view was default.
  // That produced confident-looking screenshots of the wrong page.
  await page
    .getByText("control plane", { exact: true })
    .waitFor({ state: "visible", timeout: 15000 })
    .catch(() => errs.push(`[mount] ${s.name}: app never rendered`));
  if (s.expire && !s.expire_at_load) {
    // Let the first reads succeed, then refuse everything after.
    await page.getByText("broker ready").waitFor({ timeout: 10000 })
      .catch(() => errs.push(`[expire] ${s.name}: the first reads never succeeded`));
    EXPIRE = s.expire;
  }
  if (s.nav) {
    await navigateTo(s.name, s.nav);
  } else {
    await page.waitForTimeout(800);
  }
  if (s.settle) await page.waitForTimeout(s.settle);
  if (s.click) {
    await page.getByText(s.click, { exact: true }).first().click().catch(() => {});
    await page.waitForTimeout(500);
  }
  // A tab, by its accessible name: a substring click would hit the page
  // subtitle ("kube events · pod logs") first.
  if (s.tab) {
    await page.getByRole("tab", { name: s.tab }).click().catch(() => {});
    await page.waitForTimeout(700);
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
  // Extra settle time after the actions: a second /metrics poll, or the
  // drawer's own fetches.
  if (s.settle) await page.waitForTimeout(s.settle);
  // A second click *inside* whatever the first one opened (the modal's "Run
  // diagnostic" button). Substring match: the button label carries a glyph.
  if (s.then_click) {
    await page.getByText(s.then_click).first().click().catch(() => {});
    await page.waitForTimeout(900);
  }
  await page.waitForTimeout(700);
  await page.screenshot({ path: `${OUT}/${s.name}.png`, fullPage: true });
  console.log(`shot: ${s.name}`);

  // The session scenes assert what the screenshot shows: exactly the
  // expected banner (or none), and the indicator text.
  if (s.expire) {
    let total = 0;
    for (const b of BANNERS) total += await page.getByText(b).count();
    const mine = s.banner ? await page.getByText(s.banner).count() : 0;
    const want = s.banner ? 1 : 0;
    const shown = await page.getByText(s.indicator, { exact: true }).count();
    if (total !== want || mine !== want || shown !== 1) {
      errs.push(`[assert] ${s.name}: ${total} session banner(s), want ${want}; "${s.indicator}" shown ${shown}x`);
    } else {
      console.log(`  assert: ${want} session banner, indicator "${s.indicator}" ✓`);
    }
  }
  // Under a session banner, a panel that got 401/403 waits quietly: no
  // "Not authorized" with the raw broker body (BROKKR-T-0347).
  if (s.expire && s.banner) {
    const raw = await visibleCount("Not authorized");
    const waiting = await visibleCount(WAITING);
    if (raw !== 0 || waiting < 1) {
      errs.push(`[assert] ${s.name}: "Not authorized" shown ${raw}x, "${WAITING}" shown ${waiting}x`);
    } else {
      console.log(`  assert: no "Not authorized", ${waiting} panel(s) wait for a new session ✓`);
    }
  }
  // Any other error keeps Aurora's error state and its Retry.
  if (s.assert_error) {
    const title = await visibleCount(s.assert_error);
    const retry = await page.getByRole("button", { name: "Retry" }).count();
    const waiting = await visibleCount(WAITING);
    if (title < 1 || retry < 1 || waiting !== 0) {
      errs.push(`[assert] ${s.name}: "${s.assert_error}" shown ${title}x, Retry ${retry}x, waiting ${waiting}x`);
    } else {
      console.log(`  assert: "${s.assert_error}" with Retry, no waiting panel ✓`);
    }
  }
  // Reload against a healthy broker: the page gets a new token, the banner goes.
  if (s.assert_reload) {
    EXPIRE = 0;
    await page.getByRole("button", { name: "Reload" }).click();
    await page.getByText("broker ready").waitFor({ timeout: 10000 }).catch(() => {});
    const left = await page.getByText(s.banner).count();
    if (left) errs.push(`[assert] ${s.name}: the banner is still there after Reload`);
    else console.log("  assert: Reload clears the banner ✓");
  }
  EXPIRE = 0;

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
  // The selected scope and the Overview layout persist in localStorage; clear
  // them so scenes stay independent.
  await page.evaluate(() => {
    localStorage.removeItem("brokkr_scope");
    localStorage.removeItem("brokkr_overview_layout");
  });
}

console.log(errs.length ? `CONSOLE ERRORS:\n${errs.join("\n")}` : "no console errors");
await browser.close();
