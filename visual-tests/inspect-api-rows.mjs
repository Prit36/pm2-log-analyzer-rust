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

const reactRows = await reactPage.$$eval("section:has(h2:has-text('Slow API endpoints')) div.grid", els => els.map(e => e.innerText.replace(/\n/g, " | ")));
const rustRows = await rustPage.$$eval("section:has(h2:has-text('Slow API endpoints')) div.grid", els => els.map(e => e.innerText.replace(/\n/g, " | ")));

console.log("React API Rows Count:", reactRows.length);
console.log("Rust API Rows Count:", rustRows.length);
console.log("\nFirst 5 React API Rows:");
console.log(reactRows.slice(0, 5));
console.log("\nFirst 5 Rust API Rows:");
console.log(rustRows.slice(0, 5));

await browser.close();
await cdpBrowser.close();
