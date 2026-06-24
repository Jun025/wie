// POST /api/auth/resend { login_id } — resend the email-verification link.
//
// No session required (a pending user cannot log in yet). Always returns ok so
// the endpoint can't be used to probe which ids exist. Rate-limited.

import { ok, readJson, handleError, str } from "../../_lib/http.js";
import { rateLimit } from "../../_lib/ratelimit.js";
import { emailConfigured, sendEmail, verifyEmailTemplate } from "../../_lib/email.js";
import { createToken, invalidateTokens, VERIFY_TTL_MS } from "../../_lib/tokens.js";

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "resend", { limit: 5, windowSec: 600 });
    const body = await readJson(context.request);
    const loginId = str(body.login_id, { name: "login_id", min: 1, max: 254 });

    const user = await context.env.DB.prepare("SELECT id, email, status FROM users WHERE login_id = ?").bind(loginId).first();
    if (user && user.email && user.status === "pending" && emailConfigured(context.env)) {
      await invalidateTokens(context.env, user.id, "verify");
      const raw = await createToken(context.env, user.id, "verify", VERIFY_TTL_MS);
      const origin = new URL(context.request.url).origin;
      const url = `${origin}/api/auth/verify?token=${encodeURIComponent(raw)}`;
      await sendEmail(context.env, { to: user.email, ...verifyEmailTemplate(url) });
    }
    // Always ok — never reveal whether the id exists or its state.
    return ok({ emailConfigured: emailConfigured(context.env) });
  } catch (e) {
    return handleError(e);
  }
}
