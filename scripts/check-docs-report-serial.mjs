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
// ★★**⑴ 은 «작성 시점 조회»로 안 없어졌다 — 2026-09-16 채택 제안이 그것을 실측했다**
//   (`docs/worklog/2026-09-16-adopt-slice-d-base-swap-fix2-p1.json#p0`): `--next-serial` 의 답은
//   **묻는 순간의 사실**인데 파일은 **회차 끝에** 커밋된다 ⇒ ★그 사이에 남이 같은 번호로 PR 을 연다.
//   실측 ⓐ #164·#165 가 «8분» 사이에 둘 다 `0122` · 실측 ⓑ #176·#177 이 «5분» 사이에 둘 다 `0134`.
//   ★**창의 크기는 «커밋→PR» 이 아니다** — 그 구간은 실측 **p50 28초 · max 70초**(PR 32건)로 거의 0이고,
//   진짜 창은 ★**«조회→커밋»(회차 작업 시간, 분 단위)** 이다. ⇒ ★**claim 조회를 «브랜치까지» 넓혀도 ≤70초밖에 못 산다.**
//   ⇒ 그래서 이 파일이 닫는 자리는 «조회»가 아니라 ★**«내가 쓴 번호를 다른 열린 PR 이 이미 들고 있는가»** 이고,
//     그 판정은 **기본 모드**(= 회차가 커밋 전에 돌리고 CI 가 PR 마다 돌린다)에 붙는다 — 아래 `crossPrCollisions`.
//   ★**착지 «후»에만 울던 축이 착지 «전»에 운다** — 새 잡·새 도구 0.
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

/**
 * ★내 트리가 «새로 더한» `docs/report` 파일명 = 디스크 − `origin/main`.
 * ★다른 열린 PR 의 claim 과 대조할 때 **이 집합만** 쓴다 — 전체 디스크로 대조하면
 *   ★**나와 무관한 두 PR 의 충돌이 내 PR 을 red 로 만든다**(오탐).
 * ★git 을 못 물으면 «판정 불가»이므로 **빈 목록**(축 건너뜀)이다 — 막지 않는다.
 */
export function myAddedNames(disk) {
  try {
    const base = execFileSync("git", ["ls-tree", "-r", "origin/main", "--name-only", "--", REPORT_DIR], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    })
      .split("\n")
      .filter(Boolean)
      .map((f) => path.basename(f));
    const known = new Set(base);
    return { ok: true, out: disk.filter((n) => !known.has(n)) };
  } catch (e) {
    return { ok: false, out: [], why: String(e.message || e).split("\n")[0] };
  }
}

