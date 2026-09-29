#!/usr/bin/env bash
# Builds one demo game: <game-dir>/out/<jar>.jar from arcade-common/src + <game-dir>/src,
# plus the images <game-dir>/art/*.java draws at build time. No WTK, no preverify.
#
# Why no preverify: CLDC's preverifier only adds StackMap attributes that a KVM needs.
# wie's JVM (RustJava) interprets bytecode without a verifier pass, so the plain javac
# output runs as-is. The jar is therefore for wie (and desktop MIDP emulators), not for
# a real CLDC handset.
#
# Why --release 7: it is the oldest target a current JDK still emits (class file 51),
# and it keeps the language to what CLDC-era code looks like — no lambdas, no
# invokedynamic string concatenation (javac 9+ uses indy for `+` from target 9 up).
# `stubs/` declares only the MIDP surface the game touches — and only what the engine
# (wie-midp) implements — so compiling against it also proves the game calls nothing
# else; the stubs are never packaged.
#
# The art step runs on the build JDK (full Java2D, single-file source launch = JDK 11+)
# and writes PNGs into the class directory, so they ship inside the jar.
#
# Reproducible: `jar --date` pins every entry timestamp and ImageIO writes no timestamp,
# so the same JDK gives the same bytes. Usage: demo/arcade-common/build-game.sh <game-dir>
# <jar-name>   (JAVA_HOME or a Homebrew openjdk@17; JDK 11~19)
set -euo pipefail
common="$(cd "$(dirname "$0")" && pwd)"
game="$(cd "$1" && pwd)"
name="$2"
if [[ -n "${JAVA_HOME:-}" ]]; then bin="$JAVA_HOME/bin"
elif [[ -x /opt/homebrew/opt/openjdk@17/bin/javac ]]; then bin=/opt/homebrew/opt/openjdk@17/bin
else echo "need a JDK (11 ~ 19, for --release 7 and source launch): set JAVA_HOME" >&2; exit 2; fi

out="$game/out"
rm -rf "${out:?}"
mkdir -p "$out/stubs" "$out/classes"
"$bin/javac" -nowarn -Xlint:-options --release 7 -d "$out/stubs" $(find "$common/stubs" -name '*.java' | sort)
"$bin/javac" -nowarn -Xlint:-options --release 7 -encoding UTF-8 -cp "$out/stubs" -d "$out/classes" \
  $(find "$common/src" "$game/src" -name '*.java' | sort)
for art in $(find "$game/art" -name '*.java' 2>/dev/null | sort); do
  "$bin/java" -Djava.awt.headless=true "$art" "$out/classes"
done
cp "$game/LICENSE" "$out/classes/LICENSE.txt"
"$bin/jar" --create --file "$out/$name.jar" --manifest "$game/MANIFEST.MF" \
  --date 2026-01-01T00:00:00Z -C "$out/classes" .
shasum -a 256 "$out/$name.jar"
