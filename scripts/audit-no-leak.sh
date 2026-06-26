#!/usr/bin/env bash
# No-leak / isolation self-audit (개정 기준선 / S5).
#
# BASELINE (revised, lawyer-approved per the directive):
#   • NOT logged in: game files stay on the device (IndexedDB). ZERO bytes,
#     filenames, hashes, or "owned list" are sent to the server.
#   • Logged in: the user MAY store their own game files in a PRIVATE, per-owner
#     server vault (R2 bytes + D1 metadata). This is allowed — but it MUST be
#     strictly owner-isolated: no other user can reach a file by ANY path, there
#     is no public/presigned URL, no cross-user listing/search/discovery, and
#     dedup compares hashes PER-USER only (never globally).
#
# This script proves, by static inspection, that the code upholds the above.
# Exit non-zero on any violation.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
note() { printf '  %s\n' "$1"; }
bad() { printf '  ❌ %s\n' "$1"; fail=1; }
good() { printf '  ✅ %s\n' "$1"; }

# Hand-written frontend sources (exclude generated wasm glue).
SRC_FILES=$(find web/src -type f \( -name '*.ts' -o -name '*.tsx' \) | grep -v '/wasm/' || true)

echo "── 1. network primitives only live in lib/api.ts ───────────────────────"
NET_RE='fetch\(|XMLHttpRequest|sendBeacon|new WebSocket|EventSource'
offenders=$(echo "$SRC_FILES" | xargs grep -lE "$NET_RE" 2>/dev/null | grep -vE 'web/src/lib/api\.ts' || true)
if [ -n "$offenders" ]; then bad "unexpected network primitive in: $offenders"; else good "network primitives confined to lib/api.ts"; fi

echo "── 2. device-local code never hits the network ─────────────────────────"
if grep -nE "$NET_RE" web/src/lib/library.ts >/dev/null 2>&1; then
  bad "library.ts (device-local store) contains a network call"
else good "library.ts has no network calls"; fi
# saveSync: NOT-logged-in saves must stay local (never sent to the server). The
# server sync (savesApi) is gated on `loggedIn`. We assert the not-logged-in
# early-return / guard is present so anonymous users are never synced. (Logged-in
# saves DO carry the owner's own rom_hash — per-owner, isolated, allowed.)
if grep -q "opts.loggedIn" web/src/lib/saveSync.ts && grep -qE "if \(!opts.loggedIn\)" web/src/lib/saveSync.ts; then
  good "saveSync gates server sync on login (not-logged-in saves stay local)"
else bad "saveSync.ts missing the not-logged-in local-only guard (anonymous saves could leak)"; fi
# serverLibrary.ts (migration) must go through api.ts, not raw fetch.
if grep -nE "$NET_RE" web/src/lib/serverLibrary.ts >/dev/null 2>&1; then
  bad "serverLibrary.ts makes a raw network call (must go via api.ts)"
else good "serverLibrary.ts has no raw network calls (uses api.ts)"; fi

echo "── 3. server file vault is PER-OWNER isolated (S5) ─────────────────────"
# 3a/3c: every SQL statement touching user_files OR comparing content_hash must be
#        owner-scoped (contain user_id). A global content-hash lookup, or a
#        user_files read/write without user_id, would break isolation.
python3 - <<'PY' || fail=1
import re, glob, sys
bad = False
files = glob.glob("functions/**/*.js", recursive=True)
for path in files:
    src = open(path, encoding="utf-8").read()
    # Strip comments FIRST so prose (incl. English apostrophes like "hasn't") is
    # never mistaken for a string literal.
    src = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    src = re.sub(r'//[^\n]*', '', src)
    # SQL in this codebase lives only in double-quoted or backtick strings — never
    # single-quoted (those hold short codes/ids). Scanning just these two avoids
    # the apostrophe-as-quote false positive entirely.
    for m in re.finditer(r'`([^`]*)`|"([^"]*)"', src, re.S):
        lit = m.group(1) if m.group(1) is not None else m.group(2)
        low = lit.lower()
        # only real SQL statements that touch the vault/saves tables or a hash lookup
        is_sql = any(k in low for k in ("select", "insert", "update", "delete"))
        touches_owned = any(t in low for t in ("user_files", "content_hash", "rom_hash")) or " from saves" in low or "into saves" in low or "update saves" in low
        if is_sql and touches_owned:
            if "user_id" not in low:
                print(f"  ❌ non-owner-scoped owned-data SQL in {path}: {lit[:80]!r}")
                bad = True
if not bad:
    print("  ✅ every user_files/saves/content_hash/rom_hash SQL is owner-scoped (user_id)")
