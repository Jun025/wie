import { ok, err, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { hashPassword, uuid } from "../../_lib/crypto.js";
import { createSession, sessionCookie } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";
import { emailConfigured, sendEmail, verifyEmailTemplate } from "../../_lib/email.js";
import { createToken, VERIFY_TTL_MS } from "../../_lib/tokens.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "register", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request);

    const loginId = str(body.login_id, { name: "login_id", min: 3, max: 254 });
    const password = str(body.password, { name: "password", min: 8, max: 256, trim: false });
    const email = body.email == null || body.email === "" ? null : str(body.email, { name: "email", max: 254 });

    if (!/^[\w.+@-]+$/.test(loginId)) {
      throw new HttpError("login_id may contain letters, digits and . _ + - @ only", 400);
    }
    if (email && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)) {
      throw new HttpError("이메일 형식이 올바르지 않습니다", 400, "bad_email");
    }

    const existing = await context.env.DB.prepare("SELECT id FROM users WHERE login_id = ?").bind(loginId).first();
    if (existing) return err("That id is already taken", 409, "duplicate");

    const pw = await hashPassword(password);
    const now = Date.now();
    const id = uuid();

    // When email delivery is configured AND the user supplied an email, the
    // account starts 'pending' and must verify before it becomes 'active'. With
    // no email service (or no email given) we cannot verify, so the account is
    // 'active' immediately — graceful degradation, never a lockout.
    const willVerify = !!email && emailConfigured(context.env);

    // Insert tentatively; the verification token's FK needs the row to exist
    // before we attempt to send. We finalize the status after the send result.
    const initialStatus = willVerify ? "pending" : "active";
    await context.env.DB.prepare(
      `INSERT INTO users (id, login_id, email, email_verified, password_algo, password_iter, password_salt, password_hash, status, created_at, updated_at)
       VALUES (?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?)`,
    )
      .bind(id, loginId, email, pw.algo, pw.iter, pw.salt, pw.hash, initialStatus, now, now)
      .run();

    let emailSent = false;
    let emailStatus; // numeric Resend HTTP status when a send was attempted (diagnostic, not secret)
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
    // delivers to the account owner, so other recipients 403), un-gate to
    // 'active' and log the user in — otherwise that account could never verify
    // and would be locked out forever.
    const pending = willVerify && emailSent;
    if (!pending) {
      if (initialStatus === "pending") {
        await context.env.DB.prepare("UPDATE users SET status = 'active', updated_at = ? WHERE id = ?").bind(Date.now(), id).run();
      }
      const { cookieValue } = await createSession(context.env, id);
      return ok(
        { user: { id, login_id: loginId, email, email_verified: false }, emailSent, emailStatus, emailConfigured: emailConfigured(context.env) },
        { "Set-Cookie": sessionCookie(cookieValue, 30 * 24 * 3600) },
      );
    }
    // Email sent → must verify before first login (no session yet).
    return ok({ user: { id, login_id: loginId, email, email_verified: false }, pending: true, emailSent, emailStatus });
  } catch (e) {
    return handleError(e);
  }
}
