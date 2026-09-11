import { chromium } from "playwright";
import fs from "fs";

const sampleLogText = fs.readFileSync("smoke.log", "utf-8");

const cdpBrowser = await chromium.connectOverCDP("http://localhost:9222");
const rustContext = cdpBrowser.contexts()[0];
const rustPage = rustContext.pages()[0] || (await rustContext.newPage());

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

const tallElements = await rustPage.evaluate(() => {
  const all = Array.from(document.querySelectorAll("*"));
  return all
    .map((el) => {
      const b = el.getBoundingClientRect();
      return {
        tag: el.tagName,
        id: el.id || el.getAttribute("data-ui-id") || el.className.slice(0, 30),
        scrollHeight: el.scrollHeight,
        clientHeight: el.clientHeight,
        rectHeight: b.height,
        rectBottom: b.bottom,
      };
    })
    .filter((e) => e.scrollHeight > 500 || e.rectBottom > 1550);
});

console.log("=== TALL ELEMENTS IN RUST APP ===");
console.table(tallElements);

await cdpBrowser.close();
