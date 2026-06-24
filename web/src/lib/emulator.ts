// Thin wrapper around the wasm `WieEmulator`.
//
// Loads the wasm module once, builds an emulator from in-memory game bytes,
// drives the requestAnimationFrame tick loop, forwards input, and autosaves the
// opaque (RMS + filesystem) snapshot to IndexedDB keyed by the game's content
// hash. The bytes come straight from the device-local library and are never
// transmitted anywhere.

import init, { WieEmulator } from "../wasm/wie_web.js";
import * as lib from "./library";
import { autosaveLocal, getLocalSnapshot } from "./saveSync";

const SCREEN_W = 240;
const SCREEN_H = 320;

let initPromise: Promise<unknown> | null = null;
function ensureInit(): Promise<unknown> {
  if (!initPromise) initPromise = init();
  return initPromise;
}

export interface LoadableGame {
  hash: string;
  name: string;
  bytes: ArrayBuffer;
}

// Structured runtime/load error for the on-screen diagnostic panel. Carries the
// raw message, the JS error name, an optional stack, the phase it happened in,
// and the detected platform backend — but NEVER any game-file identity
// (filename/hash/bytes/title stay out of this, per the no-leak baseline).
export interface EmuError {
  message: string;
  name: string;
  stack?: string;
  phase: "load" | "runtime";
  platformKind?: string;
}

export function normalizeError(e: unknown, phase: EmuError["phase"], platformKind?: string): EmuError {
  if (typeof e === "string") return { message: e, name: "Error", phase, platformKind };
  if (e instanceof Error) return { message: e.message || String(e), name: e.name || "Error", stack: e.stack, phase, platformKind };
  return { message: String(e), name: "Error", phase, platformKind };
}

// Volume + mute are a device-local preference (localStorage only, never sent).
const VOL_KEY = "wie-volume";
const MUTE_KEY = "wie-muted";
const clamp01 = (v: number) => Math.max(0, Math.min(1, v));
function loadVolume(): number {
  const raw = localStorage.getItem(VOL_KEY);
  if (raw === null) return 0.8; // default 80% (Number(null) === 0 would wrongly pass below)
  const v = Number(raw);
  return Number.isFinite(v) && v >= 0 && v <= 1 ? v : 0.8;
}
function loadMuted(): boolean {
  return localStorage.getItem(MUTE_KEY) === "1";
}

// Validate that bytes are a loadable game by dry-constructing the emulator (which
// runs the format detection + archive/jar loader) on a throwaway canvas. Returns
// null on success, or the loader's error message if the file isn't a valid game.
// The bytes never leave the browser; nothing is stored here.
export async function validateGame(name: string, bytes: ArrayBuffer): Promise<string | null> {
  await ensureInit();
  const u8 = new Uint8Array(bytes);
  const lower = name.toLowerCase();

  // Fast structural check: .jar/.zip must carry the ZIP signature "PK".
  if ((lower.endsWith(".jar") || lower.endsWith(".zip")) && !(u8[0] === 0x50 && u8[1] === 0x4b)) {
    return "유효한 ZIP/JAR 형식이 아닙니다 (압축 파일 시그니처를 찾을 수 없습니다).";
  }

  const probe = document.createElement("canvas");
  let emu: WieEmulator | null = null;
  try {
    // Construct + tick a few frames so the loader actually parses the archive /
    // boots the main class; a bad file throws here instead of being stored.
    emu = new WieEmulator(name, u8, probe, undefined, undefined, SCREEN_W, SCREEN_H);
    for (let i = 0; i < 6; i++) emu.tick();
    return null;
  } catch (e) {
    return typeof e === "string" ? e : ((e as Error)?.message ?? "알 수 없는 형식");
  } finally {
    try {
      emu?.free();
    } catch {
      /* already dropped */
    }
  }
}

export class EmulatorSession {
  private emu: WieEmulator | null = null;
  private rafId = 0;
  private running = false;
  private gameHash = "";
  private saveTimer = 0;
  private audioCtx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private volume = loadVolume();
  private muted = loadMuted();

  onError?: (err: EmuError) => void;
  // Fired whenever volume/mute changes (slider, pad, keyboard) so the UI gauge
  // stays in sync — the session is the single source of truth.
  onVolumeChange?: (volume: number, muted: boolean) => void;

