// Structural comparison of two MongoDB Excel exports (reference vs native).
// Usage: node export-compare-mongo.mjs <ref.xlsx> <native.xlsx>
import ExcelJS from "../../pm2-log-analyzer/node_modules/.pnpm/exceljs@4.4.0/node_modules/exceljs/excel.js";
import path from "node:path";

const refPath = path.resolve(process.argv[2]);
const natPath = path.resolve(process.argv[3]);

async function dump(file) {
  const wb = new ExcelJS.Workbook();
  await wb.xlsx.readFile(file);
  const sheets = [];
  wb.eachSheet((ws) => {
    const rows = [];
    ws.eachRow({ includeEmpty: false }, (row, rowNumber) => {
      const values = [];
      row.eachCell({ includeEmpty: false }, (cell) => {
        const raw = cell.value;
        const value = raw && typeof raw === "object" && "result" in raw ? raw.result : raw;
        values.push(value === null || value === undefined ? "" : value);
      });
      rows.push({ row: rowNumber, values });
    });
    sheets.push({
      name: ws.name,
      frozen: ws.views?.[0]?.state === "frozen" ? ws.views[0].ySplit : null,
      rows,
    });
  });
  return sheets;
}

const [ref, nat] = await Promise.all([dump(refPath), dump(natPath)]);
const problems = [];

const refNames = ref.map((s) => s.name);
const natNames = nat.map((s) => s.name);
if (refNames.join("|") !== natNames.join("|")) {
  problems.push(`sheet names differ:\n  ref: ${refNames.join(", ")}\n  nat: ${natNames.join(", ")}`);
}

for (const refSheet of ref) {
  const natSheet = nat.find((s) => s.name === refSheet.name);
  if (!natSheet) continue;
  if (refSheet.frozen !== natSheet.frozen) {
    problems.push(`${refSheet.name}: frozen panes ${refSheet.frozen} vs ${natSheet.frozen}`);
  }

  const refHeader = refSheet.rows[0]?.values ?? [];
  const natHeader = natSheet.rows[0]?.values ?? [];
  if (refHeader.join("|") !== natHeader.join("|")) {
    problems.push(
      `${refSheet.name}: header mismatch\n  ref: ${refHeader.join(" | ")}\n  nat: ${natHeader.join(" | ")}`,
    );
  }

  // Slow Queries is capped at different lengths only if the source differs.
  const refRows = refSheet.rows.length;
  const natRows = natSheet.rows.length;
  if (refRows !== natRows) {
    problems.push(`${refSheet.name}: row count ${refRows} vs ${natRows}`);
  }

  // Compare every cell of the first three data rows (numeric formatting included).
  for (let i = 1; i < Math.min(4, refSheet.rows.length, natSheet.rows.length); i += 1) {
    const a = refSheet.rows[i].values;
    const b = natSheet.rows[i].values;
    for (let c = 0; c < Math.max(a.length, b.length); c += 1) {
      const av = a[c] ?? "";
      const bv = b[c] ?? "";
      if (typeof av === "number" && typeof bv === "number") {
        if (Math.abs(av - bv) > 0.011) {
          problems.push(`${refSheet.name} r${i + 1}c${c + 1}: ${av} vs ${bv}`);
        }
      } else if (String(av) !== String(bv)) {
        problems.push(`${refSheet.name} r${i + 1}c${c + 1}: "${av}" vs "${bv}"`);
      }
    }
  }
}

if (problems.length > 0) {
  console.error(`EXPORT MISMATCH (${problems.length}):`);
  for (const problem of problems) console.error(`- ${problem}`);
  process.exit(1);
}
console.log(`MONGO EXPORT PARITY OK: ${ref.length} sheets, headers + first rows identical`);
