// Capture the reference app's Excel export for parity comparison.
// Usage: node export-ref.mjs <log-file> <out.xlsx> [--layout wide] [--mongo]
import { chromium } from "playwright";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RUST_DIR = path.resolve(__dirname, "..");
const logFile = process.argv[2] ? path.resolve(process.argv[2]) : path.resolve(RUST_DIR, "smoke.log");
const outFile = process.argv[3] ? path.resolve(process.argv[3]) : path.resolve(RUST_DIR, "target/parity/export_ref.xlsx");
const text = fs.readFileSync(logFile, "utf-8");
const mongo = process.argv.includes("--mongo");

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });

if (mongo) {
  // Load the mongod log through the hidden file input and export the workbook.
  await page.click("button:has-text('MongoDB Logs')");
  await page.setInputFiles("input[type=file]", logFile);
  await page.waitForSelector("text=Slow Queries", { timeout: 300000 });
  await page.waitForTimeout(1500);
  const downloadPromise = page.waitForEvent("download", { timeout: 300000 });
  await page.click("button[title*='Export'], button:has-text('Export')");
  const download = await downloadPromise;
  fs.mkdirSync(path.dirname(outFile), { recursive: true });
  await download.saveAs(outFile);
  console.log(`reference mongo export: ${outFile} (${fs.statSync(outFile).size} bytes)`);
  await browser.close();
  process.exit(0);
}

// Open the paste panel and submit the log.
const pasteBtn = await page.$("button:has-text('Paste logs')");
if (pasteBtn) await pasteBtn.click();
let textarea = await page.$("textarea#paste-logs, textarea[placeholder*='Paste']");
if (!textarea) {
  const btn = await page.$("button:has-text('Paste logs')");
  if (btn) await btn.click();
  textarea = await page.$("textarea#paste-logs, textarea[placeholder*='Paste']");
}
if (!textarea) throw new Error("paste textarea not found");
await textarea.fill("");
await textarea.focus();
await page.keyboard.insertText(text);
await page.waitForTimeout(200);
const analyze = await page.$("button:has-text('Analyze paste')");
if (analyze) await analyze.click();

await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", {
  timeout: 30000,
});
await page.waitForTimeout(1500);

const downloadPromise = page.waitForEvent("download", { timeout: 120000 });
await page.click("button[title*='Export'], button:has-text('Export')");
const download = await downloadPromise;
fs.mkdirSync(path.dirname(outFile), { recursive: true });
await download.saveAs(outFile);
console.log(`reference export: ${outFile} (${fs.statSync(outFile).size} bytes)`);
await browser.close();
