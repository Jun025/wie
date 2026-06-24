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
      "SELECT id, login_id, email, password_algo, password_iter, password_salt, password_hash, status FROM users WHERE login_id = ?",
    )
      .bind(loginId)
      .first();

    // Same generic error whether the id is unknown or the password is wrong.
    if (!user || user.status !== "active" || !(await verifyPassword(password, user))) {
      return err("Invalid id or password", 401, "bad_credentials");
    }

    const { cookieValue } = await createSession(context.env, user.id);
    return ok(
      { user: { id: user.id, login_id: user.login_id, email: user.email } },
      { "Set-Cookie": sessionCookie(cookieValue, 30 * 24 * 3600) },
    );
  } catch (e) {
    return handleError(e);
  }
}
