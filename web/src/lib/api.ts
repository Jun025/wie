// Backend client (Cloudflare Pages Functions).
//
// ┌─ BASELINE (개정 / S5) ──────────────────────────────────────────────────────┐
// │ • NOT logged in: game files stay on the device (IndexedDB) — ZERO bytes,    │
// │   filenames, hashes or "owned list" are ever sent. (unchanged)              │
// │ • Logged in: the user MAY upload their own game files to a PRIVATE, per-     │
// │   owner server vault (files.*). Strictly owner-isolated — no sharing,       │
// │   public URL, listing, search, or discovery. Dedup is per-user only.        │
// │ Plus: account info, opaque save payloads + a user alias, inquiry text, and  │
// │ rights-holder takedown reports.                                             │
// └────────────────────────────────────────────────────────────────────────────┘

export interface User {
  id: string;
  login_id: string;
  email: string | null;
  email_verified?: boolean;
}

// A registered device (no game identity — only counts/sizes + timestamps).
export interface Device {
  device_id: string;
  label: string;
  last_login_at: number;
  last_seen_at: number;
  item_count: number;
  total_bytes: number;
  last_run_at: number;
  last_save_at: number;
  slot_count: number;
}

// Anonymous storage aggregate the client reports for the CURRENT device. Contains
// counts/sizes only — never a filename, hash, or title.
export interface DeviceHeartbeat {
  device_id: string;
  label: string;
  item_count: number;
  total_bytes: number;
  last_run_at: number;
  last_save_at: number;
  login?: boolean;
}

export interface CloudSave {
  id: string;
  slot_label: string;
  device_label: string;
  payload?: string; // base64, only when ?include=payload
  payload_bytes: number;
  checksum: string;
  updated_at: number;
  created_at: number;
}

export interface DeviceSlot {
  device_label: string;
  slot_count: number;
  last_updated: number;
}

export interface Inquiry {
  id: string;
  title: string;
  body: string;
  env_info?: string;
  attachment_name?: string;
  attachment_mime?: string;
  has_attachment?: number | boolean;
  status: string;
  created_at: number;
}

export interface InquiryAttachment {
  name: string;
  mime: string;
  data: string; // base64
}

