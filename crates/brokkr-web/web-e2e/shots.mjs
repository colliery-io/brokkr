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

// scene = { name, nav?: sidebar label to click, mocks: { "/path": json } }
const SCENES = [
  { name: "overview", mocks: {} },
  { name: "fleet", nav: "Fleet", mocks: { "/fleet": FLEET } },
  { name: "fleet-empty", nav: "Fleet", mocks: { "/fleet": [] } },
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
