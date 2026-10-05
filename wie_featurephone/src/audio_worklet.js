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
//   { t: "stats" }              — replies { t: "stats", sequences, playbacks, voices, soundfont, synths, idle, built, work, held } on the port
//   { t: "sfwait" }             — a soundfont is on its way (audio.rs sends it before any play when it has a URL)
//   { t: "sf", data: ArrayBuffer } — the soundfont (sf2/sf3); replies { t: "sf", ok, ms?, error? }
//   { t: "sfoff" }              — it will not come (fetch, HTTP or prelude failed): FM for the session
// `ev` rides only on the first play of a handle (a handle's sequence never changes); later plays
// reuse it. audio.rs keeps at most RESIDENT_SEQUENCES handles here and evicts the least recently
// played, because nothing tells it when a handle is retired. `stats` is for measuring that from a
// page (nothing in the engine asks). Everything below runs on the audio thread, so timing is sample-accurate and does not
// depend on the emulator's tick rate.
//
// The built-in synth is deliberately small: one 2-operator FM voice per note with a patch per General
// MIDI family (smaf_player already maps SMAF tones onto GM programs), and a synthesized kit on MIDI
// channel 10. It needs nothing but this file, so the first sound is never late.
//
// A soundfont is the optional second synth (operator A/B verdict "B is better", docs/report 0317).
// When audio.rs is built with the soundfont prelude (spessasynth_core, published as
// `globalThis.wieSoundfont`) and the host passed a soundfont URL, audio.rs starts fetching it when
// the engine boots, loads the prelude as a SECOND module into this same global scope, and only then
// posts the file here — this module is always loaded alone, so a session without a soundfont never
// waits for the prelude (docs/report 0355). Each MIDI play renders through a SpessaSynthProcessor of
// its own (so each handle keeps its own 16 channels, and Stop and the game's gain stay per handle, as
// with FM). PCM always stays on the path below.
//
// ONE SYNTH PER SESSION: in a session told a soundfont is coming (`sfwait`), every MIDI play renders
// through it, the first included — until 2026-10-04 a song's first play was FM and its next one the
// soundfont, which players heard as the song changing instruments (docs/report 0441). So a MIDI play
// that cannot start on the soundfont yet is HELD — not played on FM — until the soundfont has parsed,
// the samples its notes reach are decoded, and a synth is free; its samples go to the front of the
// work queue. The hold is bounded by HOLD_MAX_MS. If the soundfont has not parsed by then, or it fails
// (`sfoff`, or a parse/build error), the whole session is FM — every play, start to end, as with no
// URL. A soundfont that parses only after that is refused. If it has parsed and only decoding is
// left, the play starts on the soundfont and the synth decodes the rest itself. The only FM play in
// a soundfont session is one past MAX_SF_SYNTHS with every synth live (below).
//
// Nothing slow runs inside a message or a play (docs/report 0359 measured a first soundfont play
// holding the audio thread 156 ms on an Android emulator, over its 90.8 ms output buffer). Parsing,
// building a synth and decoding a sample (sf3 is Vorbis, decoded on first use) are queued as work
// items that `process()` runs one at a time, resting after each (WORK_REST), while anything sounds, and
// back to back for up to SILENT_WORK_MS a quantum while nothing does — exactly the samples a
// play's notes reach, so the memory is what lazy decoding would have used. Sequences already resident
// when the soundfont arrives are queued at once.
//
// Those synths are bounded (MAX_SF_SYNTHS) and reused. Nearly all of a synth's cost is its effects
// (reverb/chorus), paid whether it has 0 voices or 8, so the cost grows with the number of synths,
// not notes; building one also allocates enough to drop an audio quantum. A play that would need a
// synth past the cap, with every synth live, plays FM instead — it still sounds.
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
// Level match with the FM synth, so the hand-over from FM to the soundfont is not a volume drop.
// On the three songs the operator judged (docs/report 0317), the soundfont's raw RMS is 2.00x /
// 2.73x / 2.85x below FM's on the same events; 2.7 is their median, rounded. The listening files were
// matched the same way (to FM's RMS), so this is the level the verdict was given at.
const SF_GAIN = 2.7;
// How long a soundfont synth keeps rendering after its last voice ends: its reverb/chorus tail.
const SF_TAIL_S = 1.0;
// At most this many soundfont synths render at once (sounding, fading, or playing their tail). One
// synth costs ~3-4% of a core in CPU time, ~90% of it effects (docs/report 0355 has the table), so 3
// bounds the soundfont at ~12% on the desktop it was measured on — BGM plus two MIDI effects at once.
// A 4th concurrent MIDI play takes the slot of a synth that is only playing its tail, else plays FM.
const MAX_SF_SYNTHS = 3;
// After a work item that held the thread d ms, the next waits until WORK_REST × d ms of audio has
// been rendered: work takes at most 1/(1+WORK_REST) of the thread, and the output buffer that the
// item drained refills before the next one. The largest single sample decodes in ~2-5 ms on a
// desktop (~15x that on the emulator), so one item stays under a 90 ms buffer where a whole
// instrument did not.
const WORK_REST = 2;
// While nothing sounds — no playback, voice or synth, only plays held for the soundfont — a stall
// cannot be heard, so work runs back to back for up to this long per `process()` and does not rest.
// A held play is otherwise as slow to start as one item per rest makes it, and a game that stops a
// short sound before it starts loses that sound entirely (docs/report 0441 measured 56 such plays
// in 38 titles with one item per rest, against 7 without holding).
const SILENT_WORK_MS = 10;
// The longest a MIDI play is held for the soundfont (see the header). It covers what a held play
// waits for on a slow device — prelude evaluation, parse (median 90 ms, max 187 ms on the Android
// emulator — docs/report 0360) and decoding the samples of a song, one per rest — with the fetch
// already started at boot. Past it the session gives up the soundfont rather than mix synths.
// docs/report 0441 has the measured holds behind the number.
const HOLD_MAX_MS = 3000;

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
    this.soundfont = "none"; // none | pending (sfwait) | loading | ready | failed
    this.bank = null; // the parsed soundfont, shared by every synth
    this.synths = []; // { handle, pb, synth, stopping, fade, tail } — soundfont synths still rendering
    this.idle = []; // reset synths ready for the next play; synths.length + idle.length <= MAX_SF_SYNTHS
    this.built = 0; // synths ever constructed (stats — the pool keeps this at MAX_SF_SYNTHS or below)
    this.building = 0; // synths queued for construction
    this.work = []; // slow soundfont steps, run one per rest from process()
    this.queued = new Set(); // samples already in `work`
    this.rest = 0; // frames to render before the next work item
    this.scratch = null;
    this.held = new Map(); // handle -> { repeat, since } — MIDI plays waiting for the soundfont
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
    else if (message.t === "sf") this.loadSoundfont(message.data);
    else if (message.t === "sfwait") {
      if (this.soundfont === "none") this.soundfont = "pending";
    } else if (message.t === "sfoff") {
      if (this.soundfont === "pending") this.giveUp(null);
    }
    else if (message.t === "stats")
      this.port.postMessage({
        t: "stats",
        sequences: this.sequences.size,
        playbacks: this.playbacks.size,
        voices: this.voices.length,
        soundfont: this.soundfont,
        synths: this.synths.length,
        idle: this.idle.length,
        built: this.built,
        work: this.work.length,
        held: this.held.size,
      });
  }

  // Parses on the audio thread, as spessasynth_lib does: the bank object is not transferable, and a
  // second copy on the main thread would stall the emulator instead. Parsing and the first synth are
  // two work items, so the two stalls do not add up. Anything that goes wrong leaves `bank` null,
  // which is exactly the FM path.
  async loadSoundfont(data) {
    const lib = globalThis.wieSoundfont;
    if (!lib || (this.soundfont !== "none" && this.soundfont !== "pending")) {
      this.port.postMessage({ t: "sf", ok: false, error: lib ? `already ${this.soundfont}` : "no soundfont prelude in this build" });
      return;
    }
    this.soundfont = "loading";
    let ms = 0;
    let bank = null;
    const fail = (error) => this.giveUp(String(error));
    try {
      await lib.ready;
    } catch (error) {
      fail(error);
      return;
    }
    this.work.push(() => {
      if (this.soundfont !== "loading") return; // given up while queued (HOLD_MAX_MS)
      const started = Date.now();
      // The loader formats the file's creation date with toLocaleString for a log line that is
      // switched off, and a thread's first toLocaleString builds an ICU formatter: parse holds the
      // audio thread ~30% longer for it (27-37 ms → 18-21 ms, headless Chromium, docs/report 0360).
      const toLocaleString = Date.prototype.toLocaleString;
      Date.prototype.toLocaleString = Date.prototype.toISOString;
      try {
        bank = lib.SoundBankLoader.fromArrayBuffer(data);
      } catch (error) {
        fail(error);
      } finally {
        Date.prototype.toLocaleString = toLocaleString;
      }
      ms += Date.now() - started;
    });
    // A synth that cannot be built fails here, once, rather than on every later play.
    this.work.push(() => {
      if (!bank || this.soundfont !== "loading") return;
      const started = Date.now();
      try {
        this.idle.push(this.newSynth(lib, bank));
      } catch (error) {
        fail(error);
        return;
      }
      this.bank = bank;
      this.soundfont = "ready";
      this.port.postMessage({ t: "sf", ok: true, ms: ms + Date.now() - started });
      for (const seq of this.sequences.values()) this.decoded(seq);
    });
  }

  // The soundfont will not be used in this session: every held play starts now, on FM, and so does
  // every later one. `error` = reply to audio.rs (null when audio.rs is the one that said so).
  giveUp(error) {
    this.soundfont = "failed";
    if (error !== null) this.port.postMessage({ t: "sf", ok: false, error });
    this.releaseHeld();
  }

  // Runs at most one work item per call — none while resting from the last one — while anything
  // sounds; while nothing does, items up to SILENT_WORK_MS (see there).
  runWork(frames) {
    const silent = this.playbacks.size === 0 && this.voices.length === 0 && this.synths.length === 0;
    if (this.rest > 0 && !silent) {
      this.rest -= frames;
      return;
    }
    const started = Date.now();
    do this.work.shift()();
    while (silent && this.work.length && Date.now() - started < SILENT_WORK_MS);
    this.rest = silent ? 0 : ((Date.now() - started) * WORK_REST * sampleRate) / 1000;
  }

  // True when every sample this sequence's notes reach is decoded; otherwise queues the missing
  // ones. The set is computed once per sequence, from its own program changes and bank selects —
  // the same patch the synth would select (drums on channel 10).
  decoded(seq, urgent = false) {
    if (!seq.samples) {
      const samples = new Set();
      const patches = [];
      for (let ch = 0; ch < 16; ch++) patches.push({ program: 0, bankMSB: 0, bankLSB: 0, isGMGSDrum: ch === 9 });
      const presets = new Map();
      for (const event of seq.events) {
        const data = event.midi;
        if (!data) continue;
        const type = data[0] & 0xf0;
        const patch = patches[data[0] & 0x0f];
        if (type === 0xc0) patch.program = data[1];
        else if (type === 0xb0 && data[1] === 0) patch.bankMSB = data[2];
        else if (type === 0xb0 && data[1] === 32) patch.bankLSB = data[2];
        else if (type === 0x90 && data[2] > 0) {
          const key = `${patch.isGMGSDrum}:${patch.bankMSB}:${patch.bankLSB}:${patch.program}`;
          if (!presets.has(key)) presets.set(key, this.bank.getPreset(patch, "gs"));
          const preset = presets.get(key);
          if (preset) for (const params of preset.getVoiceParameters(data[1], data[2])) samples.add(params.sample);
        }
      }
      seq.samples = [...samples];
    }
    let ready = true;
    for (const sample of seq.samples) {
      if (sample.audioData) continue;
      ready = false;
      if (this.queued.has(sample)) continue;
      this.queued.add(sample);
      const item = () => {
        sample.getAudioData();
        this.queued.delete(sample);
      };
      // A held play waits on these: ahead of everything else (a resident song that nobody plays).
      if (urgent) this.work.unshift(item);
      else this.work.push(item);
    }
    return ready;
  }

  newSynth(lib, bank) {
    const synth = new lib.SpessaSynthProcessor(sampleRate, { eventsEnabled: false });
    synth.soundBankManager.addSoundBank(bank, "main");
    this.built++;
    return synth;
  }

  // True when a play could take a synth now: an idle one, or one only playing its tail. Otherwise,
  // under the cap, a synth is queued for construction (the play waits for it) — one at a time, since
  // this is asked again every quantum a play is held.
  synthAvailable() {
    if (this.idle.length) return true;
    if (this.building) return false;
    if (this.synths.length < MAX_SF_SYNTHS) {
      this.building++;
      this.work.unshift(() => {
        this.building--;
        this.idle.push(this.newSynth(globalThis.wieSoundfont, this.bank));
      });
      return false;
    }
    return true; // at the cap: takeSynth finds a tail, or the play is FM — waiting would not help
  }

  // A synth for a new play: an idle one, a new one while under the cap (built here — a held play
  // past HOLD_MAX_MS), or — at the cap — the one that has been playing only its tail the longest
  // (its reverb ends early). null = play FM.
  takeSynth() {
    if (this.idle.length) return this.idle.pop();
    if (this.synths.length + this.building < MAX_SF_SYNTHS) return this.newSynth(globalThis.wieSoundfont, this.bank);
    // Already silent (stopped and faded) first, then the furthest into its tail.
    const spent = (entry) => (entry.fade === 0 ? 1e12 : 0) + entry.tail;
    let victim = -1;
    for (let i = 0; i < this.synths.length; i++) {
      const entry = this.synths[i];
      if (this.playbacks.get(entry.handle) === entry.pb) continue; // still its handle's live play
      if (victim < 0 || spent(entry) > spent(this.synths[victim])) victim = i;
    }
    if (victim < 0) return null;
    const [entry] = this.synths.splice(victim, 1);
    return this.resetSynth(entry.synth);
  }

  // Back to a fresh synth's MIDI state: the previous play's programs, controllers and notes must not
  // carry into the next one (a song that left channel volume at 0 would silence it).
  resetSynth(synth) {
    synth.stopAllChannels(true);
    synth.reset();
    return synth;
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
    const midi = out.some((event) => event.midi);
    this.sequences.set(handle, { events: out, midi, lengthFrames: Math.round(Math.max(lengthMs, MIN_LOOP_MS) * rate) });
  }

  play(message) {
    if (message.ev) this.load(message.h, message.d, message.ev);
    const seq = this.sequences.get(message.h);
    if (!seq) return;
    this.stop(message.h);
    if (this.mustWait(seq)) this.held.set(message.h, { repeat: !!message.r, since: currentFrame });
    else this.begin(message.h, seq, !!message.r);
  }

  // A MIDI play in a soundfont session that cannot start on the soundfont yet (see the header).
  mustWait(seq) {
    if (!seq.midi) return false;
    if (this.soundfont === "pending" || this.soundfont === "loading") return true;
    if (this.soundfont !== "ready") return false;
    const decoded = this.decoded(seq, true);
    return !this.synthAvailable() || !decoded;
  }

  // Starts held plays that can start; past HOLD_MAX_MS a play starts regardless — on FM, for the
  // whole session, if the soundfont has not parsed by then.
  releaseHeld(now = Infinity) {
    for (const [handle, wait] of this.held) {
      const seq = this.sequences.get(handle);
      const late = now - wait.since >= (HOLD_MAX_MS * sampleRate) / 1000;
      if (seq && !late && this.mustWait(seq)) continue;
      if (late && (this.soundfont === "pending" || this.soundfont === "loading")) {
        this.giveUp(`not ready within ${HOLD_MAX_MS} ms of a play`);
        return; // giveUp released everything
      }
      this.held.delete(handle);
      if (seq) this.begin(handle, seq, wait.repeat);
    }
  }

  begin(handle, seq, repeat) {
    const gain = this.gains.get(handle) ?? 1;
    this.gains.delete(handle);
    const playback = { seq, start: currentFrame, repeat, next: 0, channels: newChannels(), gain, sf: null };
    if (seq.midi && this.soundfont === "ready") playback.sf = this.takeSynth();
    if (playback.sf) this.synths.push({ handle, pb: playback, synth: playback.sf, stopping: false, fade: 1, tail: 0 });
    this.playbacks.set(handle, playback);
  }

  stop(handle) {
    this.held.delete(handle);
    const playback = this.playbacks.get(handle);
    // Fade the soundfont synth out over the same time FM voices get, then drop it (see mixSynths).
    for (const entry of this.synths) {
      if (entry.handle === handle && entry.pb === playback) entry.stopping = true;
    }
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
    if (playback.sf) {
      playback.sf.processMessage(data);
      return;
    }
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
          if (playback.sf) playback.sf.stopAllChannels(false);
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
    if (this.held.size) this.releaseHeld(currentFrame);
    this.advance(currentFrame + frames);

    left.fill(0);
    if (right !== left) right.fill(0);

    const alive = [];
    for (const voice of this.voices) {
      if (this.render(voice, left, right, frames)) alive.push(voice);
    }
    this.voices = alive;
    if (this.synths.length) this.mixSynths(left, right, frames);
    if (this.work.length) this.runWork(frames);

    for (let i = 0; i < frames; i++) {
      left[i] = Math.tanh(left[i]);
      if (right !== left) right[i] = Math.tanh(right[i]);
    }
    return true;
  }

  // Each soundfont synth renders into scratch and is mixed in at its handle's gain. A stopped one
  // ramps to silence over STOP_RELEASE_S (a forced cut would click); an ended one plays its release
  // and SF_TAIL_S of effects tail. Either way it goes back to the idle pool afterwards.
  //
  // A stopped one keeps rendering, unheard, for SF_TAIL_S after its fade: its reverb only drains
  // while it is processed, and a synth pooled with a full reverb would replay the stopped song's
  // tail under the next play that takes it.
  mixSynths(left, right, frames) {
    if (!this.scratch || this.scratch[0].length < frames) this.scratch = [new Float32Array(frames), new Float32Array(frames)];
    const [sl, sr] = this.scratch;
    const fadeStep = 1 / (STOP_RELEASE_S * sampleRate);
    const kept = [];
    for (const entry of this.synths) {
      sl.fill(0, 0, frames);
      sr.fill(0, 0, frames);
      entry.synth.process(sl, sr, 0, frames);
      const g = SF_GAIN * entry.pb.gain;
      let fade = entry.fade;
      for (let i = 0; i < frames; i++) {
        if (entry.stopping) fade = Math.max(0, fade - fadeStep);
        left[i] += sl[i] * g * fade;
        right[i] += sr[i] * g * fade;
      }
      if (fade === 0 && entry.fade !== 0) entry.synth.stopAllChannels(true);
      entry.fade = fade;
      if (this.playbacks.get(entry.handle) !== entry.pb && entry.synth.voiceCount === 0) {
        entry.tail += frames;
        if (entry.tail >= SF_TAIL_S * sampleRate) {
          this.idle.push(this.resetSynth(entry.synth));
          continue;
        }
      }
      kept.push(entry);
    }
    this.synths = kept;
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
