// Reads one view of the operator console of a running broker and prints what
// it shows as JSON (BROKKR-T-0400). The BDD steps run it and check the JSON,
// so the checks stay in Rust with the other steps.
//
//   node read.mjs fleet      KPI tiles, and per agent row: name, status, health
//   node read.mjs overview   KPI tiles, the rows of the Work orders panel, and
//                            the rows of the Fleet by cluster panel
//
// BROKER_URL is the broker that serves the console (default
// http://localhost:3000). The broker puts a read-only token in the page, so
// the browser needs no login.
//
// Unlike crates/brokkr-web/web-e2e, nothing is mocked: the console reads the
// real broker. Exit 0 prints the JSON; exit 1 prints the reason on stderr.

// The browser lives in node_modules, as in web-e2e (BROKKR-T-0354). The
// import comes after the variable is set, because Playwright reads it on load.
process.env.PLAYWRIGHT_BROWSERS_PATH ??= "0";
const { chromium } = await import("@playwright/test");

const VIEWS = { fleet: "Fleet", overview: "Overview" };
const view = process.argv[2];
if (!VIEWS[view]) {
  console.error(`usage: node read.mjs <${Object.keys(VIEWS).join("|")}>`);
  process.exit(2);
}
const base = (process.env.BROKER_URL || "http://localhost:3000").replace(/\/+$/, "");
const TIMEOUT = 30_000;

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  await page.goto(`${base}/#${view}`, { timeout: TIMEOUT });
  await page.waitForFunction(
    (title) => document.querySelector(".cl-page-header__title")?.textContent?.trim() === title,
    VIEWS[view],
    { timeout: TIMEOUT },
  );
  const result = view === "fleet" ? await readFleet(page) : await readOverview(page);
  console.log(JSON.stringify(result));
} catch (e) {
  console.error(`console ${view}: ${e.message}`);
  process.exitCode = 1;
} finally {
  await browser.close();
}

// Wait until `selector` holds tiles or an empty state, not a loader.
async function settled(page, selector) {
  await page.waitForFunction(
    (sel) => {
      const el = document.querySelector(sel);
      return el && !el.querySelector(".cl-loader") && el.textContent.trim() !== "";
    },
    selector,
    { timeout: TIMEOUT },
  );
}

// The label and value of each KPI tile in `scope`.
async function tiles(page, scope) {
  return page.$$eval(`${scope} .cl-stat`, (els) =>
    Object.fromEntries(
      els.map((el) => [
        el.querySelector(".cl-stat__label")?.textContent.trim(),
        el.querySelector(".cl-stat__value")?.textContent.trim(),
      ]),
    ),
  );
}

async function readFleet(page) {
  // The main element holds the KPI strip, or the empty state of no agents.
  await page.waitForFunction(
    () => document.querySelector(".brk-kpis .cl-stat") || document.querySelector(".cl-empty"),
    null,
    { timeout: TIMEOUT },
  );
  const agents = await page.$$eval(".cl-panel tbody tr", (rows) =>
    rows.map((tr) => {
      const cells = [...tr.querySelectorAll("td")];
      const pill = (td) => td?.querySelector(".cl-pill")?.textContent.trim() ?? null;
      return {
        name: cells[0]?.textContent.trim() ?? "",
        status: pill(cells[1]),
        health: pill(cells[2]),
      };
    }),
  );
  return { tiles: await tiles(page, ".brk-kpis"), agents };
}

async function readOverview(page) {
  await settled(page, ".brk-ov-kpis");
  await settled(page, ".brk-ov-jobs .cl-panel");
  const workOrders = await page.$$eval(".brk-ov-jobs .cl-panel .cl-pill", (pills) => {
    // Each row is a group: the first 8 characters of the id, then the type
    // pill, then the status pill.
    const rows = new Map();
    for (const p of pills) {
      const row = p.parentElement;
      if (!rows.has(row)) rows.set(row, []);
      rows.get(row).push(p.textContent.trim());
    }
    return [...rows.entries()].map(([row, [work_type, status]]) => ({
      id8: row.querySelector(".brk-value")?.textContent.trim() ?? "",
      work_type,
      status,
    }));
  });
  // Each cluster row of the Fleet by cluster panel: its name, and "<up>/<total> up".
  await settled(page, ".brk-ov-fleet .cl-panel");
  const clusters = await page.$$eval(".brk-ov-fleet .brk-value", (names) =>
    names.map((n) => ({
      name: n.textContent.trim(),
      caption: n.parentElement.querySelector(".brk-meta")?.textContent.trim() ?? "",
    })),
  );
  return { tiles: await tiles(page, ".brk-ov-kpis"), work_orders: workOrders, clusters };
}
