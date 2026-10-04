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
// Scenario C-img (same J2ME instance — RESOURCE-BY-NAME, ASSERTED):
//   startApp() opens a bundled PNG through Image.createImage(String) and paints
//   a rect of the dimensions the host reported, so the pixel count answers
//   "what did the widened class-loader visibility actually find" with a number.
//   This is the one call site of the six in the get_system_class_loader
//   migration whose behaviour CHANGED (it used to throw IOException every time);
//   before this fixture nothing in the repo executed either half.
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
//         wie_featurephone/src/lib.rs \
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
// Scenario E-res / F-res (same two instances — WIPI RESOURCE READ, ASSERTED):
//   The keydraw guest reads the bundled `res.bin` at boot and prints
//   `res:<size>:<byte-sum>`, and that one line distinguishes "both hops ran"
//   (MC_knlGetResourceID -> get_resource_size, MC_knlGetResource ->
//   read_resource) from "only the first did". wie_{ktf,lgt}/tests/
//   test_resource_reach.rs already assert it headlessly; these two checks add
//   the layers those tests cannot reach — the wasm build and the glue.
//
//   ── Why stdout here and pixels everywhere else (measured, not preference) ──
//   The guest paints nothing for the resource on purpose, so the key-pixel
//   assertions above stay readable; a pixel axis would need the resource drawn
//   on the same screen the key assertions own, i.e. two more fixture zips.
//   Reading stdout costs nothing instead: guest printf goes
//   MC_knlPrintk -> Platform::write_stdout, and wie_featurephone implements that as
//   `web_sys::console::log_1` (wie_featurephone/src/platform.rs), so the line is ALREADY
//   in the browser console. This file already listens (`consoleLog` below) and
//   only discarded it unless the run failed. No glue hook, no new zip, and
//   nothing added to docs/contracts/featurephone-engine-contract.json — that
//   pin has zero stdout/console entries, so Constraint 3 is untouched.
//
//   ── Why both carriers, when the resolved path is the same one ─────────────
//   For THIS fixture both carriers answer from the RustJava class loader
//   (in-memory jar) — measured, by mutation: killing only LGT's
//   System::filesystem() fallback left F-res green, because that branch is
//   never reached while the class loader finds res.bin. So the carriers are not
//   two host paths here; they are two WIPI-C shims over one resolver, and the
//   check is per-carrier because the shims are (KTF returns early on
//   `stream.is_none()`, LGT falls through to the filesystem).
//   That fallback is still the only place a host-divergent implementation sits
//   (wie_featurephone::WebFilesystem vs wie_cli::CliFilesystem), and NOTHING here covers
//   it — see the worklog for why that gap was left open rather than papered
//   over with a second fixture.
//
//   The expected numbers are DERIVED from the fixture's own recipe, never
//   restated — same rule as the Scenario E constants below.
//
// Scenario F (LGT keydraw fixture — the SAME question on LGT, ASSERTED):
//   E's twin, and the only net for a whole class of bug: it once failed at 0 px
//   while KTF reached 424, because MIDP overpainted the good WIPI frame with a
//   blank screenImage — the browser was black while the emulator worked. Neither
//   `cargo test` nor `wie_validate` sees that (wie_validate's saw_content is a
//   sticky ANY-frame predicate and never inspects the LAST frame). Root cause and
//   the 2026-09-06 fix are recorded at the scenario itself.
//
// Scenario G (resize fixture — `Screen::resize`, ASSERTED):
//   The one input in the repo that asks the host to change the screen size. Until
//   2026-09-17 nothing did: the engine's only live caller is
//   `wie-ktf/src/emulator.rs`, `if let Some((width, height)) = adf.display_size`,
//   and NEITHER committed KTF fixture declared `DisplaySize:` (both `__adf__`s were
//   AID/PID/MClass only — probed on the native host: 0 calls on the committed
//   fixture, 1 call after appending one line to a throwaway copy, so the zero was a
//   measurement and not a silent instrument). `scripts/make-resize-fixture.mjs`
//   makes that copy permanent — same guest jar, one extra ADF line — and this
//   scenario asserts the canvas actually moved to it.
//   Why that matters here rather than in `cargo test`: the call site swallows `Err`
//   into `tracing::warn!`, so a host whose resize fails boots on and every native
//   gate stays green. The browser is also where the failure is user-visible, and
//   `wie_validate`'s own `resize` is a no-op — running the fixture there asserts
//   nothing.
//   What it does NOT cover, deliberately: `WebScreen::resize` moves the visible
//   canvas AND the internal back buffer, and the back buffer is created inside the
//   wasm module, so JS cannot read its size. This asserts the visible canvas and
//   that the resized instance still ticks; a back buffer left behind would clip a
//   frame without throwing, and nothing here would see it.
//
// ── What NO scenario here reaches: `Platform::font()` ──
// Measured 2026-09-16, because a proposal asked for "boot the browser host once
// so the newly ported resize/font are verified on a real screen" and the honest
// answer was that RUNNING THIS SCRIPT DID NOT VERIFY THEM — the gap was the
// fixtures, not the host. Scenario G closed the resize half on 2026-09-17; the
// font half is still open, and the reason is unchanged:
//
//   `Screen::resize`'s OTHER caller stays uncovered — `wie_lgt/.../wipi_c/graphics.rs`,
//   whose 27-line wiring was cut when PR #161 routed LGT graphics back to the shared
//   implementation. Scenario G covers the KTF boot path only.
//
//   `Platform::font()` — the callers are NOT all MIDP, and that matters for how
//   you would cover this. Measured with `git grep -n "\.font()" -- '*.rs'`
//   (2026-09-16: 30 expressions — `wie-midp` 28, `wie-wipi-c` 2). The two are
//   `api/graphics.rs` (`MC_grpGetStringWidth`) and `api/graphics/primitives.rs`
//   (`draw_text`, the body of `MC_grpDrawString`), and BOTH are wired into the
//   very hosts these scenarios boot: `wie-ktf/.../wipi_c/method_table.rs` and
//   `wie-lgt/.../runtime/wipi_c.rs` both map `DrawString`/`GetStringWidth` onto
//   them. So a WIPI guest reaches `Platform::font()` with ONE `MC_grpDrawString`
//   call — no MIDP fixture required.
//   What keeps it at zero here is the fixtures, not the code paths: none of
//   these five draws a string. `wie_validate`'s own `font()` is
//   `unimplemented!()`, so it panics if anything reaches it; all five runner
//   fixtures pass, which prices that at zero calls without needing a probe.
//   => The cheapest cover is therefore one `MC_grpDrawString` line in the WIPI
//   keydraw fixture, whose generator (`scripts/make-wipi-keydraw-fixture.sh`)
//   this script ALREADY reads at startup and which is in `engine-contract.yml`'s
//   filter. Costed but deliberately NOT done here — it would stop this from
//   being a zero-code round; it is the next round's call.
//   What IS covered is the font's CONSTRUCTION: `WebPlatform::new` eagerly does
//   `Font::try_from_static(include_bytes!(...assets/neodgm.ttf))?`, so a missing
//   or unparseable asset fails boot and Scenario A goes red. Loading it is
//   verified; using it is not.
//
// Usage: node scripts/contract-roundtrip.mjs        (after scripts/build-wasm.sh)
//   WIE_CHROME_CHANNEL=chrome  — use a system Chrome instead of the playwright
//                                bundled chromium (local dev convenience).

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { BASE_RECT_PX, IMG_ERR_BROKEN, IMG_ERR_MISSING, IMG_H, IMG_RECT_PX, IMG_W, drawFixtureJar, keyBarPixels } from "./make-draw-fixture.mjs";
import { DRAW_RESIZE_H, DRAW_RESIZE_W, RESIZE_DRAW_FIXTURE, RESIZE_FIXTURE, RESIZE_H, RESIZE_W } from "./make-resize-fixture.mjs";
import { SOUND_MARK, soundFixtureJar } from "./make-sound-fixture.mjs";

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
// Scenario E-res/F-res's expected line, DERIVED from the recipe that writes the
// payload — the guest computes size and byte-sum from those same bytes, so the
// assertion pins the fixture's own arithmetic rather than a copied literal.
const resPayload = keydrawSh.match(/^printf '([^']*)' > "\$S\/wipi\/examples\/resources\/keydraw\/res\.bin"$/m)?.[1];
if (typeof resPayload !== "string" || resPayload.length === 0)
  throw new Error("keydraw fixture: cannot read the `printf '<payload>' > .../res.bin` line from scripts/make-wipi-keydraw-fixture.sh — refusing to fail-open");
