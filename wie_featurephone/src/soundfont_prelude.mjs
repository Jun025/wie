// Entry point of the soundfont prelude: spessasynth_core, bundled by `scripts/build-soundfont-prelude.mjs`
// into one IIFE that runs in the AudioWorkletGlobalScope *before* `audio_worklet.js` (audio.rs
// loads both strings as one Blob module). It only publishes the three things the worklet uses.
// The worklet treats `globalThis.wieSoundfont` as optional — a build without this prelude plays FM.
import { BasicSoundBank, SoundBankLoader, SpessaLog, SpessaSynthProcessor } from "spessasynth_core";

// The library logs through `console` on warnings; the audio thread has nowhere useful to send them.
SpessaLog.warnEnabled = false;

globalThis.wieSoundfont = {
  // Searched for in the built wasm by scripts/check-engine-contract.mjs (contract `soundfontPrelude`).
  marker: "wie-soundfont-prelude-v1",
  SpessaSynthProcessor,
  SoundBankLoader,
  // The sf3 (Vorbis) decoder initializes asynchronously; parsing waits on it.
  ready: BasicSoundBank.isSF3DecoderReady,
};
