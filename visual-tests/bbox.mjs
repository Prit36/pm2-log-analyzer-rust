// Ink bounding box in a region: node bbox.mjs <img.png> <x> <y> <w> <h> [threshold]
import fs from "node:fs";
import { PNG } from "pngjs";
const [, , f, xs, ys, ws, hs, ts] = process.argv;
const img = PNG.sync.read(fs.readFileSync(f));
const x = Number(xs), y = Number(ys), w = Number(ws), h = Number(hs);
const t = Number(ts ?? 180);
let minX = 1e9, minY = 1e9, maxX = -1, maxY = -1, count = 0;
for (let yy = y; yy < Math.min(y + h, img.height); yy++) {
  for (let xx = x; xx < Math.min(x + w, img.width); xx++) {
    const i = (yy * img.width + xx) * 4;
    const l = (img.data[i] * 299 + img.data[i + 1] * 587 + img.data[i + 2] * 114) / 1000;
    if (l < t) {
      count++;
      if (xx < minX) minX = xx;
      if (yy < minY) minY = yy;
      if (xx > maxX) maxX = xx;
      if (yy > maxY) maxY = yy;
    }
  }
}
if (maxX < 0) console.log(`${f} region(${x},${y},${w},${h}): no ink`);
else
  console.log(
    `${f} region(${x},${y},${w},${h}): bbox=(${minX},${minY})-(${maxX},${maxY}) size=${maxX - minX + 1}x${maxY - minY + 1} ink=${count}`,
  );
