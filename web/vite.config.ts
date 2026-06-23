import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Static client-only SPA. No SSR, no server runtime.
// NOTE: we deliberately do NOT set COOP/COEP headers — the emulator core is
// single-threaded and uses no SharedArrayBuffer/wasm-threads, so cross-origin
// isolation is unnecessary (see docs/web.md).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  build: {
    target: "es2022",
    outDir: "dist",
  },
});
