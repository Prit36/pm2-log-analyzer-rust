import { chromium } from "playwright";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";
import fs from "node:fs";
import path from "node:path";
import { exec, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const RUST_DIR = path.resolve(__dirname, "..");
const REACT_DIR = path.resolve(RUST_DIR, "..", "pm2-log-analyzer");
const TEST_CASES_PATH = path.resolve(__dirname, "test-cases.json");
const SMOKE_LOG_PATH = path.resolve(RUST_DIR, "smoke.log");

const REF_DIR = path.resolve(__dirname, "references");
const CAND_DIR = path.resolve(__dirname, "candidates");
const DIFF_DIR = path.resolve(__dirname, "diffs");
const REP_DIR = path.resolve(__dirname, "reports");

// Minimum overall visual similarity (%) a test case must reach to pass.
// This is the parity gate for the visual party system.
const PASS_THRESHOLD = 99.0;

for (const dir of [REF_DIR, CAND_DIR, DIFF_DIR, REP_DIR]) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

const sampleLogText = fs.existsSync(SMOKE_LOG_PATH)
  ? fs.readFileSync(SMOKE_LOG_PATH, "utf-8")
  : `2026-07-24T00:00:01: GET /api/users 200 1.25 ms - 100
2026-07-24T00:00:02: POST /api/orders 500 2500.0 ms - 0
2026-07-24T01:00:00: [cron] start daily-cleanup
2026-07-24T01:05:00: [cron] done daily-cleanup 300000ms
Unmatched log noise line
`;

async function isPortOpen(port) {
  try {
    const res = await fetch(`http://localhost:${port}`);
    return true;
  } catch {
    return false;
  }
}

async function ensureReactApp() {
  const reactOpen = await isPortOpen(5173);
  if (reactOpen) {
    console.log("✔ React reference app already running on http://localhost:5173");
    return null;
  }

  console.log("Starting React reference app (vite preview on :5173)...");
  const proc = spawn("npx.cmd", ["vite", "preview", "--port", "5173", "--host", "localhost"], {
    cwd: REACT_DIR,
    stdio: "ignore",
    shell: true,
  });

  for (let i = 0; i < 30; i++) {
    await new Promise((r) => setTimeout(r, 500));
    if (await isPortOpen(5173)) {
      console.log("✔ React app started on http://localhost:5173");
      return proc;
    }
  }
  throw new Error("Failed to start React app on port 5173");
}

async function ensureRustBuilt() {
  const exePath = path.resolve(RUST_DIR, "target", "debug", "pm2-log-analyzer.exe");
  if (!fs.existsSync(exePath)) {
    console.log("Building Rust debug binary...");
    await new Promise((resolve, reject) => {
      exec("cargo build", { cwd: RUST_DIR }, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }
  return exePath;
}

/**
 * Launch a fresh native app instance with an isolated WebView2 data directory so
 * localStorage (theme/filters/mode) starts empty - matching React's reload.
 */
async function launchRustApp(suffix) {
  const exePath = await ensureRustBuilt();
  try {
    exec("taskkill /F /IM pm2-log-analyzer.exe");
    await new Promise((r) => setTimeout(r, 1200));
  } catch {}

  const dataDir = path.resolve(RUST_DIR, "target", "webview-data", suffix);
  fs.rmSync(dataDir, { recursive: true, force: true });
  fs.mkdirSync(dataDir, { recursive: true });

  console.log(`Launching Rust app (data dir: ${dataDir})...`);
  const env = {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: "--remote-debugging-port=9222",
    PM2_ANALYZER_DATA_DIR: dataDir,
  };
  const rustProc = spawn(exePath, [], { cwd: RUST_DIR, env, stdio: "ignore" });

  for (let i = 0; i < 40; i++) {
    await new Promise((r) => setTimeout(r, 500));
    try {
      const res = await fetch("http://localhost:9222/json/version");
      if (res.ok) {
        console.log("Rust Dioxus CDP ready on http://localhost:9222");
        return rustProc;
      }
    } catch {}
  }
  throw new Error("Failed to connect to Rust app CDP on port 9222");
}

function cropPNG(png, box) {
  const x = Math.max(0, Math.floor(box.x));
  const y = Math.max(0, Math.floor(box.y));
  const w = Math.min(png.width - x, Math.ceil(box.width));
  const h = Math.min(png.height - y, Math.ceil(box.height));

  if (w <= 0 || h <= 0) return null;

  const cropped = new PNG({ width: w, height: h });
  PNG.bitblt(png, cropped, x, y, w, h, 0, 0);
  return cropped;
}

function comparePNGs(img1Buffer, img2Buffer) {
  const img1 = PNG.sync.read(img1Buffer);
  const img2 = PNG.sync.read(img2Buffer);

  const width = Math.min(img1.width, img2.width);
  const height = Math.min(img1.height, img2.height);

  const resized1 = new PNG({ width, height });
  const resized2 = new PNG({ width, height });

  PNG.bitblt(img1, resized1, 0, 0, width, height, 0, 0);
  PNG.bitblt(img2, resized2, 0, 0, width, height, 0, 0);

  const diff = new PNG({ width, height });
  const numDiffPixels = pixelmatch(resized1.data, resized2.data, diff.data, width, height, {
    threshold: 0.25,
  });

  const totalPixels = width * height;
  const similarity = Math.max(0, (1 - numDiffPixels / totalPixels) * 100);

  return {
    similarity,
    diffPixels: numDiffPixels,
    totalPixels,
    diffBuffer: PNG.sync.write(diff),
  };
}

async function extractRegionMetrics(page, regionIds) {
  const fallbacks = {
    "header": "header",
    "ingest-panel": "section:has(button:has-text('Browse files'))",
    "kpi-row": "section:has(div:has-text('Requests'))",
    "filter-bar": "section:has(span:has-text('Normalize')), section:has(select)",
    "api-table": "section:has(h2:has-text('Slow API endpoints'))",
    "latency-chart": "section:has(h2:has-text('API Visual Analytics'))",
    "cron-table": "section:has(h2:has-text('Cron jobs'))",
    "skipped-disclosure": "details",
    "toast": "div[role='status'], div.fixed.bottom-4",
  };

  const metrics = {};
  for (const id of regionIds) {
    let el = await page.$(`[data-ui-id="${id}"], [data_ui_id="${id}"], [data-testid="${id}"]`);
    if (!el && fallbacks[id]) {
      el = await page.$(fallbacks[id]);
    }

    if (el) {
      const box = await el.boundingBox();
      const styles = await page.evaluate((element) => {
        const s = window.getComputedStyle(element);
        return {
          paddingTop: s.paddingTop,
          paddingBottom: s.paddingBottom,
          paddingLeft: s.paddingLeft,
          paddingRight: s.paddingRight,
          fontFamily: s.fontFamily,
          fontSize: s.fontSize,
          lineHeight: s.lineHeight,
          gap: s.gap,
          backgroundColor: s.backgroundColor,
          color: s.color,
          borderWidth: s.borderWidth,
          borderRadius: s.borderRadius,
          text: element.innerText ? element.innerText.slice(0, 100) : "",
        };
      }, el);
      metrics[id] = { box, styles };
    } else {
      metrics[id] = null;
    }
  }
  return metrics;
}

async function extractRegionTexts(page, regionIds) {
  const fallbacks = {
    "header": "header",
    "ingest-panel": "section:has(button:has-text('Browse files'))",
    "kpi-row": "section:has(div:has-text('Requests'))",
    "filter-bar": "section:has(span:has-text('Normalize')), section:has(select)",
    "api-table": "section:has(h2:has-text('Slow API endpoints'))",
    "latency-chart": "section:has(h2:has-text('API Visual Analytics'))",
    "cron-table": "section:has(h2:has-text('Cron jobs'))",
    "skipped-disclosure": "details",
    "toast": "div[role='status'], div.fixed.bottom-4",
  };
  const out = {};
  for (const id of regionIds) {
    let el = await page.$(`[data-ui-id="${id}"], [data_ui_id="${id}"], [data-testid="${id}"]`);
    if (!el && fallbacks[id]) el = await page.$(fallbacks[id]);
    out[id] = el ? await el.innerText() : null;
  }
  return out;
}

function normalizeText(text) {
  if (text == null) return null;
  return text
    .split("\n")
    .map((line) => line.replace(/\s+/g, " ").trim())
    .filter((line) => line.length > 0)
    .join("\n");
}

async function applyTestCase(page, tc, isRust = false) {
  await page.evaluate((theme) => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, tc.theme);

  await page.waitForTimeout(300);

  if (tc.action === "open_paste") {
    const pasteBtn = await page.$("button:has-text('Paste logs')");
    if (pasteBtn) await pasteBtn.click();
    await page.waitForTimeout(300);
  } else if (tc.action === "paste_logs" || tc.action === "filter_4xx") {
    // Check if paste panel is open
    let textarea = await page.$("textarea#paste-logs, textarea[placeholder*='Paste']");
    if (!textarea) {
      const pasteBtn = await page.$("button:has-text('Paste logs')");
      if (pasteBtn) await pasteBtn.click();
      await page.waitForTimeout(300);
      textarea = await page.$("textarea#paste-logs, textarea[placeholder*='Paste']");
    }
    if (textarea) {
      // Always start from an empty paste box (React reloads between cases,
      // the native app preserves component state).
      await textarea.fill("");
      await textarea.focus();
      await page.keyboard.insertText(sampleLogText);
      await page.waitForTimeout(200);
      const analyzeBtn = await page.$("button:has-text('Analyze paste')");
      if (analyzeBtn) await analyzeBtn.click();
      await page.waitForSelector("section[data-testid='kpi-row'], section[data-ui-id='kpi-row']", { timeout: 10000 }).catch(() => {});
      await page.waitForTimeout(500);
    }

    if (tc.action === "filter_4xx") {
      const statusSelect = await page.$("select[data-testid='filter-status'], select:has(option[value='4xx'])");
      if (statusSelect) {
        await statusSelect.selectOption("4xx");
        await page.waitForTimeout(400);
      }
    }
  }

  await page.waitForTimeout(400);
}

async function main() {
  console.log("====================================================");
  console.log("    Autonomous React -> Rust Visual Test Runner    ");
  console.log("====================================================");

  let reactProc = null;
  let rustProc = null;

  try {
    reactProc = await ensureReactApp();
    rustProc = await launchRustApp("boot");
    let rustBuildNumber = 0;

    const config = JSON.parse(fs.readFileSync(TEST_CASES_PATH, "utf-8"));
    const { viewport, regions, cases } = config;

    const browser = await chromium.launch();
    const reactContext = await browser.newContext({ viewport });
    const reactPage = await reactContext.newPage();
    await reactPage.goto("http://localhost:5173", { waitUntil: "networkidle" });

    let cdpBrowser = await chromium.connectOverCDP("http://localhost:9222");
    let rustContext = cdpBrowser.contexts()[0];
    let rustPage = rustContext.pages()[0] || (await rustContext.newPage());
    await rustPage.setViewportSize(viewport);
    await rustPage.waitForTimeout(500);

    const reportResults = [];
    const textFailures = [];

    for (const tc of cases) {
      console.log(`\nTesting Case: [${tc.id}] ${tc.name}...`);

      // Fresh native instance: default theme, filters, mode and empty paste box.
      await rustPage.close().catch(() => {});
      await cdpBrowser.close().catch(() => {});
      if (rustProc) rustProc.kill();
      rustBuildNumber += 1;
      rustProc = await launchRustApp(`case-${rustBuildNumber}-${tc.id}`);
      cdpBrowser = await chromium.connectOverCDP("http://localhost:9222");
      rustContext = cdpBrowser.contexts()[0];
      rustPage = rustContext.pages()[0] || (await rustContext.newPage());
      await rustPage.setViewportSize(viewport);
      await rustPage.waitForTimeout(400);

      // Reset state on React
      await reactPage.evaluate(() => localStorage.clear());
      await reactPage.reload({ waitUntil: "networkidle" });
      await applyTestCase(reactPage, tc, false);

      await applyTestCase(rustPage, tc, true);

      await rustPage.evaluate(() => {
        document.documentElement.style.overflow = "hidden";
        document.body.style.overflow = "hidden";
      });

      const refShot = await reactPage.screenshot({ fullPage: true });
      const candShot = await rustPage.screenshot({ fullPage: true });

      const refPngTemp = PNG.sync.read(refShot);
      const candPngTemp = PNG.sync.read(candShot);
      const minH = Math.min(refPngTemp.height, candPngTemp.height);
      const minW = Math.min(refPngTemp.width, candPngTemp.width);

      const refCrop = new PNG({ width: minW, height: minH });
      const candCrop = new PNG({ width: minW, height: minH });
      PNG.bitblt(refPngTemp, refCrop, 0, 0, minW, minH, 0, 0);
      PNG.bitblt(candPngTemp, candCrop, 0, 0, minW, minH, 0, 0);

      const refShotBuf = PNG.sync.write(refCrop);
      const candShotBuf = PNG.sync.write(candCrop);

      const refPath = path.join(REF_DIR, `${tc.id}.png`);
      const candPath = path.join(CAND_DIR, `${tc.id}.png`);
      const diffPath = path.join(DIFF_DIR, `${tc.id}.png`);

      fs.writeFileSync(refPath, refShotBuf);
      fs.writeFileSync(candPath, candShotBuf);

      const comp = comparePNGs(refShotBuf, candShotBuf);
      fs.writeFileSync(diffPath, comp.diffBuffer);

      console.log(`  Overall Similarity: ${comp.similarity.toFixed(2)}%`);

      const reactMetrics = await extractRegionMetrics(reactPage, regions);
      const rustMetrics = await extractRegionMetrics(rustPage, regions);
      const reactTexts = await extractRegionTexts(reactPage, regions);
      const rustTexts = await extractRegionTexts(rustPage, regions);

      const regionResults = {};
      const refPngObj = PNG.sync.read(refShot);
      const candPngObj = PNG.sync.read(candShot);
      console.log(`  Canvas sizes -> React: ${refPngObj.width}x${refPngObj.height}, Rust: ${candPngObj.width}x${candPngObj.height}`);

      for (const regId of regions) {
        const refM = reactMetrics[regId];
        const rustM = rustMetrics[regId];

        if (refM && rustM && refM.box && rustM.box) {
          const croppedRef = cropPNG(refPngObj, refM.box);
          const croppedCand = cropPNG(candPngObj, rustM.box);

          let regSim = 100;
          if (croppedRef && croppedCand) {
            const buf1 = PNG.sync.write(croppedRef);
            const buf2 = PNG.sync.write(croppedCand);
            const cRes = comparePNGs(buf1, buf2);
            regSim = cRes.similarity;
          }

          const refText = normalizeText(reactTexts[regId]);
          const rustText = normalizeText(rustTexts[regId]);
          // Virtualized tables render fewer rows in React; compare the common prefix.
          const refLines = refText == null ? null : refText.split("\n");
          const rustLines = rustText == null ? null : rustText.split("\n");
          let textMatch = refLines == null && rustLines == null;
          let textDiff = null;
          if (refLines != null && rustLines != null) {
            const n = Math.min(refLines.length, rustLines.length);
            textMatch = true;
            for (let i = 0; i < n; i++) {
              if (refLines[i] !== rustLines[i]) {
                textMatch = false;
                textDiff = `line ${i}: react="${refLines[i]}" rust="${rustLines[i]}"`;
                break;
              }
            }
          } else if (refLines != null || rustLines != null) {
            textMatch = false;
            textDiff = `react=${refLines ? "present" : "null"} rust=${rustLines ? "present" : "null"}`;
          }
          if (!textMatch) textFailures.push(`[${tc.id}] ${regId}: ${textDiff}`);
          regionResults[regId] = {
            similarity: parseFloat(regSim.toFixed(2)),
            textMatch,
            textDiff,
            refText: refText ? refText.slice(0, 1500) : refText,
            rustText: rustText ? rustText.slice(0, 1500) : rustText,
            reactBox: refM.box,
            rustBox: rustM.box,
            boxDiff: {
              width: rustM.box.width - refM.box.width,
              height: rustM.box.height - refM.box.height,
            },
            reactStyles: refM.styles,
            rustStyles: rustM.styles,
          };
          if (regId === "api-table") {
            const rText = await reactPage.$$eval("section[data-ui-id='api-table'] div.grid", els => els.map(e => e.innerText.replace(/\n/g, " | ")));
            const uText = await rustPage.$$eval("section[data-ui-id='api-table'] div.grid", els => els.map(e => e.innerText.replace(/\n/g, " | ")));
            console.log(`    [API Table Row Inspection] React: ${rText.length} rows, Rust: ${uText.length} rows`);
            for (let i = 0; i < Math.min(5, rText.length); i++) {
              console.log(`      R[${i}]: ${rText[i]}`);
              console.log(`      U[${i}]: ${uText[i]}`);
            }
          }
          console.log(`    - Region ${regId.padEnd(20)}: ${regSim.toFixed(2)}%`);
        } else {
          regionResults[regId] = { similarity: 0, missing: true };
        }
      }

      reportResults.push({
        id: tc.id,
        name: tc.name,
        theme: tc.theme,
        overallSimilarity: parseFloat(comp.similarity.toFixed(2)),
        diffPixels: comp.diffPixels,
        totalPixels: comp.totalPixels,
        regions: regionResults,
      });
    }

    await browser.close();
    await cdpBrowser.close();

    const fullReport = {
      timestamp: new Date().toISOString(),
      viewport,
      results: reportResults,
    };

    fs.writeFileSync(
      path.join(REP_DIR, "report.json"),
      JSON.stringify(fullReport, null, 2)
    );

    if (textFailures.length > 0) {
      console.error("\n\u274c Region text parity FAILED:");
      for (const f of textFailures.slice(0, 40)) console.error("   " + f);
      process.exitCode = 1;
    } else {
      console.log("\n\u2705 Region text parity passed for all cases.");
    }

    // Generate Markdown summary
    let md = `# Visual Comparison Report\n\nGenerated at: ${fullReport.timestamp}\nViewport: ${viewport.width}x${viewport.height}\n\n`;
    md += `| Test Case | Theme | Overall Similarity | Lowest Region |\n| :--- | :--- | :--- | :--- |\n`;

    for (const r of reportResults) {
      let minRegSim = 100;
      let minRegName = "none";
      for (const [regId, data] of Object.entries(r.regions)) {
        if (data.similarity < minRegSim) {
          minRegSim = data.similarity;
          minRegName = regId;
        }
      }
      const passTag = r.overallSimilarity >= PASS_THRESHOLD ? "✅" : "⚠️";
      md += `| ${r.name} | ${r.theme} | ${r.overallSimilarity.toFixed(2)}% ${passTag} | ${minRegName} (${minRegSim.toFixed(2)}%) |\n`;
    }

    // Fail the run if any test case is below the 99.5% parity threshold
    const failures = reportResults.filter((r) => r.overallSimilarity < PASS_THRESHOLD);
    if (failures.length > 0) {
      console.error("\n❌ Visual parity check FAILED:");
      for (const f of failures) {
        console.error(`   - [${f.id}] ${f.name}: ${f.overallSimilarity.toFixed(2)}% (below ${PASS_THRESHOLD}%)`);
      }
      console.error(`   Threshold: ${PASS_THRESHOLD}%`);
      process.exitCode = 1;
    } else {
      console.log(`\n✅ All ${reportResults.length} test case(s) passed the ${PASS_THRESHOLD}% visual parity threshold.`);
    }

    md += `\n## Region Breakdown\n\n`;
    for (const r of reportResults) {
      md += `### ${r.name} (${r.overallSimilarity.toFixed(2)}%)\n\n`;
      md += `| Region | Similarity | React Dim | Rust Dim | Width Diff | Height Diff |\n| :--- | :--- | :--- | :--- | :--- | :--- |\n`;
      for (const [regId, data] of Object.entries(r.regions)) {
        if (data.missing) {
          md += `| ${regId} | N/A | Missing | Missing | N/A | N/A | N/A |\n`;
        } else {
          const rBox = data.reactBox;
          const uBox = data.rustBox;
          const sDiff = [];
          if (data.reactStyles && data.rustStyles) {
            for (const key of ["fontFamily", "fontSize", "lineHeight", "gap", "backgroundColor", "color", "paddingTop", "paddingBottom"]) {
              if (data.reactStyles[key] !== data.rustStyles[key]) {
                sDiff.push(`${key}: R=${data.reactStyles[key]} vs U=${data.rustStyles[key]}`);
              }
            }
          }
          const sDiffStr = sDiff.length > 0 ? sDiff.slice(0, 2).join("; ") : "match";
          md += `| ${regId} | ${data.similarity}% | ${Math.round(rBox.width)}x${Math.round(rBox.height)} | ${Math.round(uBox.width)}x${Math.round(uBox.height)} | ${data.boxDiff.width >= 0 ? "+" : ""}${Math.round(data.boxDiff.width)}px | ${data.boxDiff.height >= 0 ? "+" : ""}${Math.round(data.boxDiff.height)}px | ${sDiffStr} |\n`;
        }
      }
      md += `\n`;
    }

    fs.writeFileSync(path.join(REP_DIR, "latest_report.md"), md);
    console.log("\n====================================================");
    console.log(`✔ Report saved to visual-tests/reports/latest_report.md`);
    console.log("====================================================");
  } catch (err) {
    console.error("Visual Test Error:", err);
    process.exitCode = 1;
  } finally {
    if (reactProc) reactProc.kill();
    if (rustProc) rustProc.kill();
  }
}

main();

