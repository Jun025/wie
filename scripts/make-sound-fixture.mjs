// Builds the sound fixture — the one guest in this repo that PLAYS MUSIC, so the browser
// round-trip can drive the featurephone audio sink end to end (Scenario S in
// scripts/contract-roundtrip.mjs).
//
// Why it exists: the sink's soundfont path (wie_featurephone/src/audio.rs — fetch after the first
// play, prelude module, hand-over to the worklet) is wasm-only, so neither `cargo test` nor
// scripts/check-audio-worklet.mjs runs it, and until 2026-09-28 no committed fixture made a sound
// at all. It broke once already (the soundfont was never fetched) while every node case was green
// (docs/report 0349, 0355).
//
// What it does: a MIDlet shows a Canvas; every keyPressed() prints SOUND_MARK on stdout, then
// plays SMAF_BYTES through `Manager.createPlayer(InputStream, "application/vnd.smaf").start()` —
// a NEW player each time, so each key is a new handle and a new `Play`. Nothing plays before the
// first key: the round-trip decides when the first sound happens, which is what lets it tell
// "fetched after the first play" from "fetched at boot".
//
// Built, never committed — the round-trip imports `soundFixtureJar()` and serves it from memory,
// like drawFixtureJar(). Run directly to write test_data/sound_j2me.jar (git-ignored) for
// `wie_validate --inject`.

import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ConstantPool, classFile, method, u2, zip } from "./make-draw-fixture.mjs";

export const SOUND_MARK = "snd:play";

// A minimal SMAF: one `SEQU` (handyphone sequence, 20 ms timebase) chunk holding one note —
// channel 0, octave 2, C, gate 250 × 20 ms = 5 s (two-byte handy number: ((0x80 & 0x7f) + 1) << 7
// | 0x7a) — then end of sequence. The same shape as the hand-built SMAF in wie-skvm's
// wie_audio_clip tests. 5 s, not less: at host load ~340 a 2 s note had ended before the
// round-trip's `stats` came back (measured — 1 run in 3).
const SEQUENCE = [0x00, 0x2c, 0x80, 0x7a, 0x00, 0x00, 0x00, 0x00];
export const SMAF_BYTES = Buffer.concat([
  Buffer.from("MMMD\0\0\0\0SEQU", "latin1"),
  Buffer.from([0, 0, 0, SEQUENCE.length]),
  Buffer.from(SEQUENCE),
  Buffer.from([0, 0]),
]);

const CANVAS = "javax/microedition/lcdui/Canvas";
const DISPLAY = "javax/microedition/lcdui/Display";
const MIDLET = "javax/microedition/midlet/MIDlet";
const PLAYER = "javax/microedition/media/Player";
const b = (...xs) => Buffer.from(xs);
const initCalling = (cp, parent) => Buffer.concat([b(0x2a, 0xb7), u2(cp.method(parent, "<init>", "()V")), b(0xb1)]);

// keyPressed(int): System.out.println(SOUND_MARK);
//                  Manager.createPlayer(new ByteArrayInputStream(SMAF_BYTES), "application/vnd.smaf").start();
const soundCanvas = () => {
  const cp = new ConstantPool();
  const array = [b(0x10, SMAF_BYTES.length, 0xbc, 8)]; // bipush len, newarray T_BYTE
  SMAF_BYTES.forEach((value, index) => array.push(b(0x59, 0x10, index, 0x10, value, 0x54))); // dup, bipush i, bipush v, bastore
  const keyPressed = Buffer.concat([
    b(0xb2), // getstatic System.out
    u2(cp.field("java/lang/System", "out", "Ljava/io/PrintStream;")),
    b(0x13), // ldc_w SOUND_MARK
    u2(cp.string(SOUND_MARK)),
    b(0xb6), // invokevirtual println
    u2(cp.method("java/io/PrintStream", "println", "(Ljava/lang/String;)V")),
    b(0xbb), // new ByteArrayInputStream
    u2(cp.class_("java/io/ByteArrayInputStream")),
    b(0x59), // dup
    ...array,
    b(0xb7), // invokespecial ByteArrayInputStream.<init>([B)V
    u2(cp.method("java/io/ByteArrayInputStream", "<init>", "([B)V")),
    b(0x13), // ldc_w the SMAF content type
    u2(cp.string("application/vnd.smaf")),
    b(0xb8), // invokestatic Manager.createPlayer
    u2(cp.method("javax/microedition/media/Manager", "createPlayer", `(Ljava/io/InputStream;Ljava/lang/String;)L${PLAYER};`)),
    b(0xb9), // invokeinterface Player.start, 1 arg slot, 0
    u2(cp.interfaceMethod(PLAYER, "start", "()V")),
    b(1, 0),
    b(0xb1),
  ]);
  return classFile(cp, "SoundCanvas", CANVAS, [
    method(cp, "<init>", "()V", 1, 1, initCalling(cp, CANVAS)),
    method(cp, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", 0, 2, b(0xb1)),
    method(cp, "keyPressed", "(I)V", 7, 2, keyPressed),
  ]);
};

// startApp(): Display.getDisplay(this).setCurrent(new SoundCanvas())
const soundMidlet = () => {
  const cp = new ConstantPool();
  const startApp = Buffer.concat([
    b(0x2a, 0xb8), // aload_0, invokestatic getDisplay
    u2(cp.method(DISPLAY, "getDisplay", `(L${MIDLET};)L${DISPLAY};`)),
    b(0xbb), // new SoundCanvas
    u2(cp.class_("SoundCanvas")),
    b(0x59, 0xb7), // dup, invokespecial <init>
    u2(cp.method("SoundCanvas", "<init>", "()V")),
    b(0xb6), // invokevirtual setCurrent
    u2(cp.method(DISPLAY, "setCurrent", "(Ljavax/microedition/lcdui/Displayable;)V")),
    b(0xb1),
  ]);
  return classFile(cp, "SoundMIDlet", MIDLET, [method(cp, "<init>", "()V", 1, 1, initCalling(cp, MIDLET)), method(cp, "startApp", "()V", 3, 1, startApp)]);
};

const MANIFEST = ["Manifest-Version: 1.0", "MIDlet-Name: SoundFixture", "MIDlet-1: SoundFixture, , SoundMIDlet", ""].join("\n");

export const soundFixtureJar = () =>
  zip([
    ["META-INF/MANIFEST.MF", Buffer.from(MANIFEST, "utf8")],
    ["SoundMIDlet.class", soundMidlet()],
    ["SoundCanvas.class", soundCanvas()],
  ]);

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "test_data", "sound_j2me.jar");
  writeFileSync(out, soundFixtureJar());
  console.log(`wrote ${out}`);
}
