#!/usr/bin/env bash
# audit-missing-tests.sh — 「개명이 조용히 떨어뜨린 시험이 더 있는가」를 기계로 묻는다.
#
# ── Why this lives in the repo but NOT in CI (decided 2026-09-19) ─────────────
# It ran once, from an evidence directory outside the repo, and found 20 test
# function names that a crate-rename/base-swap dropped. The question this round
# answered is whether to promote it to a standing check. **It is not promoted**,
# and the reason is not the one the proposal expected — so both halves are
# written down here, because the next person to ask will re-derive them.
#
#   ⒜ The objection that was RAISED does not survive measurement. The proposal
#      feared axis E ("compare test-function NAMES") would fire on every ordinary
#      test rename and therefore need an allowlist that rots. Measured over the
#      last 40 first-parent landings: **0 of 40** dropped a test-function name.
#      A check whose baseline is the PR's own merge-base needs no allowlist at
#      all, and the one it would use cannot rot — it is recomputed per PR.
#   ⒝ The objection that DOES hold is about the other end of the pipe. Run
#      against the landing that brought the base swap in (`37734e74`, #161), a
#      per-PR axis E would have printed **22 names in one go** — every one the
#      audit later found, plus two that a human had already restored by eye. So
#      detection was never the scarce thing; ADJUDICATION is. Of the 20 this
#      script reports (re-measured 2026-09-21 against `origin/main`):
#
#        9  submitted in an OPEN pull request — the 8 `canvas` assertions in
#           **#213** and `wipic_svc_0x581_…` in **#215**. ★"Submitted" is the
#           whole claim: NEITHER PR HAS LANDED, so against `origin/main` all 9
#           names are still missing and this script still prints them. Read the
#           bucket as "someone has proposed an answer", never as "restored".
#        1  false positive — the name moved *into* another test (#215 measured it)
#        1  inapplicable — its target function no longer exists at HEAD
#        9  ★NOBODY HAS ASKED YET — the genuinely undecided remainder
#
#      ★Adding a second detector while 9 of the first one's 20 have not been
#      looked at once buys nothing. ★Do not read this as 17 undecided: an
#      earlier revision of this header said so, counting #213's 8 as untouched
#      while that PR had already been open for two hours. The number that
#      carries the decision is **9/20**, not 17/20.
#   ⒞ Cost is NOT the reason. The whole script runs in ~1s and a single name-set
#      build is 0.2-0.5s. Anyone re-proposing this should not argue about speed.
#
# ── When to reopen — both conditions are OBSERVABLE, so check them, do not
#    re-argue them ──────────────────────────────────────────────────────────────
#   ⑴ **#213 and #215 land, AND the remaining 9 are then adjudicated.** Landing
#      those two does NOT spend the objection — it moves 9 names out of the
#      "submitted" bucket and leaves the 9 nobody has asked about. Adjudicating
#      those 9 (restore, or measure them to be false positives the way #215 did)
#      is what spends it, because then no output of this script sits unprocessed.
#      ★How to check rather than assume: re-run axis E against `origin/main`.
#      Today it prints 20; after both PRs land it should print 11, and when the
#      remaining 9 are adjudicated the bucket this decision rests on is empty.
#      ★That is days away, not quarters — write the re-check into the round that
#      lands the last of them. ⇒ or
#   ⑵ a second mass rename lands and the loss is found **by eye again** rather
#      than by re-running this — that is the counterfactual failing in the open.
# ★If it is reopened, build the **per-PR merge-base** form and make it
# **report-only** (`continue-on-error`, like `checker-census`). A blocking gate
# would have stopped the base-swap PR on 22 legitimate items, and a fixed
# baseline + allowlist is the shape ⒜ measured to be unnecessary.
#
# ── What it measures ─────────────────────────────────────────────────────────
# 제안 `2026-09-18-restore-lgt-reach-tests#p0` 의 이행. 되살린 두 파일은 «사람이 눈으로» 찾았다.
# 이 스크립트는 그 눈을 술어로 바꾼 것이고, ★찾은 것을 «되살리지 않는다» — 목록이 산출물이다.
#
# 술어는 다섯이다. 각각 «다른 종류의 결손»을 보고, 하나가 다른 하나를 대신하지 못한다:
#   A 수집 안 됨   : 디스크에 있는데 cargo 가 test 타깃으로 «모른다»(있으나 안 돈다)
#   B 개명 손실    : 어떤 커밋에 있었는데 지금 없고, git 이 «rename 으로 못 잇는다»(없어졌다)
#   C 역사 전수    : B 를 base swap 한 회차가 아니라 «전 역사»로 넓힌 것
#   D 빈 껍데기    : 수집은 되는데 #[test] 계열 함수가 0개
#   E 함수 이름    : 파일이 아니라 «함수 이름 집합» 비교
#
# ★★E 의 한계를 A~D 의 한계와 «대칭으로» 읽어라: 파일 축은 «합쳐진 시험»에 눈멀고,
# 함수 축은 «다른 시험 안으로 흡수된 단언»에 눈먼다. 실측 1건 —
# `test_helloworld_jar_named_application` 은 이름만 사라졌고 그 단언은 `test_helloworld`
# 안에 살아 있다(개악으로 확인: 옛 이름 기반 탐색으로 되돌리면 지금 시험이 red 다).
# ⇒ ★이 스크립트의 출력은 «결손 목록»이 아니라 «조사 대상 목록»이다.
#
# ── ★옮겨 오며 «고치지 않은» 선재 결함 2건 — 값은 맞는데 «죽은 가지»가 있다 ──────
# 이 술어는 원장 증적에서 **한 글자도 안 고치고** 옮겼다. 그래서 원본의 결함도 같이 왔다.
# ★**여기 적는 이유**: 다음 사람이 이 파일을 «정본»으로 읽는데, 둘 다 **조용하다**.
#   ⑴ ★**축 C 의 주경로는 «항상» 실패한다** — 이 맥의 `/usr/bin/grep` 은 **BSD grep 2.6.0**
#      이고 `-P` 가 **없다**(실측 `printf 'abc\n' | /usr/bin/grep -P 'a'` → **rc=2**
#      `invalid option -- P`). `2>/dev/null` 이 그 메시지를 삼키고, `set -uo pipefail` 이
#      pipeline rc 를 살려 `||` 뒤의 **awk 폴백**이 탄다. ⇒ 결과는 맞지만 **`-P` 가지는
#      단 한 번도 돈 적이 없다.** 처방은 그 가지를 지우고 awk 한 줄만 남기는 것 —
#      ★**코드 변경이라 이 회차(문면 전용)에서 하지 않았다.** worklog 제안으로 넘겼다.
#   ⑵ ★**축 A 는 fail-open 이다** — `cargo metadata` 가 죽으면 stderr 가 버려지고 python
#      `json.load` 가 죽어 **빈 목록**이 되며, `comm -23` 이 **디스크의 시험파일 전건**을
#      `DEAD` 로 고발한다. report-only 도구라 계급은 낮지만, 「재실행이 한 줄」이 완화책인
#      도구가 **조용히 거짓 경보를 내는 모양**은 알고 써라. 같은 제안에 묶었다.
#
# 사용: scripts/audit-missing-tests.sh [repo 경로]   (기본 = cwd)
#   SWAP=<sha> 로 기준 커밋을 바꾼다(기본 = 36df9c31, 2026-09-16 base swap).
set -uo pipefail
cd "${1:-.}" || exit 2

