import { ok, handleError } from "../../_lib/http.js";
import { getUser, revokeSession, clearCookie } from "../../_lib/session.js";

export async function onRequestPost(context) {
  try {
    const user = await getUser(context);
    if (user) await revokeSession(context.env, user.sessionId);
    return ok({}, { "Set-Cookie": clearCookie() });
  } catch (e) {
    return handleError(e);
  }
}
