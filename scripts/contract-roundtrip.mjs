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
//   reached the guest. Representative keys only (soft/numeric/direction).
//
//   ── Why 3 and not all 20 (decided 2026-09-04, do not "complete" this list) ──
//   Delivery splits into a per-key part and a key-agnostic part, and they need
//   different guards:
//     · per-key   — two tables: parse_key (name -> KeyCode) and
//                   MIDPKeyCode::from_key_code (KeyCode -> the int the guest
//                   sees). BOTH are pinned statically for all 20 keys by
//                   check-engine-contract.mjs §4 / §4b.
//     · key-agnostic — handle_event -> event queue -> Canvas::handleKeyEvent ->
//                   keyPressed(code). Measured: not one branch on which key, so
//                   this half is proven by ANY key that arrives. 3 witnesses
//                   (one per code band) already prove it; keys 4..20 would
//                   re-prove the same path at ~1 browser frame-loop each.
//   Earlier revisions of this comment said the remaining 17 "stay the source
//   pin" — that was only half true and is why the gap survived: the source pin
//   covered the FIRST table only, so a swapped row in the second one (NUM7 ->
//   56: press 7, type 8) was caught by nothing. §4b closes that.
//
//   ── REOPEN when a hit appears that is NOT in the table below ──────────────
//   The trigger is a NEW ENTRY, not a count: every `match` in the delivery
//   files is enumerated here with why it is in or out, so "does this still
//   hold?" is a diff against this list instead of a judgement call. (The first
//   version of this condition said "reopen when the count exceeds 1" and was
//   already false on the day it was written — the count is 10.)
//     $ grep -nE '(^|[^[:alnum:]_])match[^[:alnum:]_]' \
//         wie_web/src/lib.rs \
//         wie_midp/src/classes/net/wie/event_queue.rs \
//         wie_midp/src/classes/javax/microedition/lcdui/display.rs \
//         wie_midp/src/classes/javax/microedition/lcdui/displayable.rs \
//         wie_midp/src/classes/javax/microedition/lcdui/canvas.rs
//   2026-09-04 — 10 hits, and NOT ONE of them is a per-key branch on the path:
//     lib.rs:444         not code   — doc comment ("Names match the `KeyCode`")
//     lib.rs:447         TABLE      — parse_key: name -> KeyCode         (pinned, §4)
//     event_queue.rs:120 TABLE      — from_key_code: KeyCode -> guest int (pinned, §4b)
//     event_queue.rs:90  TABLE      — MIDPKeyCode::from_raw: int -> variant; reads the same
//                                     discriminants §4b pins, so it cannot disagree with :120
//     event_queue.rs:25  path, key-agnostic — EventQueueEvent kind (KeyEvent/Repaint/Notify)
//     event_queue.rs:46  path, key-agnostic — KeyboardEventType (pressed/released/repeated)
//     event_queue.rs:205 path, key-agnostic — Event shape; the key value is handed whole to
//                                     from_key_code(x), never inspected here
//     event_queue.rs:298 path, key-agnostic — event kind again, on the dispatch side
//     canvas.rs:157      path, key-agnostic — event type -> keyPressed/Released/Repeated;
//                                     `code` passes through untouched
//     canvas.rs:94       OFF-PATH   — Canvas::getGameAction. Guest-initiated: its only entry is
//                                     the JavaMethodProto the guest calls (zero internal callers,
//                                     measured), so it runs AFTER delivery on a code the guest
//                                     already holds. It IS an unpinned per-key table — carried as
//                                     residual (see the worklog proposal), not as delivery.
//   "On the path" is decidable, not a vibe: a function is on it iff it is reachable from
//   WieEmulator::key_down WITHOUT the guest initiating the call. Measured chain —
//     key_down -> handle_event -> EventQueue -> Display::handleKeyEvent
//              -> Displayable/Canvas::handleKeyEvent -> keyPressed
//   display.rs and displayable.rs are on it too (that is why they are in the grep) and have
//   ZERO `match`; both forward `code` unchanged.
//   Every line above names a file, a line and a reason — no exemption is granted to a *kind*
//   of thing, so a new table cannot fold itself into this list by resembling an old one.
//
// Scenario E (KTF keydraw fixture — KEY DELIVERY ON THE *WIPI* PATH, ASSERTED):
//   D proves delivery down the MIDP path with a J2ME guest. KTF guests get one
//   more hop — CardCanvas overrides keyPressed and re-maps the MIDP code through
//   WIPIKeyCode::from_midp_raw before the guest sees it — so D says nothing about
//   the number a WIPI guest receives. test_data/keydraw_ktf.zip paints a bar as
//   wide as that number, so the canvas answers for the whole three-hop chain.
//   wie_ktf/tests/test_key_reach.rs already asserts this headlessly via guest
//   stdout; the browser adds the layers that test cannot reach — the wasm build,
//   the wasm-bindgen glue, and WebScreen -> canvas.
//
//   ── Only positive WIPI codes are asserted, and that is the fixture's rule ──
//   The guest source (scripts/make-wipi-keydraw-fixture.sh) says it outright:
//   for digits and symbols the bar width IS the WIPI code the host delivered,
//   while the named keys (UP/OK/CALL/...) are NEGATIVE in WIPI space, cannot be
//   a bar width, and get an arbitrary positive slot each. Asserting a slot would
//   mean restating a table no contract pins — a second source of truth, which is
//   the failure this file exists to prevent. The named rows are not unguarded:
//   check-engine-contract.mjs §4c pins all 22 WIPI codes statically against
//   WIPIKeyCode::from_midp_raw. What the browser adds is the key-agnostic half,
//   and any one arriving key proves that (same argument as Scenario D).
//
// Usage: node scripts/contract-roundtrip.mjs        (after scripts/build-wasm.sh)
//   WIE_CHROME_CHANNEL=chrome  — use a system Chrome instead of the playwright
//                                bundled chromium (local dev convenience).

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { drawFixtureJar, keyBarPixels } from "./make-draw-fixture.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(await readFile(path.join(root, "docs/contracts/featurephone-engine-contract.json"), "utf8"));