TESTPAT='(^|/)tests/[^/]+\.rs$'
G=/usr/bin/grep                    # ★대화 셸의 grep 은 ugrep 그림자다 — 절대경로로 부른다

say() { printf '\n== %s\n' "$*"; }

# ── A. cargo 가 «아는» test 타깃 ↔ 디스크에 «있는» test 파일 ───────────────────
say "A. 수집 안 되는 test 파일 (디스크에 있는데 cargo 타깃 0)"
cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c '
import sys, json, os
m = json.load(sys.stdin); root = m["workspace_root"]
for p in m["packages"]:
    for t in p["targets"]:
        if t["kind"] == ["test"]:
            print(os.path.relpath(t["src_path"], root))
' | sort -u > /tmp/.audit_cargo.$$
git ls-files | $G -E "$TESTPAT" | sort -u > /tmp/.audit_disk.$$
printf '   cargo test 타깃: %s개 · 디스크 test 파일: %s개\n' \
  "$(wc -l < /tmp/.audit_cargo.$$ | tr -d ' ')" "$(wc -l < /tmp/.audit_disk.$$ | tr -d ' ')"
comm -23 /tmp/.audit_disk.$$ /tmp/.audit_cargo.$$ | sed 's/^/   DEAD  /'
comm -13 /tmp/.audit_disk.$$ /tmp/.audit_cargo.$$ | sed 's/^/   ★타깃인데 tracked 가 아님  /'

