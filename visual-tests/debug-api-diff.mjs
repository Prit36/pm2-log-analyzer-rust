import fs from "fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

const refImg = PNG.sync.read(fs.readFileSync("visual-tests/references/populated_light.png"));
const candImg = PNG.sync.read(fs.readFileSync("visual-tests/candidates/populated_light.png"));
const report = JSON.parse(fs.readFileSync("visual-tests/reports/report.json", "utf8"));
const popCase = report.results.find(c => c.id === "populated_light");
const apiBox = popCase.regions["api-table"].reactBox;

const x = Math.floor(apiBox.x);
const y = Math.floor(apiBox.y);
const w = Math.floor(apiBox.width);
const h = Math.floor(apiBox.height);

console.log(`Cropping API Table: x=${x}, y=${y}, w=${w}, h=${h}`);

const cropRef = new PNG({ width: w, height: h });
const cropCand = new PNG({ width: w, height: h });

PNG.bitblt(refImg, cropRef, x, y, w, h, 0, 0);
PNG.bitblt(candImg, cropCand, x, y, w, h, 0, 0);

const diff = new PNG({ width: w, height: h });
const numDiff = pixelmatch(cropRef.data, cropCand.data, diff.data, w, h, { threshold: 0.1 });

console.log(`API Table Diff Pixels: ${numDiff} / ${w * h} (${((1 - numDiff / (w * h)) * 100).toFixed(2)}% similarity)`);

for (let colX = 0; colX < w; colX += 40) {
  let cDiff = 0;
  for (let dy = 64; dy < 96; dy++) {
    for (let dx = 0; dx < 40 && colX + dx < w; dx++) {
      const idx = (w * dy + (colX + dx)) << 2;
      if (diff.data[idx] > 200 && diff.data[idx + 1] < 100) {
        cDiff++;
      }
    }
  }
  if (cDiff > 0) {
    console.log(`  Row 0 X [${colX}..${colX + 40}]: ${cDiff} diff pixels`);
  }
}
