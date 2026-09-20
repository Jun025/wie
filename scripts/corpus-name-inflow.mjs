#!/usr/bin/env node
// corpus-name-inflow.mjs — "did this round write a git-ignored corpus game name into the repo?"
//
// ── Why this exists at all ───────────────────────────────────────────────────
// Rounds in the `game_lab/` lineage report a number for this and have been doing it BY
// HAND. The predicate lived only in prose, in round files — `docs/report/0173` says
// "코퍼스 고유 stem 184개와 NFC 완전일치로 전수 대조해 0건을 실측했다", and `0170`/`0174`
// say the same thing in their own words. Nothing executed it, so every round re-derived
// it, and a round that re-derived it wrong is how this lineage got its `-fix`: it claimed
// "유입 0" and the claim was false. ★A predicate that only exists in prose is a predicate
// that drifts; this file is that prose, executable.
//
// ── What the hand predicate got wrong, measured ──────────────────────────────
// It was a plain SUBSTRING test, and substrings do not have word boundaries. Measured
// 2026-09-19 over 838 tracked text files × 184 unique corpus stems: 392 occurrences, of
// which 48 have the stem glued to a Korean syllable ON THE LEFT. The single worst offender
// is the 1-syllable stem that sits inside `인스턴스` (20 hits) and `패턴` (~15) — ordinary
// prose, nothing to do with any game. A round reading "35 files already contain this name"
// cannot use that number, and the round that hit it had to split the list by hand.
// ★That "184" is the population as it stood on 2026-09-19 and is NOT what this file measures
// now — the default widened to the whole corpus on 2026-09-20 (451 stems today). The paragraph
// is kept at its original numbers because it is the record of the substring problem, not a
// current reading; the population block below has the live one.
//
// ── The three buckets, and why the third one is NOT auto-excluded ────────────
// ★This is the whole design, and it is asymmetric on purpose.
//
//   BOUNDED          both neighbours are non-word (or the match is at a file edge).
//                    → counted as inflow. 328/392 occurrences.
//   PREFIX-EMBEDDED  the LEFT neighbour is a word character.
//                    → excluded, and the exclusion is SAFE: Korean attaches particles as
//                      SUFFIXES, never prefixes, so a real mention of a title is never
//                      glued to a preceding syllable. 48/392.
//   SUFFIX-ATTACHED  only the RIGHT neighbour is a word character.
//                    → ★NOT excluded. PRINTED for a human to split, because the machine
//                      provably cannot: measured, this bucket holds BOTH
//                        · a longer, DIFFERENT title  (`<stem>2`, `<stem>3`, `<stem>1.04`
//                          — sequels/versions that are their own games), and
//                        · a REAL mention with a Korean particle glued on (`<stem>의 …`),
//                      and the two are indistinguishable by shape. 16/392.
//
// ★Erring toward FALSE POSITIVES is a choice, and here is the reason. A false negative
// here reports "유입 0" when a name is present — which is EXACTLY the failure that got this
// lineage's earlier round rejected. A false positive costs a round sixty seconds of
// reading. The costs are not symmetric, so the tool never silently drops the ambiguous
// bucket; it puts it on screen and makes the round say which it is.
//
// ── What this is NOT ─────────────────────────────────────────────────────────
// ★Not a check. It has no failing state on findings and CI does not run it — it cannot,
// because `game_lab/` is git-ignored real game bytes (Constraint 9), the same reason
// `smoke_gate.sh` is local-only and structurally so. Exit 2 means "could not measure"
// (no corpus, no git), never "found nothing".
// ★Not a name filter either. Constraint 9 is about BYTES, not names, and this repo
// deliberately commits 283 stems in `scripts/smoke_gate_baseline.tsv` with AGENTS.md's
// blessing. The question this answers is narrower: "is a name from the LOCAL corpus
// appearing in the repo because of what I just wrote".
//
// Usage:
//   node scripts/corpus-name-inflow.mjs                  # files changed vs origin/main
//   node scripts/corpus-name-inflow.mjs --base <ref>     # …vs another ref
//   node scripts/corpus-name-inflow.mjs <path> [path…]   # exactly these files
//   node scripts/corpus-name-inflow.mjs --all-tracked    # every tracked text file
//   [--corpus <dir>]   default game_lab  (★the WHOLE corpus, minus `vendor_sdk/` — below)
// Exit: 0 = measured (whatever the counts) · 2 = could not measure.
//
// ── The freshness marker (2026-09-20, docs/report/0196) ─────────────────────
// In the default mode this also prints ONE line to paste into the round's doc. It carries the
// three counts and a digest of the content they were measured over, and
// `scripts/check-inflow-marker.mjs` reddens the PR when that content has moved since — the
// failure this lineage actually had, 4 times out of 4 ("measured, then wrote more prose, never
// re-measured"). ★It proves FRESHNESS, not truth: re-deriving the counts needs the corpus,
// which no runner has (Constraint 9). See scripts/lib/inflow-marker.mjs.

