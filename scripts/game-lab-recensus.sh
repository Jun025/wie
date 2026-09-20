#!/usr/bin/env bash
# Re-census an ALREADY-CLASSIFIED `game_lab/` corpus, leaving the per-game rows behind.
#
# ── Why this exists ──────────────────────────────────────────────────────────
# The 2026-09-18 census re-ran all 187 games in `game_lab/broken/` (45 minutes of
# this machine, which doubles as the self-hosted CI runner — 5 open PRs waited on
# it), published the bucket TOTALS, and wrote its per-game rows to a `mktemp -d`
# scratch that it then deleted. Deleting the scratch was correct discipline; the
# problem is that the scratch was the only place the per-game data ever lived. So
# "which games are the 32 `no frame rendered` ones" became unanswerable, the
# low-load re-measure gate 2 had asked for became structurally impossible, and the
# next round paid an hour to rebuild a mapping out of July's leftovers
# (`scripts/game-lab-census-map.mjs`).
#
# The root cause is not "the mapping was missing" — it is that THE RE-RUN PATH
# LEAVES NOTHING BEHIND. This script is that path, and its output is durable by
# construction: one JSON per game, written as the run proceeds, into a dated
# directory beside the corpus. `game-lab-census-map.mjs --reports <that dir>`
# then produces today's mapping with no re-run at all.
#
# ── Why a NEW dated directory, and not `game_lab/reports/` ───────────────────
# `game_lab/reports/` is the July baseline (mtimes 2026-06-25 .. 2026-07-01) and
# it is the input the census reconstructed its 7/7 totals from. Overwriting it
# would destroy the only column there is to compare against — the proposal that
# asked for this script named that cost itself. So the default output is
# `game_lab/reports-YYYY-MM-DD/`, the two sit side by side, and comparing two
# censuses is two `--reports` invocations of the generator. Pointing `--out` at
# the baseline is refused outright (exit 3), before any game runs: this is the one
# irreversible thing the script could do, so it is fail-closed rather than warned.
#
# ── Why this is NOT `game_lab/classify.sh` ───────────────────────────────────
# Two reasons, both measured. (1) `classify.sh` is an INGEST path: it reads
# `game_lab/inbox/` and `mv`s each file into `working/` or `broken/`. Re-censusing
# a corpus that is already sorted is a different job — this script never writes to
# the corpus at all, and it takes the files where they already are. (2)
# `classify.sh` is inside `game_lab/`, which `.gitignore:23` excludes wholesale —
# `git ls-files game_lab` is 0. A fix there is invisible to CI, to review, and to
# every other checkout, which is precisely why the sibling generator was put in
# `scripts/` instead. `classify.sh` is deliberately left untouched.
#
# ── What this does NOT buy ───────────────────────────────────────────────────
# It does not make the 45 minutes cheaper. A full re-census still costs a full
# re-census; what changes is that you only pay it once. (`--from-stdin` does make
# a PARTIAL re-run possible — feed it `game-lab-census-map.mjs --bucket X` — and
# that is the thing that makes a targeted low-load re-measure affordable. The
# resulting directory is partial, and the map correctly reports every un-run game
# as NO-REPORT rather than pretending.)
# It is also invisible to CI, like everything else that touches `game_lab/`: the
# corpus holds real game bytes (Constraint 9), so no check can ever run this.
#
# ── Usage ────────────────────────────────────────────────────────────────────
#   bash scripts/game-lab-recensus.sh                       # full corpus -> game_lab/reports-<today>/
#   bash scripts/game-lab-recensus.sh --limit 3 --dry-run   # what would run, writing nothing
#   bash scripts/game-lab-recensus.sh --resume              # skip games already done in --out
#   node scripts/game-lab-census-map.mjs --bucket unimpl-stub \
#     | bash scripts/game-lab-recensus.sh --from-stdin      # re-run one bucket only
#
# Then:  node scripts/game-lab-census-map.mjs --reports game_lab/reports-<today>
set -euo pipefail

cd "$(dirname "$0")/.."          # repo root

ROOT="game_lab"
BASELINE="$ROOT/reports"         # ★ the July column — never a legal --out
BIN="target/debug/wie_validate"
CORPUS="$ROOT/broken"
OUT="$ROOT/reports-$(date +%F)"
TIMEOUT=20
KILL=50
NICE=15
LIMIT=0
RESUME=0
SHOTS=0
DRYRUN=0
FROM_STDIN=0