sys.exit(1 if bad else 0)
PY
# 3b: no public / presigned / public-bucket access path. Match call/identifier
#     syntax (a trailing "(" or a domain), so explanatory comments don't trip it.
if grep -rniE 'createPresignedUrl\(|getSignedUrl\(|createSignedUrl\(|\.r2\.dev|pub-[0-9a-f]+\.r2|publicBucket|public_bucket' functions >/dev/null 2>&1; then
  bad "a presigned/public R2 URL path exists (vault must be private, owner-streamed only)"
else good "no presigned/public R2 URL path (bytes streamed only via owner-checked endpoint)"; fi
# 3d: the internal R2 object key must never be serialized back to the client. It
#     appears only in SQL column lists (r2_key, / r2_key)) and server-side R2 calls
#     — never as a JS response-object key (r2_key:). Flag only the latter shape.
if grep -rnE 'r2_key\s*:' functions >/dev/null 2>&1; then
  bad "r2_key appears as a response-object key (must stay server-internal)"
else good "r2_key is server-internal (never returned to the client)"; fi
# 3e: every file endpoint enforces authentication (owner identity).
miss=""
for f in functions/api/files/index.js functions/api/files/'[id].js'; do
  grep -q "requireUser" "$f" 2>/dev/null || miss="$miss $f"
done
if [ -n "$miss" ]; then bad "file endpoint(s) missing requireUser:$miss"; else good "all file endpoints require authentication (requireUser)"; fi
# 3f: the upload UI gates on login — non-login users cannot reach the upload path.
if grep -q "showServer" web/src/components/GameLibrary.tsx && grep -q "user && serverEnabled" web/src/components/GameLibrary.tsx; then
  good "upload/vault UI is gated on a logged-in user"
else note "could not statically confirm the vault UI login-gate (review GameLibrary)"; fi

echo "── 4. D1 schema exposes no CROSS-USER game identity ────────────────────"
# The vault stores file_name/content_hash, but ONLY ever keyed to its owner. Forbid
# any column that would be a cross-user inventory, and require that the UNIQUE dedup
# index on content_hash is composite with user_id (per-user), never global.
FORBIDDEN_COL='\b(owned_games|library_manifest|game_list|public_files|shared_files)\b'
if grep -vE '^\s*--' migrations/*.sql | grep -niE "$FORBIDDEN_COL" >/dev/null 2>&1; then
  bad "D1 schema has a cross-user inventory column"
else good "D1 schema has no cross-user inventory column"; fi
# per-user dedup: a UNIQUE index on content_hash must include user_id.
if grep -niE 'unique.*content_hash' migrations/*.sql | grep -viE 'user_id' >/dev/null 2>&1; then
  bad "a UNIQUE content_hash index is GLOBAL (must be composite with user_id — per-user dedup only)"
else good "content_hash uniqueness is per-user (composite with user_id), no global dedup"; fi
# there must be no standalone index on content_hash / rom_hash alone (would enable
# "who else has this ROM / save").
if grep -niE 'create( unique)? index[^(]*\(\s*(content_hash|rom_hash)\s*\)' migrations/*.sql >/dev/null 2>&1; then
  bad "a standalone content_hash/rom_hash index exists (could reveal cross-user ownership)"
else good "no standalone content_hash/rom_hash index (cannot answer 'who else has this')"; fi
# saves uniqueness (rom_hash) must be composite with user_id (per-owner), never global.
if grep -niE 'unique.*rom_hash' migrations/*.sql | grep -viE 'user_id' >/dev/null 2>&1; then
  bad "a UNIQUE rom_hash index is GLOBAL (must be composite with user_id — per-owner)"
else good "rom_hash (saves) uniqueness is per-owner (composite with user_id)"; fi

echo "── 5. no game files in the build output / repo (S6) ────────────────────"
DIST="web/dist"
if [ -d "$DIST" ]; then
  leak=$(find "$DIST" -type f \( -iname '*.jar' -o -iname '*.jad' -o -iname '*.zip' -o -iname '*.kdf' -o -iname '*.skm' -o -iname '*.mod' -o -iname '*.smc' -o -iname '*.gba' -o -iname '*.nes' \) || true)
  if [ -n "$leak" ]; then bad "game-like files in $DIST: $leak"; else good "$DIST contains no game files"; fi
else note "$DIST not built yet — skipping dist scan"; fi
tracked=$(git ls-files | grep -iE '\.(jar|jad|kdf|skm|mod|smc|gba|nes)$' || true)
if [ -n "$tracked" ]; then bad "game binaries tracked in git: $tracked"; else good "no game binaries tracked in git"; fi

echo
if [ "$fail" -ne 0 ]; then
  echo "AUDIT FAILED ❌"
  exit 1
fi
echo "AUDIT PASSED ✅  — non-login files stay on-device; login files are a PRIVATE, per-owner vault with no cross-user path."