import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import path from "node:path";
import { formatMarker, treeDigest } from "./lib/inflow-marker.mjs";

const nfc = (s) => s.normalize("NFC");
const argv = process.argv.slice(2);
const flag = (n, d) => {
  const i = argv.indexOf(`--${n}`);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : d;
};
const has = (n) => argv.includes(`--${n}`);
const corpusDir = flag("corpus", "game_lab");
const base = flag("base", "origin/main");
const paths = argv.filter((a) => !a.startsWith("--") && a !== corpusDir && a !== base);

const die = (m) => {
  console.error(`corpus-name-inflow: ${m}`);
  process.exit(2);
};

let root;
try {
  root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
} catch {
  die("not a git work tree");
}
if (!existsSync(corpusDir)) die(`${corpusDir} does not exist — this needs the corpus, which is git-ignored and local-only`);

// ── Population: unique NFC stems of the archives under the corpus dir ────────
// ★The default is the WHOLE corpus, not `broken/`, and that was decided by measurement on
// 2026-09-20 (`docs/report/0194`). The first version read `game_lab/broken` only — 184 of the
// 450 game stems — so a round asking "is this name in the corpus" got "no" for the other 266.
// That is not hypothetical: `docs/report/0187`'s own hand-split of SUFFIX-ATTACHED dismissed
// 엑스맨3 · 크로이센1.04 · 하이브리드2 · 일지매영웅전기2 · 붕어빵타이쿤3작은화면 as "a longer
// DIFFERENT title, measured not to be a corpus stem" — and all five are archives in
// `game_lab/working/`. The narrow population produced a wrong sentence in the very round that
// built this tool.
//
// ★The cost feared when widening was measured and is ZERO where it would be paid. Over the 25
// most recent landed rounds, in the default (this-round's-diff) mode, the SUFFIX-ATTACHED bucket
// — the one a human must split by hand — is IDENTICAL under both populations in all 25. BOUNDED
// grows by a median of 1 pair (mean 1.44), and those are printed lines, not work. The "283 blessed
// stems in smoke_gate_baseline.tsv would drown it" objection does not materialise either: 235 of
// the 266 added stems occur nowhere but that file, and 0 of those 25 rounds touched it.
// ★That 235 is re-measurable only if the ruler comes with it, which is the whole point of this
// file: the population is the 266 stems `working/` adds that `broken/` did not already have; an
// "occurrence" is any hit this tool PRINTS (BOUNDED ∪ SUFFIX-ATTACHED — for this population
// PREFIX-EMBEDDED is 0, verified, so nothing is hidden by not printing it); and the tree is this
// PR's branch (905 tracked / 852 text). Counting BOUNDED alone gives 238 over the same tree —
// quote the ruler or the number means nothing.
//
// ★`vendor_sdk/` is excluded, and it is the one exclusion because it is not games — it holds
// emulator/SDK jars (`agent.jar`, `KEmulator-mmpp.jar`, `lwjgl-glfw-natives-linux.jar`). Its stem
// `agent` is an ordinary English word in a repo whose instructions live in `AGENTS.md`: measured,
// that one directory adds 21 SUFFIX-ATTACHED pairs across the tracked tree — MORE hand-splitting
// than the entire `broken/` population produces (26) — and every one of them is false. The
// exclusion is applied to DESCENDED directories only, so `--corpus game_lab/vendor_sdk` still
// works if you ever want to look at it on purpose.
const EXCLUDED_BUCKETS = new Set(["vendor_sdk"]);
const excludedSeen = [];
const walk = (d) =>
  readdirSync(d, { withFileTypes: true }).flatMap((e) => {
    const p = path.join(d, e.name);
    if (e.isDirectory()) {
      if (EXCLUDED_BUCKETS.has(e.name)) {
        excludedSeen.push(p);
        return [];
      }
      return walk(p);
    }
    return [p];
  });
const stems = [...new Set(walk(corpusDir).filter((p) => /\.(zip|jar|kdp)$/i.test(p)).map((p) => nfc(path.basename(p).replace(/\.[^.]+$/, ""))))].sort();
if (stems.length === 0) die(`no archives under ${corpusDir} — an empty population would report 0 for everything`);

