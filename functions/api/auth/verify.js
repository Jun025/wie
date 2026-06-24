// GET /api/auth/verify?token=... — email verification landing.
//
// A GET link so it works from any email client. The token is one-time + hashed +
// expiring (see _lib/tokens.js). On success the account flips pending→active and
// email_verified→1. Returns a small self-contained HTML page (no SPA needed).

import { handleError } from "../../_lib/http.js";
import { consumeToken } from "../../_lib/tokens.js";

function page(title, message, okState) {
  const color = okState ? "#16a34a" : "#dc2626";
  return new Response(
    `<!doctype html><html lang="ko"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title}</title></head>
<body style="font-family:system-ui,-apple-system,sans-serif;background:#0f172a;color:#f1f5f9;display:flex;min-height:100vh;margin:0;align-items:center;justify-content:center">
<div style="max-width:420px;padding:32px;text-align:center">
<div style="font-size:40px;margin-bottom:12px">${okState ? "✅" : "⚠️"}</div>
<h1 style="font-size:20px;color:${color};margin:0 0 8px">${title}</h1>
<p style="color:#94a3b8;line-height:1.6">${message}</p>
<p style="margin-top:24px"><a href="/" style="display:inline-block;padding:10px 18px;background:#2563eb;color:#fff;border-radius:8px;text-decoration:none">WIE 열기</a></p>
</div></body></html>`,
    { status: okState ? 200 : 400, headers: { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" } },
  );
}

export async function onRequestGet(context) {
  try {
    const token = new URL(context.request.url).searchParams.get("token") || "";
    const userId = await consumeToken(context.env, token, "verify");
    if (!userId) {
      return page("인증 링크가 유효하지 않습니다", "링크가 만료되었거나 이미 사용되었습니다. 앱에서 인증 메일을 다시 받아 주세요.", false);
    }
    await context.env.DB.prepare("UPDATE users SET email_verified = 1, status = 'active', updated_at = ? WHERE id = ?")
      .bind(Date.now(), userId)
      .run();
    return page("이메일 인증 완료", "이제 WIE에서 이 계정으로 로그인할 수 있습니다.", true);
  } catch (e) {
    return handleError(e);
  }
}
