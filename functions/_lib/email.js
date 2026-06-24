// Transactional email via Resend (https://resend.com).
//
// WHY Resend: it exposes a plain HTTPS JSON API (no SMTP socket, which Workers
// cannot open), works from Cloudflare Pages Functions with a single `fetch`, and
// has a real free tier — at time of writing 3,000 emails/month and 100/day, which
// is ample for verification + password-reset mail on a hobby service. (MailChannels'
// formerly-free Workers route was discontinued in 2024, so it is no longer viable.)
//
// FREE-TIER REQUIREMENTS the user must satisfy (documented in CLOUDFLARE_SETUP):
//   • RESEND_API_KEY — a Resend API key (Pages secret; never committed — S3).
//   • EMAIL_FROM     — a verified sender, e.g. "WIE <noreply@yourdomain>".
//                      Resend requires a verified domain to send to arbitrary
//                      recipients; the sandbox `onboarding@resend.dev` only
//                      delivers to the account owner.
//
// GRACEFUL DEGRADATION: if RESEND_API_KEY/EMAIL_FROM are absent, emailConfigured()
// is false and callers skip all mail steps — registration/login keep working,
// only the verify/reset features are disabled (with a clear in-app notice).
//
// Secrets are referenced from `env` only and are NEVER logged.

export function emailConfigured(env) {
  return !!(env && env.RESEND_API_KEY && env.EMAIL_FROM);
}

// Send one email. Returns { ok: true } on success, or { ok:false, error } — it
// never throws, so a mail failure cannot 500 an auth request.
export async function sendEmail(env, { to, subject, html, text }) {
  if (!emailConfigured(env)) return { ok: false, error: "email_not_configured" };
  try {
    // Endpoint is overridable (RESEND_API_BASE) for local testing / a compatible
    // proxy; defaults to Resend's API. Never logged.
    const base = env.RESEND_API_BASE || "https://api.resend.com";
    const res = await fetch(`${base}/emails`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${env.RESEND_API_KEY}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({ from: env.EMAIL_FROM, to, subject, html, text }),
    });
    if (!res.ok) {
      // Log only the status, never the key or recipient PII. The numeric status
      // is returned (not secret) so the operator can tell a valid key in test
      // mode (403 = recipient restricted) from a bad key (401).
      console.error("resend send failed:", res.status);
      return { ok: false, error: `send_failed_${res.status}`, status: res.status };
    }
    return { ok: true, status: 200 };
  } catch (e) {
    console.error("resend send error:", e && e.message);
    return { ok: false, error: "send_error" };
  }
}

// Plain, link-based templates. The link points at the GET verify/reset endpoints
// so it works from any email client without the SPA being loaded first.
export function verifyEmailTemplate(verifyUrl) {
  return {
    subject: "WIE 이메일 인증",
    text: `WIE 계정 이메일을 인증하려면 아래 링크를 여세요 (24시간 유효):\n\n${verifyUrl}\n\n본인이 요청하지 않았다면 이 메일을 무시하세요.`,
    html: `<p>WIE 계정 이메일을 인증하려면 아래 버튼을 누르세요. (24시간 유효)</p>
<p><a href="${verifyUrl}" style="display:inline-block;padding:10px 16px;background:#2563eb;color:#fff;border-radius:6px;text-decoration:none">이메일 인증</a></p>
<p style="color:#666;font-size:12px">버튼이 안 되면 이 주소를 여세요:<br>${verifyUrl}</p>
<p style="color:#666;font-size:12px">본인이 요청하지 않았다면 이 메일을 무시하세요.</p>`,
  };
}

export function resetEmailTemplate(resetUrl) {
  return {
    subject: "WIE 비밀번호 재설정",
    text: `WIE 비밀번호를 재설정하려면 아래 링크를 여세요 (1시간 유효):\n\n${resetUrl}\n\n본인이 요청하지 않았다면 이 메일을 무시하세요. 비밀번호는 변경되지 않습니다.`,
    html: `<p>WIE 비밀번호를 재설정하려면 아래 버튼을 누르세요. (1시간 유효)</p>
<p><a href="${resetUrl}" style="display:inline-block;padding:10px 16px;background:#2563eb;color:#fff;border-radius:6px;text-decoration:none">비밀번호 재설정</a></p>
<p style="color:#666;font-size:12px">버튼이 안 되면 이 주소를 여세요:<br>${resetUrl}</p>
<p style="color:#666;font-size:12px">본인이 요청하지 않았다면 이 메일을 무시하세요. 비밀번호는 변경되지 않습니다.</p>`,
  };
}