// ── Subjects: what to search ────────────────────────────────────────────────
let subjects;
let subjectLabel;
try {
  if (paths.length) {
    subjects = paths;
    subjectLabel = `명시한 ${paths.length}경로`;
  } else if (has("all-tracked")) {
    subjects = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" }).split("\0").filter(Boolean);
    subjectLabel = `추적 파일 전건 ${subjects.length}`;
  } else {
    subjects = execFileSync("git", ["diff", "--name-only", "--diff-filter=ACMR", `${base}...HEAD`], { cwd: root, encoding: "utf8" })
      .split("\n")
      .filter(Boolean);
    subjectLabel = `${base}...HEAD 의 추가·수정 ${subjects.length}파일`;
  }
} catch (e) {
  die(`could not list subjects (${e.message})`);
}

// A binary subject is skipped and SAID, never silently dropped — a skipped file that
// held a name would otherwise read as "0".
// ★The binary test is "a NUL in the first 8 KiB", not "a NUL anywhere", and that is not
// fussiness: the first version of this file tested the whole buffer and then classified
// ITSELF as binary, because a JS source that writes `split("\0")` contains a real NUL
// byte. It was caught within a minute — by the skip line this block exists to print,
// which is the design working on its own author. Source files that embed a NUL literal
// are text; decoding them with replacement is harmless, since U+0000 matches no stem.
const texts = new Map();
const skipped = [];
for (const f of subjects) {
  const abs = path.join(root, f);
  try {
    if (!existsSync(abs) || !statSync(abs).isFile()) {
      skipped.push([f, "부재"]);
      continue;
    }
    const b = readFileSync(abs);
    if (b.subarray(0, 8192).includes(0)) {
      skipped.push([f, "이진"]);
      continue;
    }
    texts.set(f, nfc(b.toString("utf8")));
  } catch {
    skipped.push([f, "읽기 실패"]);
  }
}

// ── The predicate ───────────────────────────────────────────────────────────
// Hangul syllables plus alphanumerics count as word characters. The boundary test is
// applied per SIDE, and only when the stem's own edge is itself a word character — a stem
// that ends in `)` or `!` (the corpus has both) has no right-hand boundary to violate.
const isHangul = (ch) => ch >= "가" && ch <= "힣";
const wordish = (ch) => !!ch && (isHangul(ch) || /[0-9A-Za-z]/.test(ch));

const B = { BOUNDED: [], "PREFIX-EMBEDDED": [], "SUFFIX-ATTACHED": [] };
for (const [f, c] of texts) {
  for (const s of stems) {
    let i = c.indexOf(s);
    while (i !== -1) {
      const j = i + s.length;
      const L = i > 0 ? c[i - 1] : "";
      const R = j < c.length ? c[j] : "";
      const leftGlued = wordish(L) && wordish(s[0]);
      const rightGlued = wordish(R) && wordish(s[s.length - 1]);
      const excerpt = c.slice(Math.max(0, i - 8), j + 8).replace(/\n/g, "⏎");
      const rec = { stem: s, file: f, excerpt };
      if (leftGlued) B["PREFIX-EMBEDDED"].push(rec);
      else if (rightGlued) B["SUFFIX-ATTACHED"].push(rec);
      else B.BOUNDED.push(rec);
      i = c.indexOf(s, i + 1);
    }
  }
}

// ── Output: the predicate is printed WITH the numbers, on purpose ────────────
// The next person must be able to see "what ruler was this measured with" on the same
// screen as the measurement. That is the whole reason the hand version drifted.
const uniq = (recs) => new Set(recs.map((r) => `${r.stem} ${r.file}`)).size;
console.log("corpus-name-inflow  — 코퍼스 게임명이 이 변경으로 repo 에 들어왔는가");
console.log(`  술어: 코퍼스 고유 stem(NFC) × 대상 파일 본문(NFC) 부분문자열 일치 후, ★«단어 경계»로 세 갈래로 가른다.`);
console.log(`        단어문자 = 한글 음절 ∪ [0-9A-Za-z] · 경계 판정은 stem 의 «그 쪽 끝»이 단어문자일 때만 적용한다.`);
console.log(`  모집단: ${corpusDir} 의 고유 stem ${stems.length}개 · 대상: ${subjectLabel} (텍스트 ${texts.size} · 건너뜀 ${skipped.length})`);
console.log(
  `  ★모집단에서 «뺀» 하위 버킷: ${excludedSeen.length ? excludedSeen.join(" · ") : "없음"}` +
    ` (제외 대상 = ${[...EXCLUDED_BUCKETS].join(" · ")} — 게임이 아니라 에뮬·SDK 아카이브라서다. 조용히 빼지 않는다)`,
);
for (const [f, why] of skipped) console.log(`    · 건너뜀 ${f} (${why})`);

