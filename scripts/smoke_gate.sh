#!/usr/bin/env bash
# scripts/smoke_gate.sh — LOCAL regression smoke gate for the working game catalog.
#
# The GATE VERDICT judges boot + render only (a title boots without crash/hang and
# paints real, non-blank content), NOT visual correctness. This is the deterministic
# part of wie_validate's bar. It bundles NO game files: point it at a local corpus via
# env. The committed baseline (scripts/smoke_gate_baseline.tsv) lists title identifiers
# and their expected status only — no paths, no binaries, no game bytes.
#
# Why boot+render and not input-survival: the validator's scripted input injection fires
# keys at wall-clock-scheduled times, but emulation speed varies per run (machine load),
# so each key lands at a different emulation tick. A tail of titles (~5% measured) crash
# nondeterministically on a particular input — a clean boot+render title FAILs ~1-in-N
# runs purely on input timing. That makes input-survival unsuitable as a hard gate. The
# boot+render verdict is reproducible (verified: titles that flake under --inject pass
# boot+render across repeated runs). Set INJECT=1 to ALSO run the input sequence as a
# non-gating advisory (printed, never affects the exit code).
#
# Usage:
#   WORKING_DIR=game_lab/working scripts/smoke_gate.sh                  # all platforms
#   WORKING_DIR=game_lab/working PLATFORM_FILTER=ktf scripts/smoke_gate.sh
#   WORKING_DIR=game_lab/working UPDATE_BASELINE=1 scripts/smoke_gate.sh # (re)write baseline
#   WORKING_DIR=game_lab/working INJECT=1 scripts/smoke_gate.sh          # +advisory input run
#
# "Regression" = a title the baseline records as PASS that now FAILs. The gate prints the
# offenders and exits 1. Titles in the baseline but absent from the local corpus are
# skipped (reported), not treated as regressions — but if more than half of the in-scope
# baseline is absent, or none of it was checked, the gate prints UNMEASURED and exits 2
# instead of OK. Titles are compared after Unicode NFC on both sides (see nfc());
# scripts/test-smoke-gate.sh checks that. A FAIL is retried (RETRY, default 2) to absorb
# transient load flake near the render deadline; only a title that FAILs every attempt counts.
# The committed baseline is the stable core: titles that boot+render in two
# independent full runs. Cache JSON is never trusted; every run is live.
#
# macOS has no `timeout`, and a single ARM tick can spin, so a background `kill -9`
# watchdog bounds each run. No `set -u`: bash 3.2 (macOS default) has no associative
# arrays and trips nounset on empty-array expansion — this script uses temp files instead.
set -eo pipefail
cd "$(dirname "$0")/.."

WORKING_DIR="${WORKING_DIR:-game_lab/working}"
PLATFORM_FILTER="${PLATFORM_FILTER:-all}"
BASELINE="${BASELINE:-scripts/smoke_gate_baseline.tsv}"
TIMEOUT="${TIMEOUT:-15}"
KILL="${KILL:-50}"
RETRY="${RETRY:-2}"
UPDATE_BASELINE="${UPDATE_BASELINE:-0}"
INJECT="${INJECT:-0}"   # 1 = also run the scripted input sequence as a non-gating advisory

BIN="${BIN:-target/debug/wie_validate}"   # override only for scripts/test-smoke-gate.sh's fake validator
[ -x "$BIN" ] || cargo build -p wie_cli --bin wie_validate >&2

if [ ! -d "$WORKING_DIR" ]; then
  echo "smoke_gate: WORKING_DIR not found: $WORKING_DIR" >&2
  echo "  set WORKING_DIR to your local working/ catalog (e.g. game_lab/working)" >&2
  exit 2
fi

case "$PLATFORM_FILTER" in
  all) PLATS="ktf lgt skt j2me" ;;
  *)   PLATS="$PLATFORM_FILTER" ;;
esac

# Unicode NFC on stdin → stdout. APFS hands back filenames in NFD while the committed baseline
# is (mostly) NFC, so a byte compare matched 0-1 of 292 titles and the gate printed
# "0 regressions / OK" having measured nothing (2026-09-23, 2026-09-29). Every title on
# both sides of the compare goes through here — the one place that knows about it.
nfc() { perl -CSD -MUnicode::Normalize -pe '$_ = NFC($_)'; }

