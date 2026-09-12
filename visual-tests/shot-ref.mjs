// Reference screenshot capture (Chromium) for pixel-parity checks.
// Usage: node shot-ref.mjs <log-file> <out.png> [width] [height] [--mongo]
import { chromium } from "playwright";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RUST_DIR = path.resolve(__dirname, "..");
const logFile = path.resolve(process.argv[2] ?? path.join(RUST_DIR, "smoke.log"));
const outFile = path.resolve(process.argv[3] ?? "/tmp/ref.png");
const width = Number(process.argv[4] ?? 1024);
const height = Number(process.argv[5] ?? 768);
const mongo = process.argv.includes("--mongo");
const viewIndex = process.argv.indexOf("--view");
const view = viewIndex > 0 ? process.argv[viewIndex + 1] : null;

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width, height },
  deviceScaleFactor: 2,
});
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });

if (mongo) {
  await page.click("button:has-text('MongoDB Logs')");
  await page.setInputFiles("input[type=file]", logFile);
  await page.waitForSelector("text=Slow Queries", { timeout: 300000 });
  if (view) {
    await page.click(`button:has-text('${view}')`);
  }
} else {
  // Load through the hidden file input so the paste panel stays closed, which
  // matches the native preview state (no extra card in the layout).
  await page.setInputFiles("input[type=file]", logFile);
  await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", {
    timeout: 120000,
  });
}

// Let the toast auto-hide so nothing overlaps the layout.
await page.waitForTimeout(4200);
await page.screenshot({ path: outFile });
console.log(`reference shot: ${outFile}`);
await browser.close();
