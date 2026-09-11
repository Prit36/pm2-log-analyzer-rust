/**
 * Debug probe: paste smoke.log into both apps and dump a region's innerText and
 * (optionally) outerHTML for direct comparison.
 *
 * Usage: node probe.mjs <region> [--html]
 */
import fs from "node:fs";
import { chromium } from "playwright";

const region = process.argv[2] ?? "latency-chart";
const withHtml = process.argv.includes("--html");
const sample = fs.readFileSync("../smoke.log", "utf8");

const FALLBACKS = {
  header: "header",
  "ingest-panel": "section:has(button:has-text('Browse files'))",
  "kpi-row": "section:has(div:has-text('Requests'))",
  "filter-bar": "section:has(span:has-text('Normalize')), section:has(select)",
  "api-table": "section:has(h2:has-text('Slow API endpoints'))",
  "latency-chart": "section:has(h2:has-text('API Visual Analytics'))",
  "cron-table": "section:has(h2:has-text('Cron jobs'))",
  "skipped-disclosure": "details",
};

async function prepare(page) {
  const clearBtn = await page.$("button:has-text('Clear')");
  if (clearBtn && (await clearBtn.isEnabled())) {
    await clearBtn.click();
    await page.waitForTimeout(250);
  }
  const pasteBtn = await page.$("button:has-text('Paste logs'), button:has-text('Paste')");
  const textarea = await page.$("textarea#paste-logs");
  if (!textarea && pasteBtn) {
    await pasteBtn.click();
    await page.waitForTimeout(250);
  }
  const ta = await page.$("textarea#paste-logs");
  if (ta) {
    await ta.fill("");
    await ta.focus();
    await page.keyboard.insertText(sample);
    await page.waitForTimeout(150);
    const analyze = await page.$("button:has-text('Analyze paste')");
    if (analyze) await analyze.click();
  }
  await page.waitForTimeout(1800);
}

async function findRegion(page) {
  const el = await page.$(`[data-testid="${region}"], ${FALLBACKS[region] ?? "section"}`);
  return el;
}

const browser = await chromium.launch();
const react = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await react.goto("http://localhost:5173", { waitUntil: "networkidle" });
await react.evaluate(() => localStorage.clear());
await react.reload({ waitUntil: "networkidle" });
await prepare(react);
const reactEl = await findRegion(react);
console.log("=============== REACT innerText ===============");
console.log(await reactEl.innerText());
if (withHtml) {
  fs.writeFileSync(`../parity/out/react_${region}.html`, await reactEl.evaluate((e) => e.outerHTML));
}
await react.screenshot({ path: `../parity/out/probe_react_${region}.png`, fullPage: true });

const cdp = await chromium.connectOverCDP("http://localhost:9222");
const ctx = cdp.contexts()[0];
const rust = ctx.pages()[0];
await rust.setViewportSize({ width: 1440, height: 900 });
await prepare(rust);
const rustEl = await findRegion(rust);
console.log("=============== RUST innerText ===============");
console.log(await rustEl.innerText());
if (withHtml) {
  fs.writeFileSync(`../parity/out/rust_${region}.html`, await rustEl.evaluate((e) => e.outerHTML));
}
await rust.screenshot({ path: `../parity/out/probe_rust_${region}.png`, fullPage: true });

await browser.close();
await cdp.close();
