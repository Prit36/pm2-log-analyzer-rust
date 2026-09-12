// Inspect computed font metrics up the tree for a selector.
import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });
await page.setInputFiles("input[type=file]", "C:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer-rust/smoke.log");
await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", { timeout: 120000 });
await page.waitForTimeout(500);
const info = await page.evaluate(() => {
  const pick = (el) => {
    const cs = getComputedStyle(el);
    return {
      tag: el.tagName,
      cls: (el.getAttribute("class") || "").slice(0, 90),
      fs: cs.fontSize,
      lh: cs.lineHeight,
      ff: cs.fontFamily.slice(0, 60),
      fw: cs.fontWeight,
      ls: cs.letterSpacing,
    };
  };
  const out = [];
  const badge = [...document.querySelectorAll("span")].find((s) => s.textContent === "POST" && s.className.includes("rounded"));
  out.push(["MethodBadge", pick(badge)]);
  let p = badge.parentElement;
  let depth = 0;
  while (p && depth < 4) { out.push([`parent${depth}`, pick(p)]); p = p.parentElement; depth++; }
  const kpiLabel = [...document.querySelectorAll("div")].find((d) => d.textContent === "Requests" && d.className.includes("uppercase"));
  out.push(["KpiLabel", pick(kpiLabel)]);
  const path = [...document.querySelectorAll("span")].find((s) => s.textContent?.startsWith("/api/admin/motor/quotes"));
  if (path) out.push(["PathSpan", pick(path)]);
  const h1 = document.querySelector("h1");
  out.push(["h1", pick(h1)]);
  const test = document.createElement("span");
  test.textContent = "x";
  test.style.cssText = "font-size:10px;font-family:var(--font-sans)";
  document.body.appendChild(test);
  out.push(["probe-sans-10", pick(test)]);
  test.style.fontFamily = "var(--font-mono)";
  out.push(["probe-mono-10", pick(test)]);
  test.style.fontFamily = "sans-serif";
  out.push(["probe-sansserif-10", pick(test)]);
  const probe2 = document.createElement("span");
  probe2.textContent = "x";
  probe2.style.cssText = "font-size:11px;font-family:var(--font-sans)";
  document.body.appendChild(probe2);
  out.push(["probe-sans-11", pick(probe2)]);
  return out;
});
for (const [name, d] of info) console.log(name.padEnd(18), JSON.stringify(d));
await browser.close();
