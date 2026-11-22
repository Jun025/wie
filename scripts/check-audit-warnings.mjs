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
// ★ASYMMETRIC since 2026-11-22, the same way the first check always was: a warning with
// no §A row is PRINTED (and written to the job summary) but is NOT red; a §A row with no
// warning is red. Until then both directions were red, so every freshly published
// unsound/yanked/unmaintained advisory reddened the daily job with no change to this repo
// (10 days, 2026-10-24 → 11-02, on one tokio warning — run 40449806714). A new
// VULNERABILITY is still red: the audit step says so, and the IGNORES block below says so
// again so this script's verdict does not depend on the step before it.
// `--selftest` holds all three shapes, hermetically (no cargo-audit, no network).
//
// THIRD CHECK (2026-10-23): the named `--ignore`s on the audit line of rust-audit.yaml and
// pr-audit.yml — same list in both, every ID still reported by the unignored audit below,
// and every ID argued in ledger §D. See the IGNORES block at the bottom.
import { appendFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { execFile, spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { promisify } from "node:util";

const EXPECTED_FILE = ".github/rust-audit-expected-warnings.json";
const LEDGER_FILE = "docs/project-kb/02_status.md";
const AUDIT_WORKFLOWS = [".github/workflows/rust-audit.yaml", ".github/workflows/pr-audit.yml"];
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
  // --selftest 전용 이음매 — 워크플로에 이 변수를 두지 마라(실제 감사 대신 파일을 읽는다).
  if (process.env.WIE_AUDIT_JSON) return JSON.parse(await readFile(process.env.WIE_AUDIT_JSON, "utf8"));
  try {
    const { stdout } = await promisify(execFile)("cargo", ["audit", "--json"], { maxBuffer: 64 * 1024 * 1024 });
    return JSON.parse(stdout);
  } catch (e) {
    if (e.stdout) return JSON.parse(e.stdout);
    throw e;
  }
};

if (process.argv.includes("--selftest")) process.exit(selftest());

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

  // 새 경고(§A 에 행이 없다)는 red 가 아니다 — «보이게»만 한다(stdout + 잡 요약). 새 자문은 이 repo 의
  // 변경 없이 뜨므로, 그것이 매일 잡을 붉히면 red 가 신호이기를 그친다(머리 주석 ★ASYMMETRIC).
  if (onlyAudit.length) {
    const line = `대장에 없는 새 경고 ${onlyAudit.length}건: ${onlyAudit.map(idOf).join(", ")}`;
    console.log(`\n  ★${line}`);
    console.log("  이것은 red 가 아니다. 살펴본 뒤 «A 에 등재하고 도달성 판정을 써라»(등재한 뒤에는 그 행이 낡으면 red 다).");
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${line}\n`);
  }

  const miscounted = declared !== undefined && Number(declared) !== rows.length;
  if (onlyRow.length || miscounted) {
    failed = true;
    console.log("\n  ❌ 대장 A 가 낡았다 — §A 에 있는 행을 cargo audit 이 더는 내지 않거나, 절 머리의 건수가 행 수와 다르다:");
    for (const r of onlyRow) console.log(`    - §A 에만 있다(경고가 없다 = 해소됐다): ${r.id} ${idOf(r)}`);
    if (miscounted) console.log(`    ! 절 머리가 ${declared}건이라 적었는데 실제 행은 ${rows.length}건이다`);
    console.log("  ★처방: 해소분은 «C 로 옮겨라»(행을 지우지 마라 — 구 ID 병기).");
    console.log("  ★번호를 재사용하지 마라 — 회차 기록·로드맵이 그 ID 로 이 표를 가리킨다.");
  } else {
    console.log(`\nOK 대장 A 에 낡은 행이 없다${onlyAudit.length ? " (대장에 없는 새 경고는 위에 갈라 적었다)" : " — 실제와 일치한다"}`);
  }
}

// ── 이름 억제(--ignore) ↔ 실제 · 두 워크플로 · 대장 §D ─────────────────────────────
// Constraint 5 는 억제를 «이름 + 도달성 논증»으로만 허락한다. 그 억제가 조용히 낡는 길이 셋이라 셋 다 red:
//   ⑴ rust-audit.yaml 과 pr-audit.yml 의 목록이 다르다 — 두 잡이 «무엇이 실패인가»에서 갈라진다.
//   ⑵ 억제한 ID 를 cargo audit 이 더는 내지 않는다(위 auditJson 은 무-억제로 돈다) — 해소됐으니 지워라.
//      남겨 두면 같은 ID 가 나중에 «다른 판본»에서 되살아나도 아무도 못 본다.
//   ⑶ 억제한 ID 에 대장 §D 행(줄머리 `| **D-<n>** |`)이 없다 — 논증 없는 억제다.
console.log(`\nIGNORES  --ignore ↔ cargo audit 실제 · ${AUDIT_WORKFLOWS.length} 워크플로 · ${LEDGER_FILE} §D`);
const ignoreSets = [];
for (const f of AUDIT_WORKFLOWS) {
  const runLines = (await readFile(f, "utf8")).split("\n").filter((l) => /^\s*run:\s*cargo audit(\s|$)/.test(l));
  if (runLines.length !== 1) {
    failed = true;
    console.log(`  ❌ ${f}: \`run: cargo audit…\` 줄이 ${runLines.length}개다(1개여야 읽는다) — «못 쟀다».`);
    continue;
  }
  ignoreSets.push({ f, ids: new Set([...runLines[0].matchAll(/--ignore\s+(\S+)/g)].map((m) => m[1])) });
}
if (ignoreSets.length === AUDIT_WORKFLOWS.length) {
  const [a, b] = ignoreSets;
  const diff = [...a.ids].filter((x) => !b.ids.has(x)).concat([...b.ids].filter((x) => !a.ids.has(x)));
  const reported = new Set((report.vulnerabilities?.list ?? []).map((v) => v.advisory?.id));
  const dRows = ledger.split("\n").filter((l) => /^\|\s*\*\*D-\d+\*\*\s*\|/.test(l));
  const stale = [...a.ids].filter((x) => !reported.has(x));
  const unargued = [...a.ids].filter((x) => !dRows.some((l) => l.includes(x)));
  // 억제되지 않은 취약점 = 새 취약점. 워크플로에서는 앞의 `cargo audit` 단계가 먼저 붉지만, 위에서 새 «경고»를
  // 풀어 준 이 검사기가 «취약점»까지 푼 것으로 읽히지 않게 여기서도 red 로 낸다(--selftest ⒝ 가 묶는다).
  const unignored = [...reported].filter((x) => !a.ids.has(x));
  console.log(`  억제 ${a.ids.size}건 · §D 행 ${dRows.length}건`);
  for (const x of a.ids) if (!diff.includes(x) && !stale.includes(x) && !unargued.includes(x)) console.log(`    · 유효: ${x}`);
  if (diff.length) {
    failed = true;
    console.log(`  ❌ 두 워크플로의 --ignore 가 다르다: ${diff.join(" ")} — 같은 목록이어야 한다`);
  }
  if (stale.length) {
    failed = true;
    console.log(`  ❌ 억제했는데 cargo audit 이 «더는 내지 않는» 자문: ${stale.join(" ")} — 두 워크플로에서 지우고 §D 행을 C 로 옮겨라`);
  }
  if (unargued.length) {
    failed = true;
    console.log(`  ❌ §D 행이 없는 억제: ${unargued.join(" ")} — 도달성 논증 없이 억제하지 마라(Constraint 5)`);
  }
  if (unignored.length) {
    failed = true;
    console.log(`  ❌ 억제되지 않은 취약점 ${unignored.length}건: ${unignored.join(" ")} — 취약점은 red 다(올려서 닫거나, 못 올리면 §D 에 논증을 쓰고 이름으로 억제)`);
  }
  if (!diff.length && !stale.length && !unargued.length && !unignored.length) console.log("\nOK 억제 목록이 두 워크플로·실제·§D 와 일치한다");
}

