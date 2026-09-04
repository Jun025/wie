// Static engine↔featurephone contract check (node-only, no browser, no deps).
//
// The otterpebble featurephone shell consumes the wie engine as a prebuilt
// artifact pair (wie_web.js glue + wie_web_bg.wasm) and calls a pinned surface
// on it (docs/contracts/featurephone-engine-contract.json). This script fails
// CI when an engine change drifts from that surface, so a break is caught HERE
// — before the artifact is published and propagated to the web shell.
//
// What this catches (static): artifact naming, glue export set, WieEmulator
// method set, constructor arity, key vocabulary (source pin), save magic
// (source pin), publish-workflow dispatch payload keys.
// What it cannot catch: runtime behavior — scripts/contract-roundtrip.mjs
// covers the behavioral slice in a real browser.
//
// Usage: node scripts/check-engine-contract.mjs   (after scripts/build-wasm.sh)

import { readFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(await readFile(path.join(root, "docs/contracts/featurephone-engine-contract.json"), "utf8"));

const violations = [];
const passes = [];
const ok = (msg) => passes.push(msg);
const bad = (msg) => violations.push(msg);

// ── 1. Artifact pair exists under the pinned names ───────────────────────────
const wasmDir = path.join(root, contract.artifacts.dir);
for (const f of contract.artifacts.files) {
  try {
    await access(path.join(wasmDir, f));
    ok(`artifact exists: ${contract.artifacts.dir}/${f}`);
  } catch {
    bad(`artifact missing: ${contract.artifacts.dir}/${f} — run scripts/build-wasm.sh first, or the build output names drifted`);
  }
}

// ── 2. Glue module surface (import the real build output in Node) ────────────
let glue = null;
try {
  glue = await import(pathToFileURL(path.join(wasmDir, "wie_web.js")).href);
  ok("glue is a Node-importable ES module");
} catch (e) {
  bad(`glue failed to import as an ES module: ${e.message}`);
}
if (glue) {
  if (typeof glue.default === "function") ok("glue export: default (init) is a function");
  else bad("glue export drift: `default` (wasm-bindgen __wbg_init) missing or not a function");
  if (typeof glue.init === "function") ok("glue export: init (panic hook) is a function");
  else bad("glue export drift: `init` missing or not a function");
  if (typeof glue.WieEmulator === "function") {
    ok("glue export: WieEmulator class present");
    const arity = glue.WieEmulator.length;
    if (arity === contract.constructorArity) ok(`WieEmulator constructor arity = ${arity}`);
    else bad(`WieEmulator constructor arity drift: expected ${contract.constructorArity} (${contract.constructorShape}), got ${arity}`);
    for (const m of contract.methods) {
      if (typeof glue.WieEmulator.prototype[m] === "function") ok(`WieEmulator.${m}() present`);
      else bad(`WieEmulator method drift: ${m}() missing — featurephone lib/engine.ts calls this`);
    }
  } else {
    bad("glue export drift: `WieEmulator` class missing");
  }
}

// ── 3. wasm artifact is a valid module ───────────────────────────────────────
try {
  const bytes = await readFile(path.join(wasmDir, "wie_web_bg.wasm"));
  await WebAssembly.compile(bytes);
  ok(`wie_web_bg.wasm compiles as WebAssembly (${bytes.length} bytes)`);
} catch (e) {
  bad(`wie_web_bg.wasm is not a compilable wasm module: ${e.message}`);
}

// ── 4. Source pins that the JS surface cannot reveal ─────────────────────────
// key_down("NUM5") with an unmapped code is a SILENT no-op (parse_key → None),
// so vocabulary loss is unobservable from JS — pin it at the source level.
const libRs = await readFile(path.join(root, "wie_web/src/lib.rs"), "utf8");
// Scope to the parse_key fn body so stray string matches elsewhere in the file
// can't satisfy (or confuse) the check. Fail-closed: if the fn can't be located
// or an arm can't be parsed, that is a violation — never a silent pass.
const parseKeyStart = libRs.indexOf("fn parse_key(");
const parseKeyEnd = parseKeyStart === -1 ? -1 : libRs.indexOf("\n}", parseKeyStart);
if (parseKeyStart === -1 || parseKeyEnd === -1) {
  bad("key mapping unverifiable: `fn parse_key(` not found (or unterminated) in wie_web/src/lib.rs — refusing to fail-open; fix the checker's locator if the fn moved");
} else {
  const parseKeyBody = libRs.slice(parseKeyStart, parseKeyEnd);
  for (const key of contract.keyVocabulary) {
    // Pair check: the arm's RIGHT side must be the same-named KeyCode variant
    // ("UP" => KeyCode::UP). A left-side-only check would pass a miswired
    // "UP" => KeyCode::DOWN.
    const arms = [...parseKeyBody.matchAll(new RegExp(`"${key}"\\s*=>\\s*([A-Za-z0-9_:]+)`, "g"))];
    if (arms.length === 0) bad(`key vocabulary drift: parse_key no longer maps "${key}" — featurephone KEY_MAP sends this code`);
    else if (arms.length > 1) bad(`key mapping unverifiable: "${key}" has ${arms.length} match arms in parse_key — refusing to fail-open`);
    else if (arms[0][1] === `KeyCode::${key}`) ok(`parse_key maps "${key}" => KeyCode::${key}`);
    else bad(`key mapping miswired: parse_key maps "${key}" => ${arms[0][1]}, expected KeyCode::${key}`);
  }
}

// ── 4b. Second hop: KeyCode -> the MIDP int the GUEST is handed ──────────────
// parse_key above is only half the wiring. The number the guest actually sees
// comes from MIDPKeyCode::from_key_code, and nothing pinned it: a swapped row
// there (NUM7 -> 56) makes pressing 7 type 8, which the JS surface cannot see
// and 4's pair check does not look at. contract-roundtrip.mjs's Scenario D
// proves it behaviorally for 3 representative keys only — the delivery PATH is
// key-agnostic so 3 witnesses suffice for it, but the TABLE is per-key, so the
// remaining 17 rows are pinned here. Resolved in two steps (arm -> variant ->
// discriminant) rather than by name, because the names differ on purpose:
// OK => FIRE, NUM0 => KEY_NUM0, HASH => KEY_POUND.
const eventQueueRs = "wie_midp/src/classes/net/wie/event_queue.rs";
const eqRs = await readFile(path.join(root, eventQueueRs), "utf8");
const enumStart = eqRs.indexOf("pub enum MIDPKeyCode");
const enumEnd = enumStart === -1 ? -1 : eqRs.indexOf("\n}", enumStart);
const fromKeyStart = eqRs.indexOf("fn from_key_code(");
const fromKeyEnd = fromKeyStart === -1 ? -1 : eqRs.indexOf("\n    }", fromKeyStart);
if (enumStart === -1 || enumEnd === -1 || fromKeyStart === -1 || fromKeyEnd === -1) {
  bad(`guest key codes unverifiable: \`pub enum MIDPKeyCode\` or \`fn from_key_code(\` not found (or unterminated) in ${eventQueueRs} — refusing to fail-open; fix the checker's locator if it moved`);
} else {
  const discriminants = new Map(
    [...eqRs.slice(enumStart, enumEnd).matchAll(/^\s*([A-Z][A-Z0-9_]*)\s*=\s*(-?\d+)\s*,/gm)].map((m) => [m[1], Number(m[2])]),
  );
  const fromKeyBody = eqRs.slice(fromKeyStart, fromKeyEnd);
  for (const [key, expected] of Object.entries(contract.keyMidpCodes)) {
    const arms = [...fromKeyBody.matchAll(new RegExp(`KeyCode::${key}\\s*=>\\s*Self::([A-Za-z0-9_]+)`, "g"))];
    if (arms.length === 0) bad(`guest key code drift: from_key_code no longer maps KeyCode::${key} — that key would stop reaching the guest`);
    else if (arms.length > 1) bad(`guest key code unverifiable: KeyCode::${key} has ${arms.length} match arms in from_key_code — refusing to fail-open`);
    else if (!discriminants.has(arms[0][1])) bad(`guest key code unverifiable: from_key_code maps KeyCode::${key} => Self::${arms[0][1]}, which has no literal discriminant in enum MIDPKeyCode`);
    else if (discriminants.get(arms[0][1]) === expected) ok(`from_key_code hands the guest ${expected} for "${key}" (Self::${arms[0][1]})`);
    else bad(`guest key code miswired: "${key}" now reaches the guest as ${discriminants.get(arms[0][1])} (Self::${arms[0][1]}), contract pins ${expected} — pressing this key would input a different one`);
  }
  for (const key of contract.keyVocabulary) {
    if (!(key in contract.keyMidpCodes)) bad(`contract gap: keyVocabulary lists "${key}" but keyMidpCodes does not pin its guest-visible code`);
  }
}

// ── 4c. Third hop, KTF only: the MIDP int -> the WIPI int the guest is handed ─
// net.wie.CardCanvas overrides Canvas.keyPressed and converts once more through
// WIPIKeyCode::from_midp_raw before handing the code to the guest's
// Card.keyNotify. Nothing pinned that table either (measured: zero checkers,
// zero tests, zero contract entries), and a swapped row there means pressing 5
// types 8 — WIPI numerics are ASCII too, so NUM5's 53 goes out as NUM8's 56.
// Same shape as §4b on purpose: contract value vs. the product's own literals,
// resolved arm -> variant -> discriminant. This block re-reads MIDPKeyCode
// itself rather than reusing §4b's parse, so the two are independent.
const cardCanvasRs = "wie_wipi_java/src/classes/net/wie/card_canvas.rs";
const ccRs = await readFile(path.join(root, cardCanvasRs), "utf8");
const wipiEnumStart = ccRs.indexOf("pub enum WIPIKeyCode");
const wipiEnumEnd = wipiEnumStart === -1 ? -1 : ccRs.indexOf("\n}", wipiEnumStart);
const fromMidpStart = ccRs.indexOf("pub fn from_midp_raw(");
const fromMidpEnd = fromMidpStart === -1 ? -1 : ccRs.indexOf("\n    }", fromMidpStart);
// enumStart/enumEnd are §4b's locators for MIDPKeyCode; re-derived here so a
// failure over there cannot silently turn this block into a no-op.
const midpEnumStart = eqRs.indexOf("pub enum MIDPKeyCode");
const midpEnumEnd = midpEnumStart === -1 ? -1 : eqRs.indexOf("\n}", midpEnumStart);
if (wipiEnumStart === -1 || wipiEnumEnd === -1 || fromMidpStart === -1 || fromMidpEnd === -1 || midpEnumStart === -1 || midpEnumEnd === -1) {
  bad(`WIPI key codes unverifiable: \`pub enum WIPIKeyCode\` or \`pub fn from_midp_raw(\` not found (or unterminated) in ${cardCanvasRs} — refusing to fail-open; fix the checker's locator if it moved`);
} else {
  const arm = /^\s*([A-Z][A-Z0-9_]*)\s*=\s*(-?\d+)\s*,/gm;
  const wipiDisc = new Map([...ccRs.slice(wipiEnumStart, wipiEnumEnd).matchAll(arm)].map((m) => [m[1], Number(m[2])]));
  // MIDP int -> variant name, because from_midp_raw's arms are written with the
  // variant, while the contract pins the int (keyMidpCodes).
  const midpByCode = new Map([...eqRs.slice(midpEnumStart, midpEnumEnd).matchAll(arm)].map((m) => [Number(m[2]), m[1]]));
  const fromMidpBody = ccRs.slice(fromMidpStart, fromMidpEnd);
  for (const [key, expected] of Object.entries(contract.keyWipiCodes)) {
    const midpVariant = midpByCode.get(contract.keyMidpCodes?.[key]);
    if (midpVariant === undefined) {
      bad(`WIPI key code unverifiable: "${key}" has no MIDPKeyCode variant carrying its pinned code ${contract.keyMidpCodes?.[key]} — the KTF hop is keyed off that variant`);
      continue;
    }
    const arms = [...fromMidpBody.matchAll(new RegExp(`Some\\(MIDPKeyCode::${midpVariant}\\)\\s*=>\\s*Self::([A-Za-z0-9_]+)`, "g"))];
    if (arms.length === 0) bad(`WIPI key code drift: from_midp_raw no longer maps MIDPKeyCode::${midpVariant} ("${key}") — the KTF guest would receive the raw MIDP code instead (the None arm falls through)`);
    else if (arms.length > 1) bad(`WIPI key code unverifiable: MIDPKeyCode::${midpVariant} ("${key}") has ${arms.length} match arms in from_midp_raw — refusing to fail-open`);
    else if (!wipiDisc.has(arms[0][1])) bad(`WIPI key code unverifiable: from_midp_raw maps "${key}" to Self::${arms[0][1]}, which has no literal discriminant in enum WIPIKeyCode`);
    else if (wipiDisc.get(arms[0][1]) === expected) ok(`from_midp_raw hands the KTF guest ${expected} for "${key}" (Self::${arms[0][1]})`);
    else bad(`WIPI key code miswired: "${key}" now reaches the KTF guest as ${wipiDisc.get(arms[0][1])} (Self::${arms[0][1]}), contract pins ${expected} — pressing this key would input a different one`);
  }
  for (const key of contract.keyVocabulary) {
    if (!(key in contract.keyWipiCodes)) bad(`contract gap: keyVocabulary lists "${key}" but keyWipiCodes does not pin its KTF-visible code`);
  }
}

// ── 4d. The other direction: the guest asks "was that key UP?" ──────────────
// getGameAction is not on the delivery path (the guest calls it, on a code it
// already holds), so §4/§4b/§4c never look at it — yet a swapped row there is
// "you pressed up and walked down". Two tables answer that question, one per
// platform, and they agree on all five shared actions, so the numbers live ONCE
// in the contract (gameActions) and both tables are checked against them.
// The two places they diverge are pinned separately as `extra`/`fallback`,
// because each is that platform's own rule — see gameActionTablesNote.
for (const [platform, spec] of Object.entries(contract.gameActionTables)) {
  const src = await readFile(path.join(root, spec.file), "utf8");
  const fnStart = src.indexOf("async fn get_game_action(");
  const fnEnd = fnStart === -1 ? -1 : src.indexOf("\n    }", fnStart);
  if (fnStart === -1 || fnEnd === -1) {
    bad(`game action table unverifiable: \`async fn get_game_action(\` not found (or unterminated) in ${spec.file} — refusing to fail-open; fix the checker's locator if it moved`);
    continue;
  }
  if (src.indexOf("async fn get_game_action(", fnStart + 1) !== -1) {
    bad(`game action table unverifiable: ${spec.file} defines \`get_game_action\` more than once — refusing to fail-open`);
    continue;
  }
  const body = src.slice(fnStart, fnEnd);
  const arms = new Map(
    [...body.matchAll(new RegExp(`Some\\(${spec.keyEnum}::([A-Z][A-Z0-9_]*)\\)\\s*=>\\s*(-?\\d+)\\s*,`, "g"))].map((m) => [m[1], Number(m[2])]),
  );
  const expected = { ...contract.gameActions, ...spec.extra };
  for (const [variant, want] of Object.entries(expected)) {
    if (!arms.has(variant)) bad(`game action drift (${platform}, ${spec.fn}): ${spec.keyEnum}::${variant} no longer returns an action — the guest would read it as "not a game key"`);
    else if (arms.get(variant) === want) ok(`${spec.fn} maps ${spec.keyEnum}::${variant} to game action ${want}`);
    else bad(`game action miswired (${platform}, ${spec.fn}): ${spec.keyEnum}::${variant} returns ${arms.get(variant)}, contract pins ${want} — the guest would move the wrong way`);
  }
  for (const variant of arms.keys()) {
    if (!(variant in expected)) bad(`game action drift (${platform}, ${spec.fn}): ${spec.keyEnum}::${variant} returns ${arms.get(variant)} but the contract pins no action for it — add it to gameActions (shared) or this table's extra`);
  }
  // The wildcard arm is half the contract: MIDP owes 0, WIPI owes the key back.
  const fallback = body.match(/_\s*=>\s*([^,\n]+)\s*,/);
  if (!fallback) bad(`game action table unverifiable: ${spec.fn} has no \`_ =>\` arm — refusing to fail-open`);
  else if (fallback[1].trim() === spec.fallback) ok(`${spec.fn} falls back to \`${spec.fallback}\` for non-game keys`);
  else bad(`game action fallback drift (${platform}, ${spec.fn}): \`_ => ${fallback[1].trim()}\`, contract pins \`${spec.fallback}\` — see gameActionTablesNote before "fixing" this`);
}

if (libRs.includes(`b"${contract.saveMagic}"`)) ok(`save magic pinned: ${contract.saveMagic}`);
else bad(`save magic drift: b"${contract.saveMagic}" not found in wie_web/src/lib.rs — stored featurephone save blobs would stop importing`);

// ── 5. Publish-workflow dispatch payload (receiver validates these keys) ─────
const publishYml = await readFile(path.join(root, contract.dispatch.workflow), "utf8");
if (publishYml.includes(`event_type:"${contract.dispatch.eventType}"`) || publishYml.includes(`event_type: "${contract.dispatch.eventType}"`))
  ok(`dispatch event_type pinned: ${contract.dispatch.eventType}`);
else bad(`dispatch drift: event_type "${contract.dispatch.eventType}" not found in ${contract.dispatch.workflow} — otterpebble receiver only wakes on this type`);
for (const k of contract.dispatch.payloadKeys) {
  if (new RegExp(`${k}\\s*:\\s*\\$`).test(publishYml)) ok(`dispatch payload key present: ${k}`);
  else bad(`dispatch drift: client_payload key "${k}" missing in ${contract.dispatch.workflow} — receiver fail-closes on missing keys`);
}
for (const f of contract.artifacts.files) {
  if (publishYml.includes(f)) ok(`publish workflow uploads ${f}`);
  else bad(`publish drift: ${f} no longer referenced by ${contract.dispatch.workflow} release upload`);
}

// ── Report ───────────────────────────────────────────────────────────────────
console.log(`engine contract static check — ${passes.length} pass, ${violations.length} violation(s)`);
for (const v of violations) console.log(`  ✗ ${v}`);
if (process.env.WIE_CONTRACT_VERBOSE) for (const p of passes) console.log(`  ✓ ${p}`);
if (violations.length > 0) {
  console.log("\nThe featurephone web shell depends on this surface (docs/contracts/featurephone-engine-contract.json).");
  console.log("If the change is INTENTIONAL, update the contract file AND coordinate the otterpebble consumer in the same rollout.");
  process.exit(1);
}
console.log("OK — engine surface matches the pinned featurephone contract.");
