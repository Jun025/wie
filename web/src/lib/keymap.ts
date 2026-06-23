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
  "LEFT_SOFT_KEY", "RIGHT_SOFT_KEY", "CLEAR", "CALL", "HANGUP",
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
};

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
