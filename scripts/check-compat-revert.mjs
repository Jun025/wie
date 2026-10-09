#!/usr/bin/env node
// ── compat.json «옛 판 위 재생성» 되돌림 검사 ────────────────────────────────────────
//
// 무엇을 막나: PR 브랜치가 진행 중 main 을 머지해 형제 PR 의 compat 행을 받아 놓고, 그 «머지 전»에 잰
// 자료로 compat.json 을 다시 써 그 행을 옛 값으로 되돌리는 것. 충돌이 없고 스키마도 통과해 `contract`
// 가 green 인 채 착지한다.
//
// 판정(행 = `sha256|platform` · 필드 = status·title·model·fileTitle·axes.*·knownIssues_ko·playTips_ko·changes · 행 자체):
//   fork = 브랜치가 main 에서 처음 갈라진 점(first-parent) · mb = merge-base(브랜치, main) ·
//   landing = git merge-tree(main, 브랜치) = 실제로 착지할 내용.
//   fork ≠ mb 인 필드(= 브랜치가 머지로 «받은» main 의 변경) 중
//   landing == fork 값 · landing ≠ mb 값 · landing ≠ main 값  ⇒  REVERT.
//
// ★왜 «main 의 과거 값과 같은가»로 재지 않나 — 그 판(v1)은 최근 compat 착지 6건 중 3건(#465·#471·#476)을
//   red 로 냈다. 엔진 수정이 limited→playable 로 «옛 값을 되살리는» 것과 낡은 재생성은 값만으로 못 가른다.
//   가르는 것은 git 계보다: 브랜치가 그 변경을 «받은 적이 있는가».
// ★2026-10-05 의 두 «사고»(#475 F1 17행 · #478 F1 23행)는 이 검사로 red 가 «아니다» — 두 브랜치 다 #473 을
//   받은 적이 없고, 실제 머지 결과(merge-tree)는 PR 이 쓴 14행만 바꾼다(main 은 #473 값을 그대로 갖고 있다).
//   검수의 «되돌림»은 main↔head 2점 diff(`git diff main head`)가 main 쪽 변경을 뒤집어 보인 것이다.
//   ⇒ 행 변경 수를 인용할 때는 이 검사의 «착지 기준 바뀐 행» 줄을 쓴다(2점 diff 금지).
//
// 의도한 되돌림: 이 PR 이 더하거나 고친 `docs/worklog/*.json` 에
//   "intendedCompatReverts": ["<sha 앞 12자>:<필드>", "<sha 앞 12자>:*"]
// 로 선언한 것만 통과한다(worklog 의 자유 키 — 소비자는 읽지 않는다).
//
// 사용법:
//   node scripts/check-compat-revert.mjs                    # HEAD ↔ origin/main (red = exit 1 · 미대조 = exit 2)
//   node scripts/check-compat-revert.mjs --head <rev> --base <rev>   # 과거 사례 재현
//   node scripts/check-compat-revert.mjs --selftest
// CI(pull_request)의 HEAD 는 머지 커밋이라 HEAD^2 를 브랜치로 쓴다.
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const COMPAT = "docs/player-data/compat.json";
const ABSENT = "∅";

function flatten(e) {
  const f = { "(row)": "present" };
  for (const k of ["status", "title", "model", "fileTitle", "knownIssues_ko", "playTips_ko", "changes"]) f[k] = JSON.stringify(e[k] ?? null);
  for (const [k, v] of Object.entries(e.axes ?? {})) f[`axes.${k}`] = String(v);
  return f;
}
const rows = (doc) => new Map((doc?.entries ?? []).map((e) => [`${e.sha256}|${e.platform}`, flatten(e)]));

/** 반환 = 되돌림 목록. ★순수 함수(selftest 가 부른다). */
export function findReverts({ landing, fork, mb, base, declared = new Set() }) {
  const L = rows(landing), F = rows(fork), M = rows(mb), B = rows(base);
  const out = [];
  for (const key of new Set([...F.keys(), ...M.keys()])) {
    const l = L.get(key), f = F.get(key), m = M.get(key);
    const fields = new Set([...Object.keys(f ?? {}), ...Object.keys(m ?? {})]);
    for (const k of l ? fields : ["(row)"]) {
      const v = (r) => r?.[k] ?? ABSENT;
      const lv = v(l), fv = v(f), mv = v(m), bv = v(B.get(key));
      if (fv === mv || lv !== fv || lv === mv || lv === bv) continue;
      const sha12 = key.slice(0, 12);
      if (declared.has(`${sha12}:${k}`) || declared.has(`${sha12}:*`)) continue;
      out.push({ sha12, platform: key.split("|")[1], field: k, main: bv, landing: lv });
    }
  }
  return out;
}

/** 착지 기준 바뀐 행 수(main ↔ landing). 검수가 인용할 수 — 2점 diff 대신. */
export function changedRows(base, landing) {
  const B = rows(base), L = rows(landing);
  let n = 0;
  for (const k of new Set([...B.keys(), ...L.keys()])) if (JSON.stringify(B.get(k)) !== JSON.stringify(L.get(k))) n++;
  return n;
}

const git = (...a) => execFileSync("git", a, { cwd: ROOT, encoding: "utf8", maxBuffer: 1 << 28, stdio: ["ignore", "pipe", "pipe"] }).trim();
const tryGit = (...a) => { try { return git(...a); } catch { return null; } };
const at = (rev) => { const t = tryGit("show", `${rev}:${COMPAT}`); return t && JSON.parse(t); };

