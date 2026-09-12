// Dump reference chart geometry for a given log (smoke.log by default).
// node probe-chart.mjs <log> [--wide]
import { chromium } from "playwright";
const logFile = process.argv[2] ?? "C:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer-rust/smoke.log";
const wide = process.argv.includes("--wide");
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });
await page.evaluate(() => localStorage.clear());
await page.reload({ waitUntil: "networkidle" });
await page.setInputFiles("input[type=file]", logFile);
await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", { timeout: 120000 });
if (wide) {
  await page.click("button:has-text('Wide View')");
  await page.waitForTimeout(500);
}
await page.waitForTimeout(1200);
const geom = await page.evaluate(() => {
  const section = [...document.querySelectorAll("section")].find((s) => s.textContent?.includes("API Visual Analytics"));
  const svg = [...section.querySelectorAll("svg.recharts-surface")].sort((a, b) => b.getBoundingClientRect().width - a.getBoundingClientRect().width)[0];
  const r = (el) => {
    const b = el.getBoundingClientRect();
    return { x: +b.x.toFixed(2), y: +b.y.toFixed(2), w: +b.width.toFixed(2), h: +b.height.toFixed(2) };
  };
  const host = section.querySelector("div.px-3.py-3");
  const grid = [...svg.querySelectorAll(".recharts-cartesian-grid line")].map((l) => ({
    x1: l.getAttribute("x1"), y1: l.getAttribute("y1"), x2: l.getAttribute("x2"), y2: l.getAttribute("y2"),
  }));
  const axes = [...svg.querySelectorAll(".recharts-cartesian-axis-line")].map((l) => ({
    x1: l.getAttribute("x1"), y1: l.getAttribute("y1"), x2: l.getAttribute("x2"), y2: l.getAttribute("y2"),
  }));
  const tickLabels = [...svg.querySelectorAll(".recharts-cartesian-axis-tick-value")].map((t) => ({
    text: t.textContent,
    x: t.getAttribute("x"),
    y: t.getAttribute("y"),
    anchor: t.getAttribute("text-anchor"),
    transform: t.getAttribute("transform"),
  }));
  const legend = section.querySelector(".recharts-legend-wrapper");
  return {
    section: r(section),
    host: r(host),
    svg: r(svg),
    svgAttrs: { width: svg.getAttribute("width"), height: svg.getAttribute("height"), viewBox: svg.getAttribute("viewBox") },
    grid,
    axes,
    tickLabels: tickLabels.slice(0, 16),
    legend: legend ? r(legend) : null,
  };
});
console.log(JSON.stringify(geom, null, 1));
await browser.close();
