import fs from "node:fs";
import { PNG } from "pngjs";

const files = process.argv.slice(2);
for (const f of files) {
  const img = PNG.sync.read(fs.readFileSync(f));
  const { width: W, height: H, data } = img;
  const rows = [];
  for (let y = 0; y < H; y++) {
    let ink = 0;
    let sum = 0;
    for (let x = 0; x < W; x += 2) {
      const i = (y * W + x) * 4;
      const l = (data[i] * 299 + data[i + 1] * 587 + data[i + 2] * 114) / 1000;
      sum += l;
      if (l < 170) ink++;
    }
    rows.push({ y, avg: sum / (W / 2), ink });
  }
  const edges = [];
  for (let y = 1; y < H; y++) {
    const a = rows[y - 1].avg, b = rows[y].avg;
    if ((a - b) > 8 || (b - a) > 8) edges.push(`${y}:${a.toFixed(0)}->${b.toFixed(0)} ink=${rows[y].ink}`);
  }
  console.log(`\n=== ${f} (${W}x${H}) edges: ${edges.length}`);
  console.log(edges.slice(0, 80).join("\n"));
}
