#!/usr/bin/env bash
# LGT-AOT §7 render self-verification harness (BattleMonster et al.).
#
# Purpose: a reproducible, structured measurement of how far an LGT-AOT (ez-i) title
# gets toward rendering — so that the moment a 0x64 ordinal→native mapping is supplied
# (see docs/lgt_abi.md cp59 §resolver), one can drop it in and re-run this to see, in
# minutes, whether `field[0x74]` advances and draws appear.
#
# It does NOT bundle any game file. Point it at a local zip via env/arg:
#   GAME_ZIP=/path/to/배틀몬스터.zip scripts/lgt_render_probe.sh
#   scripts/lgt_render_probe.sh /path/to/title.zip [--shot out.png]
#
# Cache JSON is never trusted — this always runs live (the cp47 "stale report" lesson).
# Note: no `set -u` — empty-array expansion (`"${shot_args[@]}"`) trips nounset on bash 3.2.
set -eo pipefail
cd "$(dirname "$0")/.."

GAME_ZIP="${GAME_ZIP:-${1:-}}"
SHOT=""
[ "${2:-}" = "--shot" ] && SHOT="${3:-}"
if [ -z "$GAME_ZIP" ] || [ ! -f "$GAME_ZIP" ]; then
  echo "usage: GAME_ZIP=/path/to/title.zip $0   (or: $0 /path/to/title.zip [--shot out.png])" >&2
  exit 2
fi

BIN=target/debug/wie_validate
[ -x "$BIN" ] || cargo build -p wie_cli --bin wie_validate

BOOT="${BOOT_SECS:-6}"; TIMEOUT="${TIMEOUT:-12}"
# Explicit temp files (no EXIT trap — it would fire inside the backgrounded subshell).
WORK="$(mktemp -d)"; LOG="$WORK/log"; JSON="$WORK/json"

shot_args=(); [ -n "$SHOT" ] && shot_args=(--screenshot "$SHOT")

# Live run under a kill-watchdog (macOS has no `timeout`; a single tick can spin — cp45).
RUST_LOG="${RUST_LOG:-wie_lgt=debug,wie_midp=debug,wie_wipi_c=info}" \
  "$BIN" "$GAME_ZIP" --boot-secs "$BOOT" --timeout "$TIMEOUT" "${shot_args[@]}" >"$JSON" 2>"$LOG" &
pid=$!
i=0; while kill -0 "$pid" 2>/dev/null; do i=$((i+1)); [ "$i" -ge $((TIMEOUT*3+20)) ] && { kill -9 "$pid" 2>/dev/null; break; }; sleep 1; done

clean() { sed 's/\x1b\[[0-9;]*m//g' "$LOG"; }
# The key match is already exact: the leading `"` anchors the name, so `last_frame_content`
# cannot satisfy `"content":` (measured 2026-09-08 — five prefix pairs, all read correctly).
# What was unsafe is the `tail -1` that used to end this line: a key appearing twice (nested
# objects, or two JSON documents in one file) was resolved by POSITION, silently. Now the count
# is checked, so an ambiguous read says so on stderr instead of guessing. Deliberately still
# grep+cut, not jq: `scripts/smoke_gate.sh:76` reads JSON the same way and one idiom beats two.
field() {
  local m n
  m=$(grep -o "\"$1\":[0-9a-z.]*" "$JSON" || true)
  n=$(printf '%s' "$m" | grep -c . || true)
  [ "$n" = 1 ] || echo "field($1): 매치 ${n}건(기대 1) — JSON 에 같은 키가 여럿이거나 없다" >&2
  printf '%s' "$m" | head -1 | cut -d: -f2
}
cnt()   { clean | grep -oE "$1" | wc -l | tr -d ' '; }

# ── structured metrics (the §7 advance chain) ────────────────────────────────
echo "title=$(basename "$GAME_ZIP")"
echo "result=$(grep -o '"result":"[^"]*"' "$JSON" | tail -1 | cut -d'"' -f4)"
echo "paints=$(field paints)  content=$(field content)  distinct_colors=$(field distinct_colors)"
echo "getNextEvent=$(cnt 'EventQueue::getNextEvent')   # >1 => game loop ticking (TIMER live)"
echo "graphics_reset=$(cnt 'Graphics::reset')          # per-frame paint() entries"
echo "createImage=$(cnt 'createImage')  getResource=$(cnt 'MC_knlGetResource|GetResourceID')"
echo "draw_fillRect=$(cnt 'MC_grpFillRect|Graphics::fillRect')  draw_drawImage=$(cnt 'MC_grpDrawImage|Graphics::drawImage')"
echo "noop_imports=$(cnt 'import 0x(e|10|12|1f|22)\(') # render-PENDING 0x64 imports hit (cp58)"
# advance oracle: a non-blank backbuffer with real blits is the cp50/55 success signal.
if [ "$(field content)" = "true" ] && [ "$(cnt 'MC_grpDrawImage|Graphics::drawImage')" -gt 0 ]; then
  echo "ADVANCE=YES (drawImage on non-blank backbuffer — compare to title oracle)"
else
  echo "ADVANCE=NO  (field[0x74] gated; render-PENDING 0x64 imports unresolved — see docs/lgt_abi.md cp59)"
fi

rm -rf "$WORK"
