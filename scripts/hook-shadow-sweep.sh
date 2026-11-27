#!/usr/bin/env bash
# Shadow-compare the generic KTF hooks (wie-core-arm/data/binary_patches.toml, the hash-less entry)
# against each title's own code, over a set of packages — the table a hook change is judged by.
# What is compared and what is not: wie-core-arm/src/binary_patches/hook/shadow.rs.
# Local only: it needs game files.
#   cargo build --release -p wie_cli --bin wie_validate     (under build-slot; WIE_VALIDATE=<bin> to use another)
#   bash scripts/hook-shadow-sweep.sh <out-dir> <package>…  run what <out-dir> does not hold yet, then the table
#   bash scripts/hook-shadow-sweep.sh <out-dir>              the table only
#   bash scripts/hook-shadow-sweep.sh selftest               the exit code, on made-up results (no game files)
# Exit 1: a mismatch, a fault on either side (`own_fault`, `hook_fault`), a fire that was neither
# compared nor skipped, a result file with no shadow report in it (the run died), or no results at all.
# Results are named by sha256 prefix, so <out-dir> and the table carry no title names. Keep <out-dir>
# outside the repo.
# One `build-slot run --long` lease per HOOK_SHADOW_PHASE packages (default 60 ≈ 10 min: 3 at a
# time, 30 s each) — never wrap this script itself in build-slot (CLAUDE.md «측정 스윕»).
# HOOK_SHADOW_TIMEOUT=<s> (default 30) is wie_validate's --timeout; the other arguments are the
# census's probe A. HOOK_SHADOW_KEYS=<script> replaces its 27 keys (wie_validate's --keys). A starved host reaches fewer calls: read `fires` (and load1) before `mismatch`,
# and re-run a package by deleting its result. An overlapping call is reached on some runs and not
# on others: "0 mismatches" covers it only on a run whose `handed to the title's own routine` line
# names that package.
# WIE_AOT defaults to off here (the interpreter), though wie_validate itself defaults to native AOT: under AOT every
# fire rewrites the hook site twice and discards the compiled region each time, and one title reached half the calls
# in the same run (docs/report/0607 §6). `WIE_AOT=native bash scripts/hook-shadow-sweep.sh …` sweeps that arm.
set -u
cd "$(dirname "$0")/.."
BIN=${WIE_VALIDATE:-target/release/wie_validate}
SLOT=~/orchestrator-live/bin/build-slot
PHASE=${HOOK_SHADOW_PHASE:-60}
out=${1:?usage: hook-shadow-sweep.sh <out-dir> [package…]}
shift

if [[ $out == selftest ]]; then
  t=$(mktemp -d)
  trap 'rm -rf "$t"' EXIT
  row() { # <package> <compared> <mem_mismatch> <hook_fault>, of 10 fires
    echo "{\"result\":\"PASS\",\"load1\":1,\"package\":\"$1\",\"hook_shadow\":{\"memcpy\":{\"fires\":10,\"compared\":$2,\"mem_mismatch\":$3,\"result_mismatch\":0,\"callee_saved_mismatch\":0,\"own\":0,\"skipped\":0,\"own_fault\":0,\"hook_fault\":$4},\"notes\":[]}}" >"$t/$1.json"
  }
  expect() { # <exit code> <what the results are>
    bash "$0" "$t" >/dev/null 2>&1
    rc=$?
    [[ $rc == "$1" ]] && return
    echo "selftest FAIL: $2 — exit $rc, expected $1"
    exit 1
  }
  expect 1 "no results"
  row a 10 0 0 && expect 0 "every fire compared, no mismatch"
  row b 3 0 0 && expect 1 "fires 10 · compared 3 · skipped 0"
  row b 10 1 0 && expect 1 "mem_mismatch 1"
  row b 9 0 1 && expect 1 "hook_fault 1"
  : >"$t/b.json" && expect 1 "a result file with nothing in it"
  rm "$t/b.json" && expect 0 "back to the good one"
  echo "selftest ok"
  exit 0
fi

case ${1:-} in
--one) # one package
  sha=$(shasum -a 256 "$2" | cut -c1-12)
  [[ -s $out/$sha.json ]] && exit 0
  WIE_AOT=${WIE_AOT:-off} WIE_HOOK_MODE=shadow RUST_LOG=error "$BIN" --inject --keep-timeout --timeout "${HOOK_SHADOW_TIMEOUT:-30}" \
    --max-ticks 100000000000 --pacing 8 --relaunch 1 ${HOOK_SHADOW_KEYS:+--keys "$HOOK_SHADOW_KEYS"} "$2" 2>/dev/null | tail -1 |
    sed "s/}\$/,\"package\":\"$sha\",\"load1\":$(sysctl -n vm.loadavg | cut -d' ' -f2)}/" >"$out/$sha.tmp"
  mv "$out/$sha.tmp" "$out/$sha.json"
  exit 0
  ;;
--phase) # one lease
  shift
  printf '%s\0' "$@" | xargs -0 -P 3 -n 1 bash "$0" "$out" --one
  exit 0
  ;;
esac

mkdir -p "$out"
while (($#)); do
  n=$(($# < PHASE ? $# : PHASE))
  if [[ -x $SLOT ]]; then
    "$SLOT" run --long -- bash "$0" "$out" --phase "${@:1:n}"
  else
    bash "$0" "$out" --phase "${@:1:n}"
  fi
  shift "$n"
done

table=$(cat "$out"/*.json | jq -rs --argjson files "$(ls "$out"/*.json 2>/dev/null | wc -l)" '
  def stats: ["fires", "compared", "mem_mismatch", "result_mismatch", "callee_saved_mismatch", "own", "skipped", "own_fault", "hook_fault"];
  def bad: [.mem_mismatch, .result_mismatch, .callee_saved_mismatch, .own_fault, .hook_fault, .fires - .compared - .skipped] | add;
  map(select(.hook_shadow)) as $runs
  | ($runs | map(select([.hook_shadow | del(.notes) | .[] | bad] | add > 0))) as $differing
  | "packages \($runs | length) of \($files) result files (\($runs | group_by(.result) | map("\(.[0].result) \(length)") | join(", "))) · load1 \($runs | map(.load1) | min)–\($runs | map(.load1) | max)",
    (("memcpy", "memset", "strcpy", "strlen", "reg_inline_copy", "inline_copy") as $kind
      | [$runs[].hook_shadow[$kind] | select(.)] as $rows
      | "\($kind): packages \($rows | length)" + (stats | map(" · \(.) \([$rows[][.]] | add // 0)") | add)),
    "handed to the title'"'"'s own routine:" + ([$runs[] | select(.hook_shadow | del(.notes) | any(.[]; .own > 0))
      | " \(.package)" + (.hook_shadow | del(.notes) | to_entries | map(select(.value.own > 0) | " \(.key) \(.value.own)") | add)] | join(" ·")),
    ($differing[] | "  \(.package) \(.hook_shadow.notes | join(" | "))"),
    "result files without a shadow report: \($files - ($runs | length))",
    "packages with a mismatch, a fault on either side, or a fire neither compared nor skipped: \($differing | length)"')
echo "$table"
[[ $table == *"report: 0"*": 0" && $table != "packages 0 "* ]]
