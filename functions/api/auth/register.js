import { ok, err, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { hashPassword, uuid } from "../../_lib/crypto.js";
import { createSession, sessionCookie } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";
import { emailConfigured, sendEmail, verifyEmailTemplate } from "../../_lib/email.js";
import { createToken, VERIFY_TTL_MS } from "../../_lib/tokens.js";

const EMAIL_RE = /^[^@\s]+@[^@\s]+\.[^@\s]+$/;

// Insert a user keyed by EMAIL (the sole identifier). Dual-path: the post-0004
// schema has no login_id column; the pre-0004 schema still has a NOT NULL
// login_id — so if the email-only insert fails on it, retry with login_id=email.
// This keeps registration working across the brief window before migration 0004
// is applied to the remote DB (S8).
async function insertUser(env, u) {
  try {
    await env.DB.prepare(
      `INSERT INTO users (id, email, email_verified, password_algo, password_iter, password_salt, password_hash, status, created_at, updated_at)
       VALUES (?, ?, 0, ?, ?, ?, ?, ?, ?, ?)`,
    )
      .bind(u.id, u.email, u.pw.algo, u.pw.iter, u.pw.salt, u.pw.hash, u.status, u.now, u.now)
      .run();
  } catch (e) {
    if (/login_id/i.test((e && e.message) || "")) {
      await env.DB.prepare(
        `INSERT INTO users (id, login_id, email, email_verified, password_algo, password_iter, password_salt, password_hash, status, created_at, updated_at)
         VALUES (?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?)`,
      )
        .bind(u.id, u.email, u.email, u.pw.algo, u.pw.iter, u.pw.salt, u.pw.hash, u.status, u.now, u.now)
        .run();
    } else {
      throw e;
    }
  }
}

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "register", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request);

    // Email is the login identifier now (no separate username).
    const email = str(body.email, { name: "email", min: 3, max: 254 }).toLowerCase();
    const password = str(body.password, { name: "password", min: 8, max: 256, trim: false });
    if (!EMAIL_RE.test(email)) throw new HttpError("이메일 형식이 올바르지 않습니다", 400, "bad_email");

    const existing = await context.env.DB.prepare("SELECT id FROM users WHERE email = ?").bind(email).first();
    if (existing) return err("이미 가입된 이메일입니다", 409, "duplicate");

    const pw = await hashPassword(password);
    const now = Date.now();
    const id = uuid();

    // With email delivery configured, the account starts 'pending' and must verify
    // before it becomes 'active'. With no email service we cannot verify, so the
    // account is 'active' immediately — graceful degradation, never a lockout.
    const willVerify = emailConfigured(context.env);
    const initialStatus = willVerify ? "pending" : "active";
    await insertUser(context.env, { id, email, pw, status: initialStatus, now });

    let emailSent = false;
    let emailStatus;
    if (willVerify) {
      const raw = await createToken(context.env, id, "verify", VERIFY_TTL_MS);
      const origin = new URL(context.request.url).origin;
      const url = `${origin}/api/auth/verify?token=${encodeURIComponent(raw)}`;
      const r = await sendEmail(context.env, { to: email, ...verifyEmailTemplate(url) });
      emailSent = r.ok;
      emailStatus = r.status;
    }

    // CRITICAL anti-lockout rule: gate as 'pending' ONLY when the verification
    // email actually went out. If sending failed (e.g. Resend test mode only
    // delivers to the account owner, so other recipients 403), un-gate to 'active'
    // and log the user in — otherwise that account could never verify.
    const pending = willVerify && emailSent;
    if (!pending) {
      if (initialStatus === "pending") {
        await context.env.DB.prepare("UPDATE users SET status = 'active', updated_at = ? WHERE id = ?").bind(Date.now(), id).run();
      }
      const { cookieValue } = await createSession(context.env, id);
      return ok(
        { user: { id, email, email_verified: false }, emailSent, emailStatus, emailConfigured: emailConfigured(context.env) },
        { "Set-Cookie": sessionCookie(cookieValue, 30 * 24 * 3600) },
      );
    }
    // Email sent → must verify before first login (no session yet).
    return ok({ user: { id, email, email_verified: false }, pending: true, emailSent, emailStatus });
  } catch (e) {
    return handleError(e);
  }
}
