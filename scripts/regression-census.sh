#!/usr/bin/env bash
# Weekly regression watch: measure the current origin/main on every title and report only the rows
# that went DOWN against the shipped support page (docs/player-data/compat.json). Local only — the
# corpus is git-ignored game bytes (Constraint 9), so no CI can run this.
#   bash scripts/regression-census.sh           (no arguments; the ops launchd job runs exactly this)
#   bash scripts/regression-census.sh selftest  the comparison against two fake compats (regression-diff.mjs)
#
# 1. host-load-guard --status --recovered ≠ 0 → one «skipped» ledger line, rc 0.
# 2. release wie_validate from origin/main in its own worktree (build-slot, short pool), copied into the round.
# 3. probes A·B on every title; then the 600 s long run on the guards (2 + live LGT 5) and on the rows whose
#    probe axes moved since the previous round (the shipped compat when there is none).
# 4. regression-diff.mjs: DOWN rows are candidates; candidates are re-measured once with the same binary
#    (the pair) and only those that go down again are «suspects». UP rows are a note. compat is not written.
# 5. evidence md (sha12 only) + one ledger line: WIE_REGRESSION_SUSPECT n=<n> pin=<sha8> | WIE_REGRESSION_CLEAR.
# Every sweep goes through census-drive.sh (one build-slot --long lease per ~15 min, census lock, --jobs
# default ≤ 3). Nothing here is backgrounded. The whole round stops at WIE_REGRESSION_MAX_SECS (6 h).
# A round is resumable: re-running on the same date reuses its binary and every result already on disk.
set -u
cd "$(dirname "$0")/.."
REPO=$PWD
[[ ${1:-} == selftest ]] && exec node scripts/regression-diff.mjs selftest
T0=$(date +%s)
DATE=${WIE_REGRESSION_DATE:-$(date +%F)}
ROOT=${WIE_REGRESSION_ROOT:-$HOME/wie-regression} # private: game names and frames live here, never in the repo
EVID=${WIE_REGRESSION_EVIDENCE:-$HOME/orchestrator/reports/evidence/wie-weekly-regression}
BUILD=${WIE_REGRESSION_BUILD_TREE:-$HOME/worktrees/wie-regression-build}
read -r -a CORPUS <<<"${WIE_CORPUS:-$REPO/game_lab/working $REPO/game_lab/broken}"
RECIPES=${WIE_RECIPES:-$REPO/game_lab/recipes-sound}
OB=$HOME/orchestrator-live/bin
NOTE=${WIE_REGRESSION_NOTE:-$OB/note} # a dry run points this at /usr/bin/true so the ledger never hears it
export CENSUS_DEADLINE=$((T0 + ${WIE_REGRESSION_MAX_SECS:-21600}))
# Guards 2 + live LGT 5: the set every engine PR re-measures (docs/report — «라이브 LGT 5 · 가드 2»).
GUARDS='49ade89578c5 ddd885583b15 13d7e3c21856 1b107b96bf4e 4ece6eeeaa04 a30bbe008b5e b475b6399684'

note() {
  echo "regression-census: $1" >&2
  if [[ -x $NOTE ]]; then printf '%s\n' "$1" | "$NOTE"; fi
}
# A sound recipe (game_lab/recipes-sound/<sha12>.keys) prefixes the long run, as in the census rounds.
with_recipes() { while read -r s; do if [[ -f $RECIPES/$s.keys ]]; then echo "$s 600 $RECIPES/$s.keys"; else echo "$s"; fi; done; }
drive() { bash scripts/census-drive.sh --bin "$R/wie_validate" "$@"; }
census_report() { node scripts/playability-census.mjs report --out "$1" --pin "$PIN" >/dev/null; }

if [[ -x $OB/host-load-guard ]] && ! "$OB/host-load-guard" --status --recovered >&2; then
  note "WIE_REGRESSION_SKIPPED reason=host-load date=$DATE (이번 주 건너뜀)"
  exit 0
fi

