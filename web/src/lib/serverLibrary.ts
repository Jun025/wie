// Migrate device-local (IndexedDB) games into the logged-in user's PRIVATE
// server vault, then free the local copy.
//
// Rules (per the B안 directive):
//   • upload succeeds  → delete the local copy (it now lives on the server).
//   • per-user dedup (409 duplicate) → the file is already in THIS user's vault,
//     so just delete the local copy (no re-upload).
//   • quota exceeded (413) → STOP; keep the remaining games on the device
//     (never silently drop local data).
//   • any other error → keep the local copy and count it as failed.
//
// All network access goes through api.ts (files.*) — this module makes no fetch
// of its own, so the no-leak audit's "network only in api.ts" invariant holds.

import * as lib from "./library";
import { files as filesApi, ApiError } from "./api";
import { pushSaveToServer, deviceName } from "./saveSync";

export interface MigrateResult {
  uploaded: number;
  deduped: number;
  failed: number;
  stopped: boolean; // true if halted on a full vault
  message?: string;
}

// Per-file lifecycle status for the live upload UI (1번): each local game moves
// 대기(pending) → 업로드중(uploading) → 완료(uploaded|deduped) | 실패(failed).
export type FileUploadStatus = "pending" | "uploading" | "uploaded" | "deduped" | "failed";
export interface FileUploadEvent {
  hash: string;
  name: string;
  size: number;
  status: FileUploadStatus;
  reason?: string; // populated for `failed`
}

export async function migrateLocalToServer(
  onProgress?: (done: number, total: number) => void,
  onFile?: (e: FileUploadEvent) => void,
): Promise<MigrateResult> {
  const metas = await lib.listGames();
  let uploaded = 0;
  let deduped = 0;
  let failed = 0;
  let stopped = false;
  let message: string | undefined;

  // Seed the UI with every file as 대기(pending) so the full list + total shows
  // from the start.
  for (const m of metas) onFile?.({ hash: m.hash, name: m.name, size: m.size, status: "pending" });

  for (let i = 0; i < metas.length; i++) {
    const meta = metas[i];
    const g = await lib.getGame(meta.hash);
    if (!g) {
      failed++;
      onFile?.({ hash: meta.hash, name: meta.name, size: meta.size, status: "failed", reason: "로컬에서 파일을 찾지 못함" });
      onProgress?.(i + 1, metas.length);
      continue;
    }
    onFile?.({ hash: g.hash, name: g.name, size: g.size, status: "uploading" });
    try {
      // g.hash is sha-256 of g.bytes (set at import time) — the server re-hashes
      // the same bytes and matches it, so content_hash stays honest.
      await filesApi.upload(g.name, g.kind, g.hash, g.bytes);
      // Push the ROM's save to the server too (keyed by the SAME hash) and KEEP
      // the local save — so running the now-server ROM resumes the same save.
      await pushSaveToServer(g.hash, deviceName()).catch(() => {});
      await lib.deleteGame(g.hash); // free local ROM (save kept by default)
      uploaded++;
      onFile?.({ hash: g.hash, name: g.name, size: g.size, status: "uploaded" });
    } catch (e) {
      const err = e as ApiError;
      if (err.code === "duplicate") {
        await pushSaveToServer(g.hash, deviceName()).catch(() => {});
        await lib.deleteGame(g.hash); // already in vault → free local ROM (save kept)
        deduped++;
        onFile?.({ hash: g.hash, name: g.name, size: g.size, status: "deduped" });
      } else if (err.code === "quota_exceeded") {
        stopped = true;
        message = "보관함 용량(1GB)이 가득 찼습니다. 남은 게임은 이 기기에 그대로 보존했습니다.";
        onFile?.({ hash: g.hash, name: g.name, size: g.size, status: "failed", reason: "보관함 용량(1GB) 초과 — 이 기기에 보존" });
        break; // keep the rest local
      } else {
        failed++;
        message = err.message; // keep local on any other failure
        onFile?.({ hash: g.hash, name: g.name, size: g.size, status: "failed", reason: err.message });
      }
    }
    onProgress?.(i + 1, metas.length);
  }

  return { uploaded, deduped, failed, stopped, message };
}

// 1-decimal byte formatter (re-exported from the shared limits module).
export { fmtBytes1 as fmtBytes } from "./limits";
