// Thin wrapper around the wasm `WieEmulator`.
//
// Responsibilities: load the wasm module once, build an emulator from uploaded
// bytes, drive the requestAnimationFrame tick loop, forward input, and persist
// saves to IndexedDB. The uploaded bytes go straight into the wasm constructor
// and are never transmitted anywhere.

import init, { WieEmulator } from "../wasm/wie_web.js";
import { loadSaves, saveSaves } from "./idb";

const SCREEN_W = 240;
const SCREEN_H = 320;

let initPromise: Promise<unknown> | null = null;
function ensureInit(): Promise<unknown> {
  if (!initPromise) initPromise = init();
  return initPromise;
}

export class EmulatorSession {
  private emu: WieEmulator | null = null;
  private rafId = 0;
  private running = false;
  private gameKey = "";
  private saveTimer = 0;

  onError?: (message: string) => void;

  async start(file: File, canvas: HTMLCanvasElement, audioCtx: AudioContext | null): Promise<void> {
    await ensureInit();

    const bytes = new Uint8Array(await file.arrayBuffer());
    this.gameKey = file.name;

    // Construction injects the bytes directly into wasm memory.
    this.emu = new WieEmulator(file.name, bytes, canvas, audioCtx ?? undefined, SCREEN_W, SCREEN_H);

    // Restore prior saves for this game, if any.
    const snapshot = await loadSaves(this.gameKey);
    if (snapshot) {
      try {
        this.emu.import_fs(snapshot);
      } catch {
        // ignore incompatible / corrupt snapshot
      }
    }

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

  async persist(): Promise<void> {
    if (!this.emu) return;
    try {
      if (!this.emu.has_saves()) return;
      const snapshot = this.emu.export_fs() as unknown as Record<string, Uint8Array>;
      await saveSaves(this.gameKey, snapshot);
    } catch {
      // persistence is best-effort; never interrupt play
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
