import { ok, err, readJson, handleError, str } from "../../_lib/http.js";
import { verifyPassword } from "../../_lib/crypto.js";
import { createSession, sessionCookie } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "login", { limit: 15, windowSec: 300 });
    const body = await readJson(context.request);
    const loginId = str(body.login_id, { name: "login_id", min: 1, max: 254 });
    const password = str(body.password, { name: "password", min: 1, max: 256, trim: false });

    const user = await context.env.DB.prepare(
      "SELECT id, login_id, email, email_verified, password_algo, password_iter, password_salt, password_hash, status FROM users WHERE login_id = ?",
    )
      .bind(loginId)
      .first();

    // Same generic error whether the id is unknown or the password is wrong.
    if (!user || user.status === "disabled" || !(await verifyPassword(password, user))) {
      return err("Invalid id or password", 401, "bad_credentials");
    }
    // A 'pending' account proved its password but hasn't verified its email yet —
    // block login with a distinct code so the UI can offer "resend verification".
    if (user.status === "pending") {
      return err("이메일 인증이 필요합니다. 받은 인증 메일의 링크를 열어 주세요.", 403, "email_unverified");
    }

    const { cookieValue } = await createSession(context.env, user.id);
    return ok(
      { user: { id: user.id, login_id: user.login_id, email: user.email, email_verified: !!user.email_verified } },
      { "Set-Cookie": sessionCookie(cookieValue, 30 * 24 * 3600) },
    );
  } catch (e) {
    return handleError(e);
  }
}
