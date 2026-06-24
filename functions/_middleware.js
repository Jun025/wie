// Global middleware: applied to every request (static assets + API).
// Adds security headers. NOTE: COOP/COEP are intentionally NOT set — the wasm
// emulator is single-threaded (no SharedArrayBuffer), so cross-origin isolation
// is unnecessary and would only add friction.
//
// `connect-src 'self'` is defense-in-depth for the core guarantee: even if some
// bug tried to POST game bytes to a third party, the browser would block it.
// (The real guarantee is that no such code exists — see the network self-audit.)

const SECURITY_HEADERS = {
  "X-Content-Type-Options": "nosniff",
  "Referrer-Policy": "no-referrer",
  "X-Frame-Options": "DENY",
  // The app needs none of these powerful features; deny them outright.
  "Permissions-Policy": "camera=(), microphone=(), geolocation=(), usb=(), payment=(), interest-cohort=()",
  "Content-Security-Policy": [
    "default-src 'self'",
    "script-src 'self' 'wasm-unsafe-eval'",
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data: blob:",
    "connect-src 'self'",
    "worker-src 'self' blob:",
    "font-src 'self'",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
  ].join("; "),
};

export async function onRequest(context) {
  const response = await context.next();
  const headers = new Headers(response.headers);
  for (const [k, v] of Object.entries(SECURITY_HEADERS)) headers.set(k, v);
  return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
}
