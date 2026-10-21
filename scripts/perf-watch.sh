#!/usr/bin/env bash
# Weekly speed watch: is origin/main slower than the pin the last round measured? The compat census
# (regression-census.sh) watches «does it still work»; this watches «how fast», which it cannot see.
# Local only — the samples are git-ignored game bytes (Constraint 9). Judging and the axes: perf-watch.mjs header.
#   bash scripts/perf-watch.sh                      weekly (regression-census.sh calls it): A = the last round's pin
#                                                   (bd770b73 when there is none), B = origin/main
#   bash scripts/perf-watch.sh compare <A> <B> <out>  any two revs (or build dirs) — the bisect and the live check
#   bash scripts/perf-watch.sh build <rev>          build <rev> (native + wasm) and print its build dir
#   bash scripts/perf-watch.sh slow <rev> [<spin>]  build <rev> with <spin> (default 16) black_box iterations at the
#                                                   top of every `Executor::tick_for` loop pass — a deliberately slow
#                                                   engine, never committed; prints its build dir (see «개악 대조»)
#   bash scripts/perf-watch.sh selftest             the judging rules against synthetic pairs
#
# 1. host-load-guard --status --recovered ≠ 0 → one WIE_PERF_SKIPPED line carrying the skip streak, rc 0.
# 2. both revs built from `git archive` (native wie_validate + build-wasm.sh), short build-slot pool, kept by sha12.
# 3. reps 1–3 × axes, ONE build-slot --long lease per phase (≈ 4–6 min); the samples' A and B run together.
# 4. a sample that alarms is re-measured (reps 4–6); one that alarms again is a regression.
# 5. evidence md (sha12 only) + one ledger line: WIE_PERF_REGRESSION n=<n> | WIE_PERF_CLEAR.
# Resumable: the same date reuses every build and every row on disk. WIE_PERF_AXES=native skips the browser.
set -u
cd "$(dirname "$0")/.."
REPO=$PWD
[[ ${1:-} == selftest ]] && exec node scripts/perf-watch.mjs selftest
DATE=${WIE_PERF_DATE:-$(date +%F)}
ROOT=${WIE_REGRESSION_ROOT:-$HOME/wie-regression}
PERF=$ROOT/perf # builds/<sha12>[-slow<n>] and the weekly rounds; private (game names in samples.json)
EVID=${WIE_PERF_EVIDENCE:-$HOME/orchestrator/reports/evidence/wie-weekly-perf}
read -r -a CORPUS <<<"${WIE_CORPUS:-$REPO/game_lab/working $REPO/game_lab/broken} ${WIE_OSS_JARS:-$HOME/work/oss-j2me-lab/jars}"
read -r -a AXES <<<"${WIE_PERF_AXES:-native browser}"
OB=$HOME/orchestrator-live/bin
NOTE=${WIE_REGRESSION_NOTE:-$OB/note}
BASE=bd770b73 # 10-05, the reference pin of docs/report/0495 and 0500

note() {
  echo "perf-watch: $1" >&2
  if [[ -x $NOTE ]]; then printf '%s\n' "$1" | "$NOTE"; fi
}
slot() {
  if [[ -x $OB/build-slot ]]; then "$OB/build-slot" run "$@"; else
    while [[ $1 == --* ]]; do shift; done
    "$@"
  fi
}