while [ $# -gt 0 ]; do
  case "$1" in
    --corpus)     CORPUS="$2"; shift 2;;
    --out)        OUT="$2"; shift 2;;
    --timeout)    TIMEOUT="$2"; shift 2;;
    --kill)       KILL="$2"; shift 2;;
    --nice)       NICE="$2"; shift 2;;
    --limit)      LIMIT="$2"; shift 2;;
    --resume)     RESUME=1; shift;;
    --shots)      SHOTS=1; shift;;
    --dry-run)    DRYRUN=1; shift;;
    --from-stdin) FROM_STDIN=1; shift;;
    -h|--help)    sed -n '1,60p' "$0"; exit 0;;
    *) echo "unknown arg: $1" >&2; exit 2;;
  esac
done

# ── Fail-closed: the baseline column is not an output directory ──────────────
# ★Compared as RESOLVED PATHS, not as strings. The first version of this guard
# stripped a trailing slash and compared the two spellings with `[ a = b ]`, and
# gate 2 measured what that buys: `game_lab/reports` and `game_lab/reports/` were
# refused while `./game_lab/reports`, `game_lab/./reports`,
# `game_lab/reports/../reports` and the absolute path all sailed through and
# would have overwritten the baseline's `<stem>.json` in place. A guard that
# stands on one spelling of four is not fail-closed; it is a spelling test.
#
# `canon` is the ONE predicate both guards use — deliberately, because the same
# hole existed in the corpus guard and fixing one would have left the other.
#
# ⒜ SYMLINKS ARE FOLLOWED (`cd -P` / `pwd -P`). What this guard protects is the
#    baseline's *bytes*, so a symlink that lands on them must be refused too;
#    resolving is the only way to see that. The cost is stated rather than
#    hidden: a caller who deliberately keeps a symlinked alias to a *different*
#    directory is judged by where it points, not by what they typed.
# ⒝ A PATH THAT DOES NOT EXIST YET STILL NORMALISES. The dated output directory
#    is created later by `mkdir -p`, so a guard that needed the target to exist
#    would be useless here — and `realpath`/`fs.realpath` fail on absent paths,
#    which is why this resolves the longest EXISTING prefix physically and
#    appends the (already lexically normalised) remainder.
canon() {
  local p="$1" out="" comp head rest=""
  case "$p" in /*) ;; *) p="$PWD/$p" ;; esac
  # Lexical pass first: collapses `//`, `.` and `..` even inside a tail that has
  # never been created. Globbing is off for the split so a literal `*` in a path
  # component cannot expand into something else.
  set -f
  local IFS=/
  for comp in $p; do
    case "$comp" in
      '' | .) ;;
      ..) out="${out%/*}" ;;
      *) out="$out/$comp" ;;
    esac
  done
  set +f
  unset IFS
  [ -n "$out" ] || out=/
  # Physical pass: resolve symlinks as far as the path actually exists.
  head="$out"
  while [ ! -e "$head" ] && [ "$head" != "/" ]; do
    rest="${head##*/}${rest:+/$rest}"
    head="${head%/*}"
    [ -n "$head" ] || head=/
  done
  if [ -d "$head" ]; then
    head="$(cd -P -- "$head" 2>/dev/null && pwd -P)" || head="$out"
  fi
  printf '%s' "${head%/}${rest:+/$rest}"
}

_out_c="$(canon "$OUT")"
if [ "$_out_c" = "$(canon "$BASELINE")" ]; then
  echo "game-lab-recensus: refusing --out $OUT — that is the July baseline column" >&2
  echo "  (resolves to $_out_c)" >&2
  echo "  It is the only input the census's 7/7 reconciliation can be checked against;" >&2
  echo "  a re-run would overwrite it in place with no way back. Use a dated directory" >&2
  echo "  (the default is $ROOT/reports-$(date +%F)) and compare with the generator's" >&2
  echo "  --reports flag instead." >&2
  exit 3
fi
_corpus_c="$(canon "$CORPUS")/"
case "$_out_c/" in
  "$_corpus_c"*)
    echo "game-lab-recensus: refusing --out $OUT — it is inside the corpus $CORPUS" >&2
    echo "  (resolves to $_out_c, inside ${_corpus_c%/})" >&2
    exit 3 ;;
esac

[ -d "$CORPUS" ] || { echo "game-lab-recensus: no corpus at $CORPUS (it is git-ignored and local-only)" >&2; exit 2; }

# ── Population ───────────────────────────────────────────────────────────────
# One entry per FILE, never per stem: measured 2026-09-18, the corpus is 187 files
# sharing 184 stems, so a stem-keyed population silently drops one copy of each
# duplicate. `game-lab-census-map.mjs` keys its rows the same way for the same
# reason — read the comment above its `corpus` loop.
files=()
if [ "$FROM_STDIN" -eq 1 ]; then
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    if [ -f "$line" ]; then files+=("$line"); else echo "game-lab-recensus: skipping missing path: $line" >&2; fi
  done