/** 이 트리가 서 있는 PR 의 head 브랜치. CI(PR 빌드)는 detached HEAD 라 `GITHUB_HEAD_REF` 가 정본이다. */
export function selfHeadRef() {
  if (process.env.GITHUB_HEAD_REF) return process.env.GITHUB_HEAD_REF;
  try {
    const b = execFileSync("git", ["rev-parse", "--abbrev-ref", "HEAD"], { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
    return b === "HEAD" ? "" : b; // detached & no env ⇒ 모른다
  } catch {
    return "";
  }
}

/**
 * ★내가 더한 연번을 **다른** 열린 PR 이 이미 들고 있는가.
 * ★**자기 제외는 «브랜치»가 1차, «파일명 동일성»이 2차다.**
 *   파일명만으로 걸러면 ★**연번을 그대로 두고 슬러그만 바꾼 순간 자기 PR 과 충돌한다**(게이트² F2 실측:
 *   `0136--…-p1-p0.md` → `0136--…-renamed.md` 로 고치면 상대가 «자기 자신»인데 「PR 번호가 큰 쪽이 옮긴다」는
 *   **적용 불능 규칙**이 찍힌다). 브랜치가 같으면 그 claim 은 정의상 내 PR 이므로 이름과 무관하게 제외된다.
 *   ★브랜치를 모르는 형상(detached + env 없음)에서는 2차 축만 남는다 — 그때도 «덜 걸러질» 뿐 오답은 아니다.
 * ★순수 함수 — 판별력 축이 이것을 직접 부른다.
 */
export function crossPrCollisions(mine, prClaims, selfRef = "") {
  const bySerial = new Map();
  for (const n of mine) {
    const m = SERIAL_RE.exec(n);
    if (m) bySerial.set(m[1], n);
  }
  const mineSet = new Set(mine);
  const out = [];
  for (const c of prClaims) {
    if (selfRef && c.headRefName === selfRef) continue; // ★1차: 내 PR(브랜치 동일)
    if (mineSet.has(c.name)) continue; // ★2차: 같은 파일명 = 내 PR 자신
    const s = SERIAL_RE.exec(c.name)?.[1];
    if (s && bySerial.has(s)) out.push({ serial: s, mine: bySerial.get(s), theirs: c.name, pr: c.pr });
  }
  return out.sort((a, b) => a.serial.localeCompare(b.serial));
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
    // ★jq 템플릿을 쓰지 «않는다» — JS 문자열 안의 `\(`/`\t` 는 자바스크립트가 «먼저» 먹어 버려
    //   jq 가 상수 문자열을 받는다(이 회차가 그 함정을 실제로 밟았고, 대조군 ⑵ 가 잡았다). JSON 을 그대로 파싱한다.
    const prs = JSON.parse(gh(["pr", "list", "--state", "open", "--limit", "200", "--json", "number,headRefName"]));
    const out = [];
    for (const { number: n, headRefName } of prs) {
      const files = gh([
        "api",
        `repos/${slug}/pulls/${n}/files`,
        "--paginate",
        "-q",
        '.[] | select(.status=="added") | .filename',
      ])
        .split("\n")
        .filter(Boolean);
      for (const f of files) if (f.startsWith(`${REPORT_DIR}/`)) out.push({ pr: n, headRefName, name: path.basename(f) });
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
    // ── 착지 «전» 축(열린 PR claim 대조) — 양방향 ──
    [
      "★다른 열린 PR 이 같은 연번을 들면 잡는다",
      (() => {
        const h = crossPrCollisions(["0134--2026-09-17--a.md"], [{ pr: "177", name: "0134--2026-09-17--b.md" }]);
        return h.length === 1 && h[0].serial === "0134" && h[0].pr === "177";
      })(),
    ],
    [
      "★번호가 다르면 통과(오탐 0)",
      crossPrCollisions(["0134--a.md"], [{ pr: "177", name: "0135--b.md" }]).length === 0,
    ],
    [
      "★★«같은 파일명»은 내 PR 자신이라 세지 않는다(자기 충돌 금지)",
      crossPrCollisions(["0134--a.md"], [{ pr: "176", name: "0134--a.md" }]).length === 0,
    ],
    [
      "★★내가 더한 것이 없으면 «남의 충돌»을 내 것으로 읽지 않는다",
      crossPrCollisions([], [{ pr: "177", name: "0134--b.md" }]).length === 0,
    ],
    [
      "★★실 저장소에서 «내가 더한 집합»이 디스크의 부분집합이다(git 축이 살아 있다)",
      (() => {
        const disk = diskNames();
        const m = myAddedNames(disk);
        return !m.ok || m.out.every((n) => disk.includes(n));
      })(),
    ],
    [
      "★★가드 본체가 `crossPrCollisions` 를 부르고 exit 1 을 낸다(호출부 슬라이스 판정)",
      (() => {
        const src = readFileSync(fileURLToPath(import.meta.url), "utf8");
        const MARK = "\n// ── 제품 호출부 ──\n";
        if (src.split(MARK).length !== 2) return false;
        const body = src.slice(src.indexOf(MARK) + MARK.length);
        return /crossPrCollisions\(mine\.out, claims\.out, selfHeadRef\(\)\)/.test(body) && /if \(crossBad\) process\.exit\(1\);/.test(body);
      })(),
    ],
    // ── 게이트² F2: 자기 제외는 «브랜치»가 1차 ──
    [
      "★★연번을 그대로 두고 슬러그만 바꿔도 «내 PR» 은 안 문다(브랜치 축)",
      crossPrCollisions(["0136--2026-09-17--a-renamed.md"], [{ pr: "179", headRefName: "feat/x", name: "0136--2026-09-17--a.md" }], "feat/x")
        .length === 0,
    ],
    [
      "★그 축이 «남의 PR» 까지 눈멀게 하지는 않는다(오탐 0 의 반대 방향)",
      crossPrCollisions(["0136--a.md"], [{ pr: "177", headRefName: "other/y", name: "0136--b.md" }], "feat/x").length === 1,
    ],
    [
      "★브랜치를 모르면(detached + env 없음) 2차 축(파일명)만 남는다 — 덜 걸러질 뿐 오답이 아니다",
      (() => {
        const claims = [{ pr: "179", headRefName: "feat/x", name: "0136--a.md" }];
        return crossPrCollisions(["0136--a.md"], claims, "").length === 0 && crossPrCollisions(["0136--a-renamed.md"], claims, "").length === 1;
      })(),
    ],
    // ── 게이트² F1: «건너뜀» 과 «충돌 0» 이 stdout 에서 갈리는가(호출부 슬라이스 판정) ──
    [
      "★★성공 줄이 «미대조» 를 «충돌 0» 으로 적지 않는다",
      (() => {
        const src = readFileSync(fileURLToPath(import.meta.url), "utf8");
        const MARK = "\n// ── 제품 호출부 ──\n";
        if (src.split(MARK).length !== 2) return false;
        const body = src.slice(src.indexOf(MARK) + MARK.length);
        // 두 강등 갈래가 각자 crossNote 를 세우고, «충돌 0» 은 claims.ok 인 갈래 안에서만 세워진다.
        return (
          /crossNote = " · ★열린 PR 대조 «건너뜀»/.test(body) &&
          /crossNote = ` · ★내가 더한 \$\{mine\.out\.length\}건은 «미대조»/.test(body) &&
          /if \(!crossBad\) crossNote = ` · 내가 더한 \$\{mine\.out\.length\}건 ↔ 열린 PR claim 충돌 0`;/.test(body) &&
          /\$\{crossNote\}`\);/.test(body)
        );
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

// ★착지 «전» 축 — 내가 더한 번호를 다른 열린 PR 이 이미 들고 있는가.
// ★**내가 더한 것이 0 이면 네트워크를 아예 안 친다**(main push 빌드가 그 형상이다 — 비용 0).
// ★네트워크·git 실패는 «경고»다 — 이 파일의 기존 원칙(작성을 못 하게 만드는 쪽이 중복 하나보다 비싸다) 그대로.
// ★★**«건너뛰었다»를 «대조했고 0이다»로 적지 마라** — 아래 `crossNote` 가 그 구분을 stdout 까지 나른다.
//   그 구분이 없으면 이 축은 **자기 실패 모드에서만** 무력해진다(경고는 stderr 에 있는데 성공 줄은 「충돌 0」이라 말한다).
//   ★같은 관용을 `--next-serial` 이 이미 쓴다(「디스크만 보고 답한다 — 다른 PR 이 같은 번호를 들고 있을 수 있다」).
const mine = myAddedNames(diskNames());
let crossBad = 0;
let crossNote = "";
if (!mine.ok) {
  console.error(`::warning title=내가 더한 연번을 못 물었다(막지 않는다)::${mine.why} — 열린 PR 대조를 건너뛴다.`);
  crossNote = " · ★열린 PR 대조 «건너뜀»(내가 더한 연번을 못 물었다 — 위 경고)";
} else if (mine.out.length) {
  const claims = openPrAddedNames();
  if (!claims.ok) {
    console.error(`::warning title=열린 PR 을 못 물었다(막지 않는다)::${claims.why} — 열린 PR 대조를 건너뛴다.`);
    crossNote = ` · ★내가 더한 ${mine.out.length}건은 «미대조»(열린 PR 을 못 물었다 — 위 경고)`;
  } else {
    const hits = crossPrCollisions(mine.out, claims.out, selfHeadRef());
    for (const h of hits)
      console.error(
        `::error title=열린 PR 이 같은 연번을 들고 있다::${h.serial} — 내 \`${h.mine}\` ↔ **#${h.pr}** 의 \`${h.theirs}\`. 둘 다 착지하면 위 «중복» 축이 red 가 된다. ⇒ ★규칙: **나중에 claim 한 쪽이 옮긴다**(= 대개 지금 이 red 를 «먼저» 보는 쪽이다 — 체크런은 커밋에 묶여 있어 늦게 claim 한 쪽이 먼저 빨개지고, 먼저 claim 한 쪽은 **자기 CI 가 다시 돌기 전까지 모른다**). 내 쪽이 나중이면 **내가** \`node scripts/check-docs-report-serial.mjs --next-serial\` 이 주는 번호로 \`git mv\` 하라(이름만 · 내용은 고치지 마라). 내 쪽이 «먼저» claim 했다면 옮길 쪽은 **#${h.pr}** 이니 그 PR 에 알려라.`,
      );
    crossBad = hits.length;
    if (!crossBad) crossNote = ` · 내가 더한 ${mine.out.length}건 ↔ 열린 PR claim 충돌 0`;
  }
}
if (crossBad) process.exit(1);
console.log(`check-docs-report-serial: OK — ${REPORT_DIR} ${diskNames().length}건 중 중복 연번 0${crossNote}`);
