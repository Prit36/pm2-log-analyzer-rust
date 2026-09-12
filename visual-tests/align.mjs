import fs from "node:fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
const ref = PNG.sync.read(fs.readFileSync(process.argv[2]));
const nat = PNG.sync.read(fs.readFileSync(process.argv[3]));
const W = ref.width, H = ref.height;
const buf = Buffer.alloc(W * H * 4, 255);
let best = { pct: -1, dx: 0, dy: 0 };
for (let dy = -20; dy <= 20; dy += 1) {
  for (let dx = -20; dx <= 20; dx += 1) {
    for (let y = 0; y < H; y++) {
      const sy = y + dy;
      const di = y * W * 4;
      if (sy < 0 || sy >= H) continue;
      for (let x = 0; x < W; x++) {
        const sx = x + dx;
        if (sx < 0 || sx >= W) continue;
        const si = (sy * W + sx) * 4;
        const d = di + x * 4;
        buf[d] = nat.data[si]; buf[d+1] = nat.data[si+1]; buf[d+2] = nat.data[si+2]; buf[d+3] = 255;
      }
    }
    const changed = pixelmatch(ref.data, buf, null, W, H, { threshold: 0.1, includeAA: false });
    const pct = ((W * H - changed) / (W * H)) * 100;
    if (pct > best.pct) best = { pct, dx, dy };
  }
}
console.log(`aligned (dx=${best.dx}, dy=${best.dy}): ${best.pct.toFixed(2)}% pixelmatch similarity`);
