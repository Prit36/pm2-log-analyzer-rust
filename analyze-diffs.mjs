import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pngjsPath = path.join(__dirname, "visual-tests", "node_modules", "pngjs", "lib", "png.js");
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const { PNG } = require(pngjsPath);

function analyzeDiff(refPath, candPath, name) {
  if (!fs.existsSync(refPath) || !fs.existsSync(candPath)) return;
  const ref = PNG.sync.read(fs.readFileSync(refPath));
  const cand = PNG.sync.read(fs.readFileSync(candPath));
  const w = Math.min(ref.width, cand.width);
  const h = Math.min(ref.height, cand.height);
  
  let diffCount = 0;
  let minX = w, maxX = 0, minY = h, maxY = 0;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const idx = (w * y + x) << 2;
      const dr = Math.abs(ref.data[idx] - cand.data[idx]);
      const dg = Math.abs(ref.data[idx+1] - cand.data[idx+1]);
      const db = Math.abs(ref.data[idx+2] - cand.data[idx+2]);
      if (dr > 20 || dg > 20 || db > 20) {
        diffCount++;
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  }
  console.log(`[${name}] ${diffCount} diff pixels in range X:[${minX}..${maxX}], Y:[${minY}..${maxY}]`);
}

analyzeDiff("./visual-tests/references/populated_light.png", "./visual-tests/candidates/populated_light.png", "populated_light");
analyzeDiff("./visual-tests/references/populated_dark.png", "./visual-tests/candidates/populated_dark.png", "populated_dark");
