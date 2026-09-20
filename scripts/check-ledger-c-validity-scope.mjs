#!/usr/bin/env node
// check-ledger-c-validity-scope.mjs — "does every §C row still say how long its verdict holds?"
//
// ── What rule this enforces, and where the rule already was ─────────────────
// `docs/project-kb/02_status.md` §C preserves the EXPOSURE VERDICT for advisories we have
// already resolved — "do we walk that code path", not "is the version patched" — because the
// expensive part is the reasoning, and without it the next advisory re-derives it from zero.
// Those verdicts expire: they are read off a particular version's source, so a dependency
// move invalidates them. That is what the last column, 「판정 유효 범위」, records, and the
// section header has commanded it in prose since it was written:
//   ★그 칸이 빈 행을 만들지 마라 — 유효 범위 없는 판정은 낡은 채로 권위 있게 인용된다.
// Nothing read that sentence. This does.
//
// ── Honest about why it exists: the base rate is ZERO ───────────────────────
// ★This is NOT an incident response. Measured over every first-parent commit that has ever
// carried a §C row (6 commits, 4 rows): **6 columns every time, empty scope cells 0**. Nobody
// has broken this rule yet. What is being closed is the gap between a
// written obligation and anything that reads it — the decay shape AGENTS.md names about its own
// unchecked prose obligations.
//
// ★The sibling round `docs/report/0195` DECLINED to build a checker at base rate 0, and that is
// not a contradiction — the difference is the predicate, not the count:
//   · there, the only available predicate was a PROXY for "did the round run the tool", and it
//     measured 3 false positives and 0 true positives. A proxy with no true positives is
//     harmful, because it will fire falsely.
//   · here the predicate is EXACT — "is the last cell empty" is literally the rule, with no
//     inference in between. An exact guard with no true positives is merely idle, and idle
//     costs ~10ms.
//
// ── What it deliberately does NOT do (the "두 벌" line) ─────────────────────
// ★It does not compare §C to `Cargo.lock`. That was weighed and rejected in the round that
// wrote this section (`docs/report/0186`): both failure classes ALREADY redden — a gating
// downgrade fails `cargo audit`, and a non-gating one fails `scripts/check-audit-warnings.mjs`
// via §A set equality. Re-detecting them here would be the duplicate-machinery this repo keeps
// condemning. This is a FORMAT check, which neither of those two can see.
//
// ── Why here and not inside check-audit-warnings.mjs ────────────────────────
// That script already parses this very file, so folding this in would add no new file. Measured,
// two things rule it out. ⑴ It runs from `rust-audit.yaml`, which is **schedule + dispatch only**
// — so the red would land a day after the PR that caused it, in the same daily job this lineage
// measured sitting red and **unattended for three days** (`docs/report/0181`). ⑵ It calls
// `cargo audit` at module top level, so a format check folded into it would die whenever
// cargo-audit is missing or broken — a dependency this check has no business having. It runs in
// `engine-contract.yml`'s always-run `contract` job instead: node-only, on every PR.
//
// Usage: node scripts/check-ledger-c-validity-scope.mjs
// Exit: 0 = every row scoped (or the table is empty, said out loud) · 1 = a row is unscoped
//       · 2 = could not measure (the table or its last column moved — never a silent pass).

import { readFile } from "node:fs/promises";

const FILE = "docs/project-kb/02_status.md";
const COLUMN = "판정 유효 범위";

const die = (m) => {
  console.error(`check-ledger-c-validity-scope: ★못 쟀다 — ${m}`);
  console.error(`  ★이것은 «통과»가 아니다. ${FILE} 의 §C 표나 그 마지막 칸이 움직였다면 이 검사기를 같이 옮겨라.`);
  process.exit(2);
};

let text;
try {
  text = await readFile(FILE, "utf8");
} catch (e) {
  die(`${FILE} 을 읽지 못했다 (${e.message})`);
}

// The header row is located by the column NAME, not by a line number or a table index — a line
// number rots on the first edit above it, and this file is long and edited often.
const lines = text.split("\n");
const headerIdx = lines.findIndex((l) => l.trimStart().startsWith("|") && l.includes(COLUMN));
if (headerIdx < 0) die(`머리행에서 「${COLUMN}」 칸을 찾지 못했다`);

// ★Cells are read by position from the END, and only the last one. That is what makes this
// parser small enough to be right: `<br>` and the old-ID annotation (`**C-2**<br>(구 **A-2**)`)
// live in the FIRST cell, and an escaped pipe anywhere before the last cell only shifts fields
// on the left. The one thing that must be true is that 「판정 유효 범위」 IS the last column —
// asserted here rather than assumed, because checking the wrong column silently is worse than
// not checking.
const cellsOf = (line) => line.split("|").slice(1, -1).map((s) => s.trim());
const header = cellsOf(lines[headerIdx]);
if (!header.at(-1)?.includes(COLUMN)) die(`「${COLUMN}」 가 «마지막» 칸이 아니다 — 지금 마지막 칸은 「${header.at(-1)}」 이다`);

const rows = lines.filter((l) => /^\|\s*\*\*C-\d+\*\*/.test(l));
console.log(`check-ledger-c-validity-scope — §C 의 「${COLUMN}」 칸이 빈 행이 있는가`);
console.log(`  대상: ${FILE} · 머리행 ${headerIdx + 1}줄 · ${header.length}칸 · C 행 ${rows.length}건`);

if (rows.length === 0) {
  console.log(`  ★C 행 0건 — 표가 비었거나 줄머리 서식(\`| **C-<n>**\`)이 바뀌었다. ★판정이 아니라 «셀 것이 없었다»는 보고다.`);
  process.exit(0);
}

let bad = 0;
for (const line of rows) {
  const c = cellsOf(line);
  const id = c[0].replace(/<br>.*$/, "").replace(/\*/g, "").trim();
  const scope = c.at(-1) ?? "";
  // A dropped column is the other way an empty scope hides: the last cell would then hold the
  // PREVIOUS column's prose and read as "filled in". So the shape is checked, not just emptiness.
  const shaped = c.length === header.length;
  const ok = shaped && scope.length > 0;
  console.log(`  ${ok ? "OK  " : "★빈칸"} ${id.padEnd(6)} ${c.length}칸 · 유효범위 ${scope.length}자`);
  if (!ok) bad++;
}

if (bad) {
  console.error(`\n★${bad}건 — 「${COLUMN}」 가 비었거나 칸 수가 머리행과 다르다.`);
  console.error(`  처방: 그 행에 ★«이 판정이 언제 무효가 되는가»를 적어라(어떤 의존성이 움직이면 다시 재야 하는가).`);
  console.error(`  §C 머리의 그 문장이 정본이다 — 「유효 범위 없는 판정은 낡은 채로 권위 있게 인용된다」.`);
  process.exit(1);
}
console.log(`\n  ⇒ 전건 유효 범위가 적혀 있다. ★단 이것은 «그 내용이 맞다»가 아니라 «비어 있지 않다»이다 — 내용의 진위는 사람이 본다.`);
process.exit(0);
