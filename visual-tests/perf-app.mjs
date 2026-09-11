// Measure the native app's end-to-end load time via CDP (app must be started with
// PM2_ANALYZER_AUTOLOAD=<log file> and --remote-debugging-port=9223).
// Usage: node perf-app.mjs [--timeout-ms N]
import { chromium } from "playwright";

const port = process.env.CDP_PORT ?? "9223";
const timeoutMs = Number(process.argv.includes("--timeout-ms")
  ? process.argv[process.argv.indexOf("--timeout-ms") + 1]
  : 600000);

const t0 = Date.now();
let browser;
let page;
for (let attempt = 0; attempt < 60; attempt++) {
  try {
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
    const ctx = browser.contexts()[0];
    page = ctx.pages()[0] || (await ctx.newPage());
    await page.waitForLoadState("domcontentloaded");
    break;
  } catch {
    await new Promise((r) => setTimeout(r, 1000));
  }
}
if (!page) {
  console.error("webview CDP endpoint never became available");
  process.exit(1);
}

// Wait for the parse toast ("Parsed 20,315,200 requests in 4210ms").
await page.waitForFunction(
  () => {
    const t = document.body.innerText;
    return /Parsed\s[\d,]+\srequests/.test(t);
  },
  null,
  { timeout: timeoutMs },
);
console.log(`toast visible after ${Date.now() - t0} ms (script start)`);
const toast = await page.evaluate(() => {
  const el = document.querySelector("[class*='fixed']");
  return el ? el.innerText.trim() : document.body.innerText.match(/Parsed[^\n]*/)?.[0];
});
console.log("toast:", JSON.stringify(toast));
const kpi = await page.evaluate(() => {
  const rows = document.querySelectorAll("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']");
  return rows.length ? rows[0].innerText.replace(/\n+/g, " | ").slice(0, 200) : null;
});
console.log("kpi:", JSON.stringify(kpi));
await browser.close();
