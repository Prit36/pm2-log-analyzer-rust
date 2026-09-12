import fs from "node:fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
const [refPath, natPath, outPath, dxs, dys] = process.argv.slice(2);
const ref = PNG.sync.read(fs.readFileSync(refPath));
const nat = PNG.sync.read(fs.readFileSync(natPath));
const W = ref.width, H = ref.height;
const dx = Number(dxs), dy = Number(dys);
const shifted = new PNG({ width: W, height: H });
shifted.data.fill(255);
for (let y = 0; y < H; y++) {
  const sy = y + dy; if (sy < 0 || sy >= H) continue;
  for (let x = 0; x < W; x++) {
    const sx = x + dx; if (sx < 0 || sx >= W) continue;
    const si = (sy * W + sx) * 4, di = (y * W + x) * 4;
    shifted.data[di] = nat.data[si]; shifted.data[di+1] = nat.data[si+1];
    shifted.data[di+2] = nat.data[si+2]; shifted.data[di+3] = 255;
  }
}
const out = new PNG({ width: W, height: H });
pixelmatch(ref.data, shifted.data, out.data, W, H, { threshold: 0.1, includeAA: false });
fs.writeFileSync(outPath, PNG.sync.write(out));
console.log(`diff written: ${outPath}`);