# ── B. 두 부모 → 합류 커밋에서 «rename 으로 안 이어진» 삭제 ────────────────────
#    ★`git log --diff-filter=D -- <path>` 를 쓰지 마라. 두 겹으로 거짓말한다:
#      ⑴ history simplification 이 기본이라 머지 곁가지를 지운다(`--full-history` 필요)
#      ⑵ 머지 커밋 «안»의 삭제는 `-m`/`--diff-merges` 없이는 아예 안 보인다
#    실측: base swap 은 머지 커밋이라 두 함정에 다 걸려 `--diff-filter=D` 가 «0건» 을 답한다.
#    ⇒ 로그가 아니라 **트리 대 트리**로 물어라. 그리고 rename 판정은 git 의 유사도에 맡긴다.
say "B. base swap 에서 떨어진 test 파일 (부모 트리 → 합류 트리, -M 로 rename 제외)"
SWAP=${SWAP:-36df9c31}
for P in $(git rev-parse "$SWAP"^@); do
  printf '   -- 부모 %s: %s\n' "${P:0:8}" "$(git log -1 --format=%s "$P" | cut -c1-60)"
  git diff -M --diff-filter=D --name-only "$P" "$SWAP" | $G -E "$TESTPAT" | sed 's/^/      DROPPED  /'
  git diff -M --diff-filter=R --name-status "$P" "$SWAP" | $G -E "$TESTPAT" | sed 's/^/      renamed  /'
done
say "B-2. 그중 «지금도» 없는 것 (부모 → HEAD, 이후 복원분을 뺀다)"
for P in $(git rev-parse "$SWAP"^@); do
  git diff -M --diff-filter=D --name-only "$P" HEAD | $G -E "$TESTPAT" | sed "s|^|      STILL-GONE(${P:0:8})  |"
done

# ── C. 전 역사: rename 으로 안 이어진 삭제 전수 ────────────────────────────────
#    `-m` 으로 머지 내부까지 보고, `-M` 으로 rename 을 D 에서 뺀다.
say "C. 전 역사에서 «rename 아닌» 삭제로 사라진 test 파일 (HEAD 에 없는 것만)"
git log --all --full-history -m -M --name-status --format='%H' \
  | awk '/^[0-9a-f]{40}$/{c=$0; next} /^D\t/{print c"\t"$2} /^R[0-9]*\t/{print "R\t"$2"\t"$3}' \
  | $G -E "$TESTPAT" > /tmp/.audit_hist.$$
$G -P '^[0-9a-f]{40}\t' /tmp/.audit_hist.$$ 2>/dev/null | cut -f2 | sort -u > /tmp/.audit_del.$$ \
  || awk -F'\t' '$1 ~ /^[0-9a-f]{40}$/ {print $2}' /tmp/.audit_hist.$$ | sort -u > /tmp/.audit_del.$$
