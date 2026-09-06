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
//   - the test names the checker's PATH (what points the module at that file), and
//   - a #[test] that actually calls the checker (what makes it a check at all).
//
// Axis ⑶ used to assert the exact `#[path = "…"]` ATTRIBUTE. Measured 2026-09-06,
// that spelling was the wrong thing to hold: a complete `include!` refactor keeps
// every property the attribute buys — deleting the checker alone is still a compile
// error (measured rc=101) — yet `cargo test` stayed green (11 passed) while this
// guard went red. A false positive on a correct refactor is how a guard gets deleted
// by the next person, so the axis now asserts the PATH, which is what actually aims
// the module at the checker and is load-bearing under any spelling. Nothing stops
// being caught: dropping the attribute leaves `mod checker;` unresolvable, which
// this guard still reddens AND `cargo test --all` fails on in all six legs (rc=101).
//
// Axis ⑷ is NOT relaxed, and the measurement says why: with the files present and
// the #[path] intact, a test that declares the module but never calls it compiles
// and passes (measured `cargo test` rc=0, 1 passed). Cargo has no backstop there —
// axis ⑷ is the only thing between "the lock is present" and "the lock runs".
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
// `checker` must be where the test's module declaration points, whatever the spelling.
const LOCK = {
  test: "wie_cli/tests/dod_ci_parity.rs",
  checker: "wie_cli/tests/support/dod_ci_parity.rs",
  callsChecker: "checker::parity(",
};

// How the test has to spell the checker to aim a module at it — derived, not a second
// constant, so moving the checker cannot leave a stale literal behind.
LOCK.checkerRef = path.relative(path.dirname(LOCK.test), LOCK.checker);

const CEILINGS = [
  "이 가드는 «배선»을 보지 «의미»를 보지 않는다 — 아래 넷을 통과하면서 단언이 공허한 시험은 못 잡는다.",
  "그 잔여를 무는 것은 파리티 락 «자신»의 개악 대조(M1~M6)이고, 그것은 `cargo test --all` 이 6다리에서 돌린다.",
  "가드 «자신»의 삭제는 워크플로 스텝이 잡는다(스텝이 이 파일을 경로로 부른다). 스텝까지 지우면 diff 로만 보인다.",
  "축⑷ 는 문자열 «부분일치»다 — 모듈 별칭을 바꿔도 접미가 같으면 통과한다(실측: `parity_checker::parity(`). 별칭까지 보려면 Rust 파서가 필요하고, 그 비용은 재서 기각했다(docs/worklog/2026-09-06-parity-lock-guard-axes.json).",
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

// ⑶ The coupling that aims the module at the checker — asserted as the PATH, not as
//    one spelling of it (`#[path = …]` and `include!(…)` both keep the property).
if (testSrc !== null && !testSrc.includes(LOCK.checkerRef)) {
  failures.push(`${LOCK.test} 가 \`${LOCK.checkerRef}\` 를 가리키지 않는다 — 모듈이 검사기를 겨누지 않는다(락의 자기방어가 풀린다)`);
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
console.log(`\nOK 두 파일 실재 · 시험이 \`${LOCK.checkerRef}\` 를 겨눈다 · #[test] 가 검사기를 부른다 — 락이 «돈다»`);
