/**
 * Pixel-diff gate: the reference app (Chromium) vs the native iced preview
 * rendered by `cargo test --lib writes_ui_preview -- --ignored`.
 *
 * Usage: node pixel-diff.mjs <reference.png> <native.png> [--threshold 0.1] [--write-diff out.png]
 *
 * Both images must be the same logical viewport at the same device pixel ratio
 * (the capture scripts use 1024x768 @2x, i.e. 2048x1536).
 */
import fs from "node:fs";
import path from "node:path";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";

const [refPath, natPath] = process.argv.slice(2);
const threshold = Number(process.argv[process.argv.indexOf("--threshold") + 1]) || 0.1;
const diffIndex = process.argv.indexOf("--write-diff");
const diffPath = diffIndex > 0 ? process.argv[diffIndex + 1] : null;

const ref = PNG.sync.read(fs.readFileSync(path.resolve(refPath)));
const nat = PNG.sync.read(fs.readFileSync(path.resolve(natPath)));

if (ref.width !== nat.width || ref.height !== nat.height) {
  console.error(`size mismatch: reference ${ref.width}x${ref.height}, native ${nat.width}x${nat.height}`);
  process.exit(2);
}

// Four buckets, mirroring the retired harness: identical, anti-aliased only,
// slightly different, and hard-different pixels.
const identical = new PNG({ width: ref.width, height: ref.height });
const antiAliased = new PNG({ width: ref.width, height: ref.height });
const diff = new PNG({ width: ref.width, height: ref.height });

const changed = pixelmatch(ref.data, nat.data, diff.data, ref.width, ref.height, {
  threshold,
  includeAA: false,
  alpha: 0.3,
  diffColor: [255, 0, 0],
  aaColor: [255, 204, 0],
});
const changedWithAA = pixelmatch(ref.data, nat.data, null, ref.width, ref.height, {
  threshold,
  includeAA: true,
});
// Non-identical pixels, regardless of severity: the harness "similarity" metric.
let nonIdentical = 0;
let antiAliasedOnly = 0;
for (let i = 0; i < ref.data.length; i += 4) {
  const same =
    ref.data[i] === nat.data[i] &&
    ref.data[i + 1] === nat.data[i + 1] &&
    ref.data[i + 2] === nat.data[i + 2];
  if (!same) {
    nonIdentical += 1;
    // Yellow in the diff image means "anti-aliasing only" at this tolerance.
    if (diff.data[i] === 255 && diff.data[i + 1] === 204 && diff.data[i + 2] === 0) {
      antiAliasedOnly += 1;
    }
  }
}
void identical;
void antiAliased;

const total = ref.width * ref.height;
const similarity = ((total - nonIdentical) / total) * 100;

if (diffPath) {
  fs.mkdirSync(path.dirname(path.resolve(diffPath)), { recursive: true });
  fs.writeFileSync(path.resolve(diffPath), PNG.sync.write(diff));
}

console.log(
  [
    `similarity ${similarity.toFixed(3)}%`,
    `AA-only ${((antiAliasedOnly / total) * 100).toFixed(3)}%`,
    `hard-diff ${(((nonIdentical - antiAliasedOnly) / total) * 100).toFixed(3)}%`,
    `pixelmatch(no-AA)=${changed} pixelmatch(AA)=${changedWithAA}`,
  ].join("  "),
);

const required = Number(process.env.PARITY_MIN ?? 0);
if (similarity < required) {
  console.error(`FAIL: similarity ${similarity.toFixed(3)}% < ${required}%`);
  process.exit(1);
}
