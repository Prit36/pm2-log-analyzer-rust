// Local alignment of two regions: node local-align.mjs <ref> <nat> <x> <y> <w> <h> [range]
import fs from "node:fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
const [, , refPath, natPath, xs, ys, ws, hs, rangeArg] = process.argv;
const x = Number(xs), y = Number(ys), w = Number(ws), h = Number(hs);
const range = Number(rangeArg ?? 12);
const ref = PNG.sync.read(fs.readFileSync(refPath));
const nat = PNG.sync.read(fs.readFileSync(natPath));
const crop = (img, x0, y0) => {
  const buf = Buffer.alloc(w * h * 4, 255);
  for (let yy = 0; yy < h; yy++) {
    for (let xx = 0; xx < w; xx++) {
      const sx = x0 + xx, sy = y0 + yy;
      if (sx < 0 || sy < 0 || sx >= img.width || sy >= img.height) continue;
      const si = (sy * img.width + sx) * 4;
      const di = (yy * w + xx) * 4;
      buf[di] = img.data[si];
      buf[di + 1] = img.data[si + 1];
      buf[di + 2] = img.data[si + 2];
      buf[di + 3] = 255;
    }
  }
  return buf;
};
const refCrop = crop(ref, x, y);
let best = { pct: -1, dx: 0, dy: 0 };
for (let dy = -range; dy <= range; dy++) {
  for (let dx = -range; dx <= range; dx++) {
    const natCrop = crop(nat, x + dx, y + dy);
    const changed = pixelmatch(refCrop, natCrop, null, w, h, { threshold: 0.1, includeAA: false });
    const pct = ((w * h - changed) / (w * h)) * 100;
    if (pct > best.pct) best = { pct, dx, dy };
  }
}
console.log(`region ${x},${y} ${w}x${h}: best dx=${best.dx} dy=${best.dy} -> ${best.pct.toFixed(2)}% (no-AA)`);
