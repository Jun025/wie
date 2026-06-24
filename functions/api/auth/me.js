import { json, handleError } from "../../_lib/http.js";
import { getUser } from "../../_lib/session.js";
import { emailConfigured } from "../../_lib/email.js";

export async function onRequestGet(context) {
  try {
    const user = await getUser(context);
    if (!user) return json({ ok: true, authenticated: false, emailConfigured: emailConfigured(context.env) }, 200);
    return json(
      {
        ok: true,
        authenticated: true,
        emailConfigured: emailConfigured(context.env),
        user: { id: user.id, login_id: user.login_id, email: user.email, email_verified: !!user.email_verified },
      },
      200,
    );
  } catch (e) {
    return handleError(e);
  }
}
