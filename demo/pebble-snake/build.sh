#!/usr/bin/env bash
# Builds out/pebble-snake.jar from src/ — no WTK, no preverify.
#
# Why no preverify: CLDC's preverifier only adds StackMap attributes that a KVM needs.
# wie's JVM (RustJava) interprets bytecode without a verifier pass, so the plain javac
# output runs as-is. The jar is therefore for wie (and desktop MIDP emulators), not for
# a real CLDC handset.
#
# Why --release 7: it is the oldest target a current JDK still emits (class file 51),
# and it keeps the language to what CLDC-era code looks like — no lambdas, no
# invokedynamic string concatenation (javac 9+ uses indy for `+` from target 9 up).
# `stubs/` declares only the MIDP surface this game touches, so compiling against it
# also proves the game calls nothing else; the stubs are never packaged.
#
# Reproducible: `jar --date` pins every entry timestamp, so the same JDK gives the same
# bytes. Usage: demo/pebble-snake/build.sh   (JAVA_HOME or a Homebrew openjdk@17)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if [[ -n "${JAVA_HOME:-}" ]]; then bin="$JAVA_HOME/bin"
elif [[ -x /opt/homebrew/opt/openjdk@17/bin/javac ]]; then bin=/opt/homebrew/opt/openjdk@17/bin
else echo "need a JDK (>= 9, <= 19 for --release 7): set JAVA_HOME" >&2; exit 2; fi

out="$here/out"
rm -rf "${out:?}"
mkdir -p "$out/stubs" "$out/classes"
"$bin/javac" -nowarn -Xlint:-options --release 7 -d "$out/stubs" $(find "$here/stubs" -name '*.java' | sort)
"$bin/javac" -nowarn -Xlint:-options --release 7 -encoding UTF-8 -cp "$out/stubs" -d "$out/classes" $(find "$here/src" -name '*.java' | sort)
cp "$here/LICENSE" "$out/classes/LICENSE.txt"
"$bin/jar" --create --file "$out/pebble-snake.jar" --manifest "$here/MANIFEST.MF" \
  --date 2026-01-01T00:00:00Z -C "$out/classes" .
shasum -a 256 "$out/pebble-snake.jar"
