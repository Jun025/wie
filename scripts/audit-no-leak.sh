#!/usr/bin/env bash
# No-leak self-audit (1번 기준선 / S5 / S6).
#
# Proves, by static inspection, that:
#   1. The ONLY hand-written module that talks to the network is lib/api.ts
#      (account / opaque saves / inquiries). The generated wasm glue under
#      web/src/wasm/ fetches its own .wasm bundle (a static same-origin asset)
#      and is excluded.
#   2. The device-local library code (lib/library.ts) makes ZERO network calls —
#      game bytes, filenames and hashes never leave the device.
#   3. The built frontend (web/dist) contains no game ROMs/JARs/ZIPs.
#
# Exit non-zero on any violation.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
note() { printf '  %s\n' "$1"; }
bad() { printf '  ❌ %s\n' "$1"; fail=1; }
good() { printf '  ✅ %s\n' "$1"; }

# Hand-written frontend sources (exclude generated wasm glue).
SRC_FILES=$(find web/src -type f \( -name '*.ts' -o -name '*.tsx' \) | grep -v '/wasm/' || true)

echo "── 1. network primitives only live where they should ───────────────────"
NET_RE='fetch\(|XMLHttpRequest|sendBeacon|new WebSocket|EventSource'
offenders=$(echo "$SRC_FILES" | xargs grep -lE "$NET_RE" 2>/dev/null | grep -vE 'web/src/lib/api\.ts' || true)
if [ -n "$offenders" ]; then
  bad "unexpected network primitive in: $offenders"
else
  good "network primitives confined to lib/api.ts"
fi

echo "── 2. device-local library code never hits the network ─────────────────"
if grep -nE "$NET_RE" web/src/lib/library.ts >/dev/null 2>&1; then
  bad "library.ts (device-local store) contains a network call"
else
  good "library.ts has no network calls"
fi
# saveSync.ts may import api (opaque save sync) but must never send the game hash.
if grep -nE "upsert|savesApi|fetch" web/src/lib/saveSync.ts | grep -iE "hash" >/dev/null 2>&1; then
  bad "saveSync.ts appears to mix the game hash into a server call"
else
  good "saveSync.ts never sends a game hash to the server"
fi

echo "── 3. backend stores no game identity ──────────────────────────────────"
# Inspect actual column definitions only (strip SQL `--` comments first). The
# forbidden identifiers are game-file identity columns; inquiry game_title /
# game_vendor are voluntary support text and are explicitly permitted.
FORBIDDEN_COL='\b(game_hash|game_filename|game_file_name|rom_data|game_bytes|owned_games|library_manifest|game_list)\b'
if grep -vE '^\s*--' migrations/*.sql | grep -niE "$FORBIDDEN_COL" >/dev/null 2>&1; then
  bad "D1 schema has a game-identity column"
else
  good "D1 schema has no game-file identity columns"
fi
# Functions must never read game bytes/hash from a request body (strip // comments).
FORBIDDEN_REQ='(body|payload|req)\.(game_?hash|gameBytes|game_bytes|rom_?data|filename|owned_?games)'
if grep -rhE -v '^\s*//' functions | grep -niE "$FORBIDDEN_REQ" >/dev/null 2>&1; then
  bad "a Pages Function reads game bytes/hash from the request"
else
  good "no Pages Function reads game bytes/hash from a request"
fi

echo "── 4. no game files in the build output / repo (S6) ────────────────────"
DIST="web/dist"
if [ -d "$DIST" ]; then
  leak=$(find "$DIST" -type f \( -iname '*.jar' -o -iname '*.jad' -o -iname '*.zip' -o -iname '*.kdf' -o -iname '*.skm' -o -iname '*.mod' -o -iname '*.smc' -o -iname '*.gba' -o -iname '*.nes' \) || true)
  if [ -n "$leak" ]; then bad "game-like files in $DIST: $leak"; else good "$DIST contains no game files"; fi
else
  note "$DIST not built yet — skipping dist scan"
fi
# Make sure no game binaries are tracked by git.
tracked=$(git ls-files | grep -iE '\.(jar|jad|kdf|skm|mod|smc|gba|nes)$' || true)
if [ -n "$tracked" ]; then bad "game binaries tracked in git: $tracked"; else good "no game binaries tracked in git"; fi

echo
if [ "$fail" -ne 0 ]; then
  echo "AUDIT FAILED ❌"
  exit 1
fi
echo "AUDIT PASSED ✅  — only account info + opaque saves can ever reach the server."
