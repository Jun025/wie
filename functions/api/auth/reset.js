// POST /api/auth/reset { token, password } — complete a password reset.
//
// Consumes a one-time reset token, sets a new PBKDF2(100k) hash (same policy as
// register), marks the email verified (proves mailbox control) + active, and
// revokes all existing sessions so a leaked old session can't survive a reset.

import { ok, err, readJson, handleError, str } from "../../_lib/http.js";
import { hashPassword } from "../../_lib/crypto.js";
import { consumeToken } from "../../_lib/tokens.js";
import { rateLimit } from "../../_lib/ratelimit.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "reset", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request);
    const token = str(body.token, { name: "token", min: 16, max: 256, trim: false });
    const password = str(body.password, { name: "password", min: 8, max: 256, trim: false });

    const userId = await consumeToken(context.env, token, "reset");
    if (!userId) return err("재설정 링크가 만료되었거나 이미 사용되었습니다", 400, "bad_token");

    const pw = await hashPassword(password);
    const now = Date.now();
    await context.env.DB.prepare(
      "UPDATE users SET password_algo = ?, password_iter = ?, password_salt = ?, password_hash = ?, email_verified = 1, status = 'active', updated_at = ? WHERE id = ?",
    )
      .bind(pw.algo, pw.iter, pw.salt, pw.hash, now, userId)
      .run();
    await context.env.DB.prepare("UPDATE sessions SET revoked = 1 WHERE user_id = ?").bind(userId).run();

    return ok({});
  } catch (e) {
    return handleError(e);
  }
}