else
  while IFS= read -r line; do files+=("$line"); done < <(find "$CORPUS" -type f \( -name '*.zip' -o -name '*.jar' -o -name '*.jad' \) | sort)
fi

[ ${#files[@]} -gt 0 ] || { echo "game-lab-recensus: 0 games to run (corpus=$CORPUS)" >&2; exit 2; }
if [ "$LIMIT" -gt 0 ] && [ "$LIMIT" -lt ${#files[@]} ]; then
  files=("${files[@]:0:$LIMIT}")
fi

# ── Stem collisions: announced, and now SURVIVED ─────────────────────────────
# The per-game artefacts used to be keyed by `<stem>` alone, so when two carriers
# held the same title the LAST run won and overwrote the first. `summary.tsv` was
# the only lossless layer (it is keyed by PATH), and it still is — but the layer a
# human actually opens is the per-game file, and that one was losing data.
#
# ★The loss was WIDER than the per-game JSON. `<stem>` keyed five paths: `.json`,
# `.log` (the validator's stderr — the first thing anyone reads to answer "why did
# this fail"), `.png`, the `.out` scratch, and the `--resume` skip test. All of them
# were overwritten, not just the verdict.
#
# ⇒ A colliding stem is now written as `<bucket>__<stem>`, where `<bucket>` is the
# directory the file came from (`broken/ktf/놈3.zip` -> `ktf__놈3`). Non-colliding
# stems are UNCHANGED, so an existing directory and the 181 uncontested games keep
# byte-identical names; only the games that were actually losing data move.
# ★Existing directories are NOT rewritten — the July column keeps its own names
# (that is the proposal's "기존 디렉터리를 소급해 고치지는 마라"), and the generator
# keeps reading them, because it still tries the bare `<stem>` key as a fallback.
dupstems=$(printf '%s\n' "${files[@]}" | sed 's#.*/##; s#\.[^.]*$##' | sort | uniq -d)
collisions=$(printf '%s' "$dupstems" | grep -c . || true)

echo "game-lab-recensus: ${#files[@]} game(s) · corpus=$CORPUS · out=$OUT · timeout=${TIMEOUT}s kill=${KILL}s nice=$NICE"
[ "$collisions" -gt 0 ] && {
  echo "  ★ $collisions stem(s) appear under more than one carrier — those are written as <bucket>__<stem>.{json,log,png} so neither copy is overwritten"
  printf '%s\n' "$dupstems" | sed 's/^/      /'
}

if [ "$DRYRUN" -eq 1 ]; then
  echo "  --dry-run: writing nothing. Files that would run:"
  printf '    %s\n' "${files[@]}"
  exit 0
fi

# ── The validator ────────────────────────────────────────────────────────────
if [ ! -x "$BIN" ]; then
  echo ">> building wie_validate ..."
  nice -n "$NICE" cargo build -p wie_cli --bin wie_validate
fi

mkdir -p "$OUT"
[ "$SHOTS" -eq 1 ] && mkdir -p "$OUT/shots"

SUMMARY="$OUT/summary.tsv"
if [ ! -f "$SUMMARY" ]; then
  printf "result\tplatform\tfile\treason\tticks\tpaints\tms\n" > "$SUMMARY"
fi

pass=0; fail=0; skipped=0
for f in "${files[@]}"; do
  base="$(basename "$f")"
  stem="${base%.*}"
  # ★The artefact key. `<stem>` unless two carriers share it, in which case the
  # source directory disambiguates. Derived from the PATH, not from `detected`
  # below, so it is available before the resume test and needs no unzip.
  key="$stem"
  if [ -n "$dupstems" ] && printf '%s\n' "$dupstems" | grep -qxF -- "$stem"; then
    key="$(basename "$(dirname "$f")")__${stem}"
  fi
  if [ "$RESUME" -eq 1 ] && [ -s "$OUT/${key}.json" ]; then
    skipped=$((skipped+1)); continue
  fi

  # Platform detection up front, so a SIGKILLed hang — where the validator emits
  # no JSON — is still filed under the right carrier. Same markers `wie_cli` routes
  # on, and the same ones classify.sh uses: __adf__=ktf, app_info=lgt, .msd=skt.
  detected="unknown"
  case "$base" in
    *.zip)
      entries="$(unzip -Z1 "$f" 2>/dev/null || true)"
      if printf '%s\n' "$entries" | grep -qE '(^|/)__adf__$'; then detected="ktf"
      elif printf '%s\n' "$entries" | grep -qE '(^|/)app_info$'; then detected="lgt"
      elif printf '%s\n' "$entries" | grep -qE '\.msd$'; then detected="skt"
      fi;;
    *.jad|*.jar) detected="j2me";;
  esac

  # The flags MUST match what produced the baseline column, or the two censuses are
  # not comparable — `classify.sh` ran `--inject --timeout N` and that is where
  # `game_lab/reports/`'s verdicts come from (its reasons name injection steps, e.g.
  # "tick error during '10_OK'"). `--shotdir` is the one deliberate difference: it is
  # opt-in here because the baseline's shot directory is 286 MB / 6,069 PNGs (measured),
  # and that is per run. ★Whether dropping it moves a verdict is NOT measured and this
  # script does not claim it. The A/B was attempted 2026-09-19 at loadavg 170-202 and
  # cannot settle it: the SAME 3 games run twice landed in 3 different buckets
  # (UNCLASSIFIED×3 -> only-blank / panic-unwrap / no-frame), which is the unpaired-
  # sampling trap AGENTS.md names — the two arms differ on `--shots` AND on the load
  # minute, and the load moves far more. Settling it needs an idle machine and pairing.
  # Pass `--shots` when you need a run comparable to the baseline on that axis.
  shotargs=()
  [ "$SHOTS" -eq 1 ] && shotargs=(--shotdir "$OUT/shots" --screenshot "$OUT/${key}.png")

  # macOS has no `timeout`, and SIGALRM can be masked by heavy ARM emulation, so a
  # background `kill -9` is the only reliable bound on a hung tick(). Lifted from
  # classify.sh, which has run this for the whole corpus.
  nice -n "$NICE" "$BIN" "$f" --inject --timeout "$TIMEOUT" "${shotargs[@]}" \
    >"$OUT/${key}.out" 2>"$OUT/${key}.log" &
  vpid=$!
  i=0
  while kill -0 "$vpid" 2>/dev/null; do
    i=$((i+1))
    [ "$i" -ge "$KILL" ] && { kill -9 "$vpid" 2>/dev/null; break; }
    sleep 1
  done
  wait "$vpid" 2>/dev/null || true

  json="$(cat "$OUT/${key}.out" 2>/dev/null || true)"
  rm -f "$OUT/${key}.out"
  if [ -z "$json" ]; then
    json='{"result":"FAIL","platform":"'"$detected"'","reason":"hang: SIGKILLed after '"$KILL"'s (infinite tick loop)","ticks":0,"paints":0,"content":false,"ms":0}'
  fi
  printf '%s\n' "$json" > "$OUT/${key}.json"

  IFS=$'\t' read -r result platform reason ticks paints content ms < <(
    python3 - "$json" <<'PY'
import sys, json
try:
    d = json.loads(sys.argv[1])
except Exception:
    d = {}
fields = [d.get("result","FAIL"), d.get("platform","unknown"),
          (d.get("reason","parse-error") or "").replace("\t"," ").replace("\n"," "),
          d.get("ticks",0), d.get("paints",0), d.get("content",False), d.get("ms",0)]
print("\t".join(str(x) for x in fields))
PY
  )
  # ★ Keyed by PATH, not stem — this row is what survives a stem collision.
  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n" "$result" "$platform" "$f" "$reason" "$ticks" "$paints" "$ms" >> "$SUMMARY"

  if [ "$result" = "PASS" ]; then pass=$((pass+1)); else fail=$((fail+1)); fi
  echo "   $result ($platform) ticks=$ticks paints=$paints  $base"
done

echo
echo "=== done: $pass PASS / $fail FAIL / $skipped skipped ==="
echo "per-game rows: $OUT/  (one .json per game + summary.tsv, keyed by path)"
echo "corpus untouched: $(git status --porcelain "$CORPUS" 2>/dev/null | wc -l | tr -d ' ') change(s) reported by git (game_lab/ is git-ignored, so 0 is expected either way)"
echo
echo "map it:  node scripts/game-lab-census-map.mjs --reports $OUT"
echo "compare: node scripts/game-lab-census-map.mjs --reports $BASELINE --out $OUT/map-baseline.tsv"
echo "  ★ paints is a count of ticks inside a FIXED wall-clock budget, so a loaded"
echo "    machine lowers it — read AGENTS.md's four-step rule before calling a FAIL"
echo "    a regression. Prefer running this with the machine idle."
