// Structural comparison of two PM2 Excel exports (reference vs native) using exceljs.
// Usage: node export-compare.mjs <ref.xlsx> <native.xlsx>
import ExcelJS from "../../pm2-log-analyzer/node_modules/.pnpm/exceljs@4.4.0/node_modules/exceljs/excel.js";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const refPath = path.resolve(process.argv[2]);
const natPath = path.resolve(process.argv[3]);

const normMeta = (s) =>
  String(s ?? "")
    .split("  |  ")
    .filter((p) => !p.startsWith("Generated:") && !p.startsWith("Source:"))
    .join("  |  ");

async function dump(file) {
  const wb = new ExcelJS.Workbook();
  await wb.xlsx.readFile(file);
  const sheets = [];
  wb.eachSheet((ws) => {
    const cells = [];
    ws.eachRow({ includeEmpty: false }, (row, rowNumber) => {
      row.eachCell({ includeEmpty: false }, (cell, colNumber) => {
        const v = cell.value;
        const value =
          v && typeof v === "object" && "result" in v ? v.result : v;
        if (value === null || value === undefined || value === "") return;
        const isMeta = rowNumber <= 2;
        cells.push({
          r: rowNumber,
          c: colNumber,
          v: isMeta ? normMeta(value) : value,
          numFmt: cell.numFmt ?? null,
          fill: cell.fill?.fgColor?.argb ?? null,
          bold: !!cell.font?.bold,
          color: cell.font?.color?.argb ?? null,
        });
      });
    });
    const tables = Object.values(ws.tables ?? {}).map((t) => ({
      name: t.name,
      ref: t.ref,
      style: t.style?.theme ?? t.style?.name ?? null,
      totals: !!t.totalsRow,
      cols: t.columns?.map((c) => c.name),
    }));
    sheets.push({
      name: ws.name,
      cells,
      tables,
      widths: (ws.columns ?? []).map((c) => c.width),
      merges: Object.keys(ws._merges ?? {}),
      images: (ws.getImages?.() ?? []).length,
      conditionalFormats: (ws.conditionalFormattings ?? []).map((cf) => cf.ref),
    });
  });
  return sheets;
}

const [ref, nat] = await Promise.all([dump(refPath), dump(natPath)]);
let failures = 0;
const report = (ok, msg) => {
  if (!ok) failures++;
  console.log(`${ok ? "  ok  " : " FAIL "} ${msg}`);
};

console.log("=== sheets ===");
report(
  ref.length === nat.length && ref.every((s, i) => s.name === nat[i].name),
  `sheet order/names: ref=[${ref.map((s) => s.name)}] native=[${nat.map((s) => s.name)}]`,
);
console.log("=== sheet contents ===");
for (let i = 0; i < Math.max(ref.length, nat.length); i++) {
  const a = ref[i];
  const b = nat[i];
  if (!a || !b) continue;
  console.log(`-- ${a.name} --`);
  report(a.cells.length === b.cells.length, `cell count ref=${a.cells.length} native=${b.cells.length}`);
  const key = (c) => `${c.r}:${c.c}`;
  const bByKey = new Map(b.cells.map((c) => [key(c), c]));
  let diffs = [];
  for (const c of a.cells) {
    const other = bByKey.get(key(c));
    if (!other) {
      diffs.push(`missing ${key(c)} ref=${JSON.stringify(c.v)}`);
      continue;
    }
    bByKey.delete(key(c));
    if (JSON.stringify(c.v) !== JSON.stringify(other.v))
      diffs.push(`value ${key(c)} ref=${JSON.stringify(c.v)} native=${JSON.stringify(other.v)}`);
    if (c.numFmt !== other.numFmt) diffs.push(`numFmt ${key(c)} ref=${c.numFmt} native=${other.numFmt}`);
    if (c.fill !== other.fill) diffs.push(`fill ${key(c)} ref=${c.fill} native=${other.fill}`);
    if (c.bold !== other.bold) diffs.push(`bold ${key(c)} ref=${c.bold} native=${other.bold}`);
    if (c.color !== other.color) diffs.push(`color ${key(c)} ref=${c.color} native=${other.color}`);
  }
  for (const [, c] of bByKey) diffs.push(`extra ${key(c)} native=${JSON.stringify(c.v)}`);
  report(diffs.length === 0, `cells identical (${a.cells.length} checked)`);
  for (const d of diffs.slice(0, 12)) console.log(`        ${d}`);
  if (diffs.length > 12) console.log(`        ... ${diffs.length - 12} more`);
  report(JSON.stringify(a.tables) === JSON.stringify(b.tables), `tables: ref=${JSON.stringify(a.tables)} native=${JSON.stringify(b.tables)}`);
  report(JSON.stringify(a.widths) === JSON.stringify(b.widths), `widths ref=${a.widths} native=${b.widths}`);
  report(JSON.stringify(a.merges) === JSON.stringify(b.merges), `merges ref=${a.merges} native=${b.merges}`);
  report(a.conditionalFormats.length === b.conditionalFormats.length && JSON.stringify(a.conditionalFormats) === JSON.stringify(b.conditionalFormats), `conditional formats ref=${a.conditionalFormats} native=${b.conditionalFormats}`);
  report(a.images === b.images, `images ref=${a.images} native=${b.images}`);
}

console.log(failures === 0 ? "\nEXPORT PARITY OK" : `\nEXPORT PARITY FAILED (${failures} checks)`);
process.exit(failures === 0 ? 0 : 1);
