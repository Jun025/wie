#!/usr/bin/env node
// Bundles wie_featurephone/src/soundfont_prelude.mjs (spessasynth_core) into one minified IIFE for
// the featurephone audio worklet. Called by scripts/build-wasm.sh (which exports the output path
// as WIE_SOUNDFONT_PRELUDE for wie_featurephone/build.rs) and by scripts/check-audio-worklet.mjs.
//
//   node scripts/build-soundfont-prelude.mjs [out]   # default target/wie-soundfont/soundfont_prelude.js
//
// Needs the root devDependencies (`npm ci`): spessasynth_core, esbuild. Both are exact-pinned in
// package.json, so the same lockfile gives the same bytes.
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = path.resolve(process.argv[2] ?? path.join(root, "target/wie-soundfont/soundfont_prelude.js"));
await mkdir(path.dirname(out), { recursive: true });
const version = async (pkg) => JSON.parse(await readFile(path.join(root, "node_modules", pkg, "package.json"), "utf8")).version;
// Apache-2.0 §4(a): the artifact that carries this code also carries the notice (licenses/).
const banner =
  `/* soundfont prelude: spessasynth_core ${await version("spessasynth_core")} + stb-vorbis ${await version("stb-vorbis")} — ` +
  `Apache License 2.0, full text licenses/Apache-2.0.txt (Jun025/wie, attached to every engine release) */`;
const result = await build({
  entryPoints: [path.join(root, "wie_featurephone/src/soundfont_prelude.mjs")],
  bundle: true,
  format: "iife",
  minify: true,
  // Chrome/Safari AudioWorkletGlobalScope: no DOM, no TextDecoder guarantee; the library guards the latter.
  platform: "neutral",
  mainFields: ["module", "main"],
  target: "es2020",
  legalComments: "none",
  banner: { js: banner },
  write: false,
  logLevel: "warning",
});
// Rewrite only on a real change: build.rs declares rerun-if-changed on this file, and the wasm
// release profile is fat LTO — a same-bytes rewrite would relink the whole engine every build.
const bytes = result.outputFiles[0].contents;
const old = await readFile(out).catch(() => null);
if (!old || Buffer.compare(old, Buffer.from(bytes)) !== 0) await writeFile(out, bytes);
console.log(out);
