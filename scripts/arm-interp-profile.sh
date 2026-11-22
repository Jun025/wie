#!/usr/bin/env bash
# Sample-profile the ARM interpreter on one scene: run a native `wie_validate` along a key script, record
# `xctrace` Time Profiler for a window, and print where the run loop's time went (arm-interp-profile.mjs).
# Local only (it needs a game file — Constraint 9) and macOS only (xctrace). One run, ~2 minutes, and ONE AT A
# TIME on the machine: a second Time Profiler session fails on the kperf lock («could not lock kperf»).
#   ~/orchestrator-live/bin/build-slot run --long -- bash scripts/arm-interp-profile.sh \
#       <wie_validate> <game> <keys file|list> <at secs> <out dir> [window secs = 8]
# <wie_validate> must carry line tables, or every sample lands in one frame:
#   CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release -p wie_cli --bin wie_validate
# and a `.dSYM` must sit beside it: `dsymutil <wie_validate> -o <wie_validate>.dSYM`, run before anything
# else builds in that target dir (macOS keeps the debug info in the object files, and the next build overwrites them).
# <at> counts from process start; --boot-secs is 12, so key time t is process time t + 12.
# <out dir>/shots holds a frame every 10 s and at every key step — look at them: a profile of the wrong
# screen is still a profile. They are game frames; keep <out dir> outside the repo.
# RUST_LOG is passed through (default warn) and stderr goes to <out dir>/run.err, <out dir>/t0 holds the start
# epoch: with a title's frame marker in the log, samples ÷ frames in the window is CPU per frame.
# Read shares, not sample counts: the loop's share of all samples moves with host load and with how
# much of the scene the runner spends waiting on timers; the split inside the loop does not.
set -u
[[ $# -ge 5 ]] || { sed -n '2,18p' "$0" >&2; exit 2; }
BIN=$1 GAME=$2 KEYS=$3 AT=$4 OUT=$5 WIN=${6:-8}
cd "$(dirname "$0")/.." && mkdir -p "$OUT/shots" && rm -rf "$OUT/x.trace" # wie_validate does not create --shotdir
date +%s >"$OUT/t0"
RUST_LOG=${RUST_LOG:-warn} RUST_MIN_STACK=4194304 "$BIN" --inject --boot-secs 12 --max-ticks 100000000000 --keys "$KEYS" --keep-timeout --timeout $((AT + WIN + 900)) --shotdir "$OUT/shots" --shot-every 10 "$GAME" >"$OUT/run.json" 2>"$OUT/run.err" &
PID=$!
trap 'kill $PID 2>/dev/null' EXIT
perl -e "sleep $AT"
kill -0 $PID 2>/dev/null || { echo "arm-interp-profile: the run ended before ${AT}s — see $OUT/run.json" >&2; exit 1; }
# The run must outlive the recording (hence --keep-timeout above): xctrace can take tens of seconds to attach on a
# busy host, and it hangs while saving if its target has exited. The watchdog is for that and for nothing else.
xcrun xctrace record --template 'Time Profiler' --attach $PID --time-limit "${WIN}s" --output "$OUT/x.trace" >"$OUT/xctrace.log" 2>&1 &
XC=$!
for ((i = 0; i < WIN + 240; i++)); do kill -0 $XC 2>/dev/null || break; perl -e 'sleep 1'; done
if kill -0 $XC 2>/dev/null; then kill -9 $XC; echo "arm-interp-profile: xctrace did not finish within $((WIN + 240))s — killed; see $OUT/xctrace.log" >&2; exit 1; fi
wait $XC || { echo "arm-interp-profile: xctrace record failed — see $OUT/xctrace.log" >&2; exit 1; }
kill $PID 2>/dev/null; wait $PID 2>/dev/null; trap - EXIT
xcrun xctrace export --input "$OUT/x.trace" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' >"$OUT/time-profile.xml" 2>>"$OUT/xctrace.log" || { echo "arm-interp-profile: xctrace export failed — see $OUT/xctrace.log" >&2; exit 1; }
echo "load1=$(sysctl -n vm.loadavg | awk '{print $2}')"
node scripts/arm-interp-profile.mjs "$OUT/time-profile.xml"
