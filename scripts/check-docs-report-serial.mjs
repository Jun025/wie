#!/usr/bin/env node
// ── `docs/report/NNNN--…` 연번 «유일성» 가드 ────────────────────────────────────────
//
// ★**포팅이다 — 발명이 아니다.** 형제 저장소 `otterpebble` 이 같은 병을 먼저 풀었고
//   (`otterpebble-docs-report-serial-uniqueness-unguarded` · verdict approve · `scripts/docs-report-serial-check.mjs`),
//   이 파일은 그 형상·모드·판별력 축을 옮긴 것이다. 다른 점은 파일 아래 「wie 와 다른 점」 절에 적었다.
//
// ★왜 필요한가(가설 아님 · 2026-09-07 wie 실측): `REPORT.md` 를 회차별 파일로 이관하면서
//   「연번 = 디스크 최대 + 1」이 관례가 됐는데, ★**«디스크»는 머지된 트리다.**
//   ⇒ 동시에 열린 두 PR 이 각자 `max+1` 을 계산하면 **같은 번호**를 잡고,
//   파일명이 달라 **git 충돌도 안 나고** 어떤 검사도 울지 않는다.
//   ★**이관이 «없애려던» 형태가 «파일명»에서 그대로 재발한다.**
//   실제 관측 3형태:
//     ⑴열린 PR 끼리 겹침(작성 시점 조회로 회피 가능)
//     ⑵내가 번호를 잡은 **뒤** 남이 착지(사전 조회로도 못 막는다)
//     ⑶★**한 PR 이 두 번호**를 claim(PR 단위로 세면 아예 안 보인다)
//   ⇒ ★**사후 축(이 파일의 기본 모드)이 «있어야» 한다** — ⑵⑶ 은 사전 조회로 못 막는다.
//   ★그리고 사전 도우미(`--next-serial`)도 함께 둔다 — ⑴ 을 «나기 전»에 없애는 것이 제일 싸다.
//
// ★**번호 «체계»는 바꾸지 않는다.** 이 파일이 하는 일은 «유일성 보장» 하나다.
// ★**기존 파일을 재번호하지 않는다** — 착지한 참조가 깨진다. 처방은 «아직 안 착지한 쪽»을 옮기는 것이다.
//
// 사용법:
//   node scripts/check-docs-report-serial.mjs                # 기본 = 중복 검출(red = exit 1)
//   node scripts/check-docs-report-serial.mjs --next-serial  # ★작성 도우미(열린 PR 까지 세어 다음 번호)
//   node scripts/check-docs-report-serial.mjs --selftest     # 판별력
//
// ── wie 와 다른 점(형제 저장소와 갈린 자리 · 저장소 차이가 실재한다) ──
//   ⒜ 파일명 = `scripts/check-*.mjs`. wie 의 다른 검사기 셋(`check-worklog-json`·`check-worklog-coverage`·
//      `check-parity-lock-wired`)이 전부 그 꼴이라 그쪽을 따랐다(형제는 `docs-report-serial-check.mjs`).
//   ⒝ 배선 = `engine-contract.yml` 의 `contract` 잡 **한 곳**. 형제는 `ci.yml` 이 `paths-ignore: docs/**`
//      라서 «원장만 고친 docs-only 착지»를 못 봐 `docs-gate.yml` 을 따로 두어야 했는데,
//      ★**wie 의 `contract` 잡은 Constraint 4 로 «paths 필터가 없는 상시 실행 래퍼»** 라 그 문제가 없다.
//      ⇒ 잡을 새로 만들지 않았다. 형제의 `gate-checks.txt`·`axis-floors.json` 은 wie 에 **없다**.
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const REPORT_DIR = "docs/report";
/** ★파일명 규약: `NNNN--YYYY-MM-DD--<slug>.md` — 연번은 **앞 4자리**다. */
const SERIAL_RE = /^(\d{4})--/;

/** 파일명 목록 → `{serial: [파일…]}` 중 **2건 이상**인 것만. ★순수 함수(판별력 축이 이것을 부른다). */
export function duplicateSerials(names) {
  const by = new Map();
  for (const n of names) {
    const m = SERIAL_RE.exec(n);
    if (!m) continue; // ★규약 밖 이름은 이 축의 대상이 아니다(별 축 소관)
    if (!by.has(m[1])) by.set(m[1], []);
    by.get(m[1]).push(n);
  }
  return [...by.entries()].filter(([, fs]) => fs.length > 1).sort();
}

/** 파일명 목록 → 다음 빈 연번(4자리 문자열). ★비어 있으면 `0001`. */
export function nextSerial(names) {
  let max = 0;
  for (const n of names) {
    const m = SERIAL_RE.exec(n);
    if (m) max = Math.max(max, Number(m[1]));
  }
  return String(max + 1).padStart(4, "0");
}

const diskNames = () => readdirSync(path.join(ROOT, REPORT_DIR)).filter((f) => f.endsWith(".md"));

/**
 * ★열린 PR 이 «추가한» `docs/report` 파일명 — `--next-serial` 전용.
 * ★**네트워크 의존이라 실패해도 «막지 않는다»**: 경고만 내고 빈 목록을 돌려준다.
 *   작성을 못 하게 만드는 쪽이 중복 하나보다 비싸다.
 * ★`gh api …/pulls/<n>/files` 를 쓴다 — PR head 를 fetch 하지 않으므로 ★**`.git/shallow` 를 만들지 않는다**.
 */
