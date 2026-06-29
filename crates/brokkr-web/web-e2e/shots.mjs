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
  { agent_id: "1b9d6bcd", name: "prod-agent-01", status: "ACTIVE", ws_connected: true,
    heartbeat_age_seconds: 3, health_failing: 0, health_degraded: 0,
    pending_object_count: 2, pending_work_orders: 0, claimed_work_orders: 1 },
  { agent_id: "7c9e6679", name: "prod-agent-02", status: "ACTIVE", ws_connected: false,
    heartbeat_age_seconds: 42, health_failing: 0, health_degraded: 2,
    pending_object_count: 0, pending_work_orders: 1, claimed_work_orders: 0 },
  { agent_id: "a1b2c3d4", name: "staging-agent-01", status: "INACTIVE", ws_connected: false,
    heartbeat_age_seconds: 900, health_failing: 1, health_degraded: 0,
    pending_object_count: 0, pending_work_orders: 0, claimed_work_orders: 0 },
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
const SCENES = [
  { name: "overview", mocks: {} },
  { name: "fleet", nav: "Fleet", mocks: { "/fleet": FLEET } },
  { name: "fleet-empty", nav: "Fleet", mocks: { "/fleet": [] } },
  { name: "health", nav: "Broker health", mocks: { "/admin/ws/connections": WSCONN } },
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
  const suffix = new URL(route.request().url()).pathname.replace(/^\/api\/v1/, "");
  if (suffix in MOCKS) {
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(MOCKS[suffix]),
    });
  }
  return route.fulfill({
    status: 404,
    contentType: "application/json",
    body: JSON.stringify({ code: "not_found", message: `no mock for ${suffix}` }),
  });
});

for (const s of SCENES) {
  MOCKS = s.mocks || {};
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  if (s.nav) {
    await page.getByText(s.nav, { exact: true }).first().click().catch(() => {});
  }
  await page.waitForTimeout(1200);
  await page.screenshot({ path: `${OUT}/${s.name}.png`, fullPage: true });
  console.log(`shot: ${s.name}`);
}

console.log(errs.length ? `CONSOLE ERRORS:\n${errs.join("\n")}` : "no console errors");
await browser.close();
