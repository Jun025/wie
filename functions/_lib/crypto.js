// Password hashing + session-token signing.
//
// Password KDF: PBKDF2-HMAC-SHA256. PBKDF2 is the strong, salted, iterated KDF
// that is natively available in the Workers WebCrypto (SubtleCrypto) runtime —
// argon2/scrypt are not exposed by the platform's crypto and would require
// bundling unaudited JS, so PBKDF2 with a high iteration count is the correct
// "Workers 가용 검증 알고리즘" choice. Plaintext passwords are never stored or
// logged.

// Iteration count is a deliberate trade-off with the Cloudflare *free* plan's
// ~10ms CPU budget per request: 210k PBKDF2-SHA256 iterations overran it and
// made register/login fail with a 500. 75k keeps auth comfortably under budget
// while still being a strong, salted, iterated KDF. The count is stored per
// record (`password_iter`), so this can be raised later (e.g. on a paid plan)
// without breaking existing hashes — verification always uses each record's own
// iteration count.
const PBKDF2_ITERATIONS = 75_000;
const PBKDF2_ALGO = "pbkdf2-sha256";

const enc = new TextEncoder();

export function b64encode(bytes) {
  let bin = "";
  const arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  for (let i = 0; i < arr.length; i++) bin += String.fromCharCode(arr[i]);
  return btoa(bin);
}

export function b64decode(s) {
  const bin = atob(s);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export function b64urlencode(bytes) {
  return b64encode(bytes).replaceAll("+", "-").replaceAll("/", "_").replaceAll("=", "");
}

function randomBytes(n) {
  const b = new Uint8Array(n);
  crypto.getRandomValues(b);
  return b;
}

export function randomHex(n = 32) {
  return [...randomBytes(n)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export function uuid() {
  return crypto.randomUUID();
}

async function pbkdf2(password, salt, iterations) {
  const keyMaterial = await crypto.subtle.importKey("raw", enc.encode(password), "PBKDF2", false, ["deriveBits"]);
  const bits = await crypto.subtle.deriveBits(
    { name: "PBKDF2", salt, iterations, hash: "SHA-256" },
    keyMaterial,
    256,
  );
  return new Uint8Array(bits);
}

export async function hashPassword(password) {
  const salt = randomBytes(16);
  const hash = await pbkdf2(password, salt, PBKDF2_ITERATIONS);
  return {
    algo: PBKDF2_ALGO,
    iter: PBKDF2_ITERATIONS,
    salt: b64encode(salt),
    hash: b64encode(hash),
  };
}

export async function verifyPassword(password, record) {
  if (record.password_algo !== PBKDF2_ALGO) return false;
  const salt = b64decode(record.password_salt);
  const expected = b64decode(record.password_hash);
  const actual = await pbkdf2(password, salt, record.password_iter);
  return timingSafeEqual(actual, expected);
}

export function timingSafeEqual(a, b) {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a[i] ^ b[i];
  return diff === 0;
}

async function hmacSha256(secret, message) {
  const key = await crypto.subtle.importKey("raw", enc.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(message));
  return new Uint8Array(sig);
}

// Cookie value = "<sessionId>.<base64url(hmac(sessionId))>". Signature blocks
// tampering; the sessions table row enables revocation + expiry.
export async function signSessionId(sessionId, secret) {
  const mac = await hmacSha256(secret, sessionId);
  return `${sessionId}.${b64urlencode(mac)}`;
}

export async function verifySessionCookie(value, secret) {
  if (typeof value !== "string" || !value.includes(".")) return null;
  const idx = value.lastIndexOf(".");
  const sessionId = value.slice(0, idx);
  const provided = value.slice(idx + 1);
  const expected = b64urlencode(await hmacSha256(secret, sessionId));
  if (provided.length !== expected.length) return null;
  let diff = 0;
  for (let i = 0; i < provided.length; i++) diff |= provided.charCodeAt(i) ^ expected.charCodeAt(i);
  return diff === 0 ? sessionId : null;
}

export async function sha256Hex(input) {
  const data = typeof input === "string" ? enc.encode(input) : input;
  const digest = await crypto.subtle.digest("SHA-256", data);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}
