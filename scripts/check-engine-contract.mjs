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
for (const key of contract.keyVocabulary) {
  if (new RegExp(`"${key}"\\s*=>`).test(libRs)) ok(`parse_key covers "${key}"`);
  else bad(`key vocabulary drift: parse_key no longer maps "${key}" — featurephone KEY_MAP sends this code`);
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
