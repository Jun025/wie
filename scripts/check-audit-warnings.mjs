// Compares cargo-audit's CURRENT warning list against the expected one in
// .github/rust-audit-expected-warnings.json.
//
// WHY THIS EXISTS. On 2026-09-06 the header comment in rust-audit.yaml said the
// allowed warnings were "ttf-parser unmaintained, spin 0.12.0 yanked". Measured:
// spin had been upgraded away and chacha20 0.10.0 yanked had arrived. The COUNT
// was 2 both before and after, so "two current warnings" stayed literally true and
// nothing looked wrong. A count cannot see a swap; a list can.
//
// THIS IS NOT A SUPPRESSION, AND THE ASYMMETRY IS WHAT MAKES THAT TRUE:
//   - GONE (expected, no longer reported)  -> exit 1. The list is stale, which is
//     the defect this file exists to catch. Fixing it is deleting a line; it can
//     never be fixed by hiding something.
//   - NEW (reported, not expected)         -> printed, exit 0. A fresh advisory
//     must not redden the daily job, or the next person reaches for `--ignore` —
//     which AGENTS.md Constraint 5 forbids ("cargo audit with no ignores ... never
//     blanket, never continue-on-error").
// Deleting an entry here therefore HIDES NOTHING: the warning simply becomes NEW
// and keeps being printed. The audit step itself is untouched and still fails the
// job on any vulnerability.
//
// SECOND CHECK, SAME WARNING SET: the prose ledger's section A.
// docs/project-kb/02_status.md says of itself "경고 수와 이 표의 행 수가 어긋나면 둘 중
// 하나가 낡은 것이다" — and then went stale for a month anyway (A-2/A-3 were upgraded
// away on 2026-08-08 and chacha20 arrived, while the table still listed three rows).
// A human fixed it on 2026-09-08; this makes the machine say it next time. It reads
// the SAME `actual` computed above, so the two checks can never disagree about what
// the warnings are.
import { readFile } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const EXPECTED_FILE = ".github/rust-audit-expected-warnings.json";
const LEDGER_FILE = "docs/project-kb/02_status.md";
const key = (w) => `${w.package}@${w.version} ${w.kind}${w.advisory ? ` (${w.advisory})` : ""}`;

// The ledger states kind in prose ("`unmaintained`(informational)", "취약점 아님 — 레지스트리
// 신호"), so keying on it would mean parsing Korean. Advisory id is stated machine-readably
// and is the stronger discriminator anyway; a yanked row has none, and cargo-audit reports
// yanked with advisory=null, so both sides fall back to the literal "yanked".
const idOf = (w) => `${w.package}@${w.version} ${w.advisory ?? w.kind}`;

// cargo-audit exits non-zero when there are VULNERABILITIES. That is the audit
// step's job, not ours: we still want to read the JSON, so a non-zero exit with
// parseable stdout is not an error here.
const auditJson = async () => {
  try {
    const { stdout } = await promisify(execFile)("cargo", ["audit", "--json"], { maxBuffer: 64 * 1024 * 1024 });
    return JSON.parse(stdout);
  } catch (e) {
    if (e.stdout) return JSON.parse(e.stdout);
    throw e;
  }
};

const report = await auditJson();
const actual = Object.entries(report.warnings ?? {}).flatMap(([kind, items]) =>
  items.map((it) => ({ package: it.package?.name, version: it.package?.version, kind, advisory: it.advisory?.id ?? null })),
);
const expected = JSON.parse(await readFile(EXPECTED_FILE, "utf8")).expected;

const actualKeys = new Set(actual.map(key));
const expectedKeys = new Set(expected.map(key));
const gone = expected.filter((w) => !actualKeys.has(key(w)));
const fresh = actual.filter((w) => !expectedKeys.has(key(w)));

console.log("AUDIT-WARNINGS  기대 목록 ↔ cargo audit 실제");
console.log(`  기대 ${expected.length}건 · 실제 ${actual.length}건 · 취약점 ${report.vulnerabilities?.count ?? "?"}건`);
for (const w of expected.filter((w) => actualKeys.has(key(w)))) console.log(`    · 예상대로: ${key(w)}`);

if (fresh.length > 0) {
  console.log(`\n  ★처음 보는 경고 ${fresh.length}건 — 이것은 red 가 아니다(새 자문이 매일 잡을 붉히면 --ignore 압력이 생긴다):`);
  for (const w of fresh) console.log(`    + ${key(w)}`);
  console.log(`  살펴본 뒤 ${EXPECTED_FILE} 의 expected 에 아래를 그대로 추가하라:`);
  for (const w of fresh) console.log(`    ${JSON.stringify(w)}`);
}

let failed = false;

