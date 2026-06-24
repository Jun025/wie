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

export class EmulatorSession {
  private emu: WieEmulator | null = null;
  private rafId = 0;
  private running = false;
  private gameHash = "";
  private saveTimer = 0;

  onError?: (message: string) => void;

  async start(game: LoadableGame, canvas: HTMLCanvasElement, audioCtx: AudioContext | null): Promise<void> {
    await ensureInit();

    this.gameHash = game.hash;
    const bytes = new Uint8Array(game.bytes);

    // Construction injects the bytes directly into wasm memory.
    this.emu = new WieEmulator(game.name, bytes, canvas, audioCtx ?? undefined, SCREEN_W, SCREEN_H);

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
      this.stop();
      this.onError?.(typeof e === "string" ? e : (e as Error)?.message ?? "emulator error");
      return;
    }
    this.rafId = requestAnimationFrame(this.loop);
  };

  keyDown(code: string): void {
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
