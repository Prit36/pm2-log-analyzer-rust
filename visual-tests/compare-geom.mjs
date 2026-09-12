// Compare native PNG geometry against reference DOM dump.
// Finds card/panel edges in the native image and prints DOM rects for the reference.
import fs from "node:fs";
import { PNG } from "pngjs";
const [, , domPath, natPath] = process.argv;
const dom = JSON.parse(fs.readFileSync(domPath, "utf8"));
const img = PNG.sync.read(fs.readFileSync(natPath));
const W = img.width, H = img.height; // 2x
const px = (x, y) => {
  const i = (y * W + x) * 4;
  return [img.data[i], img.data[i + 1], img.data[i + 2]];
};
const S = 2; // device scale

console.log("=== reference sections (CSS px) ===");
for (const e of dom.els) {
  if (["section", "header", "footer", "main", "table", "h2", "h1", "h3"].includes(e.tag) || e.testid) {
    console.log(
      `${e.tag}${e.testid ? "#" + e.testid : ""} [${e.class.split(" ").filter((c) => /^(bg-|rounded|p-|px-|py-|gap-|grid|flex|border|shadow|text-)/.test(c)).join(" ").slice(0, 80)}] x=${e.x} y=${e.y} w=${e.w} h=${e.h} fs=${e.fs} fw=${e.fw} lh=${e.lh} bg=${e.bg} bd=${e.border} r=${e.radius} pad=${e.pad} gap=${e.gap} shadow="${e.shadow}"`,
    );
  }
}

// Detect horizontal lines (rows whose pixel at some x range is uniform-ish darker than neighbors)
console.log("\n=== native horizontal edges (physical px, /2=CSS) ===");
const sampleX = 600; // physical x in the white card area
const rows = [];
for (let y = 0; y < H; y++) {
  rows.push(px(sampleX, y));
}
for (let y = 1; y < H; y++) {
  const a = rows[y - 1], b = rows[y];
  const d = Math.abs(a[0] - b[0]) + Math.abs(a[1] - b[1]) + Math.abs(a[2] - b[2]);
  if (d > 20) {
    console.log(`y=${y} (css ${(y / S).toFixed(1)})  ${a.join(",")} -> ${b.join(",")}`);
  }
}
