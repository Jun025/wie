// Device-local storage (IndexedDB): the game library + the local save cache.
//
// ┌─ GUARDRAIL (1번 기준선 / S5) ───────────────────────────────────────────────┐
// │ EVERYTHING here stays on the device. Game bytes, filenames, content hashes  │
// │ and the "which games this device has" list live ONLY in IndexedDB. This     │
// │ module performs ZERO network requests. The only bytes that ever leave the   │
// │ browser are opaque save blobs + account info, sent from api.ts / saveSync.ts.│
// └────────────────────────────────────────────────────────────────────────────┘

const DB_NAME = "wie-local";
const DB_VERSION = 1;

export interface GameRecord {
  hash: string; // sha-256 of the runnable bytes — LOCAL key, never transmitted
  name: string;
  kind: string; // jar | zip | jad | kdf | skm
  bytes: ArrayBuffer; // runnable payload (jar/zip)
  jadBytes?: ArrayBuffer;
  size: number;
  addedAt: number;
  lastPlayedAt?: number;
}

export interface LocalSave {
  hash: string; // game hash this save belongs to (LOCAL mapping only)
  blob: ArrayBuffer; // opaque WIESAV01 snapshot
  updatedAt: number;
  slotLabel?: string; // user alias, set when synced to cloud
  serverId?: string;
  syncedAt?: number;
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains("games")) db.createObjectStore("games", { keyPath: "hash" });
      if (!db.objectStoreNames.contains("saves")) db.createObjectStore("saves", { keyPath: "hash" });
      if (!db.objectStoreNames.contains("meta")) db.createObjectStore("meta", { keyPath: "key" });
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function reqAsync<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}

async function withStore<T>(store: string, mode: IDBTransactionMode, fn: (s: IDBObjectStore) => Promise<T> | T): Promise<T> {
  const db = await openDb();
  try {
    const tx = db.transaction(store, mode);
    return await fn(tx.objectStore(store));
  } finally {
    db.close();
  }
}

// ── content hash (LOCAL identifier only — never transmitted) ──────────────────
export async function sha256Hex(buffer: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", buffer);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

// ── games ──────────────────────────────────────────────────────────────────────
export async function putGame(game: GameRecord): Promise<void> {
  await withStore("games", "readwrite", (s) => reqAsync(s.put(game)));
}

export async function getGame(hash: string): Promise<GameRecord | undefined> {
  return withStore("games", "readonly", (s) => reqAsync<GameRecord | undefined>(s.get(hash)));
}

export type GameMeta = Omit<GameRecord, "bytes" | "jadBytes">;

export async function listGames(): Promise<GameMeta[]> {
  const all = await withStore("games", "readonly", (s) => reqAsync<GameRecord[]>(s.getAll()));
  return all
    .map(({ bytes: _b, jadBytes: _j, ...meta }) => meta)
    .sort((a, b) => (b.lastPlayedAt ?? b.addedAt) - (a.lastPlayedAt ?? a.addedAt));
}

export async function deleteGame(hash: string): Promise<void> {
  await withStore("games", "readwrite", (s) => reqAsync(s.delete(hash)));
  await deleteLocalSave(hash);
}

export async function clearGames(): Promise<void> {
  await withStore("games", "readwrite", (s) => reqAsync(s.clear()));
}

export async function touchGame(hash: string): Promise<void> {
  const db = await openDb();
  try {
    const tx = db.transaction("games", "readwrite");
    const s = tx.objectStore("games");
    const g = await reqAsync<GameRecord | undefined>(s.get(hash));
    if (g) {
      g.lastPlayedAt = Date.now();
      await reqAsync(s.put(g));
    }
  } finally {
    db.close();
  }
}

export async function totalGameBytes(): Promise<number> {
  const all = await withStore("games", "readonly", (s) => reqAsync<GameRecord[]>(s.getAll()));
  return all.reduce((sum, g) => sum + (g.bytes?.byteLength ?? 0) + (g.jadBytes?.byteLength ?? 0), 0);
}

// ── local saves (opaque snapshots) ───────────────────────────────────────────
export async function putLocalSave(save: LocalSave): Promise<void> {
  await withStore("saves", "readwrite", (s) => reqAsync(s.put(save)));
}

export async function getLocalSave(hash: string): Promise<LocalSave | undefined> {
  return withStore("saves", "readonly", (s) => reqAsync<LocalSave | undefined>(s.get(hash)));
}

export async function listLocalSaves(): Promise<LocalSave[]> {
  return withStore("saves", "readonly", (s) => reqAsync<LocalSave[]>(s.getAll()));
}

export async function deleteLocalSave(hash: string): Promise<void> {
  await withStore("saves", "readwrite", (s) => reqAsync(s.delete(hash)));
}

// ── meta (capacity limit etc.) ───────────────────────────────────────────────
export async function getMeta<T>(key: string, fallback: T): Promise<T> {
  const row = await withStore("meta", "readonly", (s) => reqAsync<{ key: string; value: T } | undefined>(s.get(key)));
  return row ? row.value : fallback;
}

export async function setMeta<T>(key: string, value: T): Promise<void> {
  await withStore("meta", "readwrite", (s) => reqAsync(s.put({ key, value })));
}

export const DEFAULT_CAPACITY_MB = 5;
export const capacityMB = () => getMeta("capacityMB", DEFAULT_CAPACITY_MB);
export const setCapacityMB = (mb: number) => setMeta("capacityMB", mb);