const resBytes = Buffer.from(resPayload, "latin1");
const resLine = `res:${resBytes.length}:${resBytes.reduce((a, b) => a + b, 0)}`;

// Scenario C-img's numbers, DERIVED from the fixture's own exports (same rule as
// the Scenario E constants above: never restate a number the fixture owns).
// Crosses into the page context through page.evaluate's argument — the module
// import is Node-side only.
const IMG = { base: BASE_RECT_PX, px: IMG_RECT_PX, w: IMG_W, h: IMG_H, errMissing: IMG_ERR_MISSING, errBroken: IMG_ERR_BROKEN };

// Scenario G's numbers, DERIVED from the fixture's own generator — the same rule
// as the Scenario E constants above. `make-resize-fixture.mjs` writes the ADF
// line and exports the pair, so restating "176x220" here could drift from the
// bytes actually served.
const RESIZE = { fixture: RESIZE_FIXTURE, w: RESIZE_W, h: RESIZE_H, drawFixture: RESIZE_DRAW_FIXTURE, dw: DRAW_RESIZE_W, dh: DRAW_RESIZE_H };

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
  if (url.pathname === "/fixtures/sound_j2me.jar") {
    const jar = soundFixtureJar();
    res.writeHead(200, { "content-type": "application/java-archive", "content-length": jar.length });
    res.end(jar);
    return;
  }
  // Scenario S: the committed soundfont, served the way the shell will serve it. Any other name
  // under /soundfont/ is a 404 (S4's failure path).
  if (url.pathname === "/soundfont/GeneralUser.sf3" || url.pathname === "/soundfont/slow/GeneralUser.sf3") file = path.join(root, "wie-web/public/GeneralUser.sf3");
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
// Scenario S plays audio without a user gesture; the shell resumes its AudioContext on one.
const launchOpts = { headless: true, args: ["--autoplay-policy=no-user-gesture-required"] };
if (process.env.WIE_CHROME_CHANNEL) launchOpts.channel = process.env.WIE_CHROME_CHANNEL;
const browser = await chromium.launch(launchOpts);
const page = await browser.newPage();
const consoleLog = [];
page.on("console", (m) => consoleLog.push(`[console.${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => consoleLog.push(`[pageerror] ${e.message}`));
await page.goto(base + "/");
page.setDefaultTimeout(120_000);

const steps = await page.evaluate(async ({ contract, representativeKeys, ktfKeys, img, resLine, resize, soundMark }) => {
  const steps = [];
  // wie_featurephone 은 게스트 stdout 을 console.log 로 낸다(Platform::write_stdout ->
  // web_sys::console::log_1). 원 함수를 그대로 호출하므로 Node 쪽 진단 수집은
  // 영향받지 않는다 — 여기서는 «어느 시나리오 구간의» 줄인지 가르려고 기록한다.
  const guestOut = [];
  const realLog = console.log.bind(console);
  console.log = (...a) => {
    guestOut.push(a.map((x) => String(x)).join(" "));
    realLog(...a);
  };
  // ★한 줄이 «한 메시지»로 오지 않는다 — 실측(2026-09-06): 게스트의 `res:{}:{}` 한 줄이
  // console 에 `res:` / `9` / `:` / `602` / `\n` 다섯 메시지로 쪼개져 도착한다.
  // MC_knlPrintk 가 포맷 조각마다 Platform::write_stdout 을 부르고 wie_featurephone 이 그
  // 호출마다 console.log_1 을 내기 때문이다. 네이티브는 바이트 스트림이라 줄이 저절로
  // 이어지므로 ★이 쪼개짐은 «브라우저에만» 있다. ⇒ 메시지별이 아니라 «이어붙인» 버퍼에서 찾는다.
  const sawSince = (mark, needle) => guestOut.slice(mark).join("").includes(needle);
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

    // One RMS record seeded before boot — the "first-run marker" an LGT notice
    // title writes before it exits. B-relaunch below reads it back.
    const le32 = (n) => [n & 255, (n >> 8) & 255, (n >> 16) & 255, (n >>> 24) & 255];
    const lstr = (t) => [...le32(t.length), ...new TextEncoder().encode(t)];
    const seed = new Uint8Array([...new TextEncoder().encode(contract.saveMagic), ...le32(1), ...lstr("roundtrip"), ...lstr("marker"), ...le32(1), ...le32(1), ...le32(1), 7, ...le32(0)]);
    const b = await bootFixture(mod2, "helloworld_lgt.zip");
    check("B: import_saves(seeded DB record) → true", b.emu.import_saves(seed) === true);
    check('B: platform_kind() === "LGT"', b.emu.platform_kind() === "LGT", `got ${b.emu.platform_kind()}`);
    check('B: lgt_compile_model() === "clet"', b.emu.lgt_compile_model() === "clet", `got ${b.emu.lgt_compile_model()}`);
    const runB = await tickLoop(b.emu, b.canvas, 20_000);
    check("B: tick loop survives (no throw)", runB.threw === null, runB.threw ?? `${runB.frames} frames`);
    check("B: clean exit observed (has_exited() flips true)", b.emu.has_exited() === true, `${runB.frames} frames, ${runB.pixels} px (fixture draws nothing — pixels are info only)`);
    // ── B-relaunch: the shell's «다시 실행» after a clean exit (the LGT first-run
    // notice path, docs/report/0325) — persist the exited instance's saves, boot a
    // FRESH instance, import them. The DB must come through byte-identical, or the
    // relaunched title sees no marker and shows its notice forever.
    const afterExit = b.emu.export_saves();
    check("B: DB survives the clean exit (export == seed)", afterExit.length === seed.length && afterExit.every((v, i) => v === seed[i]), `${afterExit.length} vs ${seed.length} bytes`);
    b.emu.free();
    check("B: free() (no throw)", true);
    const b2 = await bootFixture(mod2, "helloworld_lgt.zip");
    check("B-relaunch: import_saves(exited blob) on a fresh instance → true", b2.emu.import_saves(afterExit) === true);
    const runB2 = await tickLoop(b2.emu, b2.canvas, 20_000);
    const relaunched = b2.emu.export_saves();
    check(
      "B-relaunch: relaunched instance runs to exit with the same DB",
      runB2.threw === null && b2.emu.has_exited() === true && relaunched.length === seed.length && relaunched.every((v, i) => v === seed[i]),
      runB2.threw ?? `${runB2.frames} frames, ${relaunched.length} save bytes`,
    );
    b2.emu.free();

    // ── Scenario C: J2ME draw fixture — canvas blit, ASSERTED not reported ───
    const markC = guestOut.length;
    const c = await bootFixture(mod2, "draw_j2me.jar");
    check('C: platform_kind() === "J2ME"', c.emu.platform_kind() === "J2ME", `got ${c.emu.platform_kind()}`);
    const runC = await tickLoop(c.emu, c.canvas, 30_000, (px) => px > 0);
    check("C: tick loop survives (no throw)", runC.threw === null, runC.threw ?? `${runC.frames} frames`);
    check("C: canvas blit ASSERTED — fixture's rect reaches the canvas", runC.pixels > 0, `${runC.pixels} non-black px after ${runC.frames} frames`);

    // ── Scenario C-img: does Image.createImage(String) RESOLVE? (ASSERTED) ────
    // The one site of the six get_system_class_loader call sites whose behaviour
    // actually changed. Through current_class_loader this call could never find
    // a guest resource and threw IOException every time; through the system
    // URLClassLoader it resolves. Nothing exercised either half before this —
    // a planted panic!() at image.rs left the suite and all five fixtures green.
    //
    // The guest fills a rect of the dimensions the host reported for the decoded
    // image, so the pixel count is the ANSWER, not a liveness signal: exactly
    // BASE + IMG_W*IMG_H means the name resolved AND decode_image produced those
    // dimensions. A failure on THIS name is LOUD, not a smaller number: the
    // assembler's exception table guards only the two failure-branch calls below,
    // never this one, so an unresolved /wie-img.png still aborts the boot —
    // measured 2026-09-06 as FAIL/paints 0 under wie_validate.
    const runCimg = await tickLoop(c.emu, c.canvas, 30_000, (px) => px === img.base + img.px);
    check(
      `C: Image.createImage(String) resolves the bundled resource — a ${img.w}x${img.h} image`,
      runCimg.threw === null && runCimg.pixels === img.base + img.px,
      runCimg.threw ??
        `${runCimg.pixels} px, expected ${img.base + img.px} (base ${img.base} + image ${img.px}) — ` +
          `this call is outside the exception table, so a resource that does not resolve aborts the boot: expect a throw or 0 px, not ${img.base}`,
    );

    // ── Scenario C-err: the FAILURE branches of the same call (ASSERTED) ─────
    // C-img locks "the host found it". These lock what the guest RECEIVES when it
    // does not — the half no fixture said anything about, so either branch could
    // change exception type or stop throwing and every check stayed green.
    // The markers come from narrow catches in the assembler (java/io/IOException
    // and java/lang/IllegalArgumentException, read from wie_midp/.../image.rs), so
    // a wrong type escapes the handler and aborts the boot instead of printing.
    // Split by PREFIX (`imgerr:`) on purpose: the existing assertions above match
    // on pixel COUNTS and the `res:` ones on their own prefix, so added output
    // cannot shift what any of them find.
    check(
      `C-err: absent resource name surfaces as java.io.IOException (${img.errMissing})`,
      sawSince(markC, img.errMissing),
      guestOut.slice(markC).join("") || "(no guest stdout after the C boot)",
    );
    check(
      `C-err: undecodable bytes surface as java.lang.IllegalArgumentException (${img.errBroken})`,
      sawSince(markC, img.errBroken),
      guestOut.slice(markC).join("") || "(no guest stdout after the C boot)",
    );

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

    const markE = guestOut.length;
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

    // E-res: 부팅 때 읽은 리소스가 «두 홉 다» 돌았는가 — 키 픽셀 단언과 같은 화면을 다투지 않는다.
    check(
      `E-res: KTF guest reads res.bin through the wasm build (${resLine})`,
      sawSince(markE, resLine),
      guestOut.slice(markE).filter((l) => l.startsWith("res:")).join(" | ") || "no res: line on the console at all",
    );

    e.emu.free();
    check("E: free() (no throw)", true);

    // ── Scenario F: the SAME question on LGT — and the black-screen regression ─
    // This scenario existed once and was REMOVED: it failed at 0 px while KTF
    // reached 424, for a reason that was not about keys at all. The ordered
    // browser trace was
    //   MC_grpFlushLcd -> WebScreen::paint(incoming_nonblack=424) -> draw_image ok
    //   Display::handle_paint_event disable_paint=false
    //                  -> WebScreen::paint(incoming_nonblack=0)   -> draw_image ok
    // i.e. the good WIPI frame was painted and then OVERWRITTEN by MIDP's blank
    // screenImage, so the last frame — the one you see — was black. Root cause:
    // net/wie/CardCanvas only calls Display.disablePaint() for the clet card, and
    // it compared Class.getName() (BINARY name, dots) against a slash literal, so
    // for LGT's `net.wie.CletWrapperCard` the branch had never run once.
    //
    // Fixed 2026-09-06 by normalising the name in `is_clet_card` rather than
    // adding the dot literal: KTF's `CletCard` passes only because it has no
    // package, so a packaged card class would have broken KTF the same way.
    //
    // KEEP THIS SCENARIO. It is the ONLY net for that class of bug — `cargo test`
    // and `wie_validate` both pass with the black screen present, because
    // wie_validate's `saw_content` is a sticky any-frame predicate and never looks
    // at the LAST frame, which is the one a user sees. This asserts pixels AFTER
    // the loop settles, so a re-introduced overpaint reads as 0 px here.
    const fixtureBytesF = await (await fetch("/fixtures/keydraw_lgt.zip")).arrayBuffer();
    check("F: static server delivers keydraw_lgt.zip", fixtureBytesF.byteLength > 0, `${fixtureBytesF.byteLength} bytes over HTTP`);

    const markF = guestOut.length;
    const f = await bootFixture(mod2, "keydraw_lgt.zip");
    check('F: platform_kind() === "LGT"', f.emu.platform_kind() === "LGT", `got ${f.emu.platform_kind()}`);
    // Same guest source as keydraw_ktf.zip, built by the same script, so the bar
    // widths above hold unchanged — that is why this reuses ktfKeys verbatim.
    for (const k of ktfKeys) {
      f.emu.key_down(k.code);
      const runF = await tickLoop(f.emu, f.canvas, 15_000, (px) => px === k.expectPixels);
      f.emu.key_up(k.code);
      check(
        `F: "${k.code}" reaches the LGT guest as WIPI code ${k.wipi} AND survives to the last frame`,
        runF.threw === null && runF.pixels === k.expectPixels,
        runF.threw ?? `${runF.pixels} px, expected ${k.expectPixels} (${k.wipi}*barH) after ${runF.frames} frames`,
      );
    }

    // F-res: 같은 질문을 LGT 에서 — ★이쪽은 호스트별 Platform::filesystem() 을 먼저 거친다.
    check(
      `F-res: LGT guest reads res.bin through the wasm build (${resLine})`,
      sawSince(markF, resLine),
      guestOut.slice(markF).filter((l) => l.startsWith("res:")).join(" | ") || "no res: line on the console at all",
    );

    f.emu.free();
    check("F: free() (no throw)", true);

    // ── Scenario G: does a `DisplaySize:` ADF actually resize the canvas? ─────
    // The boot size handed to the constructor is the contract's; the fixture asks
    // for a different one on BOTH axes, so a half-applied resize cannot read as a
    // pass. See this file's header for what this does and does not cover.
    const g = await bootFixture(mod, resize.fixture);
    check(
      `G: DisplaySize ADF resizes the canvas ${contract.screen.width}x${contract.screen.height} -> ${resize.w}x${resize.h}`,
      g.canvas.width === resize.w && g.canvas.height === resize.h,
      `${g.canvas.width}x${g.canvas.height}`,
    );
    // The resized instance must still run: `WebScreen::paint` sizes its ImageData
    // from the guest frame and blits the back buffer, so a surface pair left
    // inconsistent shows up as a throw here rather than as a wrong number above.
    const runG = await tickLoop(g.emu, g.canvas, 5_000);
    check("G: resized instance ticks without throwing", runG.threw === null, runG.threw ?? `${runG.frames} frames`);
    g.emu.free();
    check("G: free() (no throw)", true);

    // ── Scenario G2: did the BACK buffer resize too? ─────────────────────────
    // G above can only read `HTMLCanvasElement.width/height` — the front surface.
    // `WebScreen`'s back canvas is created by `document.create_element` inside
    // wasm and never attached to the DOM, so no selector reaches it, and a back
    // buffer left at the old size CLIPS the blit instead of throwing: both of G's
    // checks pass while the frame is silently cropped.
    //
    // This reads that miss off the FRONT canvas, using two properties of the pair:
    //   * the fixture GROWS (G's shrinks). On a shrink an oversized back canvas is
    //     clipped by the front one and the copied top-left region is exactly the
    //     frame — invisible by construction. Growing makes the region past the OLD
    //     size come from nothing.
    //   * `WebScreen::paint` forces alpha opaque across the whole guest frame,
    //     while a canvas is transparent black right after `set_width`. So the
    //     probe is ALPHA, not colour: it holds even where the guest draws nothing,
    //     which is why the sample point is a corner rather than the guest's rect.
    // Hence the source is the DRAWING guest — helloworld never paints, so nothing
    // would be blitted and both branches would read alpha 0.
    const g2 = await bootFixture(mod2, resize.drawFixture);
    check(
      `G2: drawing fixture resizes the canvas ${contract.screen.width}x${contract.screen.height} -> ${resize.dw}x${resize.dh}`,
      g2.canvas.width === resize.dw && g2.canvas.height === resize.dh,
      `${g2.canvas.width}x${g2.canvas.height}`,
    );
    // Make the guest paint at least once; it draws on key, like Scenario E.
    g2.emu.key_down(ktfKeys[0].code);
    const runG2 = await tickLoop(g2.emu, g2.canvas, 15_000, (px) => px > 0);
    g2.emu.key_up(ktfKeys[0].code);
    check("G2: resized drawing instance paints without throwing", runG2.threw === null && runG2.pixels > 0, runG2.threw ?? `${runG2.pixels} px after ${runG2.frames} frames`);
    // Sample PAST the old bounds on both axes. Opaque => the back canvas grew with
    // the front one. Transparent => `drawImage` copied an old-sized back buffer and
    // this corner was never written.
    const probe = g2.canvas.getContext("2d").getImageData(contract.screen.width + 1, contract.screen.height + 1, 1, 1).data;
    check(
      `G2: back buffer grew too — pixel past ${contract.screen.width}x${contract.screen.height} is opaque`,
      probe[3] !== 0,
      `rgba(${probe[0]},${probe[1]},${probe[2]},${probe[3]}) at (${contract.screen.width + 1},${contract.screen.height + 1})`,
    );
    g2.emu.free();
    check("G2: free() (no throw)", true);

    // ── Scenario S: the audio sink's soundfont path, end to end (docs/report 0355) ─────
    // wie_featurephone/src/audio.rs's soundfont state machine (Off / NotRequested / Requested /
    // Arrived / Prelude / Posted) is wasm-only: no cargo test and no node case reaches it, and it
    // broke once while all of those were green (the soundfont was never fetched). This drives it
    // in a browser with a guest that plays one SMAF note per key (make-sound-fixture.mjs) and the
    // committed soundfont, watching what the sink does from outside: the worklet modules it loads
    // (and whether each carries the prelude), when it fetches, what it posts, and the worklet's
    // own `stats`.
    //   S1  no URL        — one module, no prelude, no fetch, FM sounds
    //   S2  URL, file arrives while the worklet module is still loading (Arrived)
    //   S3  URL — fetched at boot; the first play renders through the soundfont, never FM
    //   S4  URL that 404s — warned, FM for every play, no prelude, no error
    //   S5  (in S3's run) an instrument not decoded yet — its first play waits, then is the soundfont
    //   S6  URL whose body takes longer than the worklet's HOLD_MAX_MS — FM for every play
    // S3, S5 and S6 are the session invariant of docs/report 0438: one synth per session, from the
    // first play of each song (until 2026-10-04 a song's first play was FM and its next the soundfont).
    const log = [];
    const t0 = performance.now();
    const at = () => performance.now() - t0;
    let holdModule = null; // S2: the next addModule resolves only once this promise does
    let bodyArrived = null;
    let slowBodyMs = 0; // S6: a /slow/ soundfont body resolves this much later
    const nativeAdd = AudioWorklet.prototype.addModule;
    AudioWorklet.prototype.addModule = async function (url, options) {
      const text = await (await nativeFetch(url)).text();
      const entry = { what: "module", at: at(), prelude: text.includes(contract.soundfontPrelude.marker), bytes: text.length };
      log.push(entry);
      await nativeAdd.call(this, url, options);
      entry.loadMs = at() - entry.at;
      if (holdModule) {
        const hold = holdModule;
        holdModule = null;
        await hold;
      }
      entry.readyAt = at();
    };
    const nativeFetch = window.fetch.bind(window);
    window.fetch = (input, init) => {
      const url = String(input && input.url ? input.url : input);
      if (url.includes("/soundfont/")) log.push({ what: "sf-fetch", at: at(), url });
      return nativeFetch(input, init);
    };
    const nativeBody = Response.prototype.arrayBuffer;
    Response.prototype.arrayBuffer = function () {
      let body = nativeBody.call(this);
      if (this.url.includes("/soundfont/slow/")) body = body.then((b) => new Promise((r) => setTimeout(() => r(b), slowBodyMs)));
      if (this.url.includes("/soundfont/"))
        body.then(() => {
          log.push({ what: "sf-body", at: at() });
          if (bodyArrived) bodyArrived();
        });
      return body;
    };
    const NativeNode = window.AudioWorkletNode;
    const nodes = [];
    window.AudioWorkletNode = class extends NativeNode {
      constructor(...args) {
        super(...args);
        log.push({ what: "node", at: at() });
        nodes.push(this);
        this.lastStats = null;
        this.port.addEventListener("message", (event) => {
          if (event.data && event.data.t === "stats") this.lastStats = event.data;
        });
        this.port.start();
        const post = this.port.postMessage.bind(this.port);
        this.rawPost = post;
        this.port.postMessage = (message, transfer) => {
          if (message && (message.t === "play" || message.t === "sf")) log.push({ what: `post-${message.t}`, at: at() });
          return transfer ? post(message, transfer) : post(message);
        };
      }
    };
    const warnOut = [];
    const errorOut = [];
    const realWarn = console.warn.bind(console);
    const realError = console.error.bind(console);
    console.warn = (...a) => {
      warnOut.push(a.map(String).join(" "));
      realWarn(...a);
    };
    console.error = (...a) => {
      errorOut.push(a.map(String).join(" "));
      realError(...a);
    };

    // Ticks the emulator per animation frame until `until()` holds or `ms` passes.
    const pump = async (emu, ms, until = () => false) => {
      const start = performance.now();
      while (performance.now() - start < ms) {
        emu.tick();
        if (until()) return true;
        await new Promise((r) => requestAnimationFrame(r));
      }
      return until();
    };
    const statsOf = async (emu, node) => {
      node.lastStats = null;
      node.rawPost({ t: "stats" });
      await pump(emu, 3000, () => node.lastStats !== null);
      return node.lastStats ?? {};
    };
    const levelOf = (analyser) => {
      const buf = new Float32Array(analyser.fftSize);
      analyser.getFloatTimeDomainData(buf);
      return Math.sqrt(buf.reduce((sum, v) => sum + v * v, 0) / buf.length);
    };
    // Polls the worklet until `want(stats, level)` holds (the play is handled on the audio thread a
    // quantum or more after it is posted, and later under load) — returns the last reading either way.
    // Level FIRST, then stats: a soundfont synth is allocated when the play message lands but its
    // notes (and any FM double render) only on the next quantum, so stats read before the level can
    // say `synths 1 · voices 0` for a play that is about to render through both (measured: the
    // double-render mutation passed 1 run in 2 with the order reversed).
    const sounding = async (run, want) => {
      let stats = {};
      let level = 0;
      for (let i = 0; i < 20; i++) {
        level = levelOf(run.analyser);
        stats = await statsOf(run.emu, run.node());
        if (want(stats, level)) break;
        await pump(run.emu, 100);
      }
      return { stats, level };
    };
    // Polls until the play renders through a soundfont synth and is audible, recording the most FM
    // voices seen on the way — 0 means no FM note sounded at any poll of this play.
    const sfOnly = async (run) => {
      let stats = {};
      let level = 0;
      let fmMax = 0;
      let heldFirst = null;
      const started = performance.now();
      for (let i = 0; i < 80; i++) {
        level = levelOf(run.analyser);
        stats = await statsOf(run.emu, run.node());
        heldFirst ??= stats.held;
        fmMax = Math.max(fmMax, stats.voices);
        if (stats.synths >= 1 && level > 1e-3) break;
        await pump(run.emu, 100);
      }
      return { stats, level, fmMax, heldFirst, waitMs: performance.now() - started };
    };
    const since = (mark) => log.slice(mark);
    const first = (entries, what) => entries.find((e) => e.what === what);
    const count = (entries, what) => entries.filter((e) => e.what === what).length;
    const ms = (x) => (x === undefined ? "-" : `${Math.round(x)} ms`);
    // One guest key = one new Play; resolves once the guest printed its mark and the sink posted it.
    const press = async (run) => {
      const marks = guestOut.join("").split(soundMark).length;
      const plays = count(log, "post-play");
      run.emu.key_down("NUM5");
      run.emu.key_up("NUM5");
      const played = await pump(run.emu, 20_000, () => guestOut.join("").split(soundMark).length > marks && count(log, "post-play") > plays);
      return played;
    };
    const boot = async (soundfontUrl) => {
      const mark = log.length;
      const ctx = new AudioContext();
      await ctx.resume();
      const gain = ctx.createGain();
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 8192;
      gain.connect(analyser);
      analyser.connect(ctx.destination);
      const bytes = new Uint8Array(await (await nativeFetch("/fixtures/sound_j2me.jar")).arrayBuffer());
      const canvas = document.createElement("canvas");
      document.body.appendChild(canvas);
      const emu = new mod2.WieEmulator("sound_j2me.jar", bytes, canvas, ctx, gain, contract.screen.width, contract.screen.height, soundfontUrl);
      return { emu, ctx, analyser, mark, outMark: guestOut.length, node: () => nodes[nodes.length - 1] };
    };
    // The FM note of an earlier play (5 s) must have ended before a soundfont play is judged by
    // `voices === 0` — otherwise that term reads the old note, not a double render.
    const fmSilent = async (run) => {
      for (let i = 0; i < 40; i++) {
        if ((await statsOf(run.emu, run.node())).voices === 0) return true;
        await pump(run.emu, 300);
      }
      return false;
    };
    const soundfontReady = (run) => pump(run.emu, 60_000, () => guestOut.slice(run.outMark).join("").includes("[wie] soundfont ready") && count(since(run.mark), "post-sf") === 1);
    const close = async (run) => {
      run.emu.free();
      await run.ctx.close();
    };

    // S1 — no URL: exactly the FM sink. One module, prelude-free; no fetch ever; the note sounds.
    {
      const run = await boot(undefined);
      await pump(run.emu, 15_000, () => count(since(run.mark), "node") === 1);
      const played = await press(run);
      const { stats, level } = await sounding(run, (st, lv) => st.voices > 0 && lv > 1e-3);
      await pump(run.emu, 1500); // room for a fetch that should not happen
      const seen = since(run.mark);
      const modules = seen.filter((e) => e.what === "module");
      check("S1: no URL — the key plays a note through the FM synth, and it is audible", played && stats.voices > 0 && stats.soundfont === "none" && level > 1e-3, `voices ${stats.voices} · soundfont ${stats.soundfont} · output rms ${level.toFixed(4)}`);
      check("S1: no URL — one worklet module, without the prelude, and no soundfont fetch", modules.length === 1 && !modules[0].prelude && count(seen, "sf-fetch") === 0, `modules ${modules.map((m) => `${m.bytes}B prelude=${m.prelude} load ${ms(m.loadMs)}`).join(", ")} · fetches ${count(seen, "sf-fetch")}`);
      await close(run);
    }

    // S3 — URL: the soundfont fetch starts when the engine boots, before any play (until 2026-10-04 it
    // waited for the first play, so that play was FM). The worklet module is still loaded alone; the
    // prelude follows once the file is in. The first play — pressed as soon as the node exists, so
    // it may well arrive before the soundfont has parsed — waits for it and renders through it only.
    {
      const run = await boot("/soundfont/GeneralUser.sf3");
      await pump(run.emu, 15_000, () => count(since(run.mark), "node") === 1);
      const idle = since(run.mark);
      const firstModule = first(idle, "module");
      check(
        "S3: URL — the worklet module is loaded ALONE (no prelude), and the soundfont fetch starts at boot, before any play",
        firstModule && !firstModule.prelude && count(idle, "sf-fetch") === 1 && count(idle, "post-play") === 0,
        `module ${firstModule ? `${firstModule.bytes}B prelude=${firstModule.prelude} load ${ms(firstModule.loadMs)}` : "none"} · fetch ${ms(first(idle, "sf-fetch")?.at)} · plays before it ${count(idle, "post-play")}`,
      );
      const played = await press(run);
      const sf1 = await sfOnly(run);
      const ready = await soundfontReady(run);
      const seen = since(run.mark);
      const bodyAt = first(seen, "sf-body")?.at;
      const prelude = seen.filter((e) => e.what === "module")[1];
      check(
        "S3: the first play renders through the soundfont only — no FM note at any point — and the prelude loaded as a second module",
        played && ready && sf1.fmMax === 0 && sf1.stats.synths >= 1 && sf1.level > 1e-3 && prelude?.prelude === true && prelude.at >= bodyAt && count(seen, "post-sf") === 1,
        `held at first look ${sf1.heldFirst} · soundfont after ${ms(sf1.waitMs)} · synths ${sf1.stats.synths} · FM voices seen ${sf1.fmMax} · rms ${sf1.level.toFixed(4)} · body ${ms(bodyAt)} · prelude module ${ms(prelude?.at)}`,
      );
      const quiet = await fmSilent(run);
      const playedSf = await press(run);
      const { stats, level } = await sounding(run, (st, lv) => st.synths >= 1 && lv > 1e-3);
      check(
        "S3: the next play renders through the soundfont only (synth 1+, FM voices 0), and it is audible",
        quiet && playedSf && stats.soundfont === "ready" && stats.synths >= 1 && stats.voices === 0 && level > 1e-3,
        `soundfont ${stats.soundfont} · synths ${stats.synths} · FM voices ${stats.voices} · output rms ${level.toFixed(4)}`,
      );

      // S5 — a play whose instrument the soundfont has not decoded yet (strings: the guest never
      // used it) waits — held, silent, no FM — while its samples are decoded as queued work on the
      // audio thread, never inside the play (docs/report 0359: a whole instrument decoded in the play
      // held an Android emulator's audio thread 156 ms), then renders through the soundfont only.
      // Posted straight to the node — the guest has one sound — on a handle audio.rs never allocates.
      const node = run.node();
      const H = 0x7fff0000;
      const strings = [[0, 0, new Uint8Array([0xc0, 48])], [0, 0, new Uint8Array([0x90, 64, 110])], [700, 0, new Uint8Array([0x80, 64, 0])]];
      const idleAgain = async () => {
        for (let i = 0; i < 60; i++) {
          const st = await statsOf(run.emu, node);
          if (st.voices === 0 && st.synths === 0 && st.work === 0) return true;
          await pump(run.emu, 200);
        }
        return false;
      };
      const quiet5 = await idleAgain();
      node.rawPost({ t: "play", h: H, r: false, d: 800, ev: strings });
      const sf5 = await sfOnly(run);
      check(
        "S5: an instrument not decoded yet — its first play waits for its samples (no FM note), then renders through the soundfont only",
        quiet5 && sf5.heldFirst === 1 && sf5.fmMax === 0 && sf5.stats.synths >= 1 && sf5.level > 1e-3,
        `idle before ${quiet5} · held at first look ${sf5.heldFirst} · soundfont after ${ms(sf5.waitMs)} · synths ${sf5.stats.synths} · FM voices seen ${sf5.fmMax} · output rms ${sf5.level.toFixed(4)}`,
      );
      await close(run);
    }

    // S2 — URL, the file arrives while the worklet module is still loading: the first module's
    // resolution is held until the soundfont body is in, so the sink sees Arrived before its node.
    {
      const arrived = new Promise((resolve) => (bodyArrived = resolve));
      holdModule = Promise.race([arrived, new Promise((r) => setTimeout(r, 30_000))]);
      const run = await boot("/soundfont/GeneralUser.sf3");
      // Key until the guest plays (its canvas may not be up on the first frames). The plays queue:
      // the held module has not resolved, so there is no node yet.
      let queued = false;
      for (let i = 0; i < 40 && !queued; i++) {
        run.emu.key_down("NUM5");
        run.emu.key_up("NUM5");
        queued = await pump(run.emu, 500, () => guestOut.slice(run.outMark).join("").includes(soundMark));
      }
      const nodeBeforePlay = count(since(run.mark), "node");
      const ready = await soundfontReady(run);
      bodyArrived = null;
      const seen = since(run.mark);
      const bodyAt = first(seen, "sf-body")?.at;
      const nodeAt = first(seen, "node")?.at;
      // Order by log position, not timestamp: the held module resolves on the body, so the node
      // follows it within the same millisecond and `bodyAt < nodeAt` failed on a tie (#454's CI).
      const bodyIdx = seen.findIndex((e) => e.what === "sf-body");
      const bodyFirst = bodyIdx >= 0 && bodyIdx < seen.findIndex((e) => e.what === "node");
      const prelude = seen.filter((e) => e.what === "module")[1];
      check(
        "S2: a soundfont that arrives before the worklet is ready is kept, then loaded and posted once it is",
        queued && nodeBeforePlay === 0 && ready && bodyFirst && prelude?.prelude === true && prelude.at >= nodeAt && count(seen, "post-sf") === 1,
        `play queued before the node ${queued && nodeBeforePlay === 0} · body ${ms(bodyAt)} · node ${ms(nodeAt)} · prelude module ${ms(prelude?.at)} · ready ${ready}`,
      );
      const quiet = await fmSilent(run);
      const playedSf = await press(run);
      // `level` too: the synth is allocated when the play message lands, its notes only on the next
      // quantum — `synths >= 1` alone can be read before anything (FM included) was dispatched.
      const { stats, level } = await sounding(run, (st, lv) => st.synths >= 1 && lv > 1e-3);
      check("S2: the next play renders through the soundfont only", quiet && playedSf && stats.synths >= 1 && stats.voices === 0 && level > 1e-3, `synths ${stats.synths} · FM voices ${stats.voices} · output rms ${level.toFixed(4)}`);
      await close(run);
    }

    // S4 — URL that 404s: a warning, FM, no prelude module, and nothing on console.error. The worklet
    // reads `failed` when the 404 lands after it told it to wait (`sfoff`), `none` when before.
    {
      const errorsBefore = errorOut.length;
      const run = await boot("/soundfont/missing.sf3");
      await pump(run.emu, 15_000, () => count(since(run.mark), "node") === 1);
      await press(run);
      await pump(run.emu, 5000, () => warnOut.some((w) => w.includes("[wie] soundfont") && w.includes("404")));
      await press(run);
      const { stats } = await sounding(run, (st) => st.voices > 0);
      const seen = since(run.mark);
      const warning = warnOut.find((w) => w.includes("[wie] soundfont") && w.includes("404"));
      check(
        "S4: a soundfont that 404s is warned about and FM keeps playing — no prelude, no console.error",
        warning && stats.voices > 0 && stats.synths === 0 && stats.held === 0 && stats.soundfont !== "ready" && count(seen, "module") === 1 && errorOut.length === errorsBefore,
        `${warning ?? "no warning"} · FM voices ${stats.voices} · synths ${stats.synths} · held ${stats.held} · worklet ${stats.soundfont} · modules ${count(seen, "module")} · console.error ${errorOut.length - errorsBefore}`,
      );
      await close(run);
    }

    // S6 — URL whose body takes longer than HOLD_MAX_MS (3 s) after the first play: that play waits,
    // then starts on FM, and the session stays FM — the soundfont arriving afterwards is refused.
    // A slow first visit gets the pre-soundfont sound throughout, never a song that changes synth.
    {
      const errorsBefore = errorOut.length;
      slowBodyMs = 8000;
      const run = await boot("/soundfont/slow/GeneralUser.sf3");
      await pump(run.emu, 15_000, () => count(since(run.mark), "node") === 1);
      const played = await press(run);
      const held = await statsOf(run.emu, run.node());
      // Up to 8 s: the hold alone is 3 s, and the guest's note is short — poll, do not sleep past it.
      const fm = { stats: {} };
      for (const t = performance.now(); performance.now() - t < 8000; ) {
        fm.stats = await statsOf(run.emu, run.node());
        if (fm.stats.voices > 0 || fm.stats.synths > 0) break;
        await pump(run.emu, 50);
      }
      const refused = await pump(run.emu, 20_000, () => warnOut.some((w) => w.includes("[wie] soundfont") && w.includes("already failed")));
      await fmSilent(run);
      const playedAgain = await press(run);
      const { stats, level } = await sounding(run, (st, lv) => st.voices > 0 && lv > 1e-3);
      slowBodyMs = 0;
      check(
        "S6: a soundfont later than HOLD_MAX_MS — the waiting play starts on FM, the late file is refused, and the next play is FM too",
        played && held.held === 1 && fm.stats.voices > 0 && fm.stats.synths === 0 && refused && playedAgain && stats.voices > 0 && stats.synths === 0 && stats.soundfont === "failed" && level > 1e-3 && errorOut.length === errorsBefore,
        `held ${held.held} (${held.soundfont}) · then FM voices ${fm.stats.voices} synths ${fm.stats.synths} · late file refused ${refused} · next play FM voices ${stats.voices} synths ${stats.synths} (${stats.soundfont}) · console.error ${errorOut.length - errorsBefore}`,
      );
      await close(run);
    }
  } catch (e) {
    check("scenario aborted by exception", false, (e && e.stack) || String(e));
  }
  return steps;
}, { contract, representativeKeys: REPRESENTATIVE_KEYS, ktfKeys: KTF_KEYS, img: IMG, resLine, resize: RESIZE, soundMark: SOUND_MARK });

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
