// One way to ask cargo what is in this workspace, and one meaning for "it did not answer".
//
// WHY THIS IS SHARED AND NOT COPIED. Two scripts need the workspace shape from cargo
// rather than from a path guess: `checker-census.mjs` (which integration-test files does
// `cargo test --all` actually compile) and `game-lab-census-map.mjs` (which paths count as
// "the engine moved"). Both had to make the same three decisions — how to invoke cargo, how
// big a buffer, and what a failure returns — and the third is the one that matters: this
// repo's standing rule is that "could not measure" must never render as a confident number.
// `checker-census.mjs` already states it ("reported as UNMEASURABLE, never folded into
// '0 callers' ... fail toward a confident wrong number"). A second hand-written copy of that
// try/catch is one edit away from returning `[]` instead of `null`, and `[]` is exactly the
// fail-open both callers exist to remove — silently, because an empty list looks measured.
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
