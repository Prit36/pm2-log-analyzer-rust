import fs from "fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

const refImg = PNG.sync.read(fs.readFileSync("visual-tests/references/populated_light.png"));
const candImg = PNG.sync.read(fs.readFileSync("visual-tests/candidates/populated_light.png"));
const report = JSON.parse(fs.readFileSync("visual-tests/reports/report.json", "utf8"));
const popCase = report.results.find(c => c.id === "populated_light");
const chartBox = popCase.regions["latency-chart"].reactBox;

const x = Math.floor(chartBox.x);
const y = Math.floor(chartBox.y);
const w = Math.floor(chartBox.width);
const h = Math.floor(chartBox.height);

console.log(`Cropping Chart: x=${x}, y=${y}, w=${w}, h=${h}`);

const cropRef = new PNG({ width: w, height: h });
const cropCand = new PNG({ width: w, height: h });

PNG.bitblt(refImg, cropRef, x, y, w, h, 0, 0);
PNG.bitblt(candImg, cropCand, x, y, w, h, 0, 0);

const diff = new PNG({ width: w, height: h });
const numDiff = pixelmatch(cropRef.data, cropCand.data, diff.data, w, h, { threshold: 0.1 });

console.log(`Chart Diff Pixels: ${numDiff} / ${w * h} (${((1 - numDiff / (w * h)) * 100).toFixed(2)}% similarity)`);

for (let y = 0; y < h; y += 20) {
  let rowDiffs = 0;
  for (let dy = 0; dy < 20 && y + dy < h; dy++) {
    for (let x = 0; x < w; x++) {
      const idx = (w * (y + dy) + x) << 2;
      if (diff.data[idx] > 200 && diff.data[idx + 1] < 100) {
        rowDiffs++;
      }
    }
  }
  if (rowDiffs > 0) {
    console.log(`  Chart Y [${y}..${y + 20}]: ${rowDiffs} diff pixels`);
  }
}
