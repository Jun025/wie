#!/usr/bin/env node
// Did the workspace graph regain a crate our Linux CI cannot build?
//
// Measured cause (2026-09-16, the upstream base swap). `wie-app`, upstream's Tauri
// desktop shell, arrived as ONE LINE in `Cargo.toml`'s `members` and dragged in
// tauri -> gtk -> atk -> glib-sys / soup3-sys. Our Linux runners apt-install
// `libasound2-dev` (rust.yml) and `libgtk-3-dev libasound2-dev` (coverage.yml) —
// gtk3 links libsoup2, not 3 — so THREE checks died there:
//   rust_ci (ubuntu, stable) · rust_ci (ubuntu, beta)  glib-2.0 not found
//   coverage                                           libsoup-3.0 not found
// while macOS and Windows stayed green, because those hosts never compile the GTK
// deps at all. That asymmetry is the whole reason this check exists: **the four
// local gates in AGENTS.md are structurally blind to it**, so the round that
// landed it saw four greens, and a human reviewer caught the red.
//
// The predicate is the LOCKFILE, not the members list, and that choice is the
// point. A members allowlist would be a second copy of `members` — every
// legitimate new crate would have to edit two places — and it would still miss the
// other way in: an EXISTING member gaining a GTK dependency. The lockfile names
// the resolved graph, so both paths show up as the same fact.
//
// Measured both directions on `origin/main` in a throwaway worktree: with
// `wie-app` excluded, every name below is absent from Cargo.lock (0 each); put it
// back into `members` and all five appear (1 each). So this file is a faithful
// witness of the graph, not a proxy for it.
//
// CEILINGS (printed on every run — a guard whose limits live in a commit message
// is a guard nobody can reason about):
//   - It knows the crate families we have ALREADY been burned by. A new system
//     dependency with a new name (openssl-sys, dbus-sys, …) passes until someone
//     adds it here — which is the moment a human should be looking anyway.
//   - It reads the lockfile with an anchored line match, not a TOML parse. That is
//     safe because cargo owns this file's formatting and writes `name = "…"` once
//     per [[package]]; `dependencies = [ "glib-sys" ]` entries are indented and do
//     not match. It is NOT safe to hand this script a hand-edited lockfile.
//   - A lockfile entry is target-independent: a crate reachable only under a
//     target we never build would still trip this. For THIS list that is the
//     desired behaviour — these arrive only with a desktop shell we decided not to
//     adopt — but do not generalise the list without re-deciding that.
//   - It does not check that the Linux runners' apt list is still what it was. If
//     someone installs libgtk-4/libsoup3 there, this guard becomes conservative
//     rather than wrong, and the entries below should be revisited.
//
// Usage: node scripts/check-linux-system-deps.mjs   (exit 0 = clean, 1 = regained)

import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// Each entry names a crate that FAILED on our Linux runners, or the root that
// brings them. Reasons are quoted from the failure, not paraphrased.
const FORBIDDEN = [
  { name: "glib-sys", why: "rust_ci(ubuntu, stable/beta): `The system library \\`glib-2.0\\` required by crate \\`glib-sys\\` was not found`" },
  { name: "soup3-sys", why: "coverage: `libsoup-3.0` not found — the runner installs gtk3, which links libsoup2" },
  { name: "gtk-sys", why: "same GTK family as glib-sys; reached through atk/gtk" },
  { name: "webkit2gtk-sys", why: "same family; the webview half of the desktop shell" },
  { name: "tauri", why: "the ROOT that pulls all of the above — upstream's desktop shell wie-app is its only consumer here" },
];

const lock = await readFile(path.join(root, "Cargo.lock"), "utf8");
const present = FORBIDDEN.filter((c) => new RegExp(`^name = "${c.name}"$`, "m").test(lock));

console.log("LINUX-SYSTEM-DEPS  Cargo.lock 이 «리눅스 CI 가 못 빌드하는» 크레이트를 되찾았는가");
console.log(`  대상: Cargo.lock · 감시 ${FORBIDDEN.length}종 — ${FORBIDDEN.map((c) => c.name).join(", ")}`);
console.log("\n  [천장 — 이 가드가 «못 보는» 것]");
console.log("    - 이미 덴 계열만 안다. 새 이름의 시스템 의존(openssl-sys 등)은 누가 여기 적기 전까지 통과한다.");
console.log("    - TOML 파서가 아니라 «앵커된 줄 일치»다(cargo 가 이 파일의 서식을 소유하므로 안전 · 손으로 고친 lock 은 대상 아님).");
console.log("    - lock 항목은 타깃 독립이라 «안 빌드하는 타깃에서만 닿는» 크레이트도 문다 — 이 목록에서는 의도된 동작이다.");
console.log("    - 러너의 apt 목록이 바뀌었는지는 보지 않는다(바뀌면 이 가드는 틀린 게 아니라 보수적이 된다).");

if (present.length > 0) {
  console.log("\nFAIL workspace 그래프가 리눅스에서 못 빌드하는 크레이트를 다시 갖고 있다:");
  for (const c of present) console.log(`  ★ ${c.name} — ${c.why}`);
  console.log("\n  누가 끌어왔는지: cargo tree --workspace --all-features -i <크레이트> --target x86_64-unknown-linux-gnu");
  console.log("  ⇒ 우리가 쓰지 않는 크레이트면 Cargo.toml 의 `members` 에서 빼고 `exclude` 로 선언하라");
  console.log("     (사유 주석은 그 자리에 이미 있다 — 2026-09-16 wie-app 처분). 채택하기로 «결정»했다면");
  console.log("     리눅스 러너의 apt 목록을 먼저 늘리고, 그때 이 목록에서 해당 줄을 지워라.");
  process.exit(1);
}
console.log("\nOK 감시 대상 전건 부재 — 리눅스 레그가 빌드할 수 없는 의존이 그래프에 없다");
