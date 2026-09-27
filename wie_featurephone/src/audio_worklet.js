// wie_featurephone audio worklet — MIDI synthesis + PCM playback off the main thread.
//
// Loaded by `audio.rs` from a Blob URL (the engine artifact is exactly two files, so this source
// travels inside the wasm as a string rather than as a third file). `audio.rs` posts one message
// per `AudioCommand`:
//   { t: "play", h: handle, r: repeat, d: durationMs, ev?: [[timeMs, 0, Uint8Array midi] |
//                                                          [timeMs, 1, channels, rate, Int16Array]] }
//   { t: "stop", h: handle }
//   { t: "evict", h: handle }   — forget the handle's sequence; audio.rs resends `ev` on its next play
//   { t: "gain", h: handle, g } — the game's volume for that handle (0..1): its playback now, else its next play
//   { t: "stats" }              — replies { t: "stats", sequences, playbacks, voices } on the port
// `ev` rides only on the first play of a handle (a handle's sequence never changes); later plays
// reuse it. audio.rs keeps at most RESIDENT_SEQUENCES handles here and evicts the least recently
// played, because nothing tells it when a handle is retired. `stats` is for measuring that from a
// page (nothing in the engine asks). Everything below runs on the audio thread, so timing is sample-accurate and does not
// depend on the emulator's tick rate.
//
// The synth is deliberately small: one 2-operator FM voice per note with a patch per General MIDI
// family (smaf_player already maps SMAF tones onto GM programs), and a synthesized kit on MIDI
// channel 10. It is not a soundfont — see docs/report for why the soundfont path was not taken.
//
// `scripts/check-audio-worklet.mjs` runs this file under node with stub globals; keep it free of
// anything but the AudioWorkletGlobalScope surface (`registerProcessor`, `sampleRate`,
// `currentFrame`, `AudioWorkletProcessor`).

const TABLE_BITS = 12;
const TABLE_SIZE = 1 << TABLE_BITS;
const TABLE_MASK = TABLE_SIZE - 1;
const SINE = new Float32Array(TABLE_SIZE);
for (let i = 0; i < TABLE_SIZE; i++) SINE[i] = Math.sin((2 * Math.PI * i) / TABLE_SIZE);

const MAX_VOICES = 48;
const STOP_RELEASE_S = 0.015;
const MIN_LOOP_MS = 20;
const VOICE_GAIN = 0.28;
const PCM_GAIN = 0.9;
const SILENT = 1e-4;

// One patch per GM family (program >> 3). ratio/index: FM modulator; idecay: modulator index
// time constant (0 = constant); isus: index floor as a fraction; a: attack s; d: decay time
// constant s; s: sustain level (0 = decays away like a struck string); r: release time constant s.
const PATCHES = [
  { ratio: 1, index: 1.2, idecay: 0.5, isus: 0.1, a: 0.002, d: 1.2, s: 0, r: 0.25, lvl: 1 }, // piano
  { ratio: 3.5, index: 2, idecay: 0.25, isus: 0, a: 0.001, d: 0.7, s: 0, r: 0.3, lvl: 0.8 }, // chromatic perc
  { ratio: 1, index: 0.9, idecay: 0, isus: 1, a: 0.01, d: 0.1, s: 0.9, r: 0.06, lvl: 0.7 }, // organ
  { ratio: 1, index: 1.6, idecay: 0.25, isus: 0.1, a: 0.002, d: 0.9, s: 0, r: 0.15, lvl: 0.9 }, // guitar
  { ratio: 1, index: 1.3, idecay: 0.15, isus: 0.3, a: 0.003, d: 0.6, s: 0.35, r: 0.08, lvl: 1 }, // bass
  { ratio: 1, index: 0.7, idecay: 0, isus: 1, a: 0.08, d: 0.3, s: 0.85, r: 0.25, lvl: 0.7 }, // strings
  { ratio: 1, index: 0.6, idecay: 0, isus: 1, a: 0.12, d: 0.3, s: 0.85, r: 0.35, lvl: 0.7 }, // ensemble
  { ratio: 1, index: 2.2, idecay: 0.2, isus: 0.5, a: 0.03, d: 0.2, s: 0.8, r: 0.12, lvl: 0.8 }, // brass
  { ratio: 2, index: 1.4, idecay: 0, isus: 1, a: 0.03, d: 0.2, s: 0.85, r: 0.1, lvl: 0.7 }, // reed
  { ratio: 1, index: 0.35, idecay: 0, isus: 1, a: 0.04, d: 0.2, s: 0.8, r: 0.12, lvl: 0.8 }, // pipe
  { ratio: 1, index: 2.5, idecay: 0, isus: 1, a: 0.005, d: 0.1, s: 0.85, r: 0.08, lvl: 0.6 }, // synth lead
  { ratio: 1, index: 1, idecay: 0, isus: 1, a: 0.2, d: 0.5, s: 0.9, r: 0.5, lvl: 0.6 }, // synth pad
  { ratio: 1.41, index: 2, idecay: 0.5, isus: 0.3, a: 0.05, d: 0.5, s: 0.6, r: 0.5, lvl: 0.6 }, // synth fx
  { ratio: 1, index: 2, idecay: 0.2, isus: 0.1, a: 0.002, d: 0.7, s: 0, r: 0.2, lvl: 0.8 }, // ethnic
  { ratio: 1.5, index: 2, idecay: 0.1, isus: 0, a: 0.001, d: 0.35, s: 0, r: 0.15, lvl: 0.9 }, // percussive
  { ratio: 3.3, index: 5, idecay: 0.3, isus: 0.2, a: 0.005, d: 0.5, s: 0, r: 0.3, lvl: 0.5 }, // sound fx
];

