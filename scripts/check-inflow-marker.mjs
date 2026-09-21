#!/usr/bin/env node
// check-inflow-marker.mjs — "is the 유입 number written in this round's docs still fresh?"
//
// ── The failure this exists for, measured rather than imagined ──────────────
// `scripts/corpus-name-inflow.mjs` made the inflow predicate executable, and a sibling round
// then measured what actually goes wrong afterwards (`docs/report/0195`). The answer was NOT
// "rounds forget to run it" — that has 0 observations, 3 of 3 rounds ran it. It is that the
// number goes STALE, and that has 4 observations out of 4:
//   0187        measured 16회/8쌍, then wrote the evidence list (game names) into the same
//               documents — which is itself an occurrence — and never re-measured. Real: 29/17.
//   0188 worklog  a `-fix` added sentences; the count moved under it.
//   0189 worklog  measured BEFORE committing, when the subject set was 0 files, and read the
//               resulting 0 as "no inflow".
//   0190 worklog  a wrong justification, not a stale count. ★This checker would NOT have
//               caught it, and that is written here rather than rounded away.
// Three of those four cost a gate② reject and a `-fix` round. This turns them into a red check
// on the PR that caused them.
//
// ── Why this CAN run in CI, when the proposal said it could not ─────────────
// The proposal (`…-token-boundary#p2`) assumed any such check needs the corpus and would
// therefore degrade to "could not measure" on every runner. That is true of TRUTH and false of
// FRESHNESS. The marker carries a digest of the content the counts were measured over; this
// re-computes that digest from the tree and compares. No corpus, no game bytes, no
// `game_lab/` — so it is a real gate in `engine-contract.yml`'s always-run `contract` job,
// not a checker that prints "skipped" forever and dies unread (`check-worklog-coverage`, 2026-09-07).
//
// ── What it deliberately does not do ────────────────────────────────────────
// ★It does not require a marker. A round that writes no marker passes — silently green would
// be wrong, so the no-marker case is PRINTED, but it is not a defect. Making the marker
// mandatory would re-open "did the round call the tool at all", which was measured and
// declined on 2026-09-20 (`docs/report/0195`): the only available predicate for that is a
// wording proxy, and it flagged 3 compliant rounds and 0 offenders.
// ★It does not re-derive the counts. A number typed into a marker by hand over unchanged
// content passes. That is the same declined axis, and claiming otherwise would make this
// check assert something it cannot see.
//
// Usage: node scripts/check-inflow-marker.mjs [--base <ref>]
// Exit: 0 = fresh (or no marker) · 1 = a marker is stale · 2 = could not measure.

import { execFileSync } from "node:child_process";
import { readFileSync, existsSync, statSync } from "node:fs";
import path from "node:path";
import { parseMarkers, treeDigest } from "./lib/inflow-marker.mjs";

const argv = process.argv.slice(2);
const flagIdx = argv.indexOf("--base");
const base = flagIdx >= 0 && argv[flagIdx + 1] ? argv[flagIdx + 1] : "origin/main";

const die = (m) => {
  console.error(`check-inflow-marker: ★못 쟀다 — ${m}`);
  process.exit(2);
};

let root;
try {
  root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
} catch {
  die("not a git work tree");
}

// The subject set is recomputed the same way the counter computes it, from the same command —
// if a round added a file after measuring, the set itself differs and the digest moves with it.
let subjects;
try {
  subjects = execFileSync("git", ["diff", "--name-only", "--diff-filter=ACMR", `${base}...HEAD`], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter(Boolean);
} catch (e) {
  die(`could not diff against ${base} (${e.message}) — CI supplies it via fetch-depth: 0`);
}

// Same text/binary split as the counter, for the same reason: the digest must cover exactly the
// files whose content produced the numbers, or a byte that never entered the count could move it.
const texts = new Map();
for (const f of subjects) {
  const abs = path.join(root, f);
  try {
    if (!existsSync(abs) || !statSync(abs).isFile()) continue;
    const b = readFileSync(abs);
    if (b.subarray(0, 8192).includes(0)) continue;
    texts.set(f, b.toString("utf8"));
  } catch {
    /* unreadable here means "not part of the measured text", same as the counter */
  }
}

const found = [];
for (const [f, c] of texts) for (const m of parseMarkers(c)) found.push({ file: f, ...m });

console.log(`check-inflow-marker — 유입 표식의 «신선도»(진위가 아니다 · 코퍼스 불요)`);
console.log(`  대상: ${base}...HEAD 의 추가·수정 ${subjects.length}파일 (텍스트 ${texts.size})`);

if (found.length === 0) {
  console.log(`  표식 0건 — 이 회차는 유입 수를 표식으로 박지 않았다. ★그것은 결함이 아니다(표식은 선택이다).`);
  console.log(`  ※「도구를 불렀는가」는 이 검사의 축이 아니다 — 재고 기각됐다(docs/report/0195).`);
  process.exit(0);
}

const actual = treeDigest(texts);
let bad = 0;
for (const m of found) {
  const ok = m.tree === actual && String(texts.size) === m.subjects;
  console.log(
    `  ${ok ? "OK  " : "★STALE"} ${m.file}  표식 subjects=${m.subjects} tree=${m.tree}  ↔  지금 subjects=${texts.size} tree=${actual}`,
  );
  if (!ok) bad++;
}

if (bad) {
  console.error(`\n★STALE ${bad}건 — 표식을 찍은 뒤 본문이 바뀌었다. 적힌 B/P/S 는 «지금 트리»의 수가 아니다.`);
  console.error(`  고치는 법: \`node scripts/corpus-name-inflow.mjs\` 를 ★다시 돌리고, 출력 마지막 줄로 그 표식을 «갈아끼워라».`);
  console.error(`  ★본문 수정과 표식 갱신의 순서가 뒤집히면 다시 red 다 — 그 순서가 이 리니지의 오류 4건 중 3건의 근인이다.`);
  process.exit(1);
}
console.log(`\n  ⇒ 신선하다. ★단 이것은 «그 수를 도구가 냈다»의 증명이 아니다 — 진위는 코퍼스가 있어야 재고, 그 축은 이 검사의 것이 아니다.`);
process.exit(0);
