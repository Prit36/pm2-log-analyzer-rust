import { chromium } from "playwright";
import fs from "fs";

const sampleLogText = fs.readFileSync("smoke.log", "utf-8");

const browser = await chromium.launch();
const reactContext = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const reactPage = await reactContext.newPage();
await reactPage.goto("http://localhost:5173");

const cdpBrowser = await chromium.connectOverCDP("http://localhost:9222");
const rustContext = cdpBrowser.contexts()[0];
const rustPage = rustContext.pages()[0] || (await rustContext.newPage());

await reactPage.evaluate(() => localStorage.clear());
await reactPage.reload({ waitUntil: "networkidle" });

// Paste logs in React
let rTextarea = await reactPage.$("textarea#paste-logs, textarea[placeholder*='Paste']");
if (!rTextarea) {
  const pasteBtn = await reactPage.$("button:has-text('Paste logs')");
  if (pasteBtn) await pasteBtn.click();
  await reactPage.waitForTimeout(300);
  rTextarea = await reactPage.$("textarea#paste-logs, textarea[placeholder*='Paste']");
}
await rTextarea.focus();
await reactPage.keyboard.insertText(sampleLogText);
await reactPage.waitForTimeout(200);
await (await reactPage.$("button:has-text('Analyze paste')")).click();
await reactPage.waitForTimeout(1000);

// Paste logs in Rust
let uTextarea = await rustPage.$("textarea#paste-logs, textarea[placeholder*='Paste']");
if (!uTextarea) {
  const pasteBtn = await rustPage.$("button:has-text('Paste logs')");
  if (pasteBtn) await pasteBtn.click();
  await rustPage.waitForTimeout(300);
  uTextarea = await rustPage.$("textarea#paste-logs, textarea[placeholder*='Paste']");
}
await uTextarea.focus();
await rustPage.keyboard.insertText(sampleLogText);
await rustPage.waitForTimeout(200);
await (await rustPage.$("button:has-text('Analyze paste')")).click();
await rustPage.waitForTimeout(1000);

const getLayout = async (page) => {
  return page.evaluate(() => {
    const sel = ["header", "section[data-ui-id='ingest-panel']", "section[data-ui-id='kpi-row']", "section[data-ui-id='filter-bar']", "section[data-ui-id='api-table']", "section[data-ui-id='latency-chart']", "section[data-ui-id='cron-table']", "section[data-ui-id='skipped-disclosure']", "footer"];
    return sel.map(s => {
      const el = document.querySelector(s);
      if (!el) return { selector: s, missing: true };
      const b = el.getBoundingClientRect();
      return { selector: s, top: b.top, left: b.left, width: b.width, height: b.height };
    });
  });
};

const rLayout = await getLayout(reactPage);
const uLayout = await getLayout(rustPage);

console.log("=== COMPONENT LAYOUT GEOMETRY COMPARISON ===");
for (let i = 0; i < rLayout.length; i++) {
  const r = rLayout[i];
  const u = uLayout[i];
  console.log(`\n${r.selector}:`);
  console.log(`  React -> Top: ${r.top.toFixed(1)}, Height: ${r.height.toFixed(1)}, Width: ${r.width.toFixed(1)}`);
  if (u && !u.missing) {
    console.log(`  Rust  -> Top: ${u.top.toFixed(1)}, Height: ${u.height.toFixed(1)}, Width: ${u.width.toFixed(1)}`);
    console.log(`  DIFF  -> Top: ${(u.top - r.top).toFixed(1)}px, Height: ${(u.height - r.height).toFixed(1)}px`);
  } else {
    console.log(`  Rust  -> MISSING!`);
  }
}

await browser.close();
await cdpBrowser.close();
