import { json, handleError } from "../../_lib/http.js";
import { getUser } from "../../_lib/session.js";

export async function onRequestGet(context) {
  try {
    const user = await getUser(context);
    if (!user) return json({ ok: true, authenticated: false }, 200);
    return json({ ok: true, authenticated: true, user: { id: user.id, login_id: user.login_id, email: user.email } }, 200);
  } catch (e) {
    return handleError(e);
  }
}
