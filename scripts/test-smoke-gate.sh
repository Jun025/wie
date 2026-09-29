#!/usr/bin/env bash
# scripts/test-smoke-gate.sh — self-check for smoke_gate.sh's title matching. Local only, no game
# bytes: a fake validator stands in for wie_validate (PASS unless the filename contains FAIL).
# Cases: NFD filenames vs NFC baseline match N/N · a planted regression goes red · too many
# absent titles is UNMEASURED · the gate with NFC normalization removed does NOT say OK.
set -eo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
T="$(mktemp -d)"; trap 'rm -rf "${T:?}"' EXIT

cat > "$T/fake" <<'F'
#!/bin/sh
case "$1" in *FAIL*) echo '{"result":"FAIL"}' ;; *) echo '{"result":"PASS"}' ;; esac
F
chmod +x "$T/fake"
nfd() { perl -CSDA -MUnicode::Normalize -e 'print NFD($ARGV[0])' "$1"; }

# corpus <name> <title...>: NFD filenames under $T/<name>/ktf, as APFS hands them back
corpus() { local d="$T/$1/ktf"; shift; mkdir -p "$d"; for t in "$@"; do : > "$d/$(nfd "$t").zip"; done; }
# baseline <name> <title...>: NFC lines, as committed
baseline() { local b="$T/$1.tsv"; shift; : > "$b"; for t in "$@"; do printf 'ktf/%s.zip\tPASS\n' "$t" >> "$b"; done; }
gate() { # gate <script> <name> → summary + verdict lines, then rc=<n>
  local o rc=0
  o="$(BIN="$T/fake" WORKING_DIR="$T/$2" BASELINE="$T/$2.tsv" PLATFORM_FILTER=ktf RETRY=0 KILL=5 \
    bash "$1" 2>/dev/null)" || rc=$?
  printf '%s\n' "$o" | grep -E '^(== smoke_gate|OK|FAIL|UNMEASURED)' || true
  echo "rc=$rc"
}
bad=0
expect() { # expect <label> <want-substring> <output>
  case "$3" in *"$2"*) echo "ok   $1" ;; *) echo "BAD  $1 — want «$2», got:"; echo "$3"; bad=1 ;; esac
}

corpus match 가나다라 한글제목 바람돌이2; baseline match 가나다라 한글제목 바람돌이2
out="$(gate "$here/smoke_gate.sh" match)"; echo "$out"
expect "NFD corpus vs NFC baseline" "checked 3 baseline titles, 0 absent, 0 regressions" "$out"
expect "  → OK rc=0" "rc=0" "$out"

corpus regress 가나다라 한글제목FAIL 바람돌이2; baseline regress 가나다라 한글제목FAIL 바람돌이2
out="$(gate "$here/smoke_gate.sh" regress)"; echo "$out"
expect "planted regression" "1 regressions" "$out"
expect "  → red rc=1" "rc=1" "$out"

corpus absent 가나다라; baseline absent 가나다라 한글제목 바람돌이2
out="$(gate "$here/smoke_gate.sh" absent)"; echo "$out"
expect "absent 2/3 → UNMEASURED" "UNMEASURED" "$out"
expect "  → rc=2" "rc=2" "$out"

# 개악: the same gate with nfc() made a no-op must not report OK on the NFD/NFC case.
mkdir -p "$T/worse/scripts"
sed 's/^nfc() {.*/nfc() { cat; }/' "$here/smoke_gate.sh" > "$T/worse/scripts/smoke_gate.sh"
grep -q '^nfc() { cat; }' "$T/worse/scripts/smoke_gate.sh" || { echo "BAD  mutation did not apply"; exit 1; }
out="$(gate "$T/worse/scripts/smoke_gate.sh" match)"; echo "$out"
expect "mutant (no NFC) is not OK" "rc=2" "$out"

[ "$bad" = 0 ] && echo "ALL OK" || { echo "FAILED"; exit 1; }
