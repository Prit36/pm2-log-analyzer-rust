import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });
await page.setInputFiles("input[type=file]", "C:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer-rust/smoke.log");
await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", { timeout: 120000 });
await page.waitForTimeout(500);
const client = await page.context().newCDPSession(page);
await client.send("DOM.enable");
await client.send("CSS.enable");
const { root } = await client.send("DOM.getDocument");
const { nodeId } = await client.send("DOM.querySelector", { nodeId: root.nodeId, selector: "span.inline-block" });
const { matchedCSSRules } = await client.send("CSS.getMatchedStylesForNode", { nodeId });
for (const m of matchedCSSRules) {
  const sel = m.rule.selectorList.text;
  const props = m.rule.style.cssProperties.filter((p) => p.name.includes("line") || p.name.includes("font") || p.name === "display").map((p) => `${p.name}:${p.value}`);
  if (props.length) console.log(sel, "=>", props.join("; "));
}
await browser.close();
