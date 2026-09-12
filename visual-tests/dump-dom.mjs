// Dump reference-app DOM geometry + computed styles in CSS px.
// Usage: node dump-dom.mjs <log-file> <out.json> [--mongo]
import { chromium } from "playwright";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RUST_DIR = path.resolve(__dirname, "..");
const logFile = path.resolve(process.argv[2] ?? path.join(RUST_DIR, "smoke.log"));
const outFile = path.resolve(process.argv[3] ?? path.join(RUST_DIR, "target/dom.json"));
const mongo = process.argv.includes("--mongo");
const viewIndex = process.argv.indexOf("--view");
const view = viewIndex > 0 ? process.argv[viewIndex + 1] : null;

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 }, deviceScaleFactor: 1 });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });

if (mongo) {
  await page.click("button:has-text('MongoDB Logs')");
  await page.setInputFiles("input[type=file]", logFile);
  await page.waitForSelector("text=Slow Queries", { timeout: 300000 });
  if (view) {
    await page.click(`button:has-text('${view}')`);
    await page.waitForTimeout(600);
  }
} else {
  await page.setInputFiles("input[type=file]", logFile);
  await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", {
    timeout: 120000,
  });
}
await page.waitForTimeout(800);

const dump = await page.evaluate(() => {
  const els = [];
  const walk = (el, depth) => {
    if (!(el instanceof Element)) return;
    const tag = el.tagName.toLowerCase();
    if (tag === "script" || tag === "style" || tag === "link" || tag === "svg") return;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    const text = (el.childNodes.length && el.firstChild?.nodeType === 3 ? el.firstChild.textContent : "").trim();
    els.push({
      depth,
      tag,
      class: (el.getAttribute("class") || "").slice(0, 200),
      testid: el.getAttribute("data-testid") || el.getAttribute("data-ui-id") || "",
      text: text.slice(0, 60),
      x: +r.x.toFixed(2), y: +r.y.toFixed(2), w: +r.width.toFixed(2), h: +r.height.toFixed(2),
      fs: cs.fontSize, fw: cs.fontWeight, lh: cs.lineHeight, ls: cs.letterSpacing,
      pad: cs.padding, mar: cs.margin, gap: cs.gap,
      color: cs.color, bg: cs.backgroundColor,
      radius: cs.borderRadius, border: cs.borderWidth + " " + cs.borderColor,
      shadow: cs.boxShadow === "none" ? "" : cs.boxShadow.slice(0, 100),
      display: cs.display,
    });
    for (const child of el.children) walk(child, depth + 1);
  };
  walk(document.body, 0);
  return { scrollHeight: document.documentElement.scrollHeight, els };
});

fs.writeFileSync(outFile, JSON.stringify(dump, null, 1));
console.log(`dom dump: ${outFile} (${dump.els.length} elements, scrollHeight=${dump.scrollHeight})`);
await browser.close();
