// Find rows/cols with many pixels near a target color in a window.
import fs from "node:fs";
import { PNG } from "pngjs";
const [, , f, xs, ys, ws, hs, cr, cg, cb, tolArg] = process.argv;
const img = PNG.sync.read(fs.readFileSync(f));
const x = Number(xs), y = Number(ys), w = Number(ws), h = Number(hs);
const tr = Number(cr), tg = Number(cg), tb = Number(cb);
const tol = Number(tolArg ?? 10);
console.log(`file=${f} window=${x},${y},${w},${h} target=${tr},${tg},${tb}`);
for (let yy = y; yy < Math.min(y + h, img.height); yy++) {
  let count = 0;
  for (let xx = x; xx < Math.min(x + w, img.width); xx++) {
    const i = (yy * img.width + xx) * 4;
    if (Math.abs(img.data[i] - tr) <= tol && Math.abs(img.data[i + 1] - tg) <= tol && Math.abs(img.data[i + 2] - tb) <= tol) count++;
  }
  if (count > w * 0.25) console.log(`  row y=${yy} css=${(yy / 2).toFixed(1)} count=${count}`);
}