if (gone.length > 0) {
  failed = true;
  console.log(`\n  ❌ 기대 목록에 있는데 cargo audit 이 «더는 내지 않는» 경고 ${gone.length}건 — 목록이 낡았다:`);
  for (const w of gone) console.log(`    - ${key(w)}`);
  console.log(`  ${EXPECTED_FILE} 에서 그 항목을 지워라(그것이 이 검사의 처방 전부다 — 무엇도 숨기지 않는다).`);
} else {
  console.log(`\nOK 기대 목록이 실제와 일치한다${fresh.length ? " (처음 보는 경고는 위에 갈라 적었다)" : ""}`);
}

// ── 대장 A ↔ 같은 경고 집합 ────────────────────────────────────────────────────
// "A 행" 의 정의: 줄머리가 `| **A-<n>**` 인 표 행 «만». C 로 옮긴 해소분은 `| **C-2**<br>(구 **A-2**)`
// 로 시작하므로 걸리지 않는다 — 구 ID 병기는 셀 «안»에 있지 줄머리가 아니다. 이것이 의도다:
// C 는 "이미 해소된 것의 노출 판정 보존" 이고 대장 자신이 "A 의 불변식 밖" 이라고 못박는다.
const ledger = await readFile(LEDGER_FILE, "utf8");
const rowLines = ledger.split("\n").filter((l) => /^\|\s*\*\*A-\d+\*\*\s*\|/.test(l));
const tick = (s) => s?.match(/`([^`]+)`/)?.[1] ?? null;
const rows = rowLines.map((l) => {
  const c = l.split("|").slice(1, -1).map((s) => s.trim());
  return {
    id: c[0]?.replace(/\*/g, "").trim(),
    package: tick(c[1]),
    version: tick(c[4]?.split("→")[0]),
    advisory: c[2]?.match(/RUSTSEC-\d{4}-\d{4}/)?.[0] ?? null,
    kind: "yanked", // 권고가 없는 행 = yanked. advisory 가 있으면 위 idOf 가 그쪽을 쓴다
  };
});

console.log(`\nLEDGER-A  ${LEDGER_FILE} §A ↔ cargo audit 실제`);

// fail-closed: 표를 못 읽었으면 "조용히 통과" 가 아니라 "못 쟀다" 로 운다.
const unparsed = rows.filter((r) => !r.package || !r.version);
if (rowLines.length === 0 || unparsed.length > 0) {
  failed = true;
  console.log(
    rowLines.length === 0
      ? `  ❌ §A 표 행을 «하나도» 못 찾았다(줄머리 \`| **A-<n>** |\`). 절이 개편됐다면 이 파서를 함께 고쳐라.`
      : `  ❌ §A 행 ${unparsed.length}건에서 대상/버전을 못 읽었다: ${unparsed.map((r) => r.id).join(" ")}`,
  );
  console.log("  ★«못 쟀다»를 «이상 없다»로 넘기지 않는다 — 이 검사가 눈멀면 대장은 다시 조용히 낡는다.");
} else {
  const rowKeys = new Set(rows.map(idOf));
  const auditKeys = new Set(actual.map(idOf));
  const onlyAudit = actual.filter((w) => !rowKeys.has(idOf(w)));
  const onlyRow = rows.filter((r) => !auditKeys.has(idOf(r)));

  // 대장이 스스로 적은 머리 수("### A. 권고·공급망 N건")도 같이 본다 — 그 수도 낡는다.
  const declared = ledger.match(/^###\s*A\.\s*권고·공급망\s*(\d+)\s*건/m)?.[1];

  console.log(`  §A 행 ${rows.length}건 · 경고 ${actual.length}건 · 절 머리 선언 ${declared ?? "?"}건`);
  for (const r of rows.filter((r) => auditKeys.has(idOf(r)))) console.log(`    · 일치: ${r.id} ${idOf(r)}`);

  if (onlyAudit.length || onlyRow.length || (declared !== undefined && Number(declared) !== rows.length)) {
    failed = true;
    console.log("\n  ❌ 대장 A 가 낡았다 — 그 절이 스스로 적은 「경고 수와 행 수가 어긋나면 둘 중 하나가 낡은 것이다」에 걸렸다:");
    for (const w of onlyAudit) console.log(`    + 경고에만 있다(§A 에 행이 없다): ${idOf(w)}`);
    for (const r of onlyRow) console.log(`    - §A 에만 있다(경고가 없다 = 해소됐다): ${r.id} ${idOf(r)}`);
    if (declared !== undefined && Number(declared) !== rows.length) {
      console.log(`    ! 절 머리가 ${declared}건이라 적었는데 실제 행은 ${rows.length}건이다`);
    }
    console.log("  ★처방: 해소분은 «C 로 옮겨라»(행을 지우지 마라 — 구 ID 병기) · 새 경고는 «A 에 등재하고 도달성 판정을 써라».");
    console.log("  ★번호를 재사용하지 마라 — 회차 기록·로드맵이 그 ID 로 이 표를 가리킨다.");
  } else {
    console.log("\nOK 대장 A 가 실제와 일치한다");
  }
}

if (failed) {
  console.log("\nAUDIT-WARNINGS FAILED ❌");
  process.exit(1);
}