comm -23 /tmp/.audit_del.$$ /tmp/.audit_disk.$$ | while read -r f; do
  last=$(awk -F'\t' -v f="$f" '$1 ~ /^[0-9a-f]{40}$/ && $2==f {print $1}' /tmp/.audit_hist.$$ | head -1)
  printf '   GONE  %-55s  마지막 삭제커밋 %s  %s\n' "$f" "${last:0:8}" \
    "$(git log -1 --format=%s "$last" 2>/dev/null | cut -c1-48)"
done

# ── D. 수집되는데 시험 함수가 0개 ──────────────────────────────────────────────
say "D. 수집되는데 #[test] 계열 함수가 0개인 파일"
while read -r f; do
  n=$($G -cE '#\[(test|futures_test::test|tokio::test|async_std::test)\]' "$f" 2>/dev/null)
  [ "${n:-0}" = "0" ] && printf '   EMPTY  %s\n' "$f"
done < /tmp/.audit_cargo.$$
echo "   (위에 아무것도 없으면 0건)"

rm -f /tmp/.audit_cargo.$$ /tmp/.audit_disk.$$ /tmp/.audit_hist.$$ /tmp/.audit_del.$$

# ── E. ★함수 단위: swap 직전 트리에 있던 #[test] 함수 중 HEAD 에 «이름이 없는» 것 ─────
#    ★A~D 는 «파일»을 본다. 그래서 `#[cfg(test)] mod` 안의 유닛시험이 통째로 빠져도 «보이지 않는다».
#    실측: 이 축이 A~D 가 못 본 결손을 찾았다. 파일 축만 돌리고 「0건」이라 적으면 그것은 거짓이다.
say "E. swap 직전(\$SWAP^1) → HEAD 에서 «이름이 사라진» 시험 함수"
SWAP=${SWAP:-36df9c31} python3 - <<'PY'
import subprocess, re, os
PAT = re.compile(rb'#\[\s*(?:futures_test::|tokio::|async_std::|actix_rt::)?test\s*[\]\(]')
FN  = re.compile(rb'\bfn\s+([A-Za-z0-9_]+)')
def names(tree):
    ls = subprocess.run(['git','ls-tree','-r','--format=%(objectname) %(path)',tree],
                        capture_output=True).stdout.decode().splitlines()
    ents = [l.split(' ',1) for l in ls if l.endswith('.rs')]
    if not ents: return {}, 0
    p = subprocess.run(['git','cat-file','--batch'],
                       input=''.join(s+'\n' for s,_ in ents).encode(), capture_output=True)
    out, pos, res = p.stdout, 0, {}
    for sha, path in ents:                      # ★cat-file --batch 한 번 — 파일마다 포크하면 2분을 넘긴다
        nl = out.index(b'\n', pos); size = int(out[pos:nl].split()[2])
        body = out[nl+1:nl+1+size]; pos = nl+1+size+1
        for m in PAT.finditer(body):
            fm = FN.search(body, m.end(), m.end()+400)
            if fm: res.setdefault(fm.group(1).decode(), set()).add(path)
    return res, len(ents)
base = subprocess.run(['git','rev-parse',os.environ['SWAP']+'^1'],
                      capture_output=True).stdout.decode().strip()
pre, npre = names(base); head, nhead = names('HEAD')
print("   swap^1=%s  .rs=%d 시험함수=%d   ↔   HEAD .rs=%d 시험함수=%d"
      % (base[:8], npre, len(pre), nhead, len(head)))
lost = sorted(set(pre) - set(head))
print("   ★이름이 사라진 시험 함수: %d" % len(lost))
for n in lost:
    print("      LOST  %-52s  (당시: %s)" % (n, ', '.join(sorted(pre[n]))))
PY
