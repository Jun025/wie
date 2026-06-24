import { ok, handleError } from "../_lib/http.js";

export async function onRequestGet(context) {
  try {
    // Touch D1 so the health check also proves the binding works.
    const row = await context.env.DB.prepare("SELECT 1 AS up").first();
    return ok({ service: "wie-web", db: row && row.up === 1 });
  } catch (e) {
    return handleError(e);
  }
}