R=$ROOT/$DATE
mkdir -p "$R" "$EVID"
if [[ ! -x $R/wie_validate ]]; then
  git fetch -q origin main || exit 1
  [[ -d $BUILD ]] || git worktree add -q --detach "$BUILD" origin/main || exit 1
  git -C "$BUILD" checkout -q --detach origin/main || exit 1
  build=(cargo build -q --release -p wie_cli --bin wie_validate)
  if [[ -x $OB/build-slot ]]; then (cd "$BUILD" && "$OB/build-slot" run -- "${build[@]}"); else (cd "$BUILD" && "${build[@]}"); fi || exit 1
  git -C "$BUILD" rev-parse HEAD >"$R/pin"
  git -C "$BUILD" show HEAD:docs/player-data/compat.json >"$R/base-compat.json"
  cp "$BUILD/target/release/wie_validate" "$R/wie_validate"
fi
PIN=$(cat "$R/pin")
TOOL_PIN=$(git rev-parse HEAD)

incomplete() {
  note "WIE_REGRESSION_INCOMPLETE stage=$1 pin=${PIN:0:8} date=$DATE (상한 ${WIE_REGRESSION_MAX_SECS:-21600}s 또는 실패 — 같은 날 다시 돌리면 이어서 잰다)"
  exit 1
}

# Probes on every title, then the long run on the guards and the rows that moved.
drive --out "$R/out" --only probe "${CORPUS[@]}" || incomplete probe
census_report "$R/out" || incomplete report
PREV=$(ls -d "$ROOT"/*/compat.json 2>/dev/null | grep -v "^$R/" | sort | tail -1)
{
  for s in $GUARDS; do echo "$s"; done
  node scripts/regression-diff.mjs changed --base "${PREV:-$R/base-compat.json}" --new "$R/out/compat.json"
} | sort -u | with_recipes >"$R/long.titles"
drive --out "$R/out" --only long --titles "$R/long.titles" "${CORPUS[@]}" || incomplete long
census_report "$R/out" || incomplete report

# The pair: every candidate once more, same binary, into its own out dir.
n=$(node scripts/regression-diff.mjs candidates --base "$R/base-compat.json" --new "$R/out/compat.json" --long "$R/long.titles" \
  --population "$R/out/population.json" --pair-corpus "$R/pair-corpus" --pair-long "$R/pair.long.titles") || incomplete candidates
pair=()
if ((n > 0)); then
  with_recipes <"$R/pair.long.titles" >"$R/pair.long.recipes"
  drive --out "$R/pair" --only probe "$R/pair-corpus" || incomplete pair-probe
  if [[ -s $R/pair.long.recipes ]]; then drive --out "$R/pair" --only long --titles "$R/pair.long.recipes" "$R/pair-corpus" || incomplete pair-long; fi
  census_report "$R/pair" || incomplete pair-report
  pair=(--pair "$R/pair/compat.json" --pair-long "$R/pair.long.titles")
fi

MD=$EVID/$DATE.md
sus=$(node scripts/regression-diff.mjs report --base "$R/base-compat.json" --new "$R/out/compat.json" --long "$R/long.titles" ${pair[@]+"${pair[@]}"} \
  --pin "$PIN" --tool-pin "$TOOL_PIN" --elapsed $(($(date +%s) - T0)) --date "$DATE" --out "$MD") || incomplete diff
cp "$R/out/compat.json" "$R/compat.json"
# Keep this round and the one before it whole; older rounds keep only their compat.json (≈300 MB of frames each).
ls -d "$ROOT"/*/ | sort -r | tail -n +3 | while read -r d; do rm -rf "${d}out" "${d}pair" "${d}pair-corpus" "${d}wie_validate"; done
echo "regression-census: $MD · suspects $sus · candidates $n · $(($(date +%s) - T0))s" >&2
if ((sus > 0)); then note "WIE_REGRESSION_SUSPECT n=$sus pin=${PIN:0:8}"; else note "WIE_REGRESSION_CLEAR pin=${PIN:0:8}"; fi
