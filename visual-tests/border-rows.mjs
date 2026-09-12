// Detect horizontal border lines in a 2x PNG and print CSS-px positions.
// A border row has a long run of pixels close to slate-200 (#e2e8f0) or is a
// near-uniform darker line. Usage: node border-rows.mjs <img.png> [minRun]
import fs from "node:fs";
import { PNG } from "pngjs";
const [, , f, minRunArg] = process.argv;
const minRun = Number(minRunArg ?? 400);
const img = PNG.sync.read(fs.readFileSync(f));
const { width: W, height: H, data } = img;

const close = (r, g, b, tr, tg, tb, tol) =>
  Math.abs(r - tr) <= tol && Math.abs(g - tg) <= tol && Math.abs(b - tb) <= tol;

const targets = [
  { name: "border-slate-200", c: [226, 232, 240] },
  { name: "canvas", c: [247, 248, 250] },
  { name: "white", c: [255, 255, 255] },
];

let prev = null;
for (let y = 0; y < H; y++) {
  let best = null;
  for (const t of targets) {
    let run = 0;
    for (let x = 0; x < W; x++) {
      const i = (y * W + x) * 4;
      if (close(data[i], data[i + 1], data[i + 2], ...t.c, 4)) {
        run++;
        if (run >= minRun) break;
      } else run = 0;
    }
    if (run >= minRun) {
      best = t.name;
      break;
    }
  }
  if (best !== prev) {
    console.log(`y=${y} (css ${(y / 2).toFixed(1)}): ${best ?? "other"}`);
    prev = best;
  }
}
