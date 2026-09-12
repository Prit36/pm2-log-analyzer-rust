import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });
await page.goto("http://localhost:5173", { waitUntil: "networkidle" });
await page.waitForTimeout(500);
const out = await page.evaluate(() => {
  const mk = (css, text = "x") => {
    const s = document.createElement("span");
    s.textContent = text;
    s.setAttribute("style", css);
    document.body.appendChild(s);
    const cs = getComputedStyle(s);
    return { css, fs: cs.fontSize, lh: cs.lineHeight, fw: cs.fontWeight, disp: cs.display, ff: cs.fontFamily.split(",")[0] };
  };
  return [
    mk("font-size:10px"),
    mk("font-size:10px;display:inline-block"),
    mk("font-size:10px;font-weight:700"),
    mk("font-size:10px;font-weight:700;display:inline-block"),
    mk("font-size:10px;font-family:'IBM Plex Mono'"),
    mk("font-size:10px;font-family:'IBM Plex Mono';display:inline-block"),
    mk("font-size:10px;line-height:normal"),
    mk("font-size:11px;display:inline-block"),
    mk("font-size:11px;font-family:'IBM Plex Mono';display:inline-block"),
    mk("font-size:12px;display:inline-block"),
  ];
});
console.log(JSON.stringify(out, null, 1));
await browser.close();
