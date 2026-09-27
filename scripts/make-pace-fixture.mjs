// Builds test_data/pace_j2me.zip — a guest whose whole job is a game loop, so the engine's
// pacing can be measured on something committed (wie-j2me/tests/test_pacing.rs reads it).
//
// The loop is the one #338 measured on an LGT title, reduced to its timing skeleton:
//
//   next = currentTimeMillis()
//   loop { painted = 0; canvas.repaint(); while (painted == 0) Thread.yield();
//          next += 50; d = next - currentTimeMillis(); Thread.sleep(max(d, 10)); }
//
// with paint() setting `painted`. A 50ms deadline loop that spins on yield until its own
// paint lands is exactly the shape that lost a host frame per game frame when the engine
// delivered repaints only at the end of a tick, and that paid a full GC per paint.
//
// Regenerate: `node scripts/make-pace-fixture.mjs` (writes the zip; byte-stable — STORED
// entries, zeroed timestamps). Verify a regeneration by re-running the test, not by diffing.

import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ConstantPool, classFile, method, u2, zip } from "./make-draw-fixture.mjs";

export const PERIOD_MS = 50;
export const FLOOR_MS = 10;

const CANVAS = "javax/microedition/lcdui/Canvas";
const GRAPHICS = "javax/microedition/lcdui/Graphics";
const DISPLAY = "javax/microedition/lcdui/Display";
const MIDLET = "javax/microedition/midlet/MIDlet";
const THREAD = "java/lang/Thread";
const SELF = "LPaceCanvas;";

const b = (...xs) => Buffer.from(xs);
// ACC_PUBLIC | ACC_STATIC field, no attributes.
const staticField = (cp, name, desc) => Buffer.concat([u2(0x0009), u2(cp.utf8(name)), u2(cp.utf8(desc)), u2(0)]);
const initCalling = (cp, parent) => Buffer.concat([b(0x2a, 0xb7), u2(cp.method(parent, "<init>", "()V")), b(0xb1)]);

// paint(): painted = 1; g.fillRect(0, 0, 32, 32) — something on screen, and the loop's release.
const paceCanvas = () => {
  const cp = new ConstantPool();
  const painted = cp.field("PaceCanvas", "painted", "I");
  const paint = Buffer.concat([
    b(0x04, 0xb3), // iconst_1, putstatic painted
    u2(painted),
    b(0x2b, 0x03, 0x03, 0x10, 32, 0x10, 32, 0xb6), // aload_1, 0, 0, 32, 32, invokevirtual fillRect
    u2(cp.method(GRAPHICS, "fillRect", "(IIII)V")),
    b(0xb1),
  ]);
  return classFile(
    cp,
    "PaceCanvas",
    CANVAS,
    [method(cp, "<init>", "()V", 1, 1, initCalling(cp, CANVAS)), method(cp, "paint", `(L${GRAPHICS};)V`, 5, 2, paint)],
    [staticField(cp, "painted", "I"), staticField(cp, "self", SELF)],
  );
};

