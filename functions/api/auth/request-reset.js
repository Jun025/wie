// POST /api/auth/request-reset { email } — send a password-reset link.
//
// Always returns ok (no account enumeration). The reset link points at the SPA
// (`/?reset=TOKEN`) which shows a "set new password" form posting to /auth/reset.
// Rate-limited; tokens are one-time, hashed, 1h-expiring.

import { ok, readJson, handleError, str } from "../../_lib/http.js";
import { rateLimit } from "../../_lib/ratelimit.js";
import { emailConfigured, sendEmail, resetEmailTemplate } from "../../_lib/email.js";
import { createToken, invalidateTokens, RESET_TTL_MS } from "../../_lib/tokens.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "request-reset", { limit: 5, windowSec: 600 });
    const body = await readJson(context.request);
    const email = str(body.email, { name: "email", min: 1, max: 254 }).toLowerCase();

    const user = await context.env.DB.prepare("SELECT id, email FROM users WHERE email = ?").bind(email).first();
    if (user && user.email && emailConfigured(context.env)) {
      await invalidateTokens(context.env, user.id, "reset");
      const raw = await createToken(context.env, user.id, "reset", RESET_TTL_MS);
      const origin = new URL(context.request.url).origin;
      const url = `${origin}/?reset=${encodeURIComponent(raw)}`;
      await sendEmail(context.env, { to: user.email, ...resetEmailTemplate(url) });
    }
    return ok({ emailConfigured: emailConfigured(context.env) });
  } catch (e) {
    return handleError(e);
  }
}
