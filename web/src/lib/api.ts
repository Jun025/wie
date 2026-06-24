// Backend client (Cloudflare Pages Functions).
//
// ┌─ GUARDRAIL (1번 기준선 / S5) ───────────────────────────────────────────────┐
// │ The ONLY things sent to the server are: account info, opaque save payloads  │
// │ + a USER-CHOSEN slot/device alias, and inquiry text. NEVER game bytes,      │
// │ filenames, content hashes, or a "games on this device" list.                │
// └────────────────────────────────────────────────────────────────────────────┘

export interface User {
  id: string;
  login_id: string;
  email: string | null;
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
  category: string;
  title: string;
  body: string;
  game_title: string;
  game_vendor: string;
  device_model: string;
  symptom: string;
  status: string;
  created_at: number;
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
  me: () => call<{ ok: boolean; authenticated: boolean; user?: User }>("/auth/me"),
  register: (login_id: string, password: string, email?: string) =>
    call<{ ok: boolean; user: User }>("/auth/register", { method: "POST", body: { login_id, password, email } }),
  login: (login_id: string, password: string) =>
    call<{ ok: boolean; user: User }>("/auth/login", { method: "POST", body: { login_id, password } }),
  logout: () => call("/auth/logout", { method: "POST" }),
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
  list: () => call<{ ok: boolean; devices: DeviceSlot[] }>("/devices"),
};

export const inquiries = {
  create: (payload: Record<string, string>) => call<{ ok: boolean; inquiry: Inquiry }>("/inquiries", { method: "POST", body: payload }),
  list: () => call<{ ok: boolean; inquiries: Inquiry[] }>("/inquiries"),
};

export const health = () => call("/health");
