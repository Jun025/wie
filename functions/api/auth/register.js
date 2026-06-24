import { ok, err, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { hashPassword, uuid } from "../../_lib/crypto.js";
import { createSession, sessionCookie } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "register", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request);

    const loginId = str(body.login_id, { name: "login_id", min: 3, max: 254 });
    const password = str(body.password, { name: "password", min: 8, max: 256, trim: false });
    // email is optional; reserved for future verification flow.
    const email = body.email == null || body.email === "" ? null : str(body.email, { name: "email", max: 254 });

    if (!/^[\w.+@-]+$/.test(loginId)) {
      throw new HttpError("login_id may contain letters, digits and . _ + - @ only", 400);
    }

    const existing = await context.env.DB.prepare("SELECT id FROM users WHERE login_id = ?").bind(loginId).first();
    if (existing) return err("That id is already taken", 409, "duplicate");

    const pw = await hashPassword(password);
    const now = Date.now();
    const id = uuid();

    await context.env.DB.prepare(
      `INSERT INTO users (id, login_id, email, email_verified, password_algo, password_iter, password_salt, password_hash, status, created_at, updated_at)
       VALUES (?, ?, ?, 0, ?, ?, ?, ?, 'active', ?, ?)`,
    )
      .bind(id, loginId, email, pw.algo, pw.iter, pw.salt, pw.hash, now, now)
      .run();

    const { cookieValue } = await createSession(context.env, id);
    return ok({ user: { id, login_id: loginId, email } }, { "Set-Cookie": sessionCookie(cookieValue, 30 * 24 * 3600) });
  } catch (e) {
    return handleError(e);
  }
}
