#!/usr/bin/env bash
# Builds one arcade demo: <game-dir>/out/<jar>.jar from arcade-common/src + <game-dir>/src.
# Same recipe as demo/pebble-snake/build.sh — read its header for why there is no
# preverify, why --release 7, and why the stubs exist. The only difference: the five
# arcade games share the frame (Arcade.java), the sound assembler (Sound.java) and the
# stubs in this directory, so each game's own build.sh is one line that calls this.
#
# Usage: demo/arcade-common/build-game.sh <game-dir> <jar-name>   (JAVA_HOME or Homebrew openjdk@17)
set -euo pipefail
common="$(cd "$(dirname "$0")" && pwd)"
game="$(cd "$1" && pwd)"
name="$2"
if [[ -n "${JAVA_HOME:-}" ]]; then bin="$JAVA_HOME/bin"
elif [[ -x /opt/homebrew/opt/openjdk@17/bin/javac ]]; then bin=/opt/homebrew/opt/openjdk@17/bin
else echo "need a JDK (>= 9, <= 19 for --release 7): set JAVA_HOME" >&2; exit 2; fi

out="$game/out"
rm -rf "${out:?}"
mkdir -p "$out/stubs" "$out/classes"
"$bin/javac" -nowarn -Xlint:-options --release 7 -d "$out/stubs" $(find "$common/stubs" -name '*.java' | sort)
"$bin/javac" -nowarn -Xlint:-options --release 7 -encoding UTF-8 -cp "$out/stubs" -d "$out/classes" \
  $(find "$common/src" "$game/src" -name '*.java' | sort)
cp "$game/LICENSE" "$out/classes/LICENSE.txt"
"$bin/jar" --create --file "$out/$name.jar" --manifest "$game/MANIFEST.MF" \
  --date 2026-01-01T00:00:00Z -C "$out/classes" .
shasum -a 256 "$out/$name.jar"
