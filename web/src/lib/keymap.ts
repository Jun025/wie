// Keyboard → emulator key mapping.
//
// Keys are KeyboardEvent.code values; values are the emulator KeyCode names
// understood by the wasm `key_down`/`key_up` API (they match the Rust `KeyCode`
// enum variants). The mapping is persisted in localStorage only — never sent
// anywhere.

export type EmuKey =
  | "UP"
  | "DOWN"
  | "LEFT"
  | "RIGHT"
  | "OK"
  | "LEFT_SOFT_KEY"
  | "RIGHT_SOFT_KEY"
  | "CLEAR"
  | "CALL"
  | "HANGUP"
  | "VOLUME_UP"
  | "VOLUME_DOWN"
  | "NUM0"
  | "NUM1"
  | "NUM2"
  | "NUM3"
  | "NUM4"
  | "NUM5"
  | "NUM6"
  | "NUM7"
  | "NUM8"
  | "NUM9"
  | "STAR"
  | "HASH";

export const ALL_KEYS: EmuKey[] = [
  "UP", "DOWN", "LEFT", "RIGHT", "OK",
  "LEFT_SOFT_KEY", "RIGHT_SOFT_KEY", "CLEAR", "CALL", "HANGUP", "VOLUME_UP", "VOLUME_DOWN",
  "NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9",
  "STAR", "NUM0", "HASH",
];

// Defaults mirror the desktop wie_cli key layout.
export const DEFAULT_KEYMAP: Record<string, EmuKey> = {
  ArrowUp: "UP",
  ArrowDown: "DOWN",
  ArrowLeft: "LEFT",
  ArrowRight: "RIGHT",
  Space: "OK",
  Enter: "OK",
  Digit1: "NUM1",
  Digit2: "NUM2",
  Digit3: "NUM3",
  KeyQ: "NUM4",
  KeyW: "NUM5",
  KeyE: "NUM6",
  KeyA: "NUM7",
  KeyS: "NUM8",
  KeyD: "NUM9",
  KeyZ: "STAR",
  KeyX: "NUM0",
  KeyC: "HASH",
  Backspace: "CLEAR",
  ShiftLeft: "LEFT_SOFT_KEY",
  ShiftRight: "RIGHT_SOFT_KEY",
  F1: "CALL",
  F2: "HANGUP",
  Backquote: "VOLUME_UP",
  Tab: "VOLUME_DOWN",
};

// ── Presets the user can apply with one click ────────────────────────────────
export interface KeymapPreset {
  id: string;
  label: string;
  map: Record<string, EmuKey>;
}

const WASD_KEYMAP: Record<string, EmuKey> = {
  KeyW: "UP",
  KeyS: "DOWN",
  KeyA: "LEFT",
  KeyD: "RIGHT",
  KeyJ: "OK",
  Space: "OK",
  KeyK: "LEFT_SOFT_KEY",
  KeyL: "RIGHT_SOFT_KEY",
  Backspace: "CLEAR",
  KeyU: "STAR",
  KeyI: "HASH",
  Digit1: "NUM1",
  Digit2: "NUM2",
  Digit3: "NUM3",
  Digit4: "NUM4",
  Digit5: "NUM5",
  Digit6: "NUM6",
  Digit7: "NUM7",
  Digit8: "NUM8",
  Digit9: "NUM9",
  Digit0: "NUM0",
};

export const PRESETS: KeymapPreset[] = [
  { id: "arrows", label: "방향키 + Z/X", map: DEFAULT_KEYMAP },
  { id: "wasd", label: "WASD + J/K/L", map: WASD_KEYMAP },
];

// Emulator keys that currently have no physical binding (for a UI warning).
export function unboundKeys(map: Record<string, EmuKey>): EmuKey[] {
  const bound = new Set(Object.values(map));
  return ALL_KEYS.filter((k) => !bound.has(k));
}

const STORAGE_KEY = "wie.keymap.v1";

export function loadKeymap(): Record<string, EmuKey> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as Record<string, EmuKey>;
  } catch {
    // ignore corrupt storage
  }
  return { ...DEFAULT_KEYMAP };
}

export function saveKeymap(map: Record<string, EmuKey>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch {
    // storage may be unavailable (private mode); non-fatal
  }
}

export function resetKeymap(): Record<string, EmuKey> {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignore
  }
  return { ...DEFAULT_KEYMAP };
}