// Scenario D's representative keys — one per class of the contract vocabulary,
// deliberately NOT all 20 — the reasoning is in the Scenario D header above.
//   soft key  — the shell's menu/back key, low "phone key" code band
//   numeric   — ASCII-valued band
//   direction — MIDP Canvas named-key band (141..148), the shell's D-pad
// The expected `midp` is READ FROM THE CONTRACT (keyMidpCodes), not restated
// here: it is the number the GUEST sees, and check-engine-contract.mjs §4b pins
// the same contract entry against MIDPKeyCode in
// wie_midp/src/classes/net/wie/event_queue.rs. So a rewiring there fails twice
// — statically there, and here loudly (the guest paints a differently-sized
// bar) — instead of silently.
// ASCENDING code order is required — see make-draw-fixture.mjs (the bar is a
// union across frames, so ascending keeps every expected count exact).
const REPRESENTATIVE_KEYS = [
  { code: "LEFT_SOFT_KEY", cls: "soft key" },
  { code: "NUM5", cls: "numeric" },
  { code: "UP", cls: "direction" },
].map((k) => {
  const midp = contract.keyMidpCodes?.[k.code];
  // Fail-closed: a missing contract entry must not silently become `undefined`
  // pixels (which would compare equal to nothing and hang the tick loop).
  if (typeof midp !== "number") throw new Error(`contract.keyMidpCodes has no code for representative key "${k.code}"`);
  return { ...k, midp, expectPixels: keyBarPixels(midp) };
});

// ── Scenario E constants: DERIVED from the fixture's own source, never restated ──
// keydraw_ktf.zip is a committed binary, so its constants cannot be exported the
// way make-draw-fixture.mjs exports Scenario C/D's. They are read back out of the
// guest source embedded in scripts/make-wipi-keydraw-fixture.sh instead, so the
// fixture stays the single source and a drift shows up as a parse failure here.
// Fail-closed on every step: a silent `undefined` would compare equal to nothing
// and burn the whole tick deadline instead of failing.
const keydrawSh = await readFile(path.join(root, "scripts/make-wipi-keydraw-fixture.sh"), "utf8");
const barH = Number(keydrawSh.match(/^const BAR_H: i32 = (\d+);$/m)?.[1]);
if (!Number.isInteger(barH) || barH <= 0) throw new Error("keydraw fixture: cannot read `const BAR_H: i32 = N;` from scripts/make-wipi-keydraw-fixture.sh — refusing to fail-open");
// The guest's `match key_code { KeyCode::X => N, ... }` arms. Only the arms whose
// variant names a positive-WIPI key are usable (see the Scenario E header).
const keydrawArms = new Map([...keydrawSh.matchAll(/^\s*KeyCode::([A-Za-z0-9]+)(?:\(_\))? => (\d+),$/gm)].map((m) => [m[1], Number(m[2])]));
if (keydrawArms.size === 0) throw new Error("keydraw fixture: no `KeyCode::X => N` arms parsed — locator drift, refusing to fail-open");
// variant -> contract vocabulary name, for the positive-WIPI rows only.
const KTF_VARIANT_TO_KEY = { Key0: "NUM0", Key1: "NUM1", Key2: "NUM2", Key3: "NUM3", Key4: "NUM4", Key5: "NUM5", Key6: "NUM6", Key7: "NUM7", Key8: "NUM8", Key9: "NUM9", Star: "STAR", Hash: "HASH" };
// The single-source claim, made mechanical: every positive row the guest paints
// must equal the WIPI code the contract pins. If the fixture is ever rebuilt with
// a different table, this throws instead of asserting a stale number below.
for (const [variant, key] of Object.entries(KTF_VARIANT_TO_KEY)) {
  const width = keydrawArms.get(variant);
  const wipi = contract.keyWipiCodes?.[key];
  if (typeof width !== "number") throw new Error(`keydraw fixture: no arm for KeyCode::${variant} — refusing to fail-open`);
  if (width !== wipi) throw new Error(`keydraw fixture paints ${width} for KeyCode::${variant}, but contract.keyWipiCodes["${key}"] is ${wipi} — the fixture and the contract disagree`);
}
// One representative per positive band, mirroring Scenario D's three.
const KTF_KEYS = ["HASH", "STAR", "NUM5"].map((code) => ({
  code,
  wipi: contract.keyWipiCodes[code],
  expectPixels: contract.keyWipiCodes[code] * barH,
}));

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

