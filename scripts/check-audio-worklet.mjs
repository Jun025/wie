#!/usr/bin/env node
// Behaviour check for wie_featurephone/src/audio_worklet.js — the featurephone host's synth.
//
// Runs the worklet under node with stub AudioWorkletGlobalScope globals, drives it with the same
// messages audio.rs posts, and measures output RMS. Every case here is a thing the host could not
// do before 2026-09-27 (MIDI was dropped, `repeat` ignored, `Stop` a no-op), so each assertion is
// the "revert it and this goes red" guard for one of them. Offline, no browser, ~0.5 s.
//
//   node scripts/check-audio-worklet.mjs      # rc 0 = all cases hold, rc 1 = a case failed
//
// Cases 10+ are the optional soundfont synth (docs/worklog/2026-09-28-featurephone-soundfont-lazy-load.json).
// They need the soundfont prelude, which is spessasynth_core bundled by build-soundfont-prelude.mjs, so
// they need the root devDependencies (`npm ci`) — and the committed wie-web/public/GeneralUser.sf3.
// Without them they are SKIPPED and say so; `--require-soundfont` turns that skip into a failure
// (engine-contract.yml passes it where `npm ci` has run). Soundfont cases add ~5 s (one parse per boot).
//   node scripts/check-audio-worklet.mjs --require-soundfont
//   node scripts/check-audio-worklet.mjs --source <file>   # run against another worklet (mutation checks)
import { readFile, mkdtemp, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const argv = process.argv.slice(2);
const requireSoundfont = argv.includes("--require-soundfont");
const sourceArg = argv.indexOf("--source");
const source = await readFile(sourceArg >= 0 ? path.resolve(argv[sourceArg + 1]) : path.join(root, "wie_featurephone/src/audio_worklet.js"), "utf8");
const RATE = 48000;
const BLOCK = 128;

function boot(prelude) {
  let Processor;
  let registrations = 0;
  const replies = [];
  const ctx = {
    sampleRate: RATE,
    currentFrame: 0,
    AudioWorkletProcessor: class {
      constructor() {
        this.port = { onmessage: null, postMessage: (message) => replies.push(message) };
      }
    },
    // Throws on a repeat name, as the real AudioWorkletGlobalScope does (NotSupportedError).
    registerProcessor: (name, cls) => {
      if (name !== "wie-audio") throw new Error(`unexpected processor name ${name}`);
      if (Processor) throw new Error(`NotSupportedError: "${name}" is already registered`);
      registrations++;
      Processor = cls;
    },
    Math,
    Float32Array,
    Map,
    Error,
    console,
  };
  vm.createContext(ctx);
  // Each load gets its own scope, as each addModule gets its own module scope; the global (the
  // AudioWorkletGlobalScope) is shared.
  const load = () => vm.runInContext(`(function () {\n${source}\n})();`, ctx);
  load();
  const proc = new Processor();
  // audio.rs loads the worklet ALONE and the processor exists before the prelude arrives: the prelude
  // is a second addModule, made only once the soundfont file is in hand (docs/report 0355). So it
  // runs here after the processor is built, into the same global.
  if (prelude) vm.runInContext(prelude, ctx);
  return {
    post: (message) => proc.port.onmessage({ data: message }),
    // Renders `seconds` and returns the RMS of the left channel.
    render(seconds) {
      const frames = Math.round(seconds * RATE);
      let sum = 0;
      let n = 0;
      for (let done = 0; done < frames; done += BLOCK) {
        const out = [[new Float32Array(BLOCK), new Float32Array(BLOCK)]];
        proc.process([], out);
        ctx.currentFrame += BLOCK;
        for (const v of out[0][0]) sum += v * v;
        n += BLOCK;
      }
      return Math.sqrt(sum / n);
    },
    voices: () => proc.voices.length,
    // Samples of the parsed soundfont that hold decoded audio (what the synths' memory grows by).
    decodedSamples: () => (proc.bank ? proc.bank.samples.filter((sample) => sample.audioData).length : 0),
    // Every sample a GM preset's zones hold — what decoding the whole instrument would cost.
    presetSamples: (program) =>
      new Set(proc.bank.getPreset({ program, bankMSB: 0, bankLSB: 0, isGMGSDrum: false }, "gs").zones.flatMap((zone) => zone.instrument.zones.map((z) => z.sample))).size,
    // Posts a soundfont buffer and waits for the worklet's { t: "sf" } reply. Parsing is queued work
    // that only `process()` runs, so this renders while it waits — as the audio thread would.
    async soundfont(buffer) {
      const before = replies.length;
      proc.port.onmessage({ data: { t: "sf", data: buffer } });
      for (let spin = 0; spin < 2000; spin++) {
        const reply = replies.slice(before).find((message) => message.t === "sf");
        if (reply) return reply;
        this.render(0.05);
        await new Promise((resolve) => setTimeout(resolve, 1));
      }
      throw new Error("no sf reply from the worklet");
    },
    // Renders until the worklet's work queue (decoding, synth construction) is empty; returns the
    // seconds of audio that took.
    settle() {
      let seconds = 0;
      while (proc.work.length && seconds < 60) {
        this.render(0.05);
        seconds += 0.05;
      }
      return seconds;
    },
    // Renders `seconds` and returns the left channel itself (for exact comparisons).
    samples(seconds) {
      const frames = Math.round(seconds * RATE);
      const all = new Float32Array(frames);
      for (let done = 0; done < frames; done += BLOCK) {
        const out = [[new Float32Array(BLOCK), new Float32Array(BLOCK)]];
        proc.process([], out);
        ctx.currentFrame += BLOCK;
        all.set(out[0][0].subarray(0, Math.min(BLOCK, frames - done)), done);
      }
      return all;
    },
    // What the worklet itself reports for `stats` — the number a page can read.
    stats() {
      proc.port.onmessage({ data: { t: "stats" } });
      return replies.at(-1);
    },
    // Loads the module a second time into the same scope, as a second sink on a reused
    // AudioContext would. Returns the error it threw, or null.
    reload() {
      try {
        load();
        return null;
      } catch (error) {
        return error;
      }
    },
    registrations: () => registrations,
  };
}

const midi = (time, ...bytes) => [time, 0, Uint8Array.from(bytes)];
const LOUD = 0.01;
const QUIET = 1e-3;
let failed = 0;
function check(name, ok, detail) {
  console.log(`${ok ? "ok  " : "FAIL"} ${name} — ${detail}`);
  if (!ok) failed++;
}

// 1. MIDI reaches the output (was dropped by the old sink).
{
  const w = boot();
  w.post({ t: "play", h: 0, r: false, d: 500, ev: [midi(0, 0xc0, 0), midi(0, 0x90, 60, 100), midi(500, 0x80, 60, 0)] });
  const during = w.render(0.4);
  w.render(2.1);
  const after = w.render(0.5);
  check("MIDI note sounds, then ends", during > LOUD && after < QUIET, `rms 0-0.4s ${during.toFixed(4)} · 2.5-3.0s ${after.toExponential(1)}`);
}

// 2. Stop silences a sustained note at once (was a no-op).
{
  const w = boot();
  w.post({ t: "play", h: 0, r: true, d: 10000, ev: [midi(0, 0xc0, 16), midi(0, 0x90, 64, 100)] });
  const before = w.render(0.3);
  w.post({ t: "stop", h: 0 });
  w.render(0.1);
  const after = w.render(0.3);
  check("Stop silences the handle", before > LOUD && after < QUIET, `rms before ${before.toFixed(4)} · after ${after.toExponential(1)}`);
}

// 3. repeat keeps playing past the sequence length (was ignored); without it the sequence ends.
for (const repeat of [true, false]) {
  const w = boot();
  w.post({ t: "play", h: 0, r: repeat, d: 200, ev: [midi(0, 0xc0, 80), midi(0, 0x90, 72, 110), midi(150, 0x80, 72, 0)] });
  w.render(2.0);
  const late = w.render(0.4);
  check(`repeat=${repeat} ${repeat ? "loops" : "ends"}`, repeat ? late > LOUD : late < QUIET, `rms 2.0-2.4s ${late.toExponential(1)} (song is 0.2s)`);
}

// 4. PCM plays through the worklet (resampled) and two handles mix; stopping one keeps the other.
{
  const pcm = new Int16Array(4000); // 0.5 s at 8 kHz
  for (let i = 0; i < pcm.length; i++) pcm[i] = Math.round(12000 * Math.sin((2 * Math.PI * 440 * i) / 8000));
  const w = boot();
  w.post({ t: "play", h: 0, r: true, d: 10000, ev: [midi(0, 0xc0, 16), midi(0, 0x90, 60, 100)] });
  const bgmOnly = w.render(0.2);
  w.post({ t: "play", h: 1, r: false, d: 0, ev: [[0, 1, 1, 8000, pcm]] });
  const both = w.render(0.2);
  w.post({ t: "stop", h: 0 });
  w.render(0.05);
  const sfxOnly = w.render(0.15);
  w.render(0.4);
  const tail = w.render(0.2);
  check(
    "PCM mixes over MIDI, survives the other handle's Stop, then ends",
    both > bgmOnly * 1.2 && sfxOnly > LOUD && tail < QUIET,
    `rms bgm ${bgmOnly.toFixed(4)} · bgm+sfx ${both.toFixed(4)} · sfx after stop(0) ${sfxOnly.toFixed(4)} · tail ${tail.toExponential(1)}`,
  );
}

// 5. A replay of a loaded handle needs no events (audio.rs sends `ev` once per handle).
{
  const w = boot();
  w.post({ t: "play", h: 3, r: false, d: 300, ev: [midi(0, 0x99, 38, 120)] });
  w.render(1.0);
  w.post({ t: "play", h: 3, r: false, d: 300 });
  const replay = w.render(0.1);
  check("replay without events reuses the loaded sequence (drum kit on ch 10)", replay > LOUD, `rms ${replay.toFixed(4)}`);
}

// 6. Voice count is bounded (a runaway sequence cannot grow the render loop without limit).
{
  const w = boot();
  const ev = [midi(0, 0xc0, 16)];
  for (let n = 0; n < 100; n++) ev.push(midi(0, 0x90, 20 + n, 100));
  w.post({ t: "play", h: 0, r: false, d: 1000, ev });
  w.render(0.1);
  check("voices stay bounded", w.voices() <= 48, `voices ${w.voices()} after 100 simultaneous notes`);
}

// 7. evict frees a sequence without cutting it, and the next play of that handle needs `ev` again
//    (audio.rs resends it). Until 2026-09-27 nothing was ever freed: one sequence per handle forever.
{
  const w = boot();
  w.post({ t: "play", h: 7, r: true, d: 10000, ev: [midi(0, 0xc0, 16), midi(0, 0x90, 64, 100)] });
  for (let h = 100; h < 110; h++) w.post({ t: "play", h, r: false, d: 50, ev: [midi(0, 0x99, 42, 90)] });
  w.render(0.2);
  const held = w.stats().sequences;
  for (let h = 100; h < 110; h++) w.post({ t: "evict", h });
  w.post({ t: "evict", h: 7 });
  const left = w.stats().sequences;
  const loopAfterEvict = w.render(0.3); // handle 7 is still looping
  w.post({ t: "stop", h: 7 });
  w.render(0.3);
  w.post({ t: "play", h: 7, r: false, d: 300 }); // no ev: the sequence is gone
  const bare = w.render(0.2);
  w.post({ t: "play", h: 7, r: false, d: 300, ev: [midi(0, 0x99, 38, 120)] });
  const resent = w.render(0.1);
  check(
    "evict frees sequences, keeps a playing one sounding, and a replay needs its events resent",
    held === 11 && left === 0 && loopAfterEvict > LOUD && bare < QUIET && resent > LOUD,
    `sequences ${held} → ${left} · looping after evict ${loopAfterEvict.toFixed(4)} · bare replay ${bare.toExponential(1)} · resent ${resent.toFixed(4)}`,
  );
}

// 8. Loading the module twice into one scope neither throws nor re-registers (a throw rejects
//    addModule and drops that sink to the MIDI-silent fallback).
{
  const w = boot();
  const error = w.reload();
  check("second module load into the same scope is a no-op", error === null && w.registrations() === 1, `error ${error} · registrations ${w.registrations()}`);
}

// 9. The game's volume: a gain sent before a play scales that play (MIDI and PCM alike), a gain
//    sent while it plays scales it at once, 0 silences it, and the next play without a gain is
//    back at 1 (backend Audio sends one before every play, so none may linger). Until 2026-09-27
//    there was no gain: Clip.setVolume and the rest were stubs.
{
  const organ = [midi(0, 0xc0, 16), midi(0, 0x90, 64, 60)];
  const pcm = new Int16Array(8000);
  for (let i = 0; i < pcm.length; i++) pcm[i] = Math.round(4000 * Math.sin((2 * Math.PI * 440 * i) / 8000));
  const level = (gain, ev) => {
    const w = boot();
    if (gain !== undefined) w.post({ t: "gain", h: 0, g: gain });
    w.post({ t: "play", h: 0, r: true, d: 10000, ev });
    w.render(0.1);
    return w.render(0.2);
  };
  const midiRatio = level(0.5, organ) / level(undefined, organ);
  const pcmRatio = level(0.5, [[0, 1, 1, 8000, pcm]]) / level(undefined, [[0, 1, 1, 8000, pcm]]);

  const w = boot();
  w.post({ t: "gain", h: 0, g: 0.5 });
  w.post({ t: "play", h: 0, r: true, d: 10000, ev: organ });
  w.render(0.1);
  const half = w.render(0.2);
  w.post({ t: "gain", h: 0, g: 0 });
  const muted = w.render(0.2);
  w.post({ t: "gain", h: 0, g: 1 });
  const full = w.render(0.2);
  w.post({ t: "gain", h: 0, g: 0 });
  w.post({ t: "stop", h: 0 });
  w.render(0.2);
  w.post({ t: "play", h: 0, r: true, d: 10000 });
  w.render(0.1);
  const replay = w.render(0.2);
  const near = (x, y) => Math.abs(x - y) < 0.05;
  check(
    "gain scales a play before and during it, 0 silences, and does not outlive the next play",
    near(midiRatio, 0.5) && near(pcmRatio, 0.5) && muted < QUIET && near(full / half, 2) && near(replay / full, 1),
    `midi ×${midiRatio.toFixed(3)} · pcm ×${pcmRatio.toFixed(3)} · muted ${muted.toExponential(1)} · 1/0.5 ×${(full / half).toFixed(3)} · replay/full ×${(replay / full).toFixed(3)}`,
  );
}

// ---- Soundfont (cases 10-19) -------------------------------------------------------------------
const soundfontPath = path.join(root, "wie-web/public/GeneralUser.sf3");
const haveDeps = existsSync(path.join(root, "node_modules/esbuild")) && existsSync(path.join(root, "node_modules/spessasynth_core"));
if (!haveDeps || !existsSync(soundfontPath)) {
  const why = !haveDeps ? "root devDependencies not installed (run `npm ci`)" : `${path.relative(root, soundfontPath)} missing`;
  console.log(`SKIP soundfont cases 10-19 — ${why}. NOT MEASURED.`);
  if (requireSoundfont) {
    console.error("check-audio-worklet: --require-soundfont but the soundfont cases could not run");
    process.exit(1);
  }
} else {
  const tmp = await mkdtemp(path.join(os.tmpdir(), "wie-sf-prelude-"));
  const preludeFile = path.join(tmp, "soundfont_prelude.js");
  const { execFileSync } = await import("node:child_process");
  execFileSync(process.execPath, [path.join(root, "scripts/build-soundfont-prelude.mjs"), preludeFile], { stdio: "ignore" });
  const prelude = await readFile(preludeFile, "utf8");
  await rm(tmp, { recursive: true, force: true });
  const sfBytes = await readFile(soundfontPath);
  const bank = () => sfBytes.buffer.slice(sfBytes.byteOffset, sfBytes.byteOffset + sfBytes.length);
  const song = () => [midi(0, 0xc0, 0), midi(0, 0x90, 60, 100), midi(0, 0x90, 64, 100), midi(400, 0x80, 60, 0), midi(400, 0x80, 64, 0)];
  const same = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
  const rmsOf = (a) => Math.sqrt(a.reduce((sum, v) => sum + v * v, 0) / a.length);
  const script = (w) => {
    w.post({ t: "gain", h: 2, g: 0.7 });
    w.post({ t: "play", h: 2, r: true, d: 600, ev: song() });
    const a = w.samples(0.5);
    w.post({ t: "stop", h: 2 });
    return [a, w.samples(0.3)];
  };

  // 10. The prelude alone changes nothing: with no soundfont posted, the output is sample-for-sample
  //     the FM output of a build without the prelude (the "no URL" path).
  {
    const [a1, b1] = script(boot());
    const [a2, b2] = script(boot(prelude));
    check("prelude without a soundfont = FM output, sample-for-sample", same(a1, a2) && same(b1, b2) && rmsOf(a1) > LOUD, `rms ${rmsOf(a1).toFixed(4)} · equal ${same(a1, a2) && same(b1, b2)}`);
  }

  // 11. A soundfont that does not parse, or a build with no prelude, leaves FM exactly as it was.
  {
    const [fa, fb] = script(boot());
    const noPrelude = boot();
    const r1 = await noPrelude.soundfont(bank());
    const [na, nb] = script(noPrelude);
    const garbage = boot(prelude);
    const r2 = await garbage.soundfont(new Uint8Array(4096).fill(7).buffer);
    const [ga, gb] = script(garbage);
    check(
      "no prelude / unparsable soundfont both reply ok:false and stay FM, sample-for-sample",
      r1.ok === false && r2.ok === false && garbage.stats().soundfont === "failed" && same(fa, na) && same(fb, nb) && same(fa, ga) && same(fb, gb),
      `no-prelude ${JSON.stringify(r1)} · garbage ok ${r2.ok} (${String(r2.error).slice(0, 60)}) · state ${garbage.stats().soundfont}`,
    );
  }

  // 12. One synth per session (docs/report 0438). In a session told a soundfont is coming (`sfwait`),
  //     a MIDI play that arrives before the soundfont parsed is HELD, not played on FM, and starts on
  //     the soundfont once it has parsed and its samples are decoded. A play of an instrument not
  //     decoded yet is held the same way while exactly the samples its notes reach are decoded — one
  //     work item per quantum (a whole instrument inside a play held the audio thread 156 ms on an
  //     Android emulator — docs/report 0359). No MIDI play of the session ever renders FM. Until
  //     2026-10-04 both first plays were FM and only the next play of each song the soundfont.
  const w = boot(prelude);
  {
    w.post({ t: "sfwait" });
    w.post({ t: "play", h: 0, r: true, d: 10000, ev: [midi(0, 0xc0, 16), midi(0, 0x90, 60, 90)] });
    const early = w.render(0.1);
    const heldEarly = w.stats();
    const reply = await w.soundfont(bank());
    let fmEver = 0;
    let started = -1;
    for (let i = 0; i < 400 && started < 0; i++) {
      w.render(BLOCK / RATE);
      const st = w.stats();
      fmEver = Math.max(fmEver, st.voices);
      if (st.synths > 0) started = i;
    }
    const resident = w.render(0.2);
    const residentStats = w.stats();
    w.post({ t: "stop", h: 0 });
    w.settle();
    const afterResident = w.decodedSamples();
    w.render(1.3);
    w.post({ t: "play", h: 1, r: false, d: 600, ev: song() });
    const first = w.stats(); // the play message itself decodes nothing: it is held and queued
    // Quantum by quantum: at most one work item per `process()`, and nothing on FM while held.
    let maxPerQuantum = 0;
    let heldQuanta = 0;
    // Until it has played 0.4 s (HOLD_MAX_MS bounds the wait).
    for (let i = 0, left = first.work, playing = 0; i < (3.5 * RATE) / BLOCK && playing < (0.4 * RATE) / BLOCK; i++) {
      const block = w.samples(BLOCK / RATE);
      const now = w.stats();
      if (now.held) heldQuanta++;
      else playing++;
      fmEver = Math.max(fmEver, now.voices);
      maxPerQuantum = Math.max(maxPerQuantum, left - now.work);
      left = now.work;
      if (now.held && rmsOf(block) > 0) fmEver = Math.max(fmEver, 1); // anything audible while held
    }
    const decoded = w.decodedSamples();
    const sfStats = w.stats();
    w.render(1.3);
    w.post({ t: "play", h: 1, r: false, d: 600 });
    const again = w.samples(0.4);
    const againStats = w.stats();
    check(
      "soundfont session: the first play of each song waits for the soundfont and plays on it — never FM",
      reply.ok === true && early === 0 && heldEarly.held === 1 && heldEarly.soundfont === "pending" && started >= 0 && resident > LOUD && residentStats.synths === 1 &&
        first.held === 1 && first.synths === 0 && first.work > 0 && heldQuanta > 0 && maxPerQuantum === 1 &&
        sfStats.held === 0 && sfStats.synths >= 1 && heldQuanta < (3 * RATE) / BLOCK && againStats.synths >= 1 && againStats.voices === 0 && rmsOf(again) > LOUD && fmEver === 0,
      `before parse: rms ${early} held ${heldEarly.held} (${heldEarly.soundfont}) · parse ${reply.ms} ms · resident song started on the soundfont after ${started} quanta (rms ${resident.toFixed(4)}) · ` +
        `new instrument held ${heldQuanta} quanta (${((heldQuanta * BLOCK * 1000) / RATE).toFixed(0)} ms) (≤ ${maxPerQuantum} work item per quantum) · next play synths ${againStats.synths} · FM voices ever ${fmEver}`,
    );
    // Memory: only the samples the notes reach — what the synth's own lazy decoding would have kept.
    const piano = w.presetSamples(0);
    check("decoding stops at the samples the notes reach, not the whole instrument", decoded > afterResident && decoded < piano, `decoded samples ${decoded} · the piano preset holds ${piano}`);
  }

  // 19. A soundfont session that loses the soundfont is FM from start to end — never some songs one
  //     way and some the other. ⒜ audio.rs reports a failure (`sfoff`): the held play starts at once,
  //     on FM. ⒝ the file does not parse. ⒞ nothing arrives within HOLD_MAX_MS: the held play starts on
  //     FM then, and a soundfont arriving afterwards is refused. Each run is FM sample-for-sample.
  {
    const fmRef = boot();
    fmRef.post({ t: "play", h: 7, r: false, d: 600, ev: song() });
    const fm = fmRef.samples(0.4);
    const off = boot(prelude);
    off.post({ t: "sfwait" });
    off.post({ t: "play", h: 7, r: false, d: 600, ev: song() });
    off.render(0.05);
    off.post({ t: "sfoff" });
    off.render(0.1);
    const offFirst = off.stats();
    off.render(3); // the first play's FM release has died away: the next play is compared alone
    off.post({ t: "play", h: 7, r: false, d: 600 });
    const offOut = off.samples(0.4);
    const bad = boot(prelude);
    bad.post({ t: "sfwait" });
    bad.post({ t: "play", h: 7, r: false, d: 600, ev: song() });
    const badReply = await bad.soundfont(new Uint8Array(4096).fill(7).buffer);
    const badStats = bad.stats();
    bad.render(3);
    bad.post({ t: "play", h: 7, r: false, d: 600 });
    const badOut = bad.samples(0.4);
    const late = boot(prelude);
    late.post({ t: "sfwait" });
    late.post({ t: "play", h: 7, r: false, d: 600, ev: song() });
    const silent = late.render(2.9);
    const heldAt29 = late.stats().held;
    late.render(0.2);
    const lateStats = late.stats();
    const lateReply = await late.soundfont(bank());
    late.render(3);
    late.post({ t: "play", h: 7, r: false, d: 600 });
    const lateOut = late.samples(0.4);
    const lateEnd = late.stats();
    check(
      "a soundfont session that loses it (sfoff / unparsable / too late) is FM for every play",
      offFirst.soundfont === "failed" && offFirst.held === 0 && offFirst.voices > 0 && offFirst.synths === 0 && same(offOut, fm),
      `sfoff: state ${offFirst.soundfont} · held play released on FM voices ${offFirst.voices} synths ${offFirst.synths} · next play = FM ${same(offOut, fm)}`,
    );
    check(
      "…unparsable: the held play is released on FM, and the next play is FM sample-for-sample",
      badReply.ok === false && badStats.soundfont === "failed" && badStats.held === 0 && same(badOut, fm),
      `reply ${badReply.ok} · state ${badStats.soundfont} · held ${badStats.held} · next play = FM ${same(badOut, fm)}`,
    );
    check(
      "…too late: held (silent) until HOLD_MAX_MS, then FM; a soundfont arriving after that is refused",
      silent === 0 && heldAt29 === 1 && lateStats.held === 0 && lateStats.soundfont === "failed" && lateStats.voices > 0 &&
        lateReply.ok === false && lateEnd.synths === 0 && same(lateOut, fm),
      `held at 2.9 s ${heldAt29} (rms ${silent}) · at 3.1 s held ${lateStats.held} state ${lateStats.soundfont} FM voices ${lateStats.voices} · late soundfont ${lateReply.ok} (${lateReply.error}) · next play = FM ${same(lateOut, fm)}`,
    );
  }

  // Instruments later cases use, decoded the way a game gets them decoded: one FM play, then the work.
  const warm = (ev) => {
    w.post({ t: "play", h: 99, r: false, d: 100, ev });
    w.render(0.1);
    w.post({ t: "stop", h: 99 });
    w.post({ t: "evict", h: 99 });
    w.settle();
  };
  warm([midi(0, 0xc0, 48), ...[55, 59, 60, 62, 64].map((n) => midi(0, 0x90, n, 110)), midi(0, 0xc0, 40), midi(0, 0x90, 50, 100), midi(0, 0xc0, 0), midi(0, 0x90, 64, 110), midi(0, 0x90, 67, 100)]);

  // 13. A non-repeating soundfont play ends: its synth plays out its release and tail, then is pooled.
  {
    w.render(3.0);
    const after = w.render(0.3);
    check("a soundfont play ends, then its synth leaves the render list", after < QUIET && w.stats().synths === 0, `rms ${after.toExponential(1)} · synths ${w.stats().synths}`);
  }

  // 14. Stop silences a looping soundfont play within the FM release time. Its synth keeps rendering
  //     unheard for SF_TAIL_S — so its reverb drains before the pool hands it to another play — and
  //     is then back in the pool.
  {
    w.post({ t: "play", h: 3, r: true, d: 10000, ev: [midi(0, 0xc0, 48), midi(0, 0x90, 64, 110)] });
    const before = w.render(0.3);
    w.post({ t: "stop", h: 3 });
    w.render(0.1);
    const after = w.render(0.3);
    const draining = w.stats().synths;
    w.render(1.0);
    const stats = w.stats();
    check(
      "Stop silences a soundfont play; its synth drains unheard, then returns to the pool",
      before > LOUD && after < QUIET && draining === 1 && stats.synths === 0 && stats.idle >= 1,
      `rms before ${before.toFixed(4)} · after ${after.toExponential(1)} · draining ${draining} → rendering ${stats.synths} idle ${stats.idle}`,
    );
  }

  // 15. The game's gain scales a soundfont play, and PCM in the same sequence still plays.
  {
    const level = (gain) => {
      if (gain !== undefined) w.post({ t: "gain", h: 4, g: gain });
      w.post({ t: "play", h: 4, r: true, d: 10000, ev: [midi(0, 0xc0, 48), midi(0, 0x90, 64, 110)] });
      w.render(0.2);
      const rms = w.render(0.2);
      w.post({ t: "stop", h: 4 });
      w.render(1.3); // back in the pool: a synth that would have to be built makes the play FM
      return rms;
    };
    const ratio = level(0.5) / level(undefined);
    const pcm = new Int16Array(4000);
    for (let i = 0; i < pcm.length; i++) pcm[i] = Math.round(12000 * Math.sin((2 * Math.PI * 440 * i) / 8000));
    w.post({ t: "play", h: 5, r: false, d: 0, ev: [[0, 1, 1, 8000, pcm]] });
    const pcmLevel = w.render(0.2);
    check("gain scales a soundfont play; PCM still plays beside it", Math.abs(ratio - 0.5) < 0.05 && pcmLevel > LOUD, `gain 0.5 → ×${ratio.toFixed(3)} · pcm rms ${pcmLevel.toFixed(4)}`);
  }

  // The synths are bounded and reused (docs/report 0355). `drain` outlasts a stopped synth's fade +
  // SF_TAIL_S, after which it is back in the idle pool.
  const drain = () => w.render(1.3);
  const loop = (h, note) => w.post({ t: "play", h, r: true, d: 10000, ev: [midi(0, 0xc0, 48), midi(0, 0x90, note, 110)] });
  drain();

  // 16. Reuse: many plays in a row construct no new synth once the pool holds one — building one
  //     allocates enough to drop an audio quantum (measured median 3.3 ms vs a 2.67 ms quantum).
  {
    const builtBefore = w.stats().built;
    for (let i = 0; i < 8; i++) {
      loop(20 + i, 60);
      w.render(0.1);
      w.post({ t: "stop", h: 20 + i });
      drain();
    }
    const stats = w.stats();
    check(
      "sequential soundfont plays reuse one pooled synth",
      stats.built === builtBefore && stats.synths === 0 && stats.idle >= 1,
      `built ${builtBefore} → ${stats.built} over 8 plays · rendering ${stats.synths} · idle ${stats.idle}`,
    );
  }

  // 17. A reused synth starts from a fresh MIDI state: a play that muted channel 0 (CC7 = 0), changed
  //     its program and bent it must not carry into the next play that gets the same synth (the
  //     pool hands back the most recently returned one).
  {
    const probe = (h) => {
      w.post({ t: "play", h, r: true, d: 10000, ev: [midi(0, 0x90, 64, 110)] }); // no program, no CC: defaults
      w.render(0.2);
      const rms = w.render(0.2);
      w.post({ t: "stop", h });
      drain();
      return rms;
    };
    const clean = probe(30);
    w.post({ t: "play", h: 31, r: true, d: 10000, ev: [midi(0, 0xc0, 40), midi(0, 0xb0, 7, 0), midi(0, 0xe0, 0, 0), midi(0, 0x90, 50, 100)] });
    w.render(0.2);
    w.post({ t: "stop", h: 31 });
    drain();
    const after = probe(32);
    check("a reused synth does not inherit the previous play's MIDI state", clean > LOUD && Math.abs(after / clean - 1) < 0.1, `rms fresh state ${clean.toFixed(4)} · after a muting play ${after.toFixed(4)} (×${(after / clean).toFixed(3)})`);
  }

  // 18. The cap: MAX_SF_SYNTHS (3) render at once. A play past it takes a synth that is only playing
  //     its tail, and with every synth live it plays FM — it still sounds, and the count never grows.
  {
    // Three synths exist first: a play that needs one built is FM (case 12's rule, for synths).
    for (const h of [40, 41, 42]) loop(h, 60);
    w.settle();
    for (const h of [40, 41, 42]) w.post({ t: "stop", h });
    drain();
    loop(40, 55);
    loop(41, 59);
    w.post({ t: "play", h: 42, r: false, d: 100, ev: [midi(0, 0xc0, 0), midi(0, 0x90, 67, 100), midi(100, 0x80, 67, 0)] });
    w.render(0.5); // 42 has ended; its synth is in its tail
    const tailing = w.stats();
    loop(43, 62); // takes 42's synth
    w.render(0.2);
    const stolen = w.stats();
    loop(44, 64); // 40, 41, 43 all live: FM
    w.render(0.1);
    const fm = w.render(0.2);
    const full = w.stats();
    for (const h of [40, 41, 43, 44]) w.post({ t: "stop", h });
    drain();
    const done = w.stats();
    check(
      "at most 3 soundfont synths render; past that a tail is taken, then FM",
      tailing.synths === 3 && stolen.synths === 3 && stolen.voices === 0 && full.synths === 3 && full.voices > 0 && fm > LOUD && full.built <= 3 && done.synths === 0 && done.idle === full.built,
      `with a tail ${tailing.synths} · after taking it ${stolen.synths} (FM voices ${stolen.voices}) · 4 live plays ${full.synths} synths + ${full.voices} FM voices (rms ${fm.toFixed(4)}) · built ${full.built} · after stop: rendering ${done.synths} idle ${done.idle}`,
    );
  }
}

if (failed) {
  console.error(`check-audio-worklet: ${failed} case(s) failed`);
  process.exit(1);
}
console.log("check-audio-worklet: OK");