// run(): the loop above. Locals: 0 this, 1-2 next (long), 3-4 d (long).
const paceThread = () => {
  const cp = new ConstantPool();
  const now = Buffer.concat([b(0xb8), u2(cp.method("java/lang/System", "currentTimeMillis", "()J"))]);
  const painted = cp.field("PaceCanvas", "painted", "I");
  const top = Buffer.concat([
    b(0x03, 0xb3), // iconst_0, putstatic painted
    u2(painted),
    b(0xb2), // getstatic self
    u2(cp.field("PaceCanvas", "self", SELF)),
    b(0xb6), // invokevirtual repaint()V
    u2(cp.method(CANVAS, "repaint", "()V")),
  ]);
  const yieldCall = Buffer.concat([b(0xb8), u2(cp.method(THREAD, "yield", "()V"))]);
  // spin: getstatic painted; ifne +(3 + 3 + 3) past the yield and the goto; yield; goto spin
  const spin = Buffer.concat([b(0xb2), u2(painted), b(0x9a), u2(3 + yieldCall.length + 3), yieldCall]);
  const spinBack = Buffer.concat([b(0xa7), u2(-spin.length & 0xffff)]);
  const deadline = Buffer.concat([
    b(0x1f, 0x10, PERIOD_MS, 0x85, 0x61, 0x40), // lload_1, bipush, i2l, ladd, lstore_1
    b(0x1f), // lload_1
    now,
    b(0x65, 0x42), // lsub, lstore_3
  ]);
  const floorSet = b(0x10, FLOOR_MS, 0x85, 0x42); // bipush, i2l, lstore_3
  // lload_3; bipush; i2l; lcmp; ifge +(3 + floorSet)
  const floor = Buffer.concat([b(0x21, 0x10, FLOOR_MS, 0x85, 0x94, 0x9c), u2(3 + floorSet.length), floorSet]);
  const sleep = Buffer.concat([b(0x21, 0xb8), u2(cp.method(THREAD, "sleep", "(J)V"))]);
  const body = Buffer.concat([top, spin, spinBack, deadline, floor, sleep]);
  const run = Buffer.concat([now, b(0x40), body, b(0xa7), u2(-body.length & 0xffff)]); // ..., lstore_1, body, goto top
  return classFile(cp, "PaceThread", THREAD, [method(cp, "<init>", "()V", 1, 1, initCalling(cp, THREAD)), method(cp, "run", "()V", 4, 5, run)]);
};

// startApp(): self = new PaceCanvas(); Display.getDisplay(this).setCurrent(self); new PaceThread().start()
const paceMidlet = () => {
  const cp = new ConstantPool();
  const self = cp.field("PaceCanvas", "self", SELF);
  const startApp = Buffer.concat([
    b(0xbb), // new PaceCanvas
    u2(cp.class_("PaceCanvas")),
    b(0x59, 0xb7), // dup, invokespecial <init>
    u2(cp.method("PaceCanvas", "<init>", "()V")),
    b(0xb3), // putstatic self
    u2(self),
    b(0x2a, 0xb8), // aload_0, invokestatic getDisplay
    u2(cp.method(DISPLAY, "getDisplay", `(L${MIDLET};)L${DISPLAY};`)),
    b(0xb2), // getstatic self
    u2(self),
    b(0xb6), // invokevirtual setCurrent
    u2(cp.method(DISPLAY, "setCurrent", "(Ljavax/microedition/lcdui/Displayable;)V")),
    b(0xbb), // new PaceThread
    u2(cp.class_("PaceThread")),
    b(0x59, 0xb7), // dup, invokespecial <init>
    u2(cp.method("PaceThread", "<init>", "()V")),
    b(0xb6), // invokevirtual start
    u2(cp.method(THREAD, "start", "()V")),
    b(0xb1),
  ]);
  return classFile(cp, "PaceMIDlet", MIDLET, [method(cp, "<init>", "()V", 1, 1, initCalling(cp, MIDLET)), method(cp, "startApp", "()V", 2, 1, startApp)]);
};

const MANIFEST = ["Manifest-Version: 1.0", "MIDlet-Name: PaceFixture", "MIDlet-1: PaceFixture, , PaceMIDlet", ""].join("\n");

export const paceFixtureJar = () =>
  zip([
    ["META-INF/MANIFEST.MF", Buffer.from(MANIFEST, "utf8")],
    ["PaceMIDlet.class", paceMidlet()],
    ["PaceCanvas.class", paceCanvas()],
    ["PaceThread.class", paceThread()],
  ]);

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "test_data", "pace_j2me.zip");
  // A zip holding the jar — the committed-fixture convention (*.jar is git-ignored, see test_boot.rs).
  writeFileSync(out, zip([["pace_j2me.jar", paceFixtureJar()]]));
  console.log(`wrote ${out}`);
}
