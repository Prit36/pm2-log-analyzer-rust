/**
 * Dumps the reference LatencyChart SVG markup for every mode/theme so the native
 * renderer can be validated against Recharts geometry.
 *
 * Usage (reference dist must be built):
 *   cd ../pm2-log-analyzer && npx vite preview --port 5173 &
 *   node parity/dump-chart.mjs http://localhost:5173 parity/out
 */
import fs from "node:fs";
import path from "node:path";
import { chromium } from "playwright";

const baseUrl = process.argv[2] ?? "http://localhost:5173";
const outDir = process.argv[3] ?? "parity/out";
fs.mkdirSync(outDir, { recursive: true });

/** Deterministic corpus: 2 days, varied endpoints/statuses, one slow outlier. */
function corpus() {
  const lines = [];
  const paths = [
    "/api/users/:id",
    "/api/orders",
    "/api/health",
    "/api/products?cat=electronics",
  ];
  const methods = ["GET", "POST", "PUT", "DELETE"];
  const statuses = [200, 201, 204, 304, 400, 404, 500, 502];
  for (let day = 24; day <= 25; day++) {
    for (let i = 0; i < 60; i++) {
      const hour = String(i % 24).padStart(2, "0");
      const method = methods[i % methods.length];
      const p = paths[i % paths.length];
      const status = statuses[i % statuses.length];
      const ms = status >= 500 ? 1500 + i * 37 : status >= 400 ? 40 + i * 3 : 8 + (i % 11) * 6.5;
      lines.push(
        `2026-07-${day}T${hour}:00:${String(i % 60).padStart(2, "0")}: ${method} ${p} ${status} ${ms.toFixed(3)} ms - 128`,
      );
    }
  }
  lines.push("2026-07-24T15:00:00: [cron] daily-report STARTED");
  lines.push("2026-07-24T15:00:31: [cron] daily-report COMPLETED in 31000ms");
  lines.push("2026-07-24T16:00:00: [cron] backup-db FAILED after 2000ms - timeout");
  lines.push("noise line that does not parse");
  return lines.join("\n");
}

const MODES = [
  { key: "timeOfDay", title: "Time of Day vs Latency Trend" },
  { key: "throughput", title: "Hourly Request Volume & Error Rate" },
  { key: "distribution", title: "Latency Distribution Buckets" },
  { key: "topP95", title: "Top p95 Slowest Endpoints" },
  { key: "dailyTrend", title: "Daily Trend (Requests, Latency, Errors across all days)" },
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto(baseUrl, { waitUntil: "networkidle" });
await page.evaluate(() => localStorage.clear());
await page.reload({ waitUntil: "networkidle" });

for (const theme of ["light", "dark"]) {
  for (const layout of ["split", "wide"]) {
    await page.evaluate(() => localStorage.clear());
    await page.reload({ waitUntil: "networkidle" });
    await page.evaluate((t) => {
      document.documentElement.classList.toggle("dark", t === "dark");
    }, theme);

    // Paste corpus.
    const pasteBtn = await page.$("button:has-text('Paste logs')");
    if (pasteBtn) await pasteBtn.click();
    await page.waitForTimeout(200);
    const textarea = await page.$("textarea#paste-logs");
    await textarea.focus();
    await page.keyboard.insertText(corpus());
    await page.waitForTimeout(150);
    await page.click("button:has-text('Analyze paste')");
    await page.waitForSelector("section[data-testid='kpi-row']", { timeout: 15000 });
    await page.waitForTimeout(800);

    if (layout === "wide") {
      await page.click("button:has-text('Wide View')");
      await page.waitForTimeout(400);
    }

    for (const mode of MODES) {
      const button = await page.$(`button[title='${mode.title}']`);
      if (!button) {
        console.warn(`mode button missing: ${mode.title}`);
        continue;
      }
      await button.click();
      await page.waitForTimeout(2200);
      const dumped = await page.evaluate(() => {
        const section = [...document.querySelectorAll("section")].find((s) =>
          s.textContent?.includes("API Visual Analytics"),
        );
        if (!section) return null;
        const plotHost = section.querySelector("div.px-3.py-3");
        const svg = [...section.querySelectorAll("svg.recharts-surface")].sort(
          (a, b) => b.getBoundingClientRect().width - a.getBoundingClientRect().width,
        )[0];
        const wrapper = section.querySelector(".recharts-wrapper");
        return {
          sectionHtml: section.outerHTML.length,
          host: plotHost
            ? { w: plotHost.clientWidth, h: plotHost.clientHeight }
            : null,
          wrapper: wrapper
            ? { w: wrapper.getBoundingClientRect().width, h: wrapper.getBoundingClientRect().height }
            : null,
          svg: svg ? svg.outerHTML : null,
          hostHtml: plotHost ? plotHost.outerHTML : null,
        };
      });
      const file = path.join(outDir, `chart_${theme}_${layout}_${mode.key}.json`);
      fs.writeFileSync(file, JSON.stringify(dumped, null, 2));
      console.log(`wrote ${file} host=${JSON.stringify(dumped?.host)} wrapper=${JSON.stringify(dumped?.wrapper)}`);
    }
  }
}

await browser.close();
