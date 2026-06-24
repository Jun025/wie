// In-browser verification: drives a real Chrome through the full user flow —
// BYOF upload → IndexedDB → wasm emulator boot → run a real WIPI app — and
// records every network request so we can prove no game bytes ever leave.
//
// Usage: node scripts/verify-browser.mjs <gameFilePath> [label]
// The game file is read from a path OUTSIDE dist/ and is never committed nor
// uploaded to any server by the app.

import { chromium } from "playwright";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const gamePath = process.argv[2] || path.join(root, "test_data", "helloworld_ktf.zip");
const label = process.argv[3] || path.basename(gamePath);
const BASE = process.env.WIE_BASE || "http://localhost:8788";

const browser = await chromium.launch({ channel: "chrome", headless: true });
const page = await browser.newPage();

const logs = [];
const requests = [];
page.on("console", (m) => logs.push(`[console.${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));
page.on("request", (req) => {
  requests.push({ method: req.method(), url: req.url(), postLen: (req.postData() || "").length, post: req.postData() || "" });
});

console.log(`▶ loading ${BASE}`);
await page.goto(BASE, { waitUntil: "networkidle" });

console.log(`▶ uploading game (BYOF): ${gamePath}`);
await page.setInputFiles('[data-testid="file-input"]', gamePath);

// Wait for it to land in the device-local library, then click Run.
await page.waitForSelector('[data-testid="run-game"]', { timeout: 10000 });
console.log("▶ game appeared in device-local library");
await page.click('[data-testid="run-game"]');
await page.waitForSelector('[data-testid="screen"]', { timeout: 10000 });

// Let the emulator run.
await page.waitForTimeout(10000);

// Screenshot the canvas + page.
await page.locator('[data-testid="screen"]').screenshot({ path: path.join(root, `verify_${label}_screen.png`) }).catch(() => {});
await page.screenshot({ path: path.join(root, `verify_${label}_page.png`), fullPage: true });

// Analyze the canvas: how many non-black pixels were drawn?
const pixelStats = await page.evaluate(() => {
  const c = document.querySelector('[data-testid="screen"]');
  if (!c) return null;
  const ctx = c.getContext("2d");
  const { data } = ctx.getImageData(0, 0, c.width, c.height);
  let nonBlack = 0;
  for (let i = 0; i < data.length; i += 4) {
    if (data[i] || data[i + 1] || data[i + 2]) nonBlack++;
  }
  return { w: c.width, h: c.height, nonBlack, total: c.width * c.height };
});

console.log("\n──────── RESULT ────────");
console.log("console/page logs:");
for (const l of logs) console.log("   " + l);
console.log("\ncanvas pixel stats:", JSON.stringify(pixelStats));

// ── Network self-audit ────────────────────────────────────────────────────────
// Read the uploaded game bytes and check no request body smuggled them out.
const fs = await import("node:fs/promises");
const gameBytes = await fs.readFile(gamePath);
const gameHead = gameBytes.subarray(0, 16).toString("latin1");

const sameOrigin = (u) => u.startsWith(BASE);
const offOrigin = requests.filter((r) => !sameOrigin(r.url));
const apiPosts = requests.filter((r) => r.method === "POST" || r.method === "PUT");
const bodyCarryingGame = requests.filter((r) => r.post && r.post.includes(gameHead) && gameHead.length > 4);

console.log("\nnetwork requests:");
for (const r of requests) console.log(`   ${r.method} ${r.url}  (body ${r.postLen}B)`);
console.log("\noff-origin requests:", offOrigin.length);
console.log("POST/PUT requests:", apiPosts.map((r) => r.url).join(", ") || "(none)");
console.log("requests whose body contains the game header bytes:", bodyCarryingGame.length);

const leak = offOrigin.length > 0 || bodyCarryingGame.length > 0;
console.log("\nNO-LEAK AUDIT:", leak ? "❌ POSSIBLE LEAK" : "✅ no game bytes left the browser, no off-origin requests");

await browser.close();
process.exit(leak ? 2 : 0);
