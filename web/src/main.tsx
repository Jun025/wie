import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./index.css";

// Apply the saved theme before React renders (the useTheme hook keeps it in sync
// afterwards). Done in bundled JS — not an inline <script> — to satisfy the CSP
// (`script-src 'self'`). Device-local preference only; never sent to the server.
try {
  const t = localStorage.getItem("wie-theme");
  document.documentElement.setAttribute("data-theme", t === "light" ? "light" : "dark");
} catch {
  document.documentElement.setAttribute("data-theme", "dark");
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
