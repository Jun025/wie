// Behavioral engine↔featurephone contract check — boots the freshly built wasm
// artifact in a real (headless) Chromium through EXACTLY the call shapes the
// featurephone shell uses (otterpebble apps/featurephone/lib/engine.ts), using
// the repo's own hello-world fixtures (test_data/ — no commercial game files).
//
// This restores, engine-side, the coverage the web shell lost when its boot
// self-test was removed (2026-07-20): if an engine change breaks the boot
// round-trip, wie CI fails BEFORE the artifact is published and propagated.
//
// Scenario A (KTF fixture — featurephone PRIMARY path):
//   precompiled WebAssembly.Module → default(module) → init() → new WieEmulator
//   → tick loop to CLEAN EXIT (the hello-world fixtures print + request exit,
//   so the full WIPI-exit → sticky has_exited() → tick-no-op chain is observed
//   end-to-end) → key vocabulary sweep → save export/import round-trip
//   (WIESAV01, still readable after exit — the shell persists post-exit) →
//   free(). NOTE the helloworld_* fixtures print and exit but never draw, so
//   their pixel count stays info only — Scenario C is what asserts the blit.
// Scenario B (LGT fixture — featurephone FALLBACK init path + fresh glue):
//   cache-busted glue re-import → default() no-arg (glue must fetch
//   wie_web_bg.wasm by its pinned name) → lgt_compile_model() === "clet".
// Scenario C (J2ME draw fixture — the canvas blit path, ASSERTED):
//   test_data/draw_j2me.jar paints one filled rect (scripts/make-draw-fixture.mjs
//   builds it), so nonBlackPixels() > 0 is a real assertion here: it fails if the
//   core stops composing frames or WebScreen::paint stops reaching the canvas.
//   This fixture never exits — the loop stops at the first painted frame.
// Scenario D (same J2ME instance — KEY DELIVERY, ASSERTED):
//   Scenario A's sweep only proves key_down/key_up don't throw, which an engine
//   that drops every event also passes. Here the fixture's keyPressed() paints a
//   bar as wide as the MIDP code it received, so the canvas says WHICH code
//   reached the guest. Representative keys only (soft/numeric/direction) — the
//   full vocabulary stays check-engine-contract.mjs's source pin.
//
// Usage: node scripts/contract-roundtrip.mjs        (after scripts/build-wasm.sh)
//   WIE_CHROME_CHANNEL=chrome  — use a system Chrome instead of the playwright
//                                bundled chromium (local dev convenience).

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { drawFixtureJar, keyBarPixels } from "./make-draw-fixture.mjs";

// Scenario D's representative keys — one per class of the contract vocabulary,
// deliberately NOT all 20 (this asserts that delivery works, not that every code
// is mapped; the mapping table is what check-engine-contract.mjs pins).
//   soft key  — the shell's menu/back key, low "phone key" code band
//   numeric   — ASCII-valued band
//   direction — MIDP Canvas named-key band (141..148), the shell's D-pad
// `midp` mirrors MIDPKeyCode in wie_midp/src/classes/net/wie/event_queue.rs: it
// is the number the GUEST sees, so a rewiring there fails here loudly (the guest
// paints a differently-sized bar) instead of silently.
// ASCENDING code order is required — see make-draw-fixture.mjs (the bar is a
// union across frames, so ascending keeps every expected count exact).
const REPRESENTATIVE_KEYS = [
  { code: "LEFT_SOFT_KEY", midp: 6, cls: "soft key" },
  { code: "NUM5", midp: 53, cls: "numeric" },
  { code: "UP", midp: 141, cls: "direction" },
].map((k) => ({ ...k, expectPixels: keyBarPixels(k.midp) }));

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(await readFile(path.join(root, "docs/contracts/featurephone-engine-contract.json"), "utf8"));