function declaredIn(mb, branch) {
  const set = new Set();
  for (const p of git("diff", "--name-only", "--diff-filter=AM", mb, branch, "--", "docs/worklog/").split("\n").filter((p) => p.endsWith(".json"))) {
    try { for (const d of JSON.parse(git("show", `${branch}:${p}`)).intendedCompatReverts ?? []) set.add(d); } catch { /* 깨진 worklog 는 check-worklog-json 소관 */ }
  }
  return set;
}

function main(argv) {
  const opt = (n, d) => (argv.includes(n) ? argv[argv.indexOf(n) + 1] : d);
  const baseRev = opt("--base", "origin/main");
  const base = tryGit("rev-parse", `${baseRev}^{commit}`);
  if (!base) { console.error(`compat-revert: ${baseRev} 가 없다 — 미대조(fetch 필요)`); return 2; }
  let branch = opt("--head", null);
  if (!branch) { // CI 의 refs/pull/N/merge 는 (main, 브랜치) 머지 커밋이다
    const p1 = tryGit("rev-parse", "HEAD^1"), p2 = tryGit("rev-parse", "HEAD^2");
    branch = p2 && tryGit("merge-base", "--is-ancestor", p1, base) !== null ? p2 : "HEAD"; // main 이 그새 움직여도
  }
  const own = git("rev-list", "--first-parent", branch, `^${base}`).split("\n").filter(Boolean);
  if (!own.length) { console.log("compat-revert: OK — 브랜치가 main 에 이미 들어 있다"); return 0; }
  const fork = tryGit("rev-parse", `${own.at(-1)}^1`), mb = git("merge-base", branch, base);
  const tree = tryGit("merge-tree", "--write-tree", base, branch);
  if (!tree) { console.log("compat-revert: 미대조 — main 과 충돌한다(충돌을 풀면 다시 잰다)"); return 0; }
  const landing = at(tree.split("\n")[0]), mainDoc = at(base);
  const n = changedRows(mainDoc, landing);
  if (fork === mb) { console.log(`compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 ${n}`); return 0; }
  const reverts = findReverts({ landing, fork: at(fork), mb: at(mb), base: mainDoc, declared: declaredIn(mb, branch) });
  if (!reverts.length) {
    console.log(`compat-revert: OK — main 에서 받은 행을 받기 전 값으로 되돌린 필드 0 (fork ${fork.slice(0, 8)} → mb ${mb.slice(0, 8)}) · 착지 기준 바뀐 행 ${n}`);
    return 0;
  }
  for (const r of reverts) console.log(`REVERT ${r.sha12} ${r.platform} ${r.field}: main=${r.main} → 착지=${r.landing} (브랜치가 main 을 받기 전 값)`);
  console.log(`compat-revert: RED — ${new Set(reverts.map((r) => r.sha12)).size}행 ${reverts.length}필드가 브랜치가 받은 main 의 변경을 되돌린다 (fork ${fork.slice(0, 8)} → mb ${mb.slice(0, 8)}).`);
  console.log("  main 을 머지하기 전에 잰 자료로 다시 썼다면 현 main 의 compat.json 위에서 다시 써라.");
  console.log('  의도한 되돌림이면 이 PR 의 docs/worklog/*.json 에 "intendedCompatReverts": ["<sha12>:<필드>"|"<sha12>:*"] 로 선언하라.');
  return 1;
}

function selftest() {
  const e = (sha, sound, status = "playable") => ({ sha256: sha.padEnd(64, "0"), platform: "KTF", status, axes: { sound } });
  const doc = (...es) => ({ entries: es });
  const fork = doc(e("aa", "no"), e("bb", "ok"));
  const mb = doc(e("aa", "ok"), e("bb", "ok"), e("cc", "ok")); // 브랜치가 main 을 받아 aa 수정·cc 추가를 얻었다
  const cases = [
    ["⑴ 정상 — 브랜치가 bb 만 바꿈", { landing: doc(e("aa", "ok"), e("bb", "no"), e("cc", "ok")), fork, mb, base: mb }, 0],
    ["⑵ 받은 aa 를 받기 전 값으로", { landing: doc(e("aa", "no"), e("bb", "ok"), e("cc", "ok")), fork, mb, base: mb }, 1],
    ["⑵' 받은 cc 행을 지움", { landing: doc(e("aa", "ok"), e("bb", "ok")), fork, mb, base: mb }, 1],
    ["⑶ 선언된 되돌림", { landing: doc(e("aa", "no"), e("bb", "ok"), e("cc", "ok")), fork, mb, base: mb, declared: new Set(["aa0000000000:axes.sound"]) }, 0],
    ["⑷ 엔진 수정이 옛 값을 되살림(v1 오탐 형) — 받은 적 없는 행", { landing: doc(e("aa", "no", "limited")), fork: doc(e("aa", "no", "playable")), mb: doc(e("aa", "no", "playable")), base: doc(e("aa", "no", "playable")) }, 0],
    ["⑸ main 이 mb 뒤에 같은 값으로 되돌렸다 — 착지 == main", { landing: doc(e("aa", "no"), e("bb", "ok"), e("cc", "ok")), fork, mb, base: doc(e("aa", "no"), e("bb", "ok"), e("cc", "ok")) }, 0],
  ];
  let bad = 0;
  for (const [name, input, want] of cases) {
    const got = findReverts(input).length;
    console.log(`${got === want ? "ok " : "BAD"} ${name}: ${got} (기대 ${want})`);
    if (got !== want) bad++;
  }
  return bad ? 1 : 0;
}

process.exit(process.argv.includes("--selftest") ? selftest() : main(process.argv.slice(2)));
