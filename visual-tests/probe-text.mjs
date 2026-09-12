// Finds dark-text rows inside an x band: node probe-text.mjs <png> <x0css> <x1css> <y0css> <y1css>
import fs from "node:fs";
import { PNG } from "pngjs";
const img = PNG.sync.read(fs.readFileSync(process.argv[2]));
const [x0, x1, y0, y1] = process.argv.slice(3).map(Number);
const { width: W, data } = img;
const rows = [];
for (let y = y0 * 2; y < y1 * 2; y += 1) {
  let dark = 0;
  for (let x = x0 * 2; x < x1 * 2; x += 1) {
    const i = (y * W + x) * 4;
    if (data[i] < 190 && data[i+1] < 190 && data[i+2] < 210) dark += 1;
  }
  if (dark > 2) rows.push(y / 2);
}
if (rows.length === 0) { console.log("no text"); process.exit(0); }
const groups = [];
for (const y of rows) {
  const last = groups[groups.length - 1];
  if (last && y - last.y1 <= 2) last.y1 = y; else groups.push({ y0: y, y1: y });
}
console.log(groups.map((g) => `${g.y0}-${g.y1}`).join("  "));
