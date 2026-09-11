/**
 * Prints Recharts geometry landmarks from dump-chart outputs so the native
 * renderer can match tick math and plot rects.
 *
 * Usage: node summary-chart.mjs ../parity/out [pattern]
 */
import fs from "node:fs";
import path from "node:path";

const dir = process.argv[2];
const pattern = process.argv[3] ?? "";
for (const file of fs.readdirSync(dir).filter((f) => f.includes(pattern) && f.endsWith(".json")).sort()) {
  const d = JSON.parse(fs.readFileSync(path.join(dir, file), "utf8"));
  if (!d.svg) continue;
  const svg = d.svg;
  console.log(`\n=== ${file} svg=${/width="([\d.]+)"/.exec(svg)?.[1]}x${/height="([\d.]+)"/.exec(svg)?.[1]} wrapper=${JSON.stringify(d.wrapper)} ===`);

  const grid = svg.match(/<g class="recharts-cartesian-grid-horizontal">(.*?)<\/g>/s)?.[1] ?? "";
  const gridLines = [...grid.matchAll(/<line[^>]*x1="([\d.]+)"[^>]*y1="([\d.]+)"[^>]*x2="([\d.]+)"[^>]*y2="([\d.]+)"/g)]
    .map((m) => `x1=${(+m[1]).toFixed(1)} y1=${(+m[2]).toFixed(1)} x2=${(+m[3]).toFixed(1)} y2=${(+m[4]).toFixed(1)}`);
  console.log("grid:", gridLines.join(" | ") || "none");

  const vgrid = svg.match(/<g class="recharts-cartesian-grid-vertical">(.*?)<\/g>/s)?.[1] ?? "";
  const vLines = [...vgrid.matchAll(/<line[^>]*x1="([\d.]+)"[^>]*y1="([\d.]+)"[^>]*x2="([\d.]+)"[^>]*y2="([\d.]+)"/g)]
    .map((m) => `x1=${(+m[1]).toFixed(1)} y1=${(+m[2]).toFixed(1)} x2=${(+m[3]).toFixed(1)} y2=${(+m[4]).toFixed(1)}`);
  console.log("vgrid:", vLines.join(" | ") || "none");

  const axisLines = [...svg.matchAll(/<line[^>]*class="recharts-cartesian-axis-line"[^>]*>/g)].map((m) => {
    const a = m[0];
    const g = (n) => /(\w+)="([\d.]+)"/.exec(a.replace(new RegExp(`.*${n}=`), `${n}=`))?.[2];
    return `x1=${a.match(/x1="([\d.]+)"/)?.[1]} y1=${a.match(/y1="([\d.]+)"/)?.[1]} x2=${a.match(/x2="([\d.]+)"/)?.[1]} y2=${a.match(/y2="([\d.]+)"/)?.[1]} ${g("height") ? "h=" + g("height") : ""}`;
  });
  console.log("axis-lines:", axisLines.join(" | ") || "none");

  const ticks = [...svg.matchAll(/<g class="recharts-layer recharts-cartesian-axis-tick">(.*?)<\/g>\s*(?=<g class="recharts-layer recharts-cartesian-axis-tick">|<\/g>|$)/gs)];
  const tickTexts = [...svg.matchAll(/<text[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*class="recharts-text recharts-cartesian-axis-tick-value"[^>]*>(?:<tspan[^>]*>)?([^<]*)/g)]
    .map((m) => `(${(+m[1]).toFixed(1)},${(+m[2]).toFixed(1)})"${m[3]}"`);
  console.log("ticks:", tickTexts.join(" | ") || "none");

  const legend = svg.match(/<div class="recharts-legend-wrapper"[^>]*>.*?<\/div>/s)?.[0];
  console.log("legend:", legend ? legend.replace(/></g, ">\n<").slice(0, 800) : "none");

  const bars = [...svg.matchAll(/<path[^>]*class="recharts-rectangle[^"]*"[^>]*d="M([\d.]+),([\d.]+)h([\d.]+)v([\-\d.]+)h-([\d.]+)Z"/g)]
    .slice(0, 8)
    .map((m) => `x=${(+m[1]).toFixed(1)} y=${(+m[2]).toFixed(1)} w=${(+m[3]).toFixed(1)} h=${(+m[4]).toFixed(1)}`);
  console.log("bars:", bars.join(" | ") || "none");

  const curve = svg.match(/class="recharts-curve recharts-line-curve"[^>]*d="([^"]+)"/)?.[1]
    ?? svg.match(/class="recharts-curve recharts-area-curve"[^>]*d="([^"]+)"/)?.[1];
  console.log("curve:", curve ? curve.slice(0, 220) : "none");

  const area = svg.match(/class="recharts-curve recharts-area-area"[^>]*d="([^"]+)"/)?.[1];
  console.log("area:", area ? area.slice(0, 160) : "none");
}
