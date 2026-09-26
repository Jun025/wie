// One way to ask cargo what is in this workspace, and one meaning for "it did not answer".
//
// WHY THIS IS A FILE. `game-lab-census-map.mjs` needs the workspace shape from cargo (which
// paths count as "the engine moved") rather than from a path guess. It was shared with
// `checker-census.mjs` until that was removed (2026-09-26). The part that matters is the
// failure semantics: "could not measure" must never render as a confident number, so a
// failure returns `null`, never `[]` — an empty list looks measured.
//
// ★So what is shared is the FAILURE SEMANTICS, not a list. No path, crate name or glob lives
// here: each caller projects the metadata itself, because they want different things out of
// it. Sharing the projection would be the "two copies of one truth" this repo condemns in the
// other direction — a helper that guesses what its callers meant.
//
// Returns the parsed `cargo metadata --no-deps` object, or `null` when cargo is absent,
// offline, refuses, or emits something unparseable. Never throws, never returns a partial.

import { execFileSync } from "node:child_process";

export function cargoMetadata(cwd) {
  let raw;
  try {
    raw = execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return null; // absent / offline / refused — reported by the caller, never folded into a count
  }
  try {
    const meta = JSON.parse(raw);
    // A shape change is a failure, not an empty workspace: every tree that has a Cargo.toml
    // has at least one package, so `packages: []` means the JSON is not what we think it is.
    if (!meta || typeof meta.workspace_root !== "string" || !Array.isArray(meta.packages) || meta.packages.length === 0) return null;
    return meta;
  } catch {
    return null;
  }
}

// `workspace_root` has no trailing slash; every caller here wants repo-relative paths.
export function workspaceRelative(meta) {
  const prefix = meta.workspace_root.endsWith("/") ? meta.workspace_root : meta.workspace_root + "/";
  return (abs) => (typeof abs === "string" && abs.startsWith(prefix) ? abs.slice(prefix.length) : null);
}
