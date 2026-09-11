/**
 * Region diff inspector: pastes smoke.log into both apps, crops a region and
 * writes ref/candidate/diff crops for visual inspection.
 *
 * Usage: node region-diff.mjs <region>
 */
import fs from "node:fs";
import { chromium } from "playwright";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";

const region = process.argv[2] ?? "latency-chart";
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
  const pasteBtn = await page.$("button:has-text('Paste logs')");
  const textarea = await page.$("textarea#paste-logs");
  if (!textarea && pasteBtn) {
    await pasteBtn.click();
    await page.waitForTimeout(300);
  }
  const ta = await page.$("textarea#paste-logs");
  await ta.fill("");
  await ta.focus();
  await page.keyboard.insertText(sample);
  await page.waitForTimeout(200);
  await (await page.$("button:has-text('Analyze paste')")).click();
  await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", {
    timeout: 15000,
  });
  await page.waitForTimeout(3500);
}

async function shot(page, path) {
  const box = await (await page.$(`[data-testid="${region}"], ${FALLBACKS[region]}`)).boundingBox();
  await page.screenshot({
    path,
    clip: { x: box.x, y: box.y, width: box.width, height: box.height },
  });
  return box;
}

const browser = await chromium.launch();
const react = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await react.goto("http://localhost:5173", { waitUntil: "networkidle" });
await react.evaluate(() => localStorage.clear());
await react.reload({ waitUntil: "networkidle" });
await prepare(react);
const refBox = await shot(react, `../parity/out/crop_ref_${region}.png`);

const cdp = await chromium.connectOverCDP("http://localhost:9222");
const ctx = cdp.contexts()[0];
const rust = ctx.pages()[0];
await rust.setViewportSize({ width: 1440, height: 900 });
await prepare(rust);
const rustBox = await shot(rust, `../parity/out/crop_rust_${region}.png`);

const a = PNG.sync.read(fs.readFileSync(`../parity/out/crop_ref_${region}.png`));
const b = PNG.sync.read(fs.readFileSync(`../parity/out/crop_rust_${region}.png`));
const w = Math.min(a.width, b.width);
const h = Math.min(a.height, b.height);
const diff = new PNG({ width: w, height: h });
const n = pixelmatch(a.data, b.data, diff.data, w, h, { threshold: 0.1 });
fs.writeFileSync(`../parity/out/crop_diff_${region}.png`, PNG.sync.write(diff));
// Row/column histograms of differing pixels to localize the difference.
const cols = new Array(w).fill(0);
const rows = new Array(h).fill(0);
for (let y = 0; y < h; y++) {
  for (let x = 0; x < w; x++) {
    const i = (w * y + x) << 2;
    if (diff.data[i] !== 0 || diff.data[i + 1] !== 0 || diff.data[i + 2] !== 0) {
      cols[x]++;
      rows[y]++;
    }
  }
}
const topCols = cols.map((c, i) => [i, c]).sort((p, q) => q[1] - p[1]).slice(0, 12);
const topRows = rows.map((c, i) => [i, c]).sort((p, q) => q[1] - p[1]).slice(0, 12);
console.log(`region=${region} ref=${refBox.width}x${refBox.height} rust=${rustBox.width}x${rustBox.height}`);
console.log(`diff pixels: ${n} / ${w * h} (${(100 - (n / (w * h)) * 100).toFixed(2)}% similar)`);
console.log("worst cols:", JSON.stringify(topCols));
console.log("worst rows:", JSON.stringify(topRows));

await browser.close();
await cdp.close();
