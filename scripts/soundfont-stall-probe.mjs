#!/usr/bin/env node
// soundfont-stall-probe — how long the featurephone audio thread stalls when the soundfont arrives,
// and how late the first note of each new instrument is. Local only; needs no game file.
//
//   node scripts/build-soundfont-prelude.mjs                      # once (npm ci first)
//   node scripts/soundfont-stall-probe.mjs [--runs 3]             # headless desktop Chromium
//   node scripts/soundfont-stall-probe.mjs --cdp http://localhost:9341 [--port 18791] [--runs 3]
//       # an Android Chrome reached through `adb forward tcp:9341 localabstract:chrome_devtools_remote`
//       # and `adb reverse tcp:18791 tcp:18791` (the page must be http://localhost — AudioWorklet needs a
//       # secure context). Use the emulator only through ~/orchestrator/bin/emu-run.
//
// It loads the SHIPPED worklet (wie_featurephone/src/audio_worklet.js), then the prelude as a second
// module, then posts wie-web/public/GeneralUser.sf3 — the order audio.rs uses (docs/report 0355) — and
// wraps `process`/`onMessage` from a module loaded before them, so the code under test is unmodified.
// Per phase it reports:
//   gapMax    longest wall-clock gap between two render quanta (ms). Whatever holds the audio thread —
//             prelude evaluation, parsing, synth construction, sample decoding — shows up here.
//   procMax   longest single `process()` call (ms).
//   lost      how far the context clock fell behind wall time over the phase (getOutputTimestamp):
//             frames that were due and not rendered, i.e. an audible dropout. ~0 = nothing lost.
//             ★Under the emulator (-no-audio) it swings by seconds either way, control runs included —
//             read gapMax/procMax against baseLatency there (docs/report 0359).
//   firstMs   play message → first non-silent output sample, wall clock (plays only).
// A soundfont row minus the same row of an --control run is the soundfont's own cost.
// Date.now() in the worklet has 1 ms resolution; the render quantum is 128 frames (2.67 ms at 48 kHz).
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const arg = (name, fallback) => {
  const i = process.argv.indexOf(name);
  return i > 0 ? process.argv[i + 1] : fallback;
};
const cdp = arg("--cdp", null);
const port = Number(arg("--port", "18791"));
const runs = Number(arg("--runs", "3"));
// --control interleaves an FM-only run (same phases and plays, no prelude, no soundfont) after each
// soundfont run: on a loaded host the noise floor is the thing to subtract.
const control = process.argv.includes("--control");

const files = {
  "/worklet.js": [path.join(root, "wie_featurephone/src/audio_worklet.js"), "text/javascript"],
  "/prelude.js": [path.join(root, "target/wie-soundfont/soundfont_prelude.js"), "text/javascript"],
  "/GeneralUser.sf3": [path.join(root, "wie-web/public/GeneralUser.sf3"), "application/octet-stream"],
};

// Loaded first into the worklet scope: wraps the processor the shipped module registers.
const INSTRUMENT = `
const real = globalThis.registerProcessor;
globalThis.registerProcessor = (name, cls) => {
  const P = cls.prototype, proc = P.process, onm = P.onMessage;
  P.process = function (i, o) {
    const t0 = Date.now();
    const r = proc.call(this, i, o);
    const t1 = Date.now();
    const s = this.__probe;
    if (s.last) s.gapMax = Math.max(s.gapMax, t0 - s.last);
    s.procMax = Math.max(s.procMax, t1 - t0);
    s.last = t1;
    if (s.pending) {
      const ch = o[0][0];
      for (let k = 0; k < ch.length; k++) if (Math.abs(ch[k]) > 1e-4) { s.firstMs = t1 - s.pending; s.pending = 0; break; }
    }
    return r;
  };
  P.onMessage = function (m) {
    const s = (this.__probe ??= { last: 0, gapMax: 0, procMax: 0, pending: 0, firstMs: -1 });
    if (m.t === "probe") {
      this.port.postMessage({ t: "probe", gapMax: s.gapMax, procMax: s.procMax, firstMs: s.firstMs, sampleRate });
      s.gapMax = 0; s.procMax = 0; s.firstMs = -1;
      return;
    }
    if (m.t === "play") { s.pending = Date.now(); s.firstMs = -1; }
    return onm.call(this, m);
  };
  const Wrapped = class extends cls { constructor(...a) { super(...a); this.__probe ??= { last: 0, gapMax: 0, procMax: 0, pending: 0, firstMs: -1 }; } };
  return real(name, Wrapped);
};`;