# build <rev> [<spin>] -> prints the build dir (wie_validate + wasm/). A dir argument is returned as is.
build() {
  if [[ -x $1/wie_validate ]]; then echo "$1"; return; fi
  local sha
  sha=$(git rev-parse --verify -q "$1^{commit}") || { echo "perf-watch: no such rev $1" >&2; return 1; }
  local d=$PERF/builds/${sha:0:12}${2:+-slow$2}
  if [[ -x $d/wie_validate && -s $d/wasm/wie_web_bg.wasm ]]; then echo "$d"; return; fi
  local t=$PERF/tree-${d##*/}
  rm -rf "$t" && mkdir -p "$t" "$d" && git archive "$sha" | tar -x -C "$t" || return 1
  if [[ -n ${2:-} ]]; then
    # The deliberately slow engine: a fixed amount of work on every pass of the executor loop, both hosts.
    # A dependent chain through black_box(u64): `black_box(())` emits no instruction and cost 0 (measured).
    perl -0pi -e "s/(        loop \{\n)(            if self\.halted)/\$1            let mut spin = 0u64;\n            for i in 0..$2u64 { spin = core::hint::black_box(spin ^ i); }\n\$2/" "$t/wie-backend/src/executor.rs"
    grep -q "0..$2u64 { spin = core::hint::black_box" "$t/wie-backend/src/executor.rs" || { echo "perf-watch: slow patch did not apply" >&2; return 1; }
  fi
  [[ -d $REPO/node_modules ]] && ln -s "$REPO/node_modules" "$t/node_modules"
  (cd "$t" && slot -- cargo build -q --release -p wie_cli --bin wie_validate && slot -- bash scripts/build-wasm.sh >/dev/null) >&2 || return 1
  cp "$t/target/release/wie_validate" "$d/" && mkdir -p "$d/wasm" && cp "$t"/web/src/wasm/wie_web{.js,_bg.wasm} "$d/wasm/" || return 1
  rm -rf "$t"
  echo "$d"
}

# measure <A dir> <B dir> <out> -> prints the regression count (evidence md written by the caller's report)
measure() {
  local a=$1 b=$2 out=$3 ax rep only
  mkdir -p "$out"
  [[ -s $out/samples.json ]] || node scripts/perf-watch.mjs resolve --out "$out" "${CORPUS[@]}" || return 1
  for rep in 1 2 3; do for ax in "${AXES[@]}"; do
    slot --long -- node scripts/perf-watch.mjs run --out "$out" --axis "$ax" --rep "$rep" --a "$a" --b "$b" || return 1
  done; done
  for ax in "${AXES[@]}"; do
    only=$(node scripts/perf-watch.mjs judge --out "$out" --reps 1-3 --axis "$ax") || return 1
    [[ -z $only ]] && continue
    for rep in 4 5 6; do
      slot --long -- node scripts/perf-watch.mjs run --out "$out" --axis "$ax" --rep "$rep" --a "$a" --b "$b" --only "$only" || return 1
    done
  done
}

case ${1:-} in
  build) build "$2"; exit ;;
  slow) build "$2" "${3:-16}"; exit ;;
  compare)
    a=$(build "$2") && b=$(build "$3") || exit 1
    measure "$a" "$b" "$4" || exit 1
    node scripts/perf-watch.mjs report --out "$4" --md "$4/perf.md" --a-pin "$(basename "$a")" --b-pin "$(basename "$b")" --tool-pin "$(git rev-parse HEAD)" --date "$DATE"
    exit ;;
  '') ;;
  *) echo "perf-watch: unknown argument «$1» — see the header (no argument = the weekly round)" >&2; exit 2 ;;
esac

SKIPS=$PERF/skips
if [[ -x $OB/host-load-guard ]] && ! "$OB/host-load-guard" --status --recovered >&2; then
  n=$(($(cat "$SKIPS" 2>/dev/null || echo 0) + 1))
  mkdir -p "$PERF" && echo "$n" >"$SKIPS"
  note "WIE_PERF_SKIPPED reason=host-load streak=$n date=$DATE (속도 감시 이번 주 건너뜀 · 연속 ${n}주)"
  exit 0
fi
incomplete() {
  note "WIE_PERF_INCOMPLETE stage=$1 date=$DATE (같은 날 다시 돌리면 이어서 잰다)"
  exit 1
}
git fetch -q origin main || incomplete fetch
R=$PERF/$DATE
mkdir -p "$R" "$EVID"
# A = the pin the last finished (judged) round measured as B; the first round has none and measures against BASE.
[[ -s $R/a-pin ]] || { prev=$(ls "$PERF"/*/judge.json 2>/dev/null | grep -v "^$R/" | sort | tail -1); [[ -n $prev ]] && cat "$(dirname "$prev")/b-pin" >"$R/a-pin"; [[ -s $R/a-pin ]] || git rev-parse "$BASE" >"$R/a-pin"; }
[[ -s $R/b-pin ]] || git rev-parse origin/main >"$R/b-pin"
A_PIN=$(cat "$R/a-pin") B_PIN=$(cat "$R/b-pin")
MD=$EVID/$DATE.md
if [[ $A_PIN == "$B_PIN" ]]; then
  printf '# wie 주간 속도 감시 — %s\n\n- 지난 회차와 같은 핀 `%s` — 잴 것이 없다.\n' "$DATE" "${B_PIN:0:8}" >"$MD"
  rm -f "$SKIPS"
  note "WIE_PERF_CLEAR pin=${B_PIN:0:8} same-pin"
  exit 0
fi
a=$(build "$A_PIN") || incomplete build-a
b=$(build "$B_PIN") || incomplete build-b
measure "$a" "$b" "$R" || incomplete measure
n=$(node scripts/perf-watch.mjs report --out "$R" --md "$MD" --a-pin "$A_PIN" --b-pin "$B_PIN" --tool-pin "$(git rev-parse HEAD)" --date "$DATE") || incomplete report
rm -f "$SKIPS"
# Keep the builds the next round can use (this round's B) and BASE; the rest are ≈ 40 MB each.
ls -d "$PERF"/builds/*/ 2>/dev/null | grep -v -e "/${B_PIN:0:12}/" -e "/$(git rev-parse "$BASE" | cut -c1-12)/" | while read -r d; do rm -rf "$d"; done
if ((n > 0)); then note "WIE_PERF_REGRESSION n=$n pin=${B_PIN:0:8} prev=${A_PIN:0:8} evidence=$MD"; else note "WIE_PERF_CLEAR pin=${B_PIN:0:8} prev=${A_PIN:0:8}"; fi