let noiseState = 0x12345678;
function noise() {
  noiseState ^= noiseState << 13;
  noiseState ^= noiseState >>> 17;
  noiseState ^= noiseState << 5;
  return (noiseState | 0) / 2147483648;
}

function midiFreq(note) {
  return 440 * Math.pow(2, (note - 69) / 12);
}

function newChannels() {
  const channels = [];
  for (let i = 0; i < 16; i++) {
    channels.push({ program: 0, volume: 100 / 127, expression: 1, pan: 0.5, bend: 0, bendRange: 2, rpn: 0x3fff, sustain: false });
  }
  return channels;
}

// Drum voice parameters on channel 10, by GM key.
function drumPatch(note) {
  if (note === 35 || note === 36) return { tone: 150, toneEnd: 45, sweep: 0.04, tdecay: 0.3, noise: 0, ndecay: 0, hp: false, lvl: 1.4 };
  if (note === 38 || note === 40 || note === 37 || note === 39)
    return { tone: 190, toneEnd: 160, sweep: 0.02, tdecay: 0.08, noise: 0.8, ndecay: 0.14, hp: false, lvl: 1 };
  if (note === 42 || note === 44) return { tone: 0, noise: 0.6, ndecay: 0.04, hp: true, lvl: 0.7 };
  if (note === 46) return { tone: 0, noise: 0.6, ndecay: 0.25, hp: true, lvl: 0.6 };
  if (note === 49 || note === 52 || note === 55 || note === 57) return { tone: 0, noise: 0.6, ndecay: 1, hp: true, lvl: 0.55 };
  if (note === 51 || note === 53 || note === 59) return { tone: 0, noise: 0.45, ndecay: 0.5, hp: true, lvl: 0.5 };
  if (note >= 41 && note <= 50) {
    const f = 70 + (note - 41) * 18;
    return { tone: f * 1.4, toneEnd: f, sweep: 0.05, tdecay: 0.3, noise: 0.1, ndecay: 0.05, hp: false, lvl: 1 };
  }
  return { tone: 400, toneEnd: 300, sweep: 0.02, tdecay: 0.06, noise: 0.4, ndecay: 0.06, hp: true, lvl: 0.6 };
}

class WieAudioProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.sequences = new Map(); // handle -> { events: [{ f, midi? , pcm? }], lengthFrames }
    this.playbacks = new Map(); // handle -> { seq, start, repeat, next, channels, gain }
    this.gains = new Map(); // handle -> gain for its next play (backend Audio sends one before every play)
    this.voices = [];
    this.port.onmessage = (event) => this.onMessage(event.data);
  }

  onMessage(message) {
    if (message.t === "play") this.play(message);
    else if (message.t === "stop") this.stop(message.h);
    // A playing sequence keeps sounding: its playback holds the sequence object itself.
    else if (message.t === "evict") {
      this.sequences.delete(message.h);
      this.gains.delete(message.h);
    } else if (message.t === "gain") {
      const playback = this.playbacks.get(message.h);
      if (playback) playback.gain = message.g;
      else this.gains.set(message.h, message.g);
    }
    else if (message.t === "stats")
      this.port.postMessage({ t: "stats", sequences: this.sequences.size, playbacks: this.playbacks.size, voices: this.voices.length });
  }

  load(handle, durationMs, events) {
    const rate = sampleRate / 1000;
    let lengthMs = durationMs;
    const out = [];
    for (const event of events) {
      if (event[1] === 0) {
        out.push({ f: Math.round(event[0] * rate), midi: event[2] });
      } else {
        const channels = Math.max(1, event[2]);
        const srcRate = event[3];
        const samples = event[4];
        if (!srcRate || !samples || samples.length < channels) continue;
        lengthMs = Math.max(lengthMs, event[0] + ((samples.length / channels) * 1000) / srcRate);
        out.push({ f: Math.round(event[0] * rate), pcm: { channels, srcRate, samples } });
      }
    }
    this.sequences.set(handle, { events: out, lengthFrames: Math.round(Math.max(lengthMs, MIN_LOOP_MS) * rate) });
  }

  play(message) {
    if (message.ev) this.load(message.h, message.d, message.ev);
    const seq = this.sequences.get(message.h);
    if (!seq) return;
    this.stop(message.h);
    const gain = this.gains.get(message.h) ?? 1;
    this.gains.delete(message.h);
    this.playbacks.set(message.h, { seq, start: currentFrame, repeat: !!message.r, next: 0, channels: newChannels(), gain });
  }

  stop(handle) {
    this.playbacks.delete(handle);
    for (const voice of this.voices) {
      if (voice.handle === handle) this.release(voice, STOP_RELEASE_S);
    }
  }

  // Releasing an already-releasing voice only ever shortens the tail — so `stop` cuts a slow
  // pad release short instead of leaving it to ring out.
  release(voice, releaseSeconds) {
    const rcoef = Math.exp(-1 / (Math.max(releaseSeconds, 0.001) * sampleRate));
    if (voice.stage === 3 && voice.rcoef <= rcoef) return;
    voice.stage = 3;
    voice.rcoef = rcoef;
  }

  dispatch(handle, playback, event) {
    if (event.pcm) {
      this.addVoice({ handle, pb: playback, kind: 2, pcm: event.pcm, pos: 0, step: event.pcm.srcRate / sampleRate, env: 1, stage: 1 });
      return;
    }
    const data = event.midi;
    const status = data[0];
    if (status < 0x80 || status >= 0xf0) return;
    const ch = status & 0x0f;
    const c = playback.channels[ch];
    const type = status & 0xf0;
    const d1 = data[1] ?? 0;
    const d2 = data[2] ?? 0;
    if (type === 0x90 && d2 > 0) {
      this.noteOn(handle, playback, ch, c, d1, d2);
    } else if (type === 0x80 || type === 0x90) {
      for (const voice of this.voices) {
        if (voice.handle === handle && voice.ch === ch && voice.note === d1 && voice.stage < 3 && !voice.held) {
          if (c.sustain) voice.held = true;
          else this.release(voice, voice.patch ? voice.patch.r : 0.05);
        }
      }
    } else if (type === 0xc0) {
      c.program = d1;
    } else if (type === 0xe0) {
      c.bend = (((d2 << 7) | d1) - 8192) / 8192;
    } else if (type === 0xb0) {
      this.controlChange(handle, ch, c, d1, d2);
    }
  }

  controlChange(handle, ch, c, control, value) {
    if (control === 7) c.volume = value / 127;
    else if (control === 11) c.expression = value / 127;
    else if (control === 10) c.pan = value / 127;
    else if (control === 101) c.rpn = (c.rpn & 0x7f) | (value << 7);
    else if (control === 100) c.rpn = (c.rpn & 0x3f80) | value;
    else if (control === 6 && c.rpn === 0) c.bendRange = value;
    else if (control === 64) {
      c.sustain = value >= 64;
      if (!c.sustain) {
        for (const voice of this.voices) {
          if (voice.handle === handle && voice.ch === ch && voice.held) {
            voice.held = false;
            this.release(voice, voice.patch ? voice.patch.r : 0.05);
          }
        }
      }
    } else if (control === 120 || control === 123) {
      for (const voice of this.voices) {
        if (voice.handle === handle && voice.ch === ch) this.release(voice, control === 120 ? STOP_RELEASE_S : 0.05);
      }
    } else if (control === 121) {
      c.expression = 1;
      c.bend = 0;
      c.sustain = false;
    }
  }

  noteOn(handle, pb, ch, c, note, velocity) {
    // Retrigger: a repeated key on the same channel cuts the previous one short.
    for (const voice of this.voices) {
      if (voice.handle === handle && voice.ch === ch && voice.note === note) this.release(voice, STOP_RELEASE_S);
    }
    const amp = (velocity / 127) * VOICE_GAIN;
    if (ch === 9) {
      this.addVoice({ handle, pb, ch, note, kind: 1, drum: drumPatch(note), t: 0, amp, phase: 0, env: 1, stage: 1, lp: 0 });
      return;
    }
    const patch = PATCHES[(c.program >> 3) & 15];
    this.addVoice({
      handle,
      pb,
      ch,
      note,
      kind: 0,
      patch,
      amp: amp * patch.lvl,
      phase: 0,
      mphase: 0,
      env: 0,
      ienv: 1,
      stage: 0,
      acoef: 1 / (Math.max(patch.a, 0.001) * sampleRate),
      dcoef: Math.exp(-1 / (patch.d * sampleRate)),
      icoef: patch.idecay > 0 ? Math.exp(-1 / (patch.idecay * sampleRate)) : 1,
      held: false,
    });
  }

  addVoice(voice) {
    if (this.voices.length >= MAX_VOICES) {
      let victim = 0;
      for (let i = 1; i < this.voices.length; i++) {
        const a = this.voices[i];
        const b = this.voices[victim];
        if (a.stage === 3 && b.stage !== 3) victim = i;
        else if ((a.stage === 3) === (b.stage === 3) && a.env < b.env) victim = i;
      }
      this.voices.splice(victim, 1);
    }
    this.voices.push(voice);
  }

  advance(blockEnd) {
    for (const [handle, playback] of this.playbacks) {
      const seq = playback.seq;
      for (;;) {
        const event = seq.events[playback.next];
        if (event && playback.start + event.f < blockEnd) {
          playback.next++;
          this.dispatch(handle, playback, event);
          continue;
        }
        if (event || playback.start + seq.lengthFrames >= blockEnd) break;
        // Sequence exhausted and its length has elapsed.
        if (!playback.repeat) {
          this.playbacks.delete(handle);
          for (const voice of this.voices) {
            if (voice.handle === handle && voice.kind !== 2) this.release(voice, voice.patch ? voice.patch.r : 0.05);
          }
          break;
        }
        playback.start += seq.lengthFrames;
        if (playback.start + seq.lengthFrames < blockEnd - 128) playback.start = blockEnd - 128;
        playback.next = 0;
      }
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0];
    const left = out[0];
    const right = out[1] ?? out[0];
    const frames = left.length;
    this.advance(currentFrame + frames);

    left.fill(0);
    if (right !== left) right.fill(0);

    const alive = [];
    for (const voice of this.voices) {
      if (this.render(voice, left, right, frames)) alive.push(voice);
    }
    this.voices = alive;

    for (let i = 0; i < frames; i++) {
      left[i] = Math.tanh(left[i]);
      if (right !== left) right[i] = Math.tanh(right[i]);
    }
    return true;
  }

  // Returns false once the voice has gone silent for good.
  render(voice, left, right, frames) {
    if (voice.kind === 2) return this.renderPcm(voice, left, right, frames);

    // A stopped handle's playback (and its channel state) is gone while its voices fade out, so
    // the last gains are kept on the voice rather than jumping to a default mid-release.
    const playback = this.playbacks.get(voice.handle);
    const c = playback ? playback.channels[voice.ch] : null;
    if (c) {
      const chGain = voice.amp * c.volume * c.expression;
      voice.gl = chGain * Math.cos((c.pan * Math.PI) / 2);
      voice.gr = chGain * Math.sin((c.pan * Math.PI) / 2);
      voice.bend = c.bend * c.bendRange;
    } else if (voice.gl === undefined) {
      voice.gl = voice.gr = voice.amp * Math.SQRT1_2;
      voice.bend = 0;
    }
    // The game's volume. `pb` is the playback object itself, so a stopped handle's release tail
    // keeps the gain it had rather than jumping to 1.
    const gl = voice.gl * voice.pb.gain;
    const gr = voice.gr * voice.pb.gain;

    if (voice.kind === 1) return this.renderDrum(voice, left, right, frames, gl, gr);

    const patch = voice.patch;
    const freq = midiFreq(voice.note + voice.bend);
    const inc = (freq * TABLE_SIZE) / sampleRate;
    const minc = inc * patch.ratio;
    const indexScale = (patch.index * TABLE_SIZE) / (2 * Math.PI);
    let { phase, mphase, env, ienv, stage } = voice;
    for (let i = 0; i < frames; i++) {
      if (stage === 0) {
        env += voice.acoef;
        if (env >= 1) {
          env = 1;
          stage = 1;
        }
      } else if (stage === 1) {
        env = patch.s + (env - patch.s) * voice.dcoef;
      } else {
        env *= voice.rcoef;
      }
      ienv *= voice.icoef;
      const index = indexScale * (patch.isus + (1 - patch.isus) * ienv);
      const mod = SINE[mphase & TABLE_MASK] * index;
      const sample = SINE[(phase + mod) & TABLE_MASK] * env;
      left[i] += sample * gl;
      right[i] += sample * gr;
      phase += inc;
      mphase += minc;
    }
    voice.phase = phase % TABLE_SIZE;
    voice.mphase = mphase % TABLE_SIZE;
    voice.env = env;
    voice.ienv = ienv;
    voice.stage = stage;
    if (stage === 3 && env < SILENT) return false;
    if (stage === 1 && patch.s === 0 && env < SILENT) return false;
    return true;
  }

  renderDrum(voice, left, right, frames, gl, gr) {
    const d = voice.drum;
    const dt = 1 / sampleRate;
    let { t, phase, lp } = voice;
    let level = 0;
    for (let i = 0; i < frames; i++) {
      let sample = 0;
      if (d.tone > 0) {
        const f = d.toneEnd + (d.tone - d.toneEnd) * Math.exp(-t / d.sweep);
        phase += (f * TABLE_SIZE) / sampleRate;
        sample += SINE[phase & TABLE_MASK] * Math.exp(-t / d.tdecay);
      }
      if (d.noise > 0) {
        const n = noise();
        const shaped = d.hp ? n - lp : n;
        lp = n;
        sample += shaped * d.noise * Math.exp(-t / d.ndecay);
      }
      if (voice.stage === 3) voice.env *= voice.rcoef;
      sample *= d.lvl * voice.env;
      left[i] += sample * gl;
      right[i] += sample * gr;
      level = Math.max(level, Math.abs(sample));
      t += dt;
    }
    voice.t = t;
    voice.phase = phase % TABLE_SIZE;
    voice.lp = lp;
    const longest = Math.max(d.tone > 0 ? d.tdecay : 0, d.noise > 0 ? d.ndecay : 0);
    return voice.env > SILENT && t < longest * 10;
  }

  renderPcm(voice, left, right, frames) {
    const { channels, samples } = voice.pcm;
    const total = samples.length / channels;
    let pos = voice.pos;
    for (let i = 0; i < frames; i++) {
      const index = pos | 0;
      if (index + 1 >= total) {
        voice.pos = total;
        return false;
      }
      if (voice.stage === 3) {
        voice.env *= voice.rcoef;
        if (voice.env < SILENT) return false;
      }
      const frac = pos - index;
      const g = (PCM_GAIN * voice.pb.gain * voice.env) / 32768;
      const l0 = samples[index * channels];
      const l1 = samples[(index + 1) * channels];
      const l = (l0 + (l1 - l0) * frac) * g;
      let r = l;
      if (channels > 1) {
        const r0 = samples[index * channels + 1];
        const r1 = samples[(index + 1) * channels + 1];
        r = (r0 + (r1 - r0) * frac) * g;
      }
      left[i] += l;
      right[i] += r;
      pos += voice.step;
    }
    voice.pos = pos;
    return true;
  }
}

// A second module load into the same AudioContext (a second sink on a reused context) would make
// registerProcessor throw NotSupportedError, reject addModule, and drop that sink to the MIDI-
// silent fallback. The scope is shared by every module in the context, so a flag on it is enough;
// the processor registered first is the same code.
if (!globalThis.wieAudioRegistered) {
  registerProcessor("wie-audio", WieAudioProcessor);
  globalThis.wieAudioRegistered = true;
}
