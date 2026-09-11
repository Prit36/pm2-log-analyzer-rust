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

const reactRows = await reactPage.$$eval("section[data-ui-id='api-table'] div.grid", els => els.map(e => ({
  text: e.innerText.replace(/\n/g, " | "),
  box: e.getBoundingClientRect()
})));
const rustRows = await rustPage.$$eval("section[data-ui-id='api-table'] div.grid", els => els.map(e => ({
  text: e.innerText.replace(/\n/g, " | "),
  box: e.getBoundingClientRect()
})));

console.log("React Rows Count:", reactRows.length);
console.log("Rust Rows Count:", rustRows.length);

for (let i = 0; i < Math.min(10, reactRows.length, rustRows.length); i++) {
  console.log(`\nRow ${i}:`);
  console.log("  React:", reactRows[i].text);
  console.log("  Rust :", rustRows[i].text);
  console.log("  React Box:", reactRows[i].box.width, reactRows[i].box.height);
  console.log("  Rust Box :", rustRows[i].box.width, rustRows[i].box.height);
}

await browser.close();
await cdpBrowser.close();
