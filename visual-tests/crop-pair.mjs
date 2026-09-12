// Side-by-side crop: node crop-pair.mjs <ref.png> <nat.png> <out.png> <x> <y> <w> <h> [scale] [--stack]
import fs from "node:fs";
import { PNG } from "pngjs";

const [refPath, natPath, outPath, xs, ys, ws, hs, ...rest] = process.argv.slice(2);
const scale = Number(rest.find((a) => /^\d+$/.test(a)) ?? 1);
const stack = rest.includes("--stack");
const x = Number(xs), y = Number(ys), w = Number(ws), h = Number(hs);

const ref = PNG.sync.read(fs.readFileSync(refPath));
const nat = PNG.sync.read(fs.readFileSync(natPath));
if (ref.width !== nat.width || ref.height !== nat.height) throw new Error("size mismatch");

const outW = stack ? w * scale : w * 2 * scale + 4;
const outH = stack ? h * 2 * scale + 4 : h * scale;
const out = new PNG({ width: outW, height: outH });
out.data.fill(255);

function blit(src, sx, sy, dx, dy, sw, sh) {
  for (let yy = 0; yy < sh * scale; yy++) {
    for (let xx = 0; xx < sw * scale; xx++) {
      const srcX = sx + Math.floor(xx / scale);
      const srcY = sy + Math.floor(yy / scale);
      if (srcX < 0 || srcY < 0 || srcX >= src.width || srcY >= src.height) continue;
      const si = (srcY * src.width + srcX) * 4;
      const di = ((dy + yy) * outW + dx + xx) * 4;
      out.data[di] = src.data[si];
      out.data[di + 1] = src.data[si + 1];
      out.data[di + 2] = src.data[si + 2];
      out.data[di + 3] = 255;
    }
  }
}

if (stack) {
  blit(ref, x, y, 0, 0, w, h);
  blit(nat, x, y, 0, h * scale + 4, w, h);
} else {
  blit(ref, x, y, 0, 0, w, h);
  blit(nat, x, y, w * scale + 4, 0, w, h);
}
fs.writeFileSync(outPath, PNG.sync.write(out));
console.log(`wrote ${outPath} (${outW}x${outH})`);
