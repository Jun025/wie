// Browser audio probe — how loud a game actually is, and what the audio worklet is holding,
// measured in a real Chromium through the same path the featurephone shell uses:
// `WieEmulator(…, audioCtx, masterGain, …)` → AudioWorklet → master GainNode → AnalyserNode.
//
// ★LOCAL ONLY — it needs a game file, and game files never enter the repo, CI or any log
// (Constraint 9). Same class as scripts/smoke_gate.sh: zero callers on purpose, declared in
// AGENTS.md's local-only list. Do not wire it into a workflow.
//
// Why it exists: the audio rounds #348 and #352 each rewrote this probe from scratch and threw it
// away, and #356's reviewer could not reproduce the browser RMS because nothing was committed.
//
// Usage:
//   node scripts/audio-probe.mjs [options] <game file>...
//     --wasm <dir>       engine build to load (a dir holding wie_web.js + wie_web_bg.wasm).
//                        Repeatable: every (build × game) pair runs AT THE SAME TIME, in its own
//                        browser, so a before/after pair shares one load minute. Default:
//                        web/src/wasm (what `npm run build:wasm` writes).
//     --secs <n>         run length, seconds (default 40)
//     --key-ms <n>       press the next key of the cycle every n ms (default 700; 0 = no keys)
//     --keys <A,B,…>     the key cycle, contract vocabulary names (default: wie_validate's 27-key
//                        script, the one every audio round used)
//     --stats-every <n>  ask the worklet for `stats` every n seconds (default 10)
//     --json             print one JSON object per run instead of the table
//     --soundfont <file> pass it to the engine as the soundfont URL (served from this probe), so the
//                        run is the shell's soundfont session rather than FM only
//     --jobs <n>         runs at once (default 3; each is its own browser)
//
// What it reports per run (engine build × game):
//   plays / stops / evicts / gains — messages audio.rs posted to the worklet port, counted by
//     wrapping MessagePort.prototype.postMessage. `gains` lists the distinct game volumes (0..1).
//   seq@Ns — the worklet's own `stats` reply (`sequences.size`) at each sample. A build older
//     than #352 has no `stats`; for it the column counts distinct handles that were sent `ev`,
//     which equals what that worklet holds (it never forgot any) — the substitute #352 used.
//   synth per play — the worklet is instrumented (a module loaded before audio.rs's) so every
//     playback it starts reports whether it renders through a soundfont synth or FM, its gain, and
//     how long the worklet held it before starting. Plays are grouped by SONG (a hash of the events
//     audio.rs sent), since a game may play one song on many handles. `firstVsNext` counts songs
//     whose first play took a different synth than a later play of the same song — what a player
//     hears as "the song sounds different the second time"; `mixed` = the session's MIDI plays
//     used both synths. Version-agnostic: it reads only `playbacks` and `pb.sf`.
//   rms mean / 2nd-half mean / silent seconds — from per-second RMS of the analyser, sampled
//     every 50 ms. "Silent" is a second whose RMS is under 1e-4.
//
// Read RMS as a comparison, never as an absolute: it depends on the key cycle, on where the game
// is when a key lands, and on host load (ticks per second). Compare builds only inside one
// invocation. Needs `npx playwright install chromium` once.

import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(await readFile(path.join(root, "docs/contracts/featurephone-engine-contract.json"), "utf8"));

// wie_validate's default --inject script (wie_cli/src/bin/wie_validate.rs), cycled.
const DEFAULT_KEYS = "OK OK LEFT_SOFT_KEY NUM5 DOWN OK DOWN OK UP OK LEFT OK RIGHT OK NUM5 LEFT_SOFT_KEY RIGHT_SOFT_KEY DOWN DOWN OK UP OK STAR HASH NUM1 OK OK";

const args = process.argv.slice(2);
const opt = { wasm: [], secs: 40, keyMs: 700, keys: DEFAULT_KEYS.split(" "), statsEvery: 10, json: false, soundfont: null, jobs: 3, games: [] };
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  const next = () => {
    if (i + 1 >= args.length) usage(`${a} needs a value`);
    return args[++i];
  };
  if (a === "--wasm") opt.wasm.push(path.resolve(next()));
  else if (a === "--secs") opt.secs = Number(next());
  else if (a === "--key-ms") opt.keyMs = Number(next());
  else if (a === "--keys") opt.keys = next().split(",").filter(Boolean);
  else if (a === "--stats-every") opt.statsEvery = Number(next());
  else if (a === "--json") opt.json = true;
  else if (a === "--soundfont") opt.soundfont = path.resolve(next());
  else if (a === "--jobs") opt.jobs = Number(next());
  else if (a === "-h" || a === "--help") usage();
  else if (a.startsWith("--")) usage(`unknown option ${a}`);
  else opt.games.push(path.resolve(a));
}
if (opt.games.length === 0) usage("no game file given");
if (opt.wasm.length === 0) opt.wasm.push(path.join(root, contract.artifacts.dir));
if (!(opt.secs > 0) || !(opt.keyMs >= 0) || !(opt.statsEvery > 0) || !(opt.jobs >= 1)) usage("--secs / --key-ms / --stats-every / --jobs must be positive numbers");
const unknownKeys = opt.keys.filter((k) => !(k in contract.keyMidpCodes));
if (unknownKeys.length) usage(`not in the contract key vocabulary: ${unknownKeys.join(", ")}`);

