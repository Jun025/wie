// Build test_data/resize_ktf.zip — the ONLY fixture that asks the host to change
// the screen size, i.e. the only input that reaches `Screen::resize`.
//
// Why it exists (measured 2026-09-16, contract-roundtrip.mjs's own header): the
// engine calls `platform.screen().resize(..)` in exactly one live place —
// `wie-ktf/src/emulator.rs`, `if let Some((width, height)) = adf.display_size` —
// and NEITHER committed KTF fixture declares `DisplaySize:` (both `__adf__`s are
// `AID`/`PID`/`MClass` only). So that call site, and every `Screen::resize`
// implementation under it, ran zero times in the whole test suite. The call site
// also swallows `Err` into `tracing::warn!`, so a host whose resize fails boots
// on regardless: nothing anywhere went red.
//
// Why a SEPARATE fixture rather than one more `DisplaySize:` line in
// helloworld_ktf.zip: Scenario E/F assert exact pixel counts against the current
// geometry, so resizing an existing fixture moves their canvas out from under
// them. This one is a derivative — same guest jar, one extra ADF line — so it
// tests the size path and nothing else.
//
// The output is BYTE-STABLE: entries are STORED with zeroed timestamps (the zip
// helper below is make-draw-fixture.mjs's, unchanged), so re-running this on the
// same source produces an identical file. Regenerate with:
//
//   node scripts/make-resize-fixture.mjs
//
// and verify with the round-trip's Scenario G, not by eyeballing the zip.

import { readFileSync, writeFileSync } from "node:fs";
import { inflateRawSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { zip } from "./make-draw-fixture.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const SRC_FIXTURE = "helloworld_ktf.zip";
export const RESIZE_FIXTURE = "resize_ktf.zip";

// ★The scenario imports these instead of restating "176x220" — same rule as
//   make-draw-fixture.mjs's exported pixel counts: the fixture owns its numbers.
//   Deliberately different on BOTH axes from the contract's 240x320 boot size, so
//   a half-applied resize (one axis, or only the front canvas) cannot read as a pass.
export const RESIZE_W = 176;
export const RESIZE_H = 220;
export const ADF_LINE = `DisplaySize:${RESIZE_W}*${RESIZE_H}`;

/** Read a STORED/DEFLATE zip by walking local headers. Enough for our own fixtures
 *  (flags=0, no data descriptors — checked), and it keeps this script dependency-free. */
function readEntries(buf) {
  const out = [];
  let off = 0;
  while (buf.readUInt32LE(off) === 0x04034b50) {
    const flag = buf.readUInt16LE(off + 6);
    const method = buf.readUInt16LE(off + 8);
    const csize = buf.readUInt32LE(off + 18);
    const nlen = buf.readUInt16LE(off + 26);
    const elen = buf.readUInt16LE(off + 28);
    if (flag & 0x08) throw new Error("data descriptor (flag bit 3) — sizes are not in the local header");
    const name = buf.subarray(off + 30, off + 30 + nlen).toString("utf8");
    const start = off + 30 + nlen + elen;
    const raw = buf.subarray(start, start + csize);
    out.push([name, method === 8 ? inflateRawSync(raw) : Buffer.from(raw)]);
    off = start + csize;
  }
  if (!out.length) throw new Error("no local file headers — not a zip we can read");
  return out;
}

export function resizeFixtureZip() {
  const src = readFileSync(path.join(root, "test_data", SRC_FIXTURE));
  const entries = readEntries(src);
  const adf = entries.find(([n]) => n === "__adf__");
  if (!adf) throw new Error(`${SRC_FIXTURE} has no __adf__ — KTF archives must carry one`);
  if (adf[1].includes(ADF_LINE)) throw new Error(`${SRC_FIXTURE} already declares ${ADF_LINE} — the source fixture changed`);
  // Append, never rewrite: AID/PID/MClass decide the jar name and main class, and
  // this fixture must stay the same guest as its source.
  adf[1] = Buffer.concat([adf[1], Buffer.from(`${ADF_LINE}\n`, "utf8")]);
  return zip(entries);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const out = path.join(root, "test_data", RESIZE_FIXTURE);
  const bytes = resizeFixtureZip();
  writeFileSync(out, bytes);
  console.log(`wrote ${path.relative(root, out)} — ${bytes.length} bytes, ${ADF_LINE}`);
}
