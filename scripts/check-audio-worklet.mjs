#!/usr/bin/env node
// Behaviour check for wie_featurephone/src/audio_worklet.js — the featurephone host's synth.
//
// Runs the worklet under node with stub AudioWorkletGlobalScope globals, drives it with the same
// messages audio.rs posts, and measures output RMS. Every case here is a thing the host could not
// do before 2026-09-27 (MIDI was dropped, `repeat` ignored, `Stop` a no-op), so each assertion is
// the "revert it and this goes red" guard for one of them. Offline, no browser, ~0.5 s.
//
//   node scripts/check-audio-worklet.mjs      # rc 0 = all cases hold, rc 1 = a case failed
import { readFile } from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = await readFile(path.join(root, "wie_featurephone/src/audio_worklet.js"), "utf8");
const RATE = 48000;
const BLOCK = 128;

function boot() {
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
  };
  vm.createContext(ctx);
  // Each load gets its own scope, as each addModule gets its own module scope; the global (the
  // AudioWorkletGlobalScope) is shared.
  const load = () => vm.runInContext(`(function () {\n${source}\n})();`, ctx);
  load();
  const proc = new Processor();
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

if (failed) {
  console.error(`check-audio-worklet: ${failed} case(s) failed`);
  process.exit(1);
}
console.log("check-audio-worklet: OK");
