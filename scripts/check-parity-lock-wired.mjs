#!/usr/bin/env node
// Guard the DoD ↔ rust.yml parity lock against being deleted.
//
// Why this exists — measured, not assumed. The lock itself is two files:
//   wie_cli/tests/dod_ci_parity.rs          the #[test]s that cargo runs
//   wie_cli/tests/support/dod_ci_parity.rs  the checker they call
// The test declares the checker with #[path], so deleting EITHER file alone is a
// compile error and `cargo test --all` goes red (measured: rc=101). Deleting BOTH
// removes the test target itself, and cargo then has nothing to fail on — the lock
// vanishes and CI stays green. Nothing in the repo referenced those paths from the
// outside, so nothing noticed.
//
// This script is that outside reference. The workflow step names it by path, so
// deleting the script reddens the step; the script names the lock by path, so
// deleting the lock reddens the script. That is the whole mechanism.
//
// "Deleted" is not only `rm`. A file that still exists but no longer wires anything
// is the same outage with a better disguise, so the checks below assert the two
// couplings that make the lock load-bearing rather than merely present:
//   - the #[path] declaration (what makes single-file deletion a compile error), and
//   - a #[test] that actually calls the checker (what makes it a check at all).
//
// What it CANNOT see is stated in CEILINGS below and printed on every run — a guard
// whose limits live only in a commit message is a guard nobody can reason about.
//
// Usage: node scripts/check-parity-lock-wired.mjs   (exit 0 = wired, 1 = not)

import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// The lock, described once. Both paths are load-bearing shapes, not just names:
// `test` must sit directly in a crate's `tests/` for cargo to auto-discover it, and
// `checker` must be where the test's #[path] points.
const LOCK = {
  test: "wie_cli/tests/dod_ci_parity.rs",
  checker: "wie_cli/tests/support/dod_ci_parity.rs",
  pathAttr: '#[path = "support/dod_ci_parity.rs"]',
  callsChecker: "checker::parity(",
};

const CEILINGS = [
  "이 가드는 «배선»을 보지 «의미»를 보지 않는다 — 아래 넷을 통과하면서 단언이 공허한 시험은 못 잡는다.",
  "그 잔여를 무는 것은 파리티 락 «자신»의 개악 대조(M1~M6)이고, 그것은 `cargo test --all` 이 6다리에서 돌린다.",
  "가드 «자신»의 삭제는 워크플로 스텝이 잡는다(스텝이 이 파일을 경로로 부른다). 스텝까지 지우면 diff 로만 보인다.",
];

const failures = [];
const notes = [];

async function readOrNull(rel) {
  try {
    return await readFile(path.join(root, rel), "utf8");
  } catch {
    return null;
  }
}

const testSrc = await readOrNull(LOCK.test);
const checkerSrc = await readOrNull(LOCK.checker);

// ⑴ Both files exist. This is the check that closes the "delete both" hole.
if (testSrc === null) failures.push(`${LOCK.test} 가 없다 — 파리티 락의 «시험»이 사라졌다(cargo 는 그것을 조용히 통과한다)`);
if (checkerSrc === null) failures.push(`${LOCK.checker} 가 없다 — 파리티 락의 «검사기»가 사라졌다`);

// ⑵ The test still lives where `cargo test --all` auto-discovers it: directly in a
//    crate's tests/ directory. A file moved one level down stops being a test target
//    while every other check here would still pass.
if (testSrc !== null) {
  const parts = LOCK.test.split("/");
  const shapeOk = parts.length === 3 && parts[1] === "tests" && parts[2].endsWith(".rs");
  if (!shapeOk) failures.push(`${LOCK.test} 가 <크레이트>/tests/<이름>.rs 형태가 아니다 — cargo 가 시험 대상으로 줍지 않는다`);
}

// ⑶ The coupling that makes single-file deletion a compile error.
if (testSrc !== null && !testSrc.includes(LOCK.pathAttr)) {
  failures.push(`${LOCK.test} 에 \`${LOCK.pathAttr}\` 선언이 없다 — 검사기를 지워도 «컴파일이 깨지지 않는다»(락의 자기방어가 풀린다)`);
}

// ⑷ It is a check, not an empty shell: at least one #[test] that calls the checker.
if (testSrc !== null) {
  const tests = (testSrc.match(/^\s*#\[test\]\s*$/gm) ?? []).length;
  if (tests === 0) failures.push(`${LOCK.test} 에 \`#[test]\` 가 하나도 없다 — 파일은 남았는데 «아무것도 돌지 않는다»`);
  else notes.push(`${LOCK.test}: #[test] ${tests}건`);
  if (!testSrc.includes(LOCK.callsChecker)) {
    failures.push(`${LOCK.test} 가 \`${LOCK.callsChecker}\` 를 부르지 않는다 — 검사기를 «선언만» 하고 «쓰지» 않는다`);
  }
}

console.log("PARITY-LOCK-WIRED  DoD ↔ rust.yml 파리티 락이 «배선돼 있는가»");
console.log(`  대상: ${LOCK.test}  ↔  ${LOCK.checker}`);
for (const n of notes) console.log(`    · ${n}`);
console.log("\n  [천장 — 이 가드가 «못 보는» 것]");
for (const c of CEILINGS) console.log(`    - ${c}`);

if (failures.length > 0) {
  console.log("\nFAIL 파리티 락이 배선돼 있지 않다:");
  for (const f of failures) console.log(`  ★ ${f}`);
  console.log("\n  ⇒ 락을 지우려면 그 «결정»을 먼저 하라. 이 가드는 「지웠는데 green」을 막으려고 있다");
  console.log("     (이 저장소에서 그 형태가 다섯 번 났다 — docs/worklog/2026-09-05-dod-ci-parity-checker.json).");
  process.exit(1);
}
console.log("\nOK 두 파일 실재 · #[path] 결합 · #[test] 가 검사기를 부른다 — 락이 «돈다»");
