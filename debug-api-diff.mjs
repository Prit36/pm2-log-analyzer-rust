import fs from "fs";
import path from "path";

const reportPath = "./visual-tests/reports/report.json";
if (fs.existsSync(reportPath)) {
  const data = JSON.parse(fs.readFileSync(reportPath, "utf-8"));
  console.log("=== Report Overview ===");
  for (const res of data.results) {
    console.log(`\nTest Case: [${res.id}] ${res.name} (Overall: ${res.overallSimilarity}%)`);
    for (const [regId, reg] of Object.entries(res.regions)) {
      if (reg.missing) {
        console.log(`  - ${regId}: N/A (Missing)`);
      } else {
        console.log(`  - ${regId}: ${reg.similarity}% | React: ${Math.round(reg.reactBox.width)}x${Math.round(reg.reactBox.height)} @ (${Math.round(reg.reactBox.x)},${Math.round(reg.reactBox.y)}) vs Rust: ${Math.round(reg.rustBox.width)}x${Math.round(reg.rustBox.height)} @ (${Math.round(reg.rustBox.x)},${Math.round(reg.rustBox.y)})`);
      }
    }
  }
} else {
  console.log("report.json not found.");
}