  // Detected emulator backend ("KTF"/"LGT"/"SKT"/"J2ME"), or null before load.
  // A runtime label for diagnostics only — not game identity.
  platformKind(): string | null {
    try {
      return this.emu?.platform_kind() ?? null;
    } catch {
      return null;
    }
  }

  async start(game: LoadableGame, canvas: HTMLCanvasElement, audioCtx: AudioContext | null): Promise<void> {
    await ensureInit();

    this.gameHash = game.hash;
    this.audioCtx = audioCtx;
    const bytes = new Uint8Array(game.bytes);

    // Master gain node: all PCM is routed through it, so the UI volume slider is
    // the single source of truth for the real output level.
    let gain: GainNode | null = null;
    if (audioCtx) {
      gain = audioCtx.createGain();
      gain.gain.value = this.muted ? 0 : this.volume;
      gain.connect(audioCtx.destination);
    }
    this.masterGain = gain;

    // Construction injects the bytes directly into wasm memory.
    this.emu = new WieEmulator(game.name, bytes, canvas, audioCtx ?? undefined, gain ?? undefined, SCREEN_W, SCREEN_H);

    // Restore prior saves (opaque blob) for this game, if any.
    const snapshot = await getLocalSnapshot(this.gameHash);
    if (snapshot) {
      try {
        this.emu.import_saves(snapshot);
      } catch {
        /* ignore incompatible / corrupt snapshot */
      }
    }

    await lib.touchGame(this.gameHash);

    this.running = true;
    this.rafId = requestAnimationFrame(this.loop);
    this.saveTimer = window.setInterval(() => void this.persist(), 5000);
  }

  private loop = (): void => {
    if (!this.running || !this.emu) return;
    try {
      this.emu.tick();
    } catch (e) {
      const kind = this.platformKind() ?? undefined;
      this.stop();
      this.onError?.(normalizeError(e, "runtime", kind));
      return;
    }
    this.rafId = requestAnimationFrame(this.loop);
  };

  // Resume the AudioContext if the browser suspended it under the autoplay
  // policy. Safe to call on every input — a no-op once running.
  resumeAudio(): void {
    if (this.audioCtx && this.audioCtx.state === "suspended") void this.audioCtx.resume();
  }

  private applyGain(): void {
    if (this.masterGain) this.masterGain.gain.value = this.muted ? 0 : this.volume;
  }
  private notifyVolume(): void {
    this.onVolumeChange?.(this.volume, this.muted);
  }

  getVolume(): number {
    return this.volume;
  }
  isMuted(): boolean {
    return this.muted;
  }

  // Set the output volume (0..1). Setting a non-zero volume implicitly unmutes.
  setVolume(v: number): void {
    this.volume = clamp01(v);
    localStorage.setItem(VOL_KEY, String(this.volume));
    if (this.volume > 0 && this.muted) {
      this.muted = false;
      localStorage.setItem(MUTE_KEY, "0");
    }
    this.resumeAudio();
    this.applyGain();
    this.notifyVolume();
  }

  // Nudge volume by a step (used by the VOL± pad buttons / keyboard).
  stepVolume(delta: number): void {
    this.setVolume(this.volume + delta);
  }

  setMuted(muted: boolean): void {
    this.muted = muted;
    localStorage.setItem(MUTE_KEY, muted ? "1" : "0");
    if (!muted) this.resumeAudio();
    this.applyGain();
    this.notifyVolume();
  }

  toggleMute(): void {
    this.setMuted(!this.muted);
  }

  keyDown(code: string): void {
    this.resumeAudio();
    this.emu?.key_down(code);
  }
  keyUp(code: string): void {
    this.emu?.key_up(code);
  }
  keyRepeat(code: string): void {
    this.emu?.key_repeat(code);
  }

  // Opaque save snapshot (RMS + filesystem), or null if nothing was written.
  exportBlob(): Uint8Array | null {
    if (!this.emu || !this.emu.has_saves()) return null;
    return this.emu.export_saves();
  }

  async persist(): Promise<void> {
    const blob = this.exportBlob();
    if (!blob) return;
    try {
      await autosaveLocal(this.gameHash, blob);
    } catch {
      /* best-effort; never interrupt play */
    }
  }

  stop(): void {
    this.running = false;
    if (this.rafId) cancelAnimationFrame(this.rafId);
    if (this.saveTimer) clearInterval(this.saveTimer);
    this.rafId = 0;
    this.saveTimer = 0;
  }

  isRunning(): boolean {
    return this.running;
  }
}
