// Per-band diff report: node region-diff.mjs <ref.png> <nat.png> [dy]
import fs from "node:fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
const [, , refPath, natPath, dyArg] = process.argv;
const dy = Number(dyArg ?? 1);
const ref = PNG.sync.read(fs.readFileSync(refPath));
const nat = PNG.sync.read(fs.readFileSync(natPath));
const W = ref.width, H = ref.height;
// shift native by dy (like align-diff)
function shifted() {
  const buf = Buffer.alloc(nat.data.length, 255);
  for (let y = 0; y < H; y++) {
    const sy = y + dy;
    if (sy < 0 || sy >= H) continue;
    nat.data.copy(buf, y * W * 4, sy * W * 4, (sy + 1) * W * 4);
  }
  return buf;
}
const natData = shifted();
const bands = [];
const bandH = 32; // physical rows
for (let y0 = 0; y0 < H; y0 += bandH) {
  const rows = Math.min(bandH, H - y0);
  const r = ref.data.subarray(y0 * W * 4, (y0 + rows) * W * 4);
  const n = natData.subarray(y0 * W * 4, (y0 + rows) * W * 4);
  const changed = pixelmatch(r, n, null, W, rows, { threshold: 0.1, includeAA: false });
  let nonIdentical = 0;
  for (let i = 0; i < r.length; i += 4) {
    if (r[i] !== n[i] || r[i + 1] !== n[i + 1] || r[i + 2] !== n[i + 2]) nonIdentical++;
  }
  const total = W * rows;
  bands.push({ y0, css: y0 / 2, changed, pct: (changed / total) * 100, exact: ((total - nonIdentical) / total) * 100 });
}
bands.sort((a, b) => b.pct - a.pct);
console.log("worst bands (physical y0, css y, pixelmatch no-AA %, exact %):");
for (const b of bands.slice(0, 25)) {
  console.log(`y0=${b.y0} css=${b.css} changed=${b.pct.toFixed(1)}% exact=${b.exact.toFixed(1)}%`);
}