export function openPrAddedNames() {
  const gh = (args) => execFileSync("gh", args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  try {
    const slug = gh(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]).trim();
    const nums = gh(["pr", "list", "--state", "open", "--limit", "200", "--json", "number", "-q", ".[].number"])
      .split("\n")
      .filter(Boolean);
    const out = [];
    for (const n of nums) {
      const files = gh([
        "api",
        `repos/${slug}/pulls/${n}/files`,
        "--paginate",
        "-q",
        '.[] | select(.status=="added") | .filename',
      ])
        .split("\n")
        .filter(Boolean);
      for (const f of files) if (f.startsWith(`${REPORT_DIR}/`)) out.push({ pr: n, name: path.basename(f) });
    }
    return { ok: true, out };
  } catch (e) {
    return { ok: false, out: [], why: String(e.message || e).split("\n")[0] };
  }
}

function selftest() {
  const F = "0007--2026-09-07--x.md";
  const checks = [
    ["★같은 번호 2건이면 잡힌다", duplicateSerials(["0001--a.md", "0001--b.md"]).length === 1],
    ["★번호가 다르면 통과(오탐 0)", duplicateSerials(["0001--a.md", "0002--b.md"]).length === 0],
    [
      "★3건 이상도 한 항목으로 모은다",
      (() => {
        const d = duplicateSerials(["0005--a.md", "0005--b.md", "0005--c.md"]);
        return d.length === 1 && d[0][1].length === 3;
      })(),
    ],
    ["★규약 밖 이름은 대상이 아니다(오탐 0)", duplicateSerials(["README.md", "abc--x.md", F]).length === 0],
    ["★★실 저장소가 지금 깨끗하다(공허한 축이 아니다)", duplicateSerials(diskNames()).length === 0],
    ["★다음 연번 = 최대 + 1(4자리 zero-pad)", nextSerial(["0008--a.md", "0012--b.md"]) === "0013"],
    ["★비어 있으면 0001", nextSerial([]) === "0001"],
    [
      "★★실 저장소의 다음 연번이 «디스크 최대 + 1» 이다",
      (() => {
        const names = diskNames();
        let max = 0;
        for (const n of names) {
          const m = SERIAL_RE.exec(n);
          if (m) max = Math.max(max, Number(m[1]));
        }
        return nextSerial(names) === String(max + 1).padStart(4, "0");
      })(),
    ],
    // ★★가드 «본체»가 그 함수를 실제로 부르고 red 를 낸다 — 함수만 두고 호출부를 지우면 이 축이 운다.
    //   ★판정 구간을 **제품 호출부(마커 뒤)로 슬라이스**한다. selftest 자신의 리터럴을 물지 않게.
    //   (형제 저장소 실사고: `src.indexOf` 가 자기 리터럴을 먼저 물어 축이 «구성상» 항상 참이었다.)
    [
      "★★가드 본체가 `duplicateSerials` 를 부르고 exit 1 을 낸다(호출부 슬라이스 판정)",
      (() => {
        const src = readFileSync(fileURLToPath(import.meta.url), "utf8");
        const MARK = "\n// ── 제품 호출부 ──\n";
        if (src.split(MARK).length !== 2) return false;
        const body = src.slice(src.indexOf(MARK) + MARK.length);
        return /const dups = duplicateSerials\(diskNames\(\)\);/.test(body) && /process\.exit\(1\);/.test(body);
      })(),
    ],
  ];
  let bad = 0;
  for (const [name, ok] of checks) {
    console.log(`${ok ? "selftest PASS" : "selftest FAIL"} — ${name}`);
    if (!ok) bad++;
  }
  console.log(`\ncheck-docs-report-serial --selftest: ${checks.length}케이스 ${bad ? `중 ${bad}건 불일치` : "전건 일치"}`);
  process.exit(bad ? 1 : 0);
}
if (process.argv.includes("--selftest")) selftest();

if (process.argv.includes("--next-serial")) {
  const names = diskNames();
  const disk = nextSerial(names);
  const pr = openPrAddedNames();
  if (!pr.ok) {
    console.error(
      `::warning title=열린 PR 을 못 물었다(막지 않는다)::${pr.why} — 디스크만 보고 답한다. ★다른 PR 이 같은 번호를 들고 있을 수 있다.`,
    );
    console.log(disk);
    process.exit(0);
  }
  const all = [...names, ...pr.out.map((x) => x.name)];
  const next = nextSerial(all);
  const claimed = pr.out.map((x) => `${SERIAL_RE.exec(x.name)?.[1] ?? "?"}(#${x.pr})`).sort().join(" ");
  console.error(`check-docs-report-serial: 디스크 기준 ${disk} · 열린 PR claim [${claimed || "없음"}] ⇒ 다음 빈 번호 ${next}`);
  console.log(next); // ★stdout 은 번호 «한 줄»뿐(스크립트가 받아 쓴다)
  process.exit(0);
}

// ── 제품 호출부 ──
const dups = duplicateSerials(diskNames());
for (const [serial, files] of dups)
  console.error(
    `::error title=docs/report 연번이 중복이다::${serial} — ${files.join(" · ")} 가 같은 번호를 쓴다. 「연번 내림차순으로 이으면 원문과 바이트 동일」이라는 이관 규약의 전제가 깨진다. ⇒ ★아직 착지하지 «않은» 쪽을 \`node scripts/check-docs-report-serial.mjs --next-serial\` 이 주는 번호로 \`git mv\` 하라(내용은 고치지 마라 · 착지한 파일은 재번호하지 마라).`,
  );
if (dups.length) process.exit(1);
console.log(`check-docs-report-serial: OK — ${REPORT_DIR} ${diskNames().length}건 중 중복 연번 0`);