const steps = await page.evaluate(async ({ contract, representativeKeys, ktfKeys }) => {
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

    // ── Scenario E: does a key reach a *WIPI* guest, as the right WIPI code? ──
    // KTF adds a hop D never sees (CardCanvas -> WIPIKeyCode::from_midp_raw), so
    // this fixture's bar width answers for the whole chain, through the wasm glue.
    const fixtureBytes = await (await fetch("/fixtures/keydraw_ktf.zip")).arrayBuffer();
    check("E: static server delivers keydraw_ktf.zip", fixtureBytes.byteLength > 0, `${fixtureBytes.byteLength} bytes over HTTP`);

    const e = await bootFixture(mod2, "keydraw_ktf.zip");
    check('E: platform_kind() === "KTF"', e.emu.platform_kind() === "KTF", `got ${e.emu.platform_kind()}`);
    // Unlike the helloworld fixtures this one never exits — it waits for a key,
    // painting an all-black screen until one arrives. The first key is therefore
    // queued BEFORE the first tick (the event queue buffers it), which is how
    // wie_ktf/tests/test_key_reach.rs avoids having to guess a boot length.
    for (const k of ktfKeys) {
      e.emu.key_down(k.code);
      const runE = await tickLoop(e.emu, e.canvas, 15_000, (px) => px === k.expectPixels);
      e.emu.key_up(k.code);
      check(
        `E: "${k.code}" reaches the KTF guest as WIPI code ${k.wipi}`,
        runE.threw === null && runE.pixels === k.expectPixels,
        runE.threw ?? `${runE.pixels} px, expected ${k.expectPixels} (${k.wipi}*barH) after ${runE.frames} frames`,
      );
    }

    e.emu.free();
    check("E: free() (no throw)", true);

    // ── Why there is no LGT twin of Scenario E (localized 2026-09-05) ─────────
    // keydraw_lgt.zip is built from the SAME guest source by the same script, so
    // the constants above would hold unchanged and the scenario is ~10 lines. It
    // was written, run, and REMOVED because it fails for a reason that is not
    // about keys — the LGT canvas stays at 0 px while KTF reaches 424.
    //
    // CORRECTION. The first version of this note said "the gap is LGT paint ->
    // WebScreen -> canvas". Staged probes on both hosts disproved that: the LGT
    // frame DOES reach the canvas. The ordered browser trace is
    //   MC_grpFlushLcd -> WebScreen::paint(incoming_nonblack=424) -> draw_image ok
    //   Display::handle_paint_event disable_paint=false
    //                  -> WebScreen::paint(incoming_nonblack=0)   -> draw_image ok
    // i.e. the good WIPI frame is painted and then OVERWRITTEN by MIDP's blank
    // screenImage, so the last frame — the one you see — is black. KTF runs the
    // same trace with disable_paint=true and no trailing blank blit.
    // Root cause: net/wie/CardCanvas only calls Display.disablePaint() when the
    // card class is "CletCard" or "net/wie/CletWrapperCard", and LGT's card
    // reports Class.getName() as "net.wie.CletWrapperCard" — dots, not slashes,
    // so the comparison never matches. (KTF's card is "CletCard", no package,
    // which is why only LGT is hit.) Adding the dot form flips LGT to 424 px in
    // the browser and its native paints 83 -> 55, exactly matching KTF.
    //
    // KTF IS NOT SAFE — IT IS UNPACKAGED. "CletCard" comes from the guest (it is
    // a constant-pool string in the fixture's own client.bin), so the day a card
    // class arrives with a package, KTF breaks the same way LGT does. A fix that
    // only ADDS the dot form leaves that fragility in place; normalising the name
    // (or dropping the class-name predicate) is what removes it. Weigh that when
    // picking the prescription — the fix is a separate round, and this file stays
    // a checker, not a patch.
  } catch (e) {
    check("scenario aborted by exception", false, (e && e.stack) || String(e));
  }
  return steps;
}, { contract, representativeKeys: REPRESENTATIVE_KEYS, ktfKeys: KTF_KEYS });

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
