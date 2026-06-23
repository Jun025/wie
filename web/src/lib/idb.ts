// Minimal IndexedDB wrapper for save persistence.
//
// IMPORTANT: this stores ONLY guest save data (the snapshot the emulator
// produces via `export_fs`). Game files are never written here — they live only
// transiently in wasm memory for the active session.

const DB_NAME = "wie-saves";
const STORE = "fs";

function open(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function loadSaves(gameKey: string): Promise<Record<string, Uint8Array> | null> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const req = tx.objectStore(STORE).get(gameKey);
    req.onsuccess = () => resolve((req.result as Record<string, Uint8Array>) ?? null);
    req.onerror = () => reject(req.error);
  });
}

export async function saveSaves(gameKey: string, snapshot: Record<string, Uint8Array>): Promise<void> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(snapshot, gameKey);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}