if (failed) {
  console.log("\nAUDIT-WARNINGS FAILED ❌");
  process.exit(1);
}

// ── --selftest ───────────────────────────────────────────────────────────────────
// 판정 «규칙»을 잰다(이 repo 의 대장이 지금 맞는가가 아니다): 임시 디렉터리에 최소 대장·기대 목록·두 워크플로를
// 쓰고, 합성 `cargo audit --json` 을 WIE_AUDIT_JSON 으로 물려 «이 스크립트 자신»을 CLI 로 돌린다.
function selftest() {
  const warn = (name, id) => ({ package: { name, version: "1.0.0" }, advisory: { id } });
  const base = () => ({
    vulnerabilities: { count: 1, list: [{ advisory: { id: "RUSTSEC-2000-0009" } }] },
    warnings: { unmaintained: [warn("aaa", "RUSTSEC-2000-0001")] },
  });
  const expectedBase = [{ package: "aaa", version: "1.0.0", kind: "unmaintained", advisory: "RUSTSEC-2000-0001" }];
  const NEW_WARNING = "대장에 없는 새 경고 1건: bbb@1.0.0 RUSTSEC-2000-0002";
  const cases = [
    { name: "기준 — 전부 일치", want: 0, out: /OK 대장 A 에 낡은 행이 없다 — 실제와 일치한다/ },
    {
      name: "⒜ 새 경고 1건 → rc=0 + 요약 줄",
      edit: (r) => (r.warnings.unsound = [warn("bbb", "RUSTSEC-2000-0002")]),
      want: 0,
      out: new RegExp(NEW_WARNING),
      summary: `${NEW_WARNING}\n`,
    },
    {
      name: "⒝ 새 취약점 1건 → red",
      edit: (r) => r.vulnerabilities.list.push({ advisory: { id: "RUSTSEC-2000-0010" } }),
      want: 1,
      out: /억제되지 않은 취약점 1건: RUSTSEC-2000-0010/,
    },
    // 기대 목록도 같이 비워, red 가 «기대 목록 GONE» 이 아니라 LEDGER-A 에서 나는지를 가른다.
    { name: "⒞ 대장에만 있는 낡은 행 → red", edit: (r) => (r.warnings = {}), expected: [], want: 1, out: /§A 에만 있다\(경고가 없다 = 해소됐다\): A-1 aaa@1\.0\.0 RUSTSEC-2000-0001/ },
  ];
  let bad = 0;
  for (const c of cases) {
    const tmp = mkdtempSync(join(tmpdir(), "audit-warnings-selftest-"));
    const put = (f, body) => (mkdirSync(dirname(join(tmp, f)), { recursive: true }), writeFileSync(join(tmp, f), body));
    const report = base();
    c.edit?.(report);
    put("audit.json", JSON.stringify(report));
    put("summary.md", "");
    put(EXPECTED_FILE, JSON.stringify({ expected: c.expected ?? expectedBase }));
    put(LEDGER_FILE, "### A. 권고·공급망 1건\n| **A-1** | `aaa` | RUSTSEC-2000-0001 | x | `1.0.0` → 없음 | x | x |\n| **D-1** | RUSTSEC-2000-0009 | x |\n");
    for (const f of AUDIT_WORKFLOWS) put(f, "        run: cargo audit --ignore RUSTSEC-2000-0009\n");
    const r = spawnSync(process.execPath, [import.meta.filename], {
      cwd: tmp,
      encoding: "utf8",
      env: { ...process.env, WIE_AUDIT_JSON: join(tmp, "audit.json"), GITHUB_STEP_SUMMARY: join(tmp, "summary.md") },
    });
    const summary = readFileSync(join(tmp, "summary.md"), "utf8");
    rmSync(tmp, { recursive: true, force: true });
    const ok = r.status === c.want && c.out.test(r.stdout) && summary === (c.summary ?? "");
    if (!ok) bad++;
    console.log(`${ok ? "selftest PASS" : "selftest FAIL"} — ${c.name} (rc=${r.status}, want ${c.want}${summary ? ` · 요약 «${summary.trim()}»` : ""})`);
    if (!ok) console.log(r.stdout.replace(/^/gm, "    | ") + r.stderr);
  }
  console.log(`\ncheck-audit-warnings --selftest: ${cases.length}케이스 ${bad ? `중 ${bad}건 불일치` : "전건 일치"}`);
  return bad ? 1 : 0;
}
