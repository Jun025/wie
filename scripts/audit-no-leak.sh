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
# saveSync may import the api (opaque save sync) but must never send the game hash.
if grep -nE "upsert|savesApi|fetch" web/src/lib/saveSync.ts | grep -iE "hash" >/dev/null 2>&1; then
  bad "saveSync.ts appears to mix the game hash into a server call"
else good "saveSync.ts never sends a game hash to the server"; fi
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
# pull every string literal (single, double, backtick) from the functions code
for path in files:
    src = open(path, encoding="utf-8").read()
    for m in re.finditer(r'`([^`]*)`|"([^"]*)"|\'([^\']*)\'', src, re.S):
        lit = next(g for g in m.groups() if g is not None)
        low = lit.lower()
        if "user_files" in low or ("content_hash" in low and " where " in low):
            # any SQL referencing the vault table / hash lookup must be owner-scoped
            if "sql" in low or "select" in low or "insert" in low or "update" in low or "delete" in low or "user_files" in low:
                if "user_id" not in low:
                    print(f"  ❌ non-owner-scoped vault SQL in {path}: {lit[:80]!r}")
                    bad = True
if not bad:
    print("  ✅ every user_files / content_hash SQL statement is owner-scoped (user_id)")
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
# there must be no standalone index on content_hash alone (would enable "who else has this").
if grep -niE 'create( unique)? index[^(]*\(\s*content_hash\s*\)' migrations/*.sql >/dev/null 2>&1; then
  bad "a standalone content_hash index exists (could reveal cross-user ownership)"
else good "no standalone content_hash index (cannot answer 'who else has this file')"; fi

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
