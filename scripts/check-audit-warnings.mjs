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
import { readFile } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const EXPECTED_FILE = ".github/rust-audit-expected-warnings.json";
const key = (w) => `${w.package}@${w.version} ${w.kind}${w.advisory ? ` (${w.advisory})` : ""}`;

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

if (gone.length > 0) {
  console.log(`\n  ❌ 기대 목록에 있는데 cargo audit 이 «더는 내지 않는» 경고 ${gone.length}건 — 목록이 낡았다:`);
  for (const w of gone) console.log(`    - ${key(w)}`);
  console.log(`  ${EXPECTED_FILE} 에서 그 항목을 지워라(그것이 이 검사의 처방 전부다 — 무엇도 숨기지 않는다).`);
  console.log("\nAUDIT-WARNINGS FAILED ❌");
  process.exit(1);
}

console.log(`\nOK 기대 목록이 실제와 일치한다${fresh.length ? " (처음 보는 경고는 위에 갈라 적었다)" : ""}`);
