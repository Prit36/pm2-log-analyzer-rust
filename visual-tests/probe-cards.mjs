// Reports horizontal card-border rows (slate-200 #e2e8f0, long runs) in a screenshot.
import fs from "node:fs";
import { PNG } from "pngjs";
const img = PNG.sync.read(fs.readFileSync(process.argv[2]));
const { width: W, height: H, data } = img;
const isBorder = (i) => Math.abs(data[i]-0xe2)<10 && Math.abs(data[i+1]-0xe8)<10 && Math.abs(data[i+2]-0xf0)<10;
const rows = [];
for (let y = 0; y < H; y += 1) {
  let best = 0, run = 0;
  for (let x = 0; x < W; x += 1) {
    const i = (y*W+x)*4;
    if (isBorder(i)) { run += 1; if (run > best) best = run; } else run = 0;
  }
  if (best > W * 0.5) rows.push({ y, run: best });
}
// Collapse adjacent rows (1px borders, 2px at DPR 2).
const groups = [];
for (const row of rows) {
  const last = groups[groups.length-1];
  if (last && row.y - last.y1 <= 2) { last.y1 = row.y; } else groups.push({ y0: row.y, y1: row.y });
}
console.log(groups.map((g) => `${(g.y0/2).toFixed(1)}${g.y1!==g.y0 ? "-"+(g.y1/2).toFixed(1) : ""}`).join("  "));