// ── Tiny static server: glue+wasm and fixtures, query string ignored ─────────
const MIME = { ".js": "text/javascript", ".wasm": "application/wasm", ".zip": "application/zip", ".html": "text/html" };
const server = createServer(async (req, res) => {
  const url = new URL(req.url, "http://x");
  let file = null;
  if (url.pathname === "/") {
    res.writeHead(200, { "content-type": "text/html" });
    res.end("<!doctype html><html><body></body></html>");
    return;
  }
  // Built in memory, not on disk: *.jar is git-ignored (Constraint 9).
  if (url.pathname === "/fixtures/draw_j2me.jar") {
    const jar = drawFixtureJar();
    res.writeHead(200, { "content-type": "application/java-archive", "content-length": jar.length });
    res.end(jar);
    return;
  }
  if (url.pathname.startsWith("/wasm/")) file = path.join(root, contract.artifacts.dir, path.basename(url.pathname));
  if (url.pathname.startsWith("/fixtures/")) file = path.join(root, "test_data", path.basename(url.pathname));
  try {
    const data = await readFile(file);
    res.writeHead(200, { "content-type": MIME[path.extname(file)] ?? "application/octet-stream", "content-length": data.length });
    res.end(data);
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

const { chromium } = await import("playwright");
const launchOpts = { headless: true };
if (process.env.WIE_CHROME_CHANNEL) launchOpts.channel = process.env.WIE_CHROME_CHANNEL;
const browser = await chromium.launch(launchOpts);
const page = await browser.newPage();
const consoleLog = [];
page.on("console", (m) => consoleLog.push(`[console.${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => consoleLog.push(`[pageerror] ${e.message}`));
await page.goto(base + "/");
page.setDefaultTimeout(120_000);

const steps = await page.evaluate(async ({ contract, representativeKeys }) => {
  const steps = [];
  const check = (name, pass, info = "") => {
    steps.push({ name, pass: !!pass, info: String(info) });
    return !!pass;
  };
  const nonBlackPixels = (canvas) => {
    const { data } = canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height);
    let n = 0;
    for (let i = 0; i < data.length; i += 4) if (data[i] || data[i + 1] || data[i + 2]) n++;
    return n;
  };
  // Drive the emulator like RunningGame does: tick per animation frame (the
  // core is an async executor — a tight loop without yielding cannot progress),
  // polling has_exited() after every tick exactly like the shell's loop.
  // `until(pixels)` is for the draw fixture, which never exits: stop as soon as
  // the canvas reaches the wanted state instead of burning the whole deadline.
  const tickLoop = async (emu, canvas, deadlineMs, until = null) => {
    const start = performance.now();
    let frames = 0;
    let threw = null;
    while (performance.now() - start < deadlineMs) {
      try {
        emu.tick();
      } catch (e) {
        threw = String(e);
        break;
      }
      frames++;
      if (emu.has_exited()) break;
      if (until && until(nonBlackPixels(canvas))) break;
      await new Promise((r) => requestAnimationFrame(r));
    }
    return { frames, threw, pixels: nonBlackPixels(canvas) };
  };
  const bootFixture = async (mod, fixture) => {
    const bytes = new Uint8Array(await (await fetch(`/fixtures/${fixture}`)).arrayBuffer());
    const canvas = document.createElement("canvas");
    document.body.appendChild(canvas);
    // Exact featurephone constructor shape: audio ctx/gain omitted = silent mode.
    const emu = new mod.WieEmulator(fixture, bytes, canvas, undefined, undefined, contract.screen.width, contract.screen.height);
    return { emu, canvas };
  };

  try {
    // ── Scenario A: KTF fixture, featurephone PRIMARY init path ──────────────
    const wasmBytes = await (await fetch("/wasm/wie_web_bg.wasm")).arrayBuffer();
    const module = await WebAssembly.compile(wasmBytes);
    const mod = await import(`/wasm/wie_web.js?v=1`);
    check("A: glue import (cache-busted, ES module)", mod && typeof mod.default === "function");
    await mod.default(module); // precompiled-Module path — featurephone's compiledModule cache pattern
    mod.init();
    check("A: default(WebAssembly.Module) + init()", true);

    const { emu, canvas } = await bootFixture(mod, "helloworld_ktf.zip");
    check("A: new WieEmulator(7 args) boots KTF fixture", true);
    check('A: platform_kind() === "KTF"', emu.platform_kind() === "KTF", `got ${emu.platform_kind()}`);
    check("A: lgt_compile_model() undefined for non-LGT", emu.lgt_compile_model() === undefined, `got ${emu.lgt_compile_model()}`);
    check("A: has_exited() false at boot", emu.has_exited() === false);

    const runA = await tickLoop(emu, canvas, 20_000);
    check("A: tick loop survives (no throw)", runA.threw === null, runA.threw ?? `${runA.frames} frames`);
    // The fixture requests a normal shutdown — this observes the whole clean-exit
    // chain the shell's exit panel depends on: core exit → sticky getter flip.
    check("A: clean exit observed (has_exited() flips true)", emu.has_exited() === true, `${runA.frames} frames, ${runA.pixels} px (fixture draws nothing — pixels are info only)`);
    let postExitThrew = "";
    try {
      emu.tick();
      emu.tick();
      emu.tick();
    } catch (e) {
      postExitThrew = String(e);
    }
    check("A: tick() after exit is a safe no-op", postExitThrew === "", postExitThrew);

    let keyFail = "";
    for (const code of contract.keyVocabulary) {
      try {
        emu.key_down(code);
        emu.key_up(code);
      } catch (e) {
        keyFail = `${code}: ${e}`;
        break;
      }
    }
    check("A: key vocabulary down/up sweep (no throw)", keyFail === "", keyFail || `${contract.keyVocabulary.length} codes`);

    // Post-exit on purpose: the shell reads the final save AFTER the exit flip
    // (persist-then-free) — saves must stay readable on an exited instance.
    const blob = emu.export_saves();
    const magic = new TextDecoder().decode(blob.slice(0, 8));
    check("A: export_saves() readable after exit → Uint8Array", blob instanceof Uint8Array, `${blob?.length} bytes`);
    check(`A: save blob magic "${contract.saveMagic}"`, magic === contract.saveMagic, `got "${magic}"`);
    check("A: import_saves(exported blob) → true", emu.import_saves(blob) === true);
    check("A: import_saves(garbage) → false (no throw)", emu.import_saves(new Uint8Array([1, 2, 3])) === false);
    check("A: has_saves() is boolean", typeof emu.has_saves() === "boolean");
    emu.free();
    check("A: free() (no throw)", true);

    // ── Scenario B: LGT fixture, FALLBACK init path + fresh glue instance ────
    const mod2 = await import(`/wasm/wie_web.js?v=2`);
    await mod2.default(); // no-arg: glue must fetch wie_web_bg.wasm by its pinned name next to itself
    mod2.init();
    check("B: fresh glue + default() no-arg (name-coupled wasm fetch)", true);

    const b = await bootFixture(mod2, "helloworld_lgt.zip");
    check('B: platform_kind() === "LGT"', b.emu.platform_kind() === "LGT", `got ${b.emu.platform_kind()}`);
    check('B: lgt_compile_model() === "clet"', b.emu.lgt_compile_model() === "clet", `got ${b.emu.lgt_compile_model()}`);
    const runB = await tickLoop(b.emu, b.canvas, 20_000);
    check("B: tick loop survives (no throw)", runB.threw === null, runB.threw ?? `${runB.frames} frames`);
    check("B: clean exit observed (has_exited() flips true)", b.emu.has_exited() === true, `${runB.frames} frames, ${runB.pixels} px (fixture draws nothing — pixels are info only)`);
    b.emu.free();
    check("B: free() (no throw)", true);

    // ── Scenario C: J2ME draw fixture — canvas blit, ASSERTED not reported ───
    const c = await bootFixture(mod2, "draw_j2me.jar");
    check('C: platform_kind() === "J2ME"', c.emu.platform_kind() === "J2ME", `got ${c.emu.platform_kind()}`);
    const runC = await tickLoop(c.emu, c.canvas, 30_000, (px) => px > 0);
    check("C: tick loop survives (no throw)", runC.threw === null, runC.threw ?? `${runC.frames} frames`);
    check("C: canvas blit ASSERTED — fixture's rect reaches the canvas", runC.pixels > 0, `${runC.pixels} non-black px after ${runC.frames} frames`);

    // ── Scenario D: does a key press REACH THE GUEST? (behavioral, not no-throw) ─
    // Scenario A only proves key_down/key_up don't throw — an engine that drops
    // every event passes that. Here the guest itself answers: its keyPressed()
    // paints a bar as wide as the MIDP code it was handed, so the canvas encodes
    // WHICH code arrived. Same instance as C: it never exits and keeps painting.
    for (const k of representativeKeys) {
      c.emu.key_down(k.code);
      const runD = await tickLoop(c.emu, c.canvas, 15_000, (px) => px === k.expectPixels);
      c.emu.key_up(k.code);
      check(
        `D: "${k.code}" (${k.cls}) reaches the guest — it paints MIDP code ${k.midp}`,
        runD.threw === null && runD.pixels === k.expectPixels,
        runD.threw ?? `${runD.pixels} px, expected ${k.expectPixels} (base + ${k.midp}*bar) after ${runD.frames} frames`,
      );
    }

    c.emu.free();
    check("C: free() (no throw)", true);
  } catch (e) {
    check("scenario aborted by exception", false, (e && e.stack) || String(e));
  }
  return steps;
}, { contract, representativeKeys: REPRESENTATIVE_KEYS });

await browser.close();
server.close();

const failed = steps.filter((s) => !s.pass);
console.log(`engine contract round-trip — ${steps.length - failed.length}/${steps.length} checks passed`);
for (const s of steps) console.log(`  ${s.pass ? "✓" : "✗"} ${s.name}${s.info ? ` — ${s.info}` : ""}`);
if (failed.length > 0) {
  console.log("\nbrowser console (diagnostics):");
  for (const l of consoleLog.slice(-40)) console.log("   " + l);
  console.log("\nThe featurephone web shell boots the engine exactly this way (docs/contracts/featurephone-engine-contract.json).");
  console.log("If the change is INTENTIONAL, update the contract + coordinate the otterpebble consumer in the same rollout.");
  process.exit(1);
}
console.log("OK — boot round-trip matches the pinned featurephone contract.");
process.exit(0);
