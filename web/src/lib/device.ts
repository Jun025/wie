// Per-device identity + environment + server heartbeat.
//
// GUARDRAIL (1번 기준선 / S5): the heartbeat and the inquiry env-info built here
// carry ONLY counts/sizes + browser/OS/screen — NEVER a game filename, hash, or
// title. The device_id is a random value WE generate (not a hardware fingerprint).

import * as lib from "./library";
import { devices as devicesApi } from "./api";
import { deviceName } from "./saveSync";

const DEVICE_ID_KEY = "wie-device-id";
export const APP_VERSION = "0.0.1";

// Stable, app-scoped random device id (NOT a hardware identifier).
export function deviceId(): string {
  let id = localStorage.getItem(DEVICE_ID_KEY);
  if (!id || id.length < 8) {
    id = (crypto.randomUUID?.() ?? `dev-${Math.abs(hashStr(String(performance.now())))}-${rand()}`).replace(/[^A-Za-z0-9_-]/g, "");
    localStorage.setItem(DEVICE_ID_KEY, id);
  }
  return id;
}

function rand(): string {
  const a = new Uint8Array(8);
  crypto.getRandomValues(a);
  return [...a].map((b) => b.toString(16).padStart(2, "0")).join("");
}
function hashStr(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return h;
}

// Client-readable environment (no game identity, no over-collection).
export interface ClientEnv {
  browser: string;
  os: string;
  screen: string;
  cores: string;
  network: string;
  language: string;
  appVersion: string;
  userAgent: string;
}

export function clientEnv(): ClientEnv {
  const ua = navigator.userAgent || "";
  const os = /iPhone|iPad|iPod/.test(ua) ? "iOS" : /Android/.test(ua) ? "Android" : /Mac OS X|Macintosh/.test(ua) ? "macOS" : /Windows/.test(ua) ? "Windows" : /Linux/.test(ua) ? "Linux" : "기타";
  const browser = /Edg\//.test(ua) ? "Edge" : /CriOS|Chrome\//.test(ua) ? "Chrome" : /FxiOS|Firefox\//.test(ua) ? "Firefox" : /Safari\//.test(ua) ? "Safari" : "기타";
  const dpr = window.devicePixelRatio || 1;
  const screen = `${window.screen?.width ?? 0}×${window.screen?.height ?? 0} @${dpr}x`;
  // navigator.connection is non-standard; guard it (same defensive lesson as vibrate/clipboard).
  const conn = (navigator as unknown as { connection?: { effectiveType?: string } }).connection;
  const network = conn?.effectiveType ?? "unknown";
  const cores = String((navigator as unknown as { hardwareConcurrency?: number }).hardwareConcurrency ?? "?");
  return { browser, os, screen, cores, network, language: navigator.language || "", appVersion: APP_VERSION, userAgent: ua };
}

// Diagnostic block auto-attached to an inquiry. platformKind is the emulator
// backend (KTF/LGT/SKT/J2ME), a runtime label — NOT game identity.
export function envInfoText(platformKind?: string | null): string {
  const e = clientEnv();
  return [
    "[자동 수집 환경정보]",
    `브라우저: ${e.browser}`,
    `OS: ${e.os}`,
    platformKind ? `플랫폼: ${platformKind}` : null,
    `화면: ${e.screen}`,
    `CPU 코어: ${e.cores}`,
    `네트워크: ${e.network}`,
    `언어: ${e.language}`,
    `앱 버전: ${e.appVersion}`,
    `UA: ${e.userAgent}`,
  ]
    .filter(Boolean)
    .join("\n");
}

// Local-only summary of THIS device's library (filenames stay local; this is for
// the on-screen "current device" detail, never sent to the server).
export async function localDeviceSummary() {
  const [games, saves] = await Promise.all([lib.listGames(), lib.listLocalSaves()]);
  const totalBytes = games.reduce((s, g) => s + (g.size ?? 0), 0);
  const lastRun = games.reduce((m, g) => Math.max(m, g.lastPlayedAt ?? 0), 0);
  const lastSave = saves.reduce((m, s) => Math.max(m, s.updatedAt ?? 0), 0);
  const saveByHash = new Map(saves.map((s) => [s.hash, s]));
  return {
    itemCount: games.length,
    totalBytes,
    lastRun,
    lastSave,
    games: games.map((g) => ({ hash: g.hash, name: g.name, size: g.size, lastPlayedAt: g.lastPlayedAt, save: saveByHash.get(g.hash) })),
  };
}

// Report this device's ANONYMOUS aggregate (counts/sizes only) to the server.
// Best-effort: never throws into the caller.
export async function sendHeartbeat(opts: { login?: boolean } = {}): Promise<void> {
  try {
    const s = await localDeviceSummary();
    await devicesApi.heartbeat({
      device_id: deviceId(),
      label: deviceName(),
      item_count: s.itemCount,
      total_bytes: s.totalBytes,
      last_run_at: s.lastRun,
      last_save_at: s.lastSave,
      login: opts.login,
    });
  } catch {
    /* offline / not logged in — ignore */
  }
}
