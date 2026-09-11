import fs from "fs";
import { PNG } from "pngjs";

const diffImg = PNG.sync.read(fs.readFileSync("visual-tests/diffs/populated_light.png"));
const report = JSON.parse(fs.readFileSync("visual-tests/reports/report.json", "utf8"));
const caseData = report.results.find(c => c.id === "populated_light");
const apiBox = caseData.regions["api-table"].reactBox;

console.log("API Table Box:", apiBox);

let minX = diffImg.width, maxX = 0, minY = diffImg.height, maxY = 0;
let redPixels = 0;

for (let y = Math.floor(apiBox.y); y < Math.floor(apiBox.y + apiBox.height); y++) {
  for (let x = Math.floor(apiBox.x); x < Math.floor(apiBox.x + apiBox.width); x++) {
    const idx = (diffImg.width * y + x) << 2;
    const r = diffImg.data[idx];
    const g = diffImg.data[idx + 1];
    const b = diffImg.data[idx + 2];
    
    if (r > 200 && g < 100) {
      redPixels++;
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }
  }
}

console.log(`API Table Red Mismatch Pixels: ${redPixels}`);
console.log(`Mismatch Bounding Box relative to API table top (${apiBox.y}): Y range [${minY - apiBox.y}, ${maxY - apiBox.y}] X range [${minX - apiBox.x}, ${maxX - apiBox.x}]`);
