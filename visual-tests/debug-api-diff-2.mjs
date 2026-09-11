import fs from "fs";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

const refImg = PNG.sync.read(fs.readFileSync("visual-tests/references/populated_light.png"));
const candImg = PNG.sync.read(fs.readFileSync("visual-tests/candidates/populated_light.png"));

// Find bounding box of section:has(h2:has-text('Slow API endpoints'))
// In report.json:
const report = JSON.parse(fs.readFileSync("visual-tests/reports/report.json", "utf8"));
const popCase = report.results.find(c => c.id === "populated_light");
console.log("Populated Case Regions:", Object.keys(popCase.regions));
const apiReg = popCase.regions["api-table"];
console.log("API Table Region Data:", apiReg);
