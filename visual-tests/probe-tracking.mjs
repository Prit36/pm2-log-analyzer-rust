import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });
await page.waitForTimeout(800);
const out = await page.evaluate(() => {
  const mk = (text, cls, family) => {
    const s = document.createElement("span");
    s.textContent = text;
    if (cls) s.className = cls;
    if (family) s.style.fontFamily = family;
    s.style.display = "inline-block";
    document.body.appendChild(s);
    const w = s.getBoundingClientRect().width;
    s.remove();
    return w;
  };
  return {
    h1_ref: mk("PM2 Log Analyzer", "text-base font-semibold tracking-tight"),
    h1_plain: mk("PM2 Log Analyzer", "text-base font-semibold"),
    requests_tracking: mk("REQUESTS", "text-[10px] font-semibold tracking-wide"),
    requests_plain: mk("REQUESTS", "text-[10px] font-semibold"),
    delete_tracking: mk("DELETE", "text-[10px] font-bold tracking-wide"),
    delete_plain: mk("DELETE", "text-[10px] font-bold"),
    h2_tracking: mk("Slow API endpoints", "text-xs font-semibold tracking-wide"),
    h2_plain: mk("Slow API endpoints", "text-xs font-semibold"),
    all_tracking: mk("All", "text-[10px] font-bold tracking-wide"),
    api_visual: mk("API Visual Analytics", "text-xs font-semibold tracking-wide"),
    date_mono_wide: mk("24/07/2026", "text-[10px] tracking-wide", '"IBM Plex Mono"'),
  };
});
console.log(JSON.stringify(out, null, 1));
await browser.close();
