// Best-effort D1-backed rate limiting. No raw IPs are ever stored — only a
// salted SHA-256 hash of the client IP, keyed per route. This is "가용 범위"
// rate limiting: D1 is eventually consistent so bursts can slip a little, but
// it caps credential-stuffing / spam without needing KV or Durable Objects.

import { sha256Hex } from "./crypto.js";
import { HttpError } from "./http.js";

export async function rateLimit(context, route, { limit = 20, windowSec = 60 } = {}) {
  const { request, env } = context;
  const ip = request.headers.get("cf-connecting-ip") || "0.0.0.0";
  const salt = env.SESSION_SECRET || "wie";
  const ipHash = (await sha256Hex(`${salt}:${ip}`)).slice(0, 24);
  const key = `${route}:${ipHash}`;
  const now = Date.now();
  const windowStart = now - now % (windowSec * 1000);

  const row = await env.DB.prepare("SELECT window_start, count FROM rate_limits WHERE bucket_key = ?").bind(key).first();

  if (!row || row.window_start !== windowStart) {
    await env.DB.prepare(
      "INSERT INTO rate_limits (bucket_key, window_start, count) VALUES (?, ?, 1) " +
        "ON CONFLICT(bucket_key) DO UPDATE SET window_start = excluded.window_start, count = 1",
    )
      .bind(key, windowStart)
      .run();
    return;
  }

  if (row.count >= limit) {
    throw new HttpError("Too many requests, slow down", 429, "rate_limited");
  }

  await env.DB.prepare("UPDATE rate_limits SET count = count + 1 WHERE bucket_key = ?").bind(key).run();
}
