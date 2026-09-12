// Print a compact DOM tree with rects. Usage: node dom-tree.mjs <dom.json> [--from-substr S] [--depth N]
import fs from "node:fs";
const [, , domPath, ...rest] = process.argv;
const fromIdx = rest.indexOf("--from-substr");
const from = fromIdx >= 0 ? rest[fromIdx + 1] : null;
const depthIdx = rest.indexOf("--depth");
const maxDepth = depthIdx >= 0 ? Number(rest[depthIdx + 1]) : 99;
const dom = JSON.parse(fs.readFileSync(domPath, "utf8"));
const els = dom.els;
let started = !from;
for (const e of els) {
  if (!started) {
    if ((e.text + " " + e.class + " " + e.testid).includes(from)) started = true;
    else continue;
  }
  if (from && e.depth > maxDepth) continue;
  const pad = "  ".repeat(Math.min(e.depth, 12));
  const cls = e.class.split(" ").filter((c) => /^(bg-|rounded|p-|px-|py-|pt-|pb-|mt-|mb-|gap-|grid|flex|border|shadow|text-|h-|min-h|w-|max-w|size-|leading|tracking|font-|space-|items-|justify-)/.test(c)).join(" ");
  console.log(
    `${pad}${e.tag}${e.testid ? "#" + e.testid : ""} (${e.x},${e.y} ${e.w}x${e.h}) fs=${e.fs} fw=${e.fw} lh=${e.lh} ls=${e.ls !== "normal" ? e.ls : ""} pad=${e.pad} gap=${e.gap} bg=${e.bg} r=${e.radius} ${e.shadow ? "shadow " : ""}${cls}${e.text ? ` "${e.text}"` : ""}`,
  );
  if (from && e.depth > maxDepth) started = false;
}
