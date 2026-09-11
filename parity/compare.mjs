/**
 * Deep-compare two JSON dumps (native kernel vs reference coordinator).
 * Prints the first mismatch path and exits non-zero when they differ.
 *
 * Usage: node compare.mjs <native.json> <reference.json>
 */
import fs from "node:fs";

const [nativePath, refPath] = process.argv.slice(2);
if (!nativePath || !refPath) {
  console.error("usage: compare.mjs <native.json> <reference.json>");
  process.exit(2);
}

const a = JSON.parse(fs.readFileSync(nativePath, "utf8"));
const b = JSON.parse(fs.readFileSync(refPath, "utf8"));

let failures = 0;
const diffs = [];

function cmp(x, y, p) {
  if (Object.is(x, y)) return;
  if (typeof x === "number" && typeof y === "number") {
    if (Number.isNaN(x) && Number.isNaN(y)) return;
    diffs.push(`${p}: native=${x} ref=${y}`);
    failures++;
    return;
  }
  if (x === null || y === null || typeof x !== "object" || typeof y !== "object") {
    diffs.push(`${p}: native=${JSON.stringify(x)} ref=${JSON.stringify(y)}`);
    failures++;
    return;
  }
  if (Array.isArray(x) !== Array.isArray(y)) {
    diffs.push(`${p}: array/non-array mismatch`);
    failures++;
    return;
  }
  if (Array.isArray(x)) {
    if (x.length !== y.length) {
      diffs.push(`${p}.length: native=${x.length} ref=${y.length}`);
      failures++;
    }
    const n = Math.min(x.length, y.length);
    for (let i = 0; i < n; i++) cmp(x[i], y[i], `${p}[${i}]`);
    return;
  }
  const keys = new Set([...Object.keys(x), ...Object.keys(y)]);
  for (const k of keys) {
    if (!(k in x) || !(k in y)) {
      diffs.push(`${p}.${k}: key missing on one side`);
      failures++;
      continue;
    }
    cmp(x[k], y[k], `${p}.${k}`);
  }
}

cmp(a, b, "$");

if (failures === 0) {
  console.log("PARITY OK: native result deep-equals reference coordinator result");
} else {
  console.error(`PARITY FAILED: ${failures} difference(s)`);
  for (const d of diffs.slice(0, 40)) console.error("  " + d);
  process.exitCode = 1;
}