const PAGE = `<!doctype html><meta name=viewport content="width=device-width"><button id=go style="font-size:40px">go</button><pre id=out></pre>
<script type=module>
const note = (h, prog, ch = 0) => ({ t: "play", h, r: false, d: 900, ev: [[0, 0, new Uint8Array([0xc0 | ch, prog])], [0, 0, new Uint8Array([0x90 | ch, 60, 100])], [700, 0, new Uint8Array([0x80 | ch, 60, 0])]] });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
window.runProbe = async (fm) => {
  const ctx = new AudioContext();
  await ctx.resume();
  const t = await (await fetch("/GeneralUser.sf3")).arrayBuffer(); // fetched up front: the network is not under test
  await ctx.audioWorklet.addModule(URL.createObjectURL(new Blob([${JSON.stringify(INSTRUMENT)}], { type: "text/javascript" })));
  await ctx.audioWorklet.addModule("/worklet.js");
  const node = new AudioWorkletNode(ctx, "wie-audio", { numberOfInputs: 0, outputChannelCount: [2] });
  node.connect(ctx.destination);
  const replies = [];
  node.port.onmessage = (e) => replies.push(e.data);
  const ask = async (t) => { const n = replies.length; node.port.postMessage({ t }); while (replies.length === n || replies[replies.length - 1].t !== t) await sleep(5); return replies[replies.length - 1]; };
  const clock = () => { const o = ctx.getOutputTimestamp(); return [o.performanceTime, o.contextTime * 1000]; };
  const rows = [];
  const phase = async (name, body) => {
    await ask("probe");
    const [p0, c0] = clock();
    const extra = (await body()) ?? {};
    await sleep(400);
    const [p1, c1] = clock();
    const r = await ask("probe");
    rows.push({ phase: name, gapMax: r.gapMax, procMax: r.procMax, lost: Math.round((p1 - p0) - (c1 - c0)), firstMs: r.firstMs, ...extra });
  };
  await sleep(1500);
  await phase("idle (FM, nothing playing)", async () => {});
  await phase("play#1 FM piano", async () => { node.port.postMessage(note(1, 0)); await sleep(900); });
  node.port.postMessage({ t: "stop", h: 1 });
  await phase("prelude addModule", async () => { const s = performance.now(); if (!fm) await ctx.audioWorklet.addModule("/prelude.js"); return { mainMs: Math.round(performance.now() - s) }; });
  await phase("sf parse (+1st synth)", async () => { if (fm) return {}; const n = replies.length; node.port.postMessage({ t: "sf", data: t }, [t]); while (!replies.slice(n).some((r) => r.t === "sf")) await sleep(5); const r = replies.slice(n).find((r) => r.t === "sf"); return { parseMs: r.ms, ok: r.ok }; });
  const progs = [[0, 0, "piano"], [48, 0, "strings"], [56, 0, "trumpet"], [73, 0, "flute"], [0, 9, "drums"], [48, 0, "strings again"], [0, 0, "piano again"]];
  let h = 10;
  for (const [prog, ch, label] of progs) {
    await sleep(1300); // past STOP_RELEASE_S + SF_TAIL_S so the synth is back in the pool
    const hh = h++;
    await phase("sf play " + label, async () => { node.port.postMessage({ t: "gain", h: hh, g: 1 }); node.port.postMessage(note(hh, prog, ch)); await sleep(900); });
    node.port.postMessage({ t: "stop", h: hh });
  }
  const stats = await ask("stats");
  const env = { ua: navigator.userAgent, sampleRate: ctx.sampleRate, baseLatencyMs: +(ctx.baseLatency * 1000).toFixed(1), outputLatencyMs: +((ctx.outputLatency ?? 0) * 1000).toFixed(1), state: ctx.state, built: stats.built, soundfont: stats.soundfont };
  await ctx.close();
  return { env, rows };
};
document.getElementById("go").onclick = async () => { document.getElementById("out").textContent = "running"; window.result = await window.runProbe(location.search.includes("fm=1")); document.getElementById("out").textContent = JSON.stringify(window.result, null, 1); };
</script>`;

const server = createServer(async (req, res) => {
  const url = req.url.split("?")[0];
  if (url === "/") return res.writeHead(200, { "content-type": "text/html" }).end(PAGE);
  const f = files[url];
  if (!f) return res.writeHead(404).end();
  res.writeHead(200, { "content-type": f[1] }).end(await readFile(f[0]));
});
await new Promise((r) => server.listen(port, "127.0.0.1", r));

const { chromium } = await import("playwright");
const browser = cdp ? await chromium.connectOverCDP(cdp) : await chromium.launch({ args: ["--autoplay-policy=no-user-gesture-required"] });
const context = cdp ? browser.contexts()[0] : await browser.newContext();
let failed = 0;
const { loadavg } = await import("node:os");
const plan = [];
for (let run = 1; run <= runs; run++) plan.push([run, false], ...(control ? [[run, true]] : []));
for (const [run, fm] of plan) {
  const page = await context.newPage();
  await page.goto(`http://localhost:${port}/${fm ? "?fm=1" : ""}`);
  const load1 = loadavg()[0].toFixed(0);
  await page.tap?.("#go").catch(() => page.click("#go"));
  let result;
  try {
    await page.waitForFunction(() => window.result, null, { timeout: 180000, polling: 500 });
    result = await page.evaluate(() => window.result);
  } catch (error) {
    failed++;
    console.log(`run ${run}${fm ? " FM-control" : ""}: FAILED ${error.message.split("\n")[0]}`);
    await page.close();
    continue;
  }
  if (run === 1 && !fm) console.log(JSON.stringify(result.env));
  console.log(`run ${run}${fm ? " FM-control" : ""} · host load1 ${load1}`);
  console.table(result.rows);
  console.log("JSON " + JSON.stringify({ run, fm, load1, env: result.env, rows: result.rows })); // one line per run, for aggregation
  await page.close();
}
await (cdp ? browser.close().catch(() => {}) : browser.close());
server.close();
process.exit(failed ? 1 : 0);