console.log(`\n  ★BOUNDED          ${String(B.BOUNDED.length).padStart(4)}회 / ${uniq(B.BOUNDED)}쌍 — 양쪽 경계 성립 ⇒ ★**이것이 «유입» 수다**`);
for (const r of B.BOUNDED) console.log(`      + ${r.stem}  ${r.file}  …${r.excerpt}…`);

console.log(`\n   PREFIX-EMBEDDED  ${String(B["PREFIX-EMBEDDED"].length).padStart(4)}회 / ${uniq(B["PREFIX-EMBEDDED"])}쌍 — 왼쪽이 붙었다 ⇒ 제외(한국어는 조사를 «앞»에 붙이지 않는다)`);

console.log(`\n  ★SUFFIX-ATTACHED  ${String(B["SUFFIX-ATTACHED"].length).padStart(4)}회 / ${uniq(B["SUFFIX-ATTACHED"])}쌍 — ★**기계가 못 가른다. 네가 갈라라**`);
if (B["SUFFIX-ATTACHED"].length) {
  console.log(`      ※이 바구니에는 «더 긴 다른 제목»(<stem>2 · <stem>3 · <stem>1.04)과`);
  console.log(`        «조사가 붙은 진짜 언급»(<stem>의 …)이 «함께» 들어 있다 — 모양으로는 구별되지 않는다.`);
  for (const r of B["SUFFIX-ATTACHED"]) console.log(`      ? ${r.stem}  ${r.file}  …${r.excerpt}…`);
} else {
  console.log(`      (없음)`);
}

console.log(`\n  ⇒ 보고할 때: 「유입 ${uniq(B.BOUNDED)}건(BOUNDED)」 + 「판단 필요 ${uniq(B["SUFFIX-ATTACHED"])}건(SUFFIX-ATTACHED)」 을 ★함께 적어라.`);
console.log(`     ★«0건» 이라고 쓰려면 SUFFIX-ATTACHED 도 0 이어야 한다 — 그 바구니를 비우지 않은 0 은 이 리니지가 반려됐던 그 0 이다.`);

// ── The freshness marker ────────────────────────────────────────────────────
// ★Only in the default (this-round's-diff) mode, because only there does "the number" mean
// "this round's number" — which is the claim a round writes down. `--all-tracked` measures the
// whole repo and explicit paths measure whatever you typed; a marker over either would assert
// something no checker can re-derive from the PR. Not emitting it is SAID, never silent.
const isDefaultMode = !paths.length && !has("all-tracked");
if (!isDefaultMode) {
  console.log(`\n  (표식 없음 — 기본 모드가 아니다. 표식은 «이 회차의 diff»를 잰 수에만 붙는다)`);
} else if (subjects.length === 0) {
  // ★This is 0189's failure verbatim: it measured before committing, got 0 subjects, and read
  // the resulting 0 as "no inflow". The counter's own design rule is that a 0 nobody could
  // have earned must not be printed quietly, so it is shouted here instead of marked.
  console.log(`\n  ★★대상이 «0파일»이다 — 이 수는 「유입이 없다」가 아니라 「아직 커밋하지 않았다」는 뜻이다.`);
  console.log(`     커밋한 «뒤» 다시 재라. 표식은 붙이지 않았다(0 을 굳히지 않는다).`);
} else {
  const digest = treeDigest(texts);
  console.log(`\n  ★아래 한 줄을 회차 문서(docs/report/… 또는 docs/worklog/…)에 ★«문장을 다 쓴 뒤 마지막에» 붙여라.`);
  console.log(`     그 뒤 본문을 한 글자라도 고치면 check-inflow-marker 가 red 로 말한다 — 그것이 이 표식의 전부다.`);
  console.log(
    `\n${formatMarker({
      subjects: texts.size,
      digest,
      b: `${B.BOUNDED.length}/${uniq(B.BOUNDED)}`,
      p: `${B["PREFIX-EMBEDDED"].length}/${uniq(B["PREFIX-EMBEDDED"])}`,
      s: `${B["SUFFIX-ATTACHED"].length}/${uniq(B["SUFFIX-ATTACHED"])}`,
    })}\n`,
  );
}
