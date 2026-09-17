#!/usr/bin/env node
// Guard the upstream-workflow guard against being un-wired.
//
// Why this exists — the same disease check-parity-lock-wired.mjs was written for,
// one file over. scripts/check-upstream-new-workflows.mjs runs from exactly ONE
// place: a step in .github/workflows/doc-liveness.yml. Delete that step and the
// script still sits in the tree, `git status` is clean, every check is green, and
// the only thing that changed is that nobody watches upstream any more.
// check-doc-liveness-parity.mjs cannot cover it: that checker compares the
// DOC-COPY regions only, and the rider lives OUTSIDE them on purpose (the rider is
// not a documented command). So the rider had no outside reference at all.
//
// ── What it asserts, and why the obvious version is vacuous ─────────────────
// ★A plain "does the workflow mention the path" test PASSES on a deleted step:
// doc-liveness.yml's header carries the literal string
// `scripts/check-upstream-new-workflows.mjs` in a COMMENT that explains the rider
// (measured: 2 hits in that file, one comment + one `run:` line). A guard that a
// leftover comment satisfies is not a guard. So a reference only counts when its
// line is not a comment line.
//
// ── Why it scans every workflow, not just doc-liveness.yml ──────────────────
// The load-bearing property is "some workflow invokes it", not "THAT workflow
// invokes it". check-parity-lock-wired learned this the expensive way on
// 2026-09-06: it pinned a spelling (`#[path = …]`), a correct refactor kept every
// property the pin bought, and the guard went red on working code — which is how a
// guard gets deleted by the next person. Pinning doc-liveness.yml by name would
// repeat that: moving the rider to another scheduled workflow keeps it running and
// would still red. So the file it lands in is REPORTED, not required.
//
// ── Ceiling, stated because a guard whose limits live in a commit message is
//    one nobody can reason about ────────────────────────────────────────────
// This sees WIRING, not CADENCE. Moving the rider onto a per-PR trigger passes
// here and is a real regression for a different reason (doc-liveness.yml's header
// explains it: an upstream-driven signal fired per-PR reddens every open PR at once
// for something none of them touched — the check-worklog-coverage outage of
// 2026-09-07). That axis is a diff review, not this script.
// And this guard's OWN deletion is caught the same way its sibling's is: the
// engine-contract.yml step names this file by path. Delete both and only the diff
// shows it — that regress is finite and deliberate, not an oversight.
//
// Usage: node scripts/check-upstream-guard-wired.mjs   (exit 0 = wired, 1 = not)

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// The guard, described once. `script` is the path a workflow has to name; it is
// also the path this script checks for existence, so a rename cannot leave a stale
// literal behind in only one of the two places.
const GUARD = { script: "scripts/check-upstream-new-workflows.mjs" };
const WORKFLOW_DIR = ".github/workflows";

const failures = [];
const notes = [];

/** Lines that are not comment lines — a leftover comment must never satisfy this. */
const invokesOn = (src, needle) =>
  src
    .split("\n")
    .map((line, i) => [i + 1, line])
    .filter(([, line]) => !line.trimStart().startsWith("#") && line.includes(needle));

let exists = true;
try {
  await readFile(path.join(root, GUARD.script), "utf8");
} catch {
  exists = false;
  failures.push(`${GUARD.script} 가 없다 — upstream 감시 가드 «자신»이 사라졌다`);
}

let files = [];
try {
  files = (await readdir(path.join(root, WORKFLOW_DIR))).filter((f) => f.endsWith(".yml") || f.endsWith(".yaml")).sort();
} catch {
  files = [];
}
// FAIL CLOSED: no workflow directory, or an unreadable one, is "cannot judge" —
// and "cannot judge" must never print as "wired".
if (files.length === 0) {
  failures.push(`${WORKFLOW_DIR}/ 에서 워크플로를 하나도 읽지 못했다 — 배선을 «판정할 수 없다»(통과가 아니다)`);
}

const wiredIn = [];
let commentOnly = 0;
for (const f of files) {
  const src = await readFile(path.join(root, WORKFLOW_DIR, f), "utf8").catch(() => null);
  if (src === null) {
    failures.push(`${WORKFLOW_DIR}/${f} 를 읽지 못했다 — 판정 불가`);
    continue;
  }
  if (!src.includes(GUARD.script)) continue;
  const hits = invokesOn(src, GUARD.script);
  if (hits.length > 0) wiredIn.push(`${f}:${hits.map(([n]) => n).join(",")}`);
  else commentOnly++;
}

if (exists && files.length > 0 && wiredIn.length === 0) {
  failures.push(
    `어떤 워크플로도 \`${GUARD.script}\` 를 «주석 아닌 줄»에서 부르지 않는다 — 파일은 남았는데 «아무 데서도 돌지 않는다»` +
      (commentOnly > 0 ? ` (★주석 언급은 ${commentOnly}개 파일에 남아 있다 — 그것으로는 돌지 않는다)` : ""),
  );
}

console.log("UPSTREAM-GUARD-WIRED  upstream 워크플로 감시 가드가 «배선돼 있는가»");
console.log(`  대상: ${GUARD.script}  ·  스캔: ${WORKFLOW_DIR}/ ${files.length}개`);
if (wiredIn.length > 0) notes.push(`호출(주석 아닌 줄): ${wiredIn.join(" · ")}`);
if (commentOnly > 0) notes.push(`주석 언급만 있는 파일: ${commentOnly}개 — ★배선으로 «세지 않는다»`);
for (const n of notes) console.log(`    · ${n}`);
console.log("\n  [천장 — 이 가드가 «못 보는» 것]");
console.log("    - 배선을 보지 «주기»를 보지 않는다 — 라이더를 PR 트리거로 옮기면 여기는 통과하고, 그것은 다른 이유로 회귀다(doc-liveness.yml 머리주석).");
console.log("    - 이 가드 «자신»의 삭제는 engine-contract.yml 스텝이 잡는다(스텝이 이 파일을 경로로 부른다). 둘 다 지우면 diff 로만 보인다.");
console.log("    - «어느» 워크플로인지는 요구하지 않고 «보고»만 한다 — 이름을 박으면 정당한 이동이 red 가 된다(check-parity-lock-wired 가 2026-09-06 에 그 오탐으로 실패했다).");

if (failures.length > 0) {
  console.log("\nFAIL upstream 감시 가드가 배선돼 있지 않다:");
  for (const f of failures) console.log(`  ★ ${f}`);
  console.log("\n  ⇒ 라이더를 내리려면 그 «결정»을 먼저 하라. 이 가드는 「지웠는데 green」을 막으려고 있다.");
  process.exit(1);
}
console.log(`\nOK ${GUARD.script} 실재 · 워크플로가 그것을 «주석 아닌 줄»에서 부른다 — 가드가 «돈다»`);
