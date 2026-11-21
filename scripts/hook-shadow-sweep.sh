#!/usr/bin/env bash
# Shadow-compare the generic KTF hooks (wie-core-arm/data/binary_patches.toml, the hash-less entry)
# against each title's own code, over a set of packages — the table a hook change is judged by.
# What is compared and what is not: wie-core-arm/src/binary_patches/hook/shadow.rs.
# Local only: it needs game files.
#   cargo build --release -p wie_cli --bin wie_validate     (under build-slot; WIE_VALIDATE=<bin> to use another)
#   bash scripts/hook-shadow-sweep.sh <out-dir> <package>…  run what <out-dir> does not hold yet, then the table
#   bash scripts/hook-shadow-sweep.sh <out-dir>              the table only (exit 1 on any mismatch)
# Results are named by sha256 prefix, so <out-dir> and the table carry no title names. Keep <out-dir>
# outside the repo.
# One `build-slot run --long` lease per HOOK_SHADOW_PHASE packages (default 60 ≈ 10 min: 3 at a
# time, 30 s each) — never wrap this script itself in build-slot (CLAUDE.md «측정 스윕»).
# HOOK_SHADOW_TIMEOUT=<s> (default 30) is wie_validate's --timeout; the other arguments are the
# census's probe A. A starved host reaches fewer calls: read `fires` (and load1) before `mismatch`,
# and re-run a package by deleting its result.
set -u
cd "$(dirname "$0")/.."
BIN=${WIE_VALIDATE:-target/release/wie_validate}
SLOT=~/orchestrator-live/bin/build-slot
PHASE=${HOOK_SHADOW_PHASE:-60}
out=${1:?usage: hook-shadow-sweep.sh <out-dir> [package…]}
shift

case ${1:-} in
--one) # one package
  sha=$(shasum -a 256 "$2" | cut -c1-12)
  [[ -s $out/$sha.json ]] && exit 0
  WIE_HOOK_MODE=shadow RUST_LOG=error "$BIN" --inject --keep-timeout --timeout "${HOOK_SHADOW_TIMEOUT:-30}" \
    --max-ticks 100000000000 --pacing 8 --relaunch 1 "$2" 2>/dev/null | tail -1 |
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

table=$(cat "$out"/*.json | jq -rs '
  def stats: ["fires", "compared", "mem_mismatch", "result_mismatch", "callee_saved_mismatch", "own", "skipped", "own_fault"];
  def bad: [.mem_mismatch, .result_mismatch, .callee_saved_mismatch, .own_fault] | add;
  map(select(.hook_shadow)) as $runs
  | ($runs | map(select([.hook_shadow | del(.notes) | .[] | bad] | add > 0))) as $differing
  | "packages \($runs | length) (\($runs | group_by(.result) | map("\(.[0].result) \(length)") | join(", "))) · load1 \($runs | map(.load1) | min)–\($runs | map(.load1) | max)",
    (("memcpy", "memset", "strcpy", "strlen", "reg_inline_copy", "inline_copy") as $kind
      | [$runs[].hook_shadow[$kind] | select(.)] as $rows
      | "\($kind): packages \($rows | length)" + (stats | map(" · \(.) \([$rows[][.]] | add // 0)") | add)),
    ($differing[] | "  \(.package) \(.hook_shadow.notes | join(" | "))"),
    "packages with a mismatch, or a fault in the title'"'"'s own code: \($differing | length)"')
echo "$table"
[[ $table == *": 0" ]]