export class ApiError extends Error {
  status: number;
  code?: string;
  constructor(message: string, status: number, code?: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

async function call<T = unknown>(path: string, opts: { method?: string; body?: unknown } = {}): Promise<T> {
  const init: RequestInit = { method: opts.method ?? "GET", credentials: "same-origin", headers: {} };
  if (opts.body !== undefined) {
    init.headers = { "content-type": "application/json" };
    init.body = JSON.stringify(opts.body);
  }
  const res = await fetch(`/api${path}`, init);
  let data: any = null;
  try {
    data = await res.json();
  } catch {
    /* non-JSON */
  }
  if (!res.ok) throw new ApiError((data && data.error) || `HTTP ${res.status}`, res.status, data && data.code);
  return data as T;
}

export const auth = {
  me: () => call<{ ok: boolean; authenticated: boolean; emailConfigured?: boolean; user?: User }>("/auth/me"),
  register: (login_id: string, password: string, email?: string) =>
    call<{ ok: boolean; user: User; pending?: boolean; emailSent?: boolean }>("/auth/register", { method: "POST", body: { login_id, password, email } }),
  login: (login_id: string, password: string) =>
    call<{ ok: boolean; user: User }>("/auth/login", { method: "POST", body: { login_id, password } }),
  logout: () => call("/auth/logout", { method: "POST" }),
  resend: (login_id: string) => call<{ ok: boolean; emailConfigured?: boolean }>("/auth/resend", { method: "POST", body: { login_id } }),
  requestReset: (login_id: string) => call<{ ok: boolean; emailConfigured?: boolean }>("/auth/request-reset", { method: "POST", body: { login_id } }),
  reset: (token: string, password: string) => call<{ ok: boolean }>("/auth/reset", { method: "POST", body: { token, password } }),
};

export const saves = {
  list: (withPayload = false) => call<{ ok: boolean; saves: CloudSave[] }>(`/saves${withPayload ? "?include=payload" : ""}`),
  get: (id: string) => call<{ ok: boolean; save: CloudSave }>(`/saves/${encodeURIComponent(id)}`),
  // payload is an opaque base64 save snapshot; slot/device are USER aliases.
  upsert: (slot_label: string, device_label: string, payload: string) =>
    call<{ ok: boolean; save: CloudSave }>("/saves", { method: "POST", body: { slot_label, device_label, payload } }),
  remove: (id: string) => call(`/saves/${encodeURIComponent(id)}`, { method: "DELETE" }),
};

export const devices = {
  list: () => call<{ ok: boolean; devices: Device[] }>("/devices"),
  // Heartbeat carries ONLY counts/sizes + timestamps — never a game identity.
  heartbeat: (hb: DeviceHeartbeat) => call<{ ok: boolean }>("/devices", { method: "POST", body: hb }),
};

export const inquiries = {
  create: (payload: { title: string; body: string; env_info?: string; attachment?: InquiryAttachment | null }) =>
    call<{ ok: boolean; inquiry: Inquiry }>("/inquiries", { method: "POST", body: payload }),
  list: () => call<{ ok: boolean; inquiries: Inquiry[] }>("/inquiries"),
};

// ── Private per-owner server file vault (B안) ──────────────────────────────────
export interface ServerFile {
  id: string;
  file_name: string;
  kind: string;
  content_hash: string;
  size: number;
  created_at: number;
  last_seen_at?: number;
}
export interface FilesUsage {
  used: number;
  quota: number;
}

export const files = {
  // List THIS user's files + quota. `enabled:false` when the R2 vault is not yet
  // provisioned on the server (S8) — the UI hides the feature in that case.
  list: () => call<{ ok: boolean; enabled: boolean; files: ServerFile[]; usage: FilesUsage }>("/files"),

  // Upload one game file: raw bytes (octet-stream) + metadata headers. The server
  // verifies the hash, screens the content, enforces the quota, and dedups per-user.
  upload: async (name: string, kind: string, contentHash: string, bytes: ArrayBuffer) => {
    const res = await fetch("/api/files", {
      method: "POST",
      credentials: "same-origin",
      headers: {
        "content-type": "application/octet-stream",
        "x-file-name": encodeURIComponent(name),
        "x-content-hash": contentHash,
        "x-kind": kind,
      },
      body: bytes,
    });
    let data: { error?: string; code?: string; file?: ServerFile; usage?: FilesUsage } | null = null;
    try {
      data = await res.json();
    } catch {
      /* non-JSON */
    }
    if (!res.ok) throw new ApiError(data?.error || `HTTP ${res.status}`, res.status, data?.code);
    return data as { ok: boolean; file: ServerFile; usage: FilesUsage };
  },

  // Download one file's bytes (owner-only) for running / restoring locally.
  download: async (id: string): Promise<ArrayBuffer> => {
    const res = await fetch(`/api/files/${encodeURIComponent(id)}`, { credentials: "same-origin" });
    if (!res.ok) {
      let data: { error?: string; code?: string } | null = null;
      try {
        data = await res.json();
      } catch {
        /* non-JSON */
      }
      throw new ApiError(data?.error || `HTTP ${res.status}`, res.status, data?.code);
    }
    return res.arrayBuffer();
  },

  remove: (id: string) => call<{ ok: boolean; usage: FilesUsage }>(`/files/${encodeURIComponent(id)}`, { method: "DELETE" }),
};

// Rights-holder takedown notice intake (compliance). Anonymous-allowed.
export const reports = {
  create: (payload: { reporter_name?: string; reporter_contact?: string; work_title?: string; statement: string; target_hint?: string }) =>
    call<{ ok: boolean; report: { id: string; status: string; created_at: number } }>("/reports", { method: "POST", body: payload }),
};

export const health = () => call("/health");