# One live run under the kill-watchdog → echoes the validator's PASS/FAIL. With no extra
# args this is the deterministic boot+render verdict; "$@" lets the advisory pass --inject.
run_one() {
  local f="$1"; shift
  local json pid i res m n
  json="$(mktemp)"
  "$BIN" "$f" --timeout "$TIMEOUT" "$@" >"$json" 2>/dev/null &
  pid=$!
  i=0
  while kill -0 "$pid" 2>/dev/null; do
    i=$((i + 1))
    [ "$i" -ge "$KILL" ] && { kill -9 "$pid" 2>/dev/null; break; }
    sleep 1
  done
  wait "$pid" 2>/dev/null || true
  # `tail -1` used to end this line: a repeated "result" key was resolved by POSITION, silently.
  # That is the same hazard `scripts/lgt_render_probe.sh` field() fixed (2026-09-08) and it bites
  # harder here — line below turns this read into the PASS/FAIL verdict, so a later occurrence
  # winning could report PASS for a run that failed. Count first, say so, then take the first.
  m="$(grep -o '"result":"[^"]*"' "$json" || true)"
  n="$(printf '%s' "$m" | grep -c . || true)"
  [ "$n" = 1 ] || echo "   ! $(basename "$f"): \"result\" 매치 ${n}건(기대 1) — 이 판정을 신뢰하지 마라" >&2
  res="$(printf '%s' "$m" | head -1 | cut -d'"' -f4)"
  rm -f "$json"
  [ "$res" = "PASS" ] && echo PASS || echo FAIL
}

RESULTS="$(mktemp)"   # lines: "<platform>/<title>\t<PASS|FAIL>"
trap 'rm -f "$RESULTS" "$RESULTS.nfc" "$RESULTS.base"' EXIT
pass=0; fail=0; total=0
echo ">> smoke_gate: WORKING_DIR=$WORKING_DIR PLATFORM_FILTER=$PLATFORM_FILTER (live, no cache)" >&2
for p in $PLATS; do
  d="$WORKING_DIR/$p"
  [ -d "$d" ] || continue
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    title="$p/$(basename "$f")"
    r="$(run_one "$f")"
    if [ "$r" = FAIL ] && [ "$RETRY" -gt 0 ]; then
      n=0
      while [ "$n" -lt "$RETRY" ] && [ "$r" = FAIL ]; do
        r="$(run_one "$f")"
        n=$((n + 1))
      done
    fi
    printf '%s\t%s\n' "$title" "$r" >> "$RESULTS"
    total=$((total + 1))
    if [ "$r" = PASS ]; then pass=$((pass + 1)); else fail=$((fail + 1)); fi
    # Optional non-gating advisory: does the title also survive the scripted input
    # sequence? Timing-nondeterministic, so reported only — never affects the verdict.
    adv=""
    if [ "$INJECT" = "1" ]; then
      ai="$(run_one "$f" --inject)"
      adv="  input-advisory=$ai"
    fi
    printf '  %-6s %s%s\n' "$r" "$title" "$adv" >&2
  done < <(find "$d" -maxdepth 1 -name '*.zip' | sort)
done

# Standalone statements so `set -e` stops the gate if nfc() fails (an `&&` list or a `< <(...)`
# would hide it and the compare would run on nothing).
nfc < "$RESULTS" > "$RESULTS.nfc"
mv "$RESULTS.nfc" "$RESULTS"

echo >&2
echo "== ran $total titles: $pass PASS / $fail FAIL ==" >&2

# Update mode: persist the current PASS set as the new baseline and stop.
if [ "$UPDATE_BASELINE" = "1" ]; then
  {
    echo "# smoke_gate baseline — titles expected to boot + render real content."
    echo "# Identifiers + expected status only; NO game files. Regenerate: UPDATE_BASELINE=1."
    awk -F'\t' '$2=="PASS"{print $1"\tPASS"}' "$RESULTS" | sort
  } > "$BASELINE"
  echo ">> wrote baseline: $BASELINE ($(grep -c '	PASS' "$BASELINE") expected-PASS titles)" >&2
  exit 0
fi

# Compare against the committed baseline.
if [ ! -f "$BASELINE" ]; then
  echo "smoke_gate: no baseline at $BASELINE (run once with UPDATE_BASELINE=1)" >&2
  exit 2
fi

nfc < "$BASELINE" > "$RESULTS.base"
regressions=0; missing=0; checked=0
while IFS=$'\t' read -r title status; do
  case "$title" in \#*|"") continue ;; esac
  [ "$status" = "PASS" ] || continue
  # A PLATFORM_FILTER run does not look at other platforms' titles: they are out of scope,
  # not absent, and must not count toward the UNMEASURED ratio below.
  case " $PLATS " in *" ${title%%/*} "*) ;; *) continue ;; esac
  actual="$(awk -F'\t' -v t="$title" '$1==t{print $2; exit}' "$RESULTS")"
  if [ -z "$actual" ]; then
    missing=$((missing + 1))
    echo "  SKIP (absent from corpus): $title" >&2
    continue
  fi
  checked=$((checked + 1))
  if [ "$actual" = "FAIL" ]; then
    regressions=$((regressions + 1))
    echo "  REGRESSION (baseline PASS -> now FAIL): $title"
  fi
done < "$RESULTS.base"

echo
echo "== smoke_gate: checked $checked baseline titles, $missing absent, $regressions regressions =="
if [ "$regressions" -gt 0 ]; then
  echo "FAIL: $regressions regression(s) detected."
  exit 1
fi
# More than half the in-scope baseline absent = the corpus and the baseline are not talking
# about the same titles (a name-form mismatch, a wrong WORKING_DIR). "OK" would be a claim
# about titles nobody ran, so refuse it — this is how the gate went vacuous unnoticed.
# checked 0 is the same claim with a zero denominator (a PLATFORM_FILTER with no baseline titles).
if [ "$checked" -eq 0 ] || [ $((missing * 2)) -gt $((checked + missing)) ]; then
  echo "UNMEASURED: $missing of $((checked + missing)) baseline titles absent from $WORKING_DIR — regressions not measured."
  exit 2
fi
echo "OK: no regressions vs baseline."
exit 0