function usage(err) {
  const help = readFileSync(fileURLToPath(import.meta.url), "utf8")
    .split("\n")
    .filter((l) => l.startsWith("//"))
    .slice(0, 48)
    .map((l) => l.slice(3))
    .join("\n");
  if (err) console.error(`audio-probe: ${err}\n`);
  console.error(help);
  process.exit(err ? 2 : 0);
}

for (const dir of opt.wasm)
  for (const f of contract.artifacts.files)
    await readFile(path.join(dir, f)).catch(() => {
      console.error(`audio-probe: ${path.join(dir, f)} is missing — run \`npm run build:wasm\` or point --wasm at a build`);
      process.exit(2);
    });

// Serves /wasm/<i>/<file> and /game/<j>. The game bytes go from this process to a page on
// 127.0.0.1 and nowhere else.
const server = createServer(async (req, res) => {
  const url = new URL(req.url, "http://x");
  if (url.pathname === "/") {
    res.writeHead(200, { "content-type": "text/html" });
    res.end("<!doctype html><html><body></body></html>");
    return;
  }
  let file = null;
  let m;
  if ((m = url.pathname.match(/^\/wasm\/(\d+)\/([\w.]+)$/)) && opt.wasm[m[1]]) file = path.join(opt.wasm[m[1]], m[2]);
  if ((m = url.pathname.match(/^\/game\/(\d+)$/)) && opt.games[m[1]]) file = opt.games[m[1]];
  if (url.pathname === "/soundfont" && opt.soundfont) file = opt.soundfont;
  try {
    const data = await readFile(file);
    const type = file.endsWith(".js") ? "text/javascript" : file.endsWith(".wasm") ? "application/wasm" : "application/octet-stream";
    res.writeHead(200, { "content-type": type, "content-length": data.length });
    res.end(data);
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

// Installed before any page script: count what audio.rs posts to the worklet, and keep the
// worklet node so the probe can ask it for `stats`.
const INIT = () => {
  const probe = (window.__probe = { plays: 0, stops: 0, evicts: 0, gains: new Set(), evHandles: new Set(), nodes: [], stats: null, song: new Map(), playLog: [], sfReady: null, t0: performance.now() });
  // FNV-1a over a play's events: one id per song, whatever handle carries it.
  const songOf = (ev) => {
    let h = 0x811c9dc5;
    const mix = (b) => (h = Math.imul(h ^ (b & 0xff), 16777619) >>> 0);
    for (const e of ev) {
      mix(e[0]);
      mix(e[0] >> 8);
      for (const part of e.slice(1)) {
        if (typeof part === "number") mix(part);
        else for (let i = 0; i < part.length; i++) mix(part[i]), mix(part[i] >> 8);
      }
    }
    return h.toString(16).padStart(8, "0");
  };
  // Loaded into the worklet scope before audio.rs's module: reports each playback the processor starts.
  const INSTRUMENT = `
    const real = globalThis.registerProcessor;
    globalThis.registerProcessor = (name, cls) => real(name, class extends cls {
      constructor(...a) {
        super(...a);
        const recv = new Map(), port = this.port, pbs = this.playbacks, set = pbs.set.bind(pbs), onm = this.onMessage.bind(this);
        this.onMessage = (m) => { if (m && m.t === "play") recv.set(m.h, currentFrame); return onm(m); };
        pbs.set = (h, pb) => {
          port.postMessage({ t: "probe-play", h, sf: !!pb.sf, g: pb.gain, midi: pb.seq.events.some((e) => e.midi), holdMs: ((currentFrame - (recv.get(h) ?? currentFrame)) * 1000) / sampleRate });
          return set(h, pb);
        };
      }
    });`;
  const addModule = AudioWorklet.prototype.addModule;
  let instrumented = false;
  AudioWorklet.prototype.addModule = async function (url, options) {
    if (!instrumented) {
      instrumented = true;
      await addModule.call(this, URL.createObjectURL(new Blob([INSTRUMENT], { type: "text/javascript" })));
    }
    return addModule.call(this, url, options);
  };
  const post = MessagePort.prototype.postMessage;
  MessagePort.prototype.postMessage = function (msg, ...rest) {
    if (msg && typeof msg === "object" && typeof msg.t === "string") {
      if (msg.t === "play") {
        probe.plays++;
        if (msg.ev) {
          probe.evHandles.add(msg.h);
          probe.song.set(msg.h, songOf(msg.ev));
        }
      } else if (msg.t === "stop") probe.stops++;
      else if (msg.t === "evict") probe.evicts++;
      else if (msg.t === "gain") probe.gains.add(Math.round(msg.g * 1000) / 1000);
    }
    return post.call(this, msg, ...rest);
  };
  const Native = window.AudioWorkletNode;
  if (Native)
    window.AudioWorkletNode = class extends Native {
      constructor(...a) {
        super(...a);
        probe.nodes.push(this);
        this.port.addEventListener("message", (e) => {
          if (e.data && e.data.t === "stats") probe.stats = e.data;
          if (e.data && e.data.t === "sf") probe.sfReady = { ok: e.data.ok, at: Math.round(performance.now() - probe.t0), error: e.data.error };
          if (e.data && e.data.t === "probe-play")
            probe.playLog.push({ at: Math.round(performance.now() - probe.t0), song: probe.song.get(e.data.h) ?? `h${e.data.h}`, sf: e.data.sf, midi: e.data.midi, g: Math.round(e.data.g * 1000) / 1000, holdMs: Math.round(e.data.holdMs) });
        });
        this.port.start();
      }
    };
};

const RUN = async ({ wasmIdx, gameIdx, gameName, secs, keyMs, keys, statsEvery, width, height, soundfontUrl }) => {
  const probe = window.__probe;
  const mod = await import(`/wasm/${wasmIdx}/wie_web.js`);
  await mod.default(`/wasm/${wasmIdx}/wie_web_bg.wasm`);
  if (typeof mod.init === "function") mod.init();

  const ctx = new AudioContext();
  await ctx.resume();
  const master = ctx.createGain();
  const analyser = ctx.createAnalyser();
  analyser.fftSize = 2048;
  master.connect(analyser);
  analyser.connect(ctx.destination);

  const bytes = new Uint8Array(await (await fetch(`/game/${gameIdx}`)).arrayBuffer());
  const canvas = document.createElement("canvas");
  document.body.appendChild(canvas);
  const emu = new mod.WieEmulator(gameName, bytes, canvas, ctx, master, width, height, soundfontUrl);

  const buf = new Float32Array(analyser.fftSize);
  const perSecond = []; // [sumSq, samples] per second
  const seq = [];
  const start = performance.now();
  let error = null;
  let keyIdx = 0;
  let nextKey = start + 2000; // let it boot before the first key
  let nextStats = start + statsEvery * 1000;
  let lastSample = 0;

  while (performance.now() - start < secs * 1000) {
    const now = performance.now();
    try {
      emu.tick();
    } catch (e) {
      error = String(e);
      break;
    }
    if (emu.has_exited()) {
      error = "guest exited";
      break;
    }
    if (keyMs > 0 && now >= nextKey) {
      const k = keys[keyIdx++ % keys.length];
      emu.key_down(k);
      setTimeout(() => emu.key_up(k), 100);
      nextKey += keyMs;
    }
    if (now - lastSample >= 50) {
      lastSample = now;
      analyser.getFloatTimeDomainData(buf);
      let s = 0;
      for (const v of buf) s += v * v;
      const sec = Math.floor((now - start) / 1000);
      perSecond[sec] ??= [0, 0];
      perSecond[sec][0] += s;
      perSecond[sec][1] += buf.length;
    }
    if (now >= nextStats) {
      const at = Math.round((now - start) / 1000);
      nextStats += statsEvery * 1000;
      probe.stats = null;
      for (const n of probe.nodes) n.port.postMessage({ t: "stats" });
      await new Promise((r) => setTimeout(r, 50));
      // No reply = a build older than #352: substitute the handles sent `ev` (see header).
      seq.push(probe.stats ? { at, sequences: probe.stats.sequences, playbacks: probe.stats.playbacks, voices: probe.stats.voices } : { at, sequences: probe.evHandles.size, substitute: true });
    }
    await new Promise((r) => requestAnimationFrame(r));
  }

  // Per song, the synth of each play in order (MIDI plays only — PCM never touches a synth).
  const songs = new Map();
  for (const p of probe.playLog) if (p.midi) songs.set(p.song, [...(songs.get(p.song) ?? []), p]);
  const firstVsNext = [...songs.values()].filter((ps) => ps.length > 1 && ps.some((p) => p.sf !== ps[0].sf)).length;
  const midiPlays = probe.playLog.filter((p) => p.midi);
  const rms = perSecond.map((x) => (x && x[1] ? Math.sqrt(x[0] / x[1]) : 0));
  const mean = (a) => (a.length ? a.reduce((p, c) => p + c, 0) / a.length : 0);
  await ctx.close();
  return {
    error,
    plays: probe.plays,
    stops: probe.stops,
    evicts: probe.evicts,
    gains: [...probe.gains].sort((a, b) => a - b),
    worklet: probe.nodes.length > 0,
    seq,
    rmsMean: mean(rms),
    rmsSecondHalf: mean(rms.slice(Math.floor(rms.length / 2))),
    rmsMax: Math.max(0, ...rms),
    silentSeconds: rms.filter((v) => v < 1e-4).length,
    seconds: rms.length,
    rmsPerSecond: rms.map((v) => Math.round(v * 10000) / 10000),
    sfReady: probe.sfReady,
    midiPlays: midiPlays.length,
    sfPlays: midiPlays.filter((p) => p.sf).length,
    songs: songs.size,
    repeatedSongs: [...songs.values()].filter((ps) => ps.length > 1).length,
    firstVsNext,
    mixed: midiPlays.some((p) => p.sf) && midiPlays.some((p) => !p.sf),
    holdMaxMs: Math.max(0, ...midiPlays.map((p) => p.holdMs)),
    playLog: probe.playLog,
  };
};

const { chromium } = await import("playwright");
const launchArgs = ["--autoplay-policy=no-user-gesture-required", "--disable-background-timer-throttling", "--disable-renderer-backgrounding"];
const pending = [];
// Game-major: the builds of one game run back to back, so a before/after pair shares the load.
for (let g = 0; g < opt.games.length; g++) for (let w = 0; w < opt.wasm.length; w++) pending.push([w, g]);
const runOne = ([w, g]) =>
      (async () => {
        // One browser per run: nothing shared (audio thread, timers) between the runs being compared.
        const browser = await chromium.launch({ headless: true, args: launchArgs });
        try {
          const page = await browser.newPage();
          const pageErrors = [];
          page.on("pageerror", (e) => pageErrors.push(e.message));
          await page.addInitScript(INIT);
          await page.goto(base + "/");
          const r = await page.evaluate(RUN, {
            wasmIdx: w,
            gameIdx: g,
            gameName: path.basename(opt.games[g]),
            secs: opt.secs,
            keyMs: opt.keyMs,
            keys: opt.keys,
            statsEvery: opt.statsEvery,
            width: contract.screen.width,
            height: contract.screen.height,
            soundfontUrl: opt.soundfont ? "/soundfont" : undefined,
          });
          return { wasm: opt.wasm[w], game: opt.games[g], ...r, pageErrors };
        } catch (e) {
          return { wasm: opt.wasm[w], game: opt.games[g], error: String(e) };
        } finally {
          await browser.close();
        }
      })();
// At most --jobs browsers at once; results keep the (build × game) order.
const results = [];
await Promise.all(
  Array.from({ length: Math.min(opt.jobs, pending.length) }, async () => {
    while (pending.length) {
      const job = pending.shift();
      const i = job[0] * opt.games.length + job[1];
      results[i] = await runOne(job);
      if (opt.json) console.log(JSON.stringify(results[i]));
    }
  }),
);
server.close();

const f4 = (v) => (typeof v === "number" ? v.toFixed(4) : "-");
for (const r of results) {
  if (opt.json) continue; // printed as each run finished
  console.log(`\n■ ${path.basename(r.game)}  @ ${path.relative(root, r.wasm) || r.wasm}`);
  if (r.error) console.log(`  stopped: ${r.error}`);
  if (r.plays === undefined) continue;
  console.log(`  plays ${r.plays} · stops ${r.stops} · evicts ${r.evicts} · gains [${r.gains.join(", ")}] · worklet ${r.worklet ? "yes" : "NO"}`);
  console.log(`  seq ${r.seq.map((s) => `${s.at}s=${s.sequences}${s.substitute ? "*" : ""}`).join(" ") || "-"}${r.seq.some((s) => s.substitute) ? "   (* no stats reply — distinct handles sent `ev`)" : ""}`);
  console.log(`  MIDI plays ${r.midiPlays} (soundfont ${r.sfPlays}) · songs ${r.songs} (repeated ${r.repeatedSongs}) · first≠next ${r.firstVsNext} · mixed ${r.mixed} · hold max ${r.holdMaxMs} ms · soundfont ${r.sfReady ? `${r.sfReady.ok ? "ready" : "failed"} at ${r.sfReady.at} ms` : "-"}`);
  console.log(`  rms mean ${f4(r.rmsMean)} · 2nd half ${f4(r.rmsSecondHalf)} · max ${f4(r.rmsMax)} · silent ${r.silentSeconds}/${r.seconds}s`);
  if (r.pageErrors?.length) console.log(`  page errors: ${r.pageErrors.slice(0, 3).join(" | ")}`);
}
process.exit(results.some((r) => r.plays === undefined) ? 1 : 0);
