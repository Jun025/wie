import { json, handleError } from "../../_lib/http.js";
import { getUser } from "../../_lib/session.js";
import { emailConfigured } from "../../_lib/email.js";
import { filesEnabled } from "../../_lib/files.js";

export async function onRequestGet(context) {
  try {
    const user = await getUser(context);
    // Presence-only email diagnostic (NEVER the values) so the operator can see
    // which env var is missing without exposing secrets.
    const emailDiag = { hasKey: !!context.env.RESEND_API_KEY, hasFrom: !!context.env.EMAIL_FROM };
    // Whether the server-side file vault (R2 binding) is provisioned (S8).
    const files = filesEnabled(context.env);
    if (!user) return json({ ok: true, authenticated: false, emailConfigured: emailConfigured(context.env), emailDiag, filesConfigured: files }, 200);
    return json(
      {
        ok: true,
        authenticated: true,
        emailConfigured: emailConfigured(context.env),
        emailDiag,
        filesConfigured: files,
        user: { id: user.id, login_id: user.login_id, email: user.email, email_verified: !!user.email_verified },
      },
      200,
    );
  } catch (e) {
    return handleError(e);
  }
}
