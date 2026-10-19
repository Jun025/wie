#!/usr/bin/env bash
# Drive one census phase in leases of ~BUDGET s: `playability-census.mjs run --budget` per
# `build-slot run --long` call, repeated while it exits 3 (more remain). Local only, like the census.
#   bash scripts/census-drive.sh <run args…>     e.g. --bin X --out Y --only progress --titles Z <corpus>
# One lease for the whole sweep held the long pool 5–7.5 h (2026-10-05~08) — never wrap this script
# itself in build-slot. No host-load polling here: waiting for the lease is the throttle.
# Keep drivers here; do not copy them into scratch (CLAUDE.md «측정 스윕»).
# CENSUS_DEADLINE=<epoch s>: no call starts past it and none is budgeted past it (exit 3 = more remain).
set -u
cd "$(dirname "$0")/.."
BUDGET=${CENSUS_BUDGET:-900} # + the longest title in flight: a 600 s progress run ends ≤ ~1700 s
SLOT=~/orchestrator-live/bin/build-slot
for ((call = 1; call <= ${CENSUS_MAX_CALLS:-200}; call++)); do
  if [[ -n ${CENSUS_DEADLINE:-} ]]; then
    left=$((CENSUS_DEADLINE - $(date +%s)))
    ((left > 0)) || { echo "census-drive: CENSUS_DEADLINE passed — stopping" >&2; exit 3; }
    ((left < BUDGET)) && BUDGET=$left
  fi
  if [[ -x $SLOT ]]; then
    "$SLOT" run --long -- node scripts/playability-census.mjs run --budget "$BUDGET" "$@"
  else
    node scripts/playability-census.mjs run --budget "$BUDGET" "$@"
  fi
  rc=$?
  echo "census-drive: call $call rc=$rc $(date +%T)" >&2
  [[ $rc == 3 ]] || exit $rc
done
echo "census-drive: ${CENSUS_MAX_CALLS:-200} calls and still more — stopping" >&2
exit 3
