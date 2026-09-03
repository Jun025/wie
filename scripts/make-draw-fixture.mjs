// Builds test_data/draw_j2me.jar — the one fixture in this repo that DRAWS.
//
// Why a generator and not a checked-in .java: the repo has no JDK (and CI has
// none either), so the two class files are emitted byte-wise here. Both are
// class-file version 47.0 on purpose — pre-StackMapTable, so no verifier
// metadata has to be synthesized.
//
// What it draws: a MIDlet whose Canvas paints one filled rect. That is enough
// to make scripts/contract-roundtrip.mjs assert nonBlackPixels() > 0 instead of
// reporting it as info (the helloworld_* fixtures never draw — untouched).
//
// It also REACTS: keyPressed() stores the MIDP code it was handed and repaints a
// bar that wide, so the canvas encodes "the guest received exactly this code".
// That is what turns the round-trip's key sweep from "no exception was thrown"
// into a delivery assertion (Scenario D).
//
// Usage: imported by scripts/contract-roundtrip.mjs; run it directly to write
// test_data/draw_j2me.jar for `wie_validate` (the jar itself is never committed
// — *.jar is git-ignored and the leak audit rejects tracked ones).

import { writeFileSync } from "node:fs";
import { crc32 } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";

const u2 = (v) => Buffer.from([(v >> 8) & 0xff, v & 0xff]);
const u4 = (v) => Buffer.from([(v >>> 24) & 0xff, (v >>> 16) & 0xff, (v >>> 8) & 0xff, v & 0xff]);

class ConstantPool {
  #items = [];
  #index = new Map();
  #add(key, bytes) {
    const hit = this.#index.get(key);
    if (hit) return hit;
    this.#items.push(bytes);
    this.#index.set(key, this.#items.length); // 1-based
    return this.#items.length;
  }
  utf8(s) {
    const b = Buffer.from(s, "utf8");
    return this.#add(`u:${s}`, Buffer.concat([Buffer.from([1]), u2(b.length), b]));
  }
  integer(v) {
    return this.#add(`i:${v}`, Buffer.concat([Buffer.from([3]), u4(v)]));
  }
  class_(name) {
    const n = this.utf8(name);
    return this.#add(`c:${name}`, Buffer.concat([Buffer.from([7]), u2(n)]));
  }
  nameAndType(name, desc) {
    const n = this.utf8(name);
    const d = this.utf8(desc);
    return this.#add(`n:${name}:${desc}`, Buffer.concat([Buffer.from([12]), u2(n), u2(d)]));
  }
  method(cls, name, desc) {
    const c = this.class_(cls);
    const nt = this.nameAndType(name, desc);
    return this.#add(`m:${cls}:${name}:${desc}`, Buffer.concat([Buffer.from([10]), u2(c), u2(nt)]));
  }
  field(cls, name, desc) {
    const c = this.class_(cls);
    const nt = this.nameAndType(name, desc);
    return this.#add(`f:${cls}:${name}:${desc}`, Buffer.concat([Buffer.from([9]), u2(c), u2(nt)]));
  }
  serialize() {
    return Buffer.concat([u2(this.#items.length + 1), ...this.#items]);
  }
}

// One method with a Code attribute. `code` is already-resolved bytecode.
const method = (cp, name, desc, maxStack, maxLocals, code) => {
  const body = Buffer.concat([u2(maxStack), u2(maxLocals), u4(code.length), code, u2(0), u2(0)]);
  return Buffer.concat([u2(0x0001), u2(cp.utf8(name)), u2(cp.utf8(desc)), u2(1), u2(cp.utf8("Code")), u4(body.length), body]);
};

// One field_info. ACC_PUBLIC | ACC_STATIC, no attributes — a static int with no
// ConstantValue starts at 0, which is the "no key seen yet" state paint() tests.
const staticIntField = (cp, name) => Buffer.concat([u2(0x0009), u2(cp.utf8(name)), u2(cp.utf8("I")), u2(0)]);

const classFile = (cp, thisClass, superClass, methods, fields = []) => {
  // Resolve every index BEFORE serializing the pool — an entry added afterwards
  // would be referenced but never written (parsers then unwrap() a None).
  const self_ = cp.class_(thisClass);
  const parent = cp.class_(superClass);
  return Buffer.concat([
    Buffer.from([0xca, 0xfe, 0xba, 0xbe]),
    u2(0), // minor
    u2(47), // major — JDK 1.3, before StackMapTable became mandatory
    cp.serialize(),
    u2(0x0021), // ACC_PUBLIC | ACC_SUPER
    u2(self_),
    u2(parent),
    u2(0), // interfaces
    u2(fields.length),
    ...fields,
    u2(methods.length),
    ...methods,
    u2(0), // class attributes
  ]);
};

// Geometry the round-trip reads back. Exported so the expected pixel count lives
// in ONE place — the fixture that draws it.
export const BASE_RECT_PX = 32 * 32; // the always-drawn rect (Scenario C)
export const KEY_BAR_Y = 32; // just below the base rect, so the two never overlap
export const KEY_BAR_H = 8; // px per unit of key code → pixels = BASE + code*8
export const keyBarPixels = (midpCode) => BASE_RECT_PX + midpCode * KEY_BAR_H;

const CANVAS = "javax/microedition/lcdui/Canvas";
const GRAPHICS = "javax/microedition/lcdui/Graphics";
const DISPLAY = "javax/microedition/lcdui/Display";
const DISPLAYABLE = "javax/microedition/lcdui/Displayable";
const MIDLET = "javax/microedition/midlet/MIDlet";

// ── DrawCanvas extends Canvas ────────────────────────────────────────────────
// paint() fills the base rect, and — once keyPressed() has stored a code — a
// second bar whose WIDTH IS THE MIDP KEY CODE the guest received. That makes the
// canvas readable from JS as an exact number: 1024 base px + code*8 bar px, so
// scripts/contract-roundtrip.mjs can assert not just "a key arrived" but "the
// guest saw exactly this code" (Scenario D).
//
// The bar is drawn WITHOUT clearing first, so the count is the union of every
// bar painted so far. Scenario D therefore presses its keys in ASCENDING code
// order — then union == widest == current, and the assertion is exact whether or
// not the host clears the framebuffer between frames.
const drawCanvas = () => {
  const cp = new ConstantPool();
  const superInit = cp.method(CANVAS, "<init>", "()V");
  const color = cp.integer(0x00ff00);
  const setColor = cp.method(GRAPHICS, "setColor", "(I)V");
  const fillRect = cp.method(GRAPHICS, "fillRect", "(IIII)V");
  const keyHit = cp.field("DrawCanvas", "keyHit", "I");
  const repaint = cp.method(CANVAS, "repaint", "()V");

  const init = Buffer.concat([Buffer.from([0x2a]), Buffer.from([0xb7]), u2(superInit), Buffer.from([0xb1])]);

  // if (keyHit != 0) g.fillRect(0, KEY_BAR_Y, keyHit, KEY_BAR_H)
  const keyBar = Buffer.concat([
    Buffer.from([0x2b, 0x03]), // aload_1 (Graphics), iconst_0 (x = 0)
    Buffer.from([0x10, KEY_BAR_Y]), // bipush  (y)
    Buffer.from([0xb2]),
    u2(keyHit), // getstatic keyHit (width = the received code)
    Buffer.from([0x10, KEY_BAR_H]), // bipush  (h)
    Buffer.from([0xb6]),
    u2(fillRect),
  ]);
  const paint = Buffer.concat([
    Buffer.from([0x2b]), // aload_1 (Graphics)
    Buffer.from([0x13]),
    u2(color), // ldc_w 0x00ff00
    Buffer.from([0xb6]),
    u2(setColor), // invokevirtual setColor(I)V
    // ── the draw itself: g.fillRect(0, 0, 32, 32) ──
    Buffer.from([0x2b, 0x03, 0x03, 0x10, 32, 0x10, 32]), // aload_1, iconst_0, iconst_0, bipush 32, bipush 32
    Buffer.from([0xb6]),
    u2(fillRect),
    Buffer.from([0xb2]),
    u2(keyHit), // getstatic keyHit
    Buffer.from([0x99]),
    u2(3 + keyBar.length), // ifeq → skip the bar (offset is from this opcode)
    keyBar,
    Buffer.from([0xb1]), // return
  ]);

  // keyPressed(int) — the guest-side proof of delivery: store the code and ask
  // for a repaint (the host only blits after the core requests a redraw).
  const keyPressed = Buffer.concat([
    Buffer.from([0x1b]), // iload_1 (key)
    Buffer.from([0xb3]),
    u2(keyHit), // putstatic keyHit
    Buffer.from([0x2a]), // aload_0
    Buffer.from([0xb6]),
    u2(repaint), // invokevirtual repaint()V
    Buffer.from([0xb1]), // return
  ]);

  return classFile(
    cp,
    "DrawCanvas",
    CANVAS,
    [
      method(cp, "<init>", "()V", 1, 1, init),
      method(cp, "paint", `(L${GRAPHICS};)V`, 5, 2, paint),
      method(cp, "keyPressed", "(I)V", 1, 2, keyPressed),
    ],
    [staticIntField(cp, "keyHit")],
  );
};

// ── DrawMIDlet extends MIDlet — startApp() shows the canvas ──────────────────
const drawMidlet = () => {
  const cp = new ConstantPool();
  const superInit = cp.method(MIDLET, "<init>", "()V");
  const getDisplay = cp.method(DISPLAY, "getDisplay", `(L${MIDLET};)L${DISPLAY};`);
  const canvasClass = cp.class_("DrawCanvas");
  const canvasInit = cp.method("DrawCanvas", "<init>", "()V");
  const setCurrent = cp.method(DISPLAY, "setCurrent", `(L${DISPLAYABLE};)V`);

  const init = Buffer.concat([Buffer.from([0x2a]), Buffer.from([0xb7]), u2(superInit), Buffer.from([0xb1])]);
  const startApp = Buffer.concat([
    Buffer.from([0x2a]), // aload_0
    Buffer.from([0xb8]),
    u2(getDisplay), // invokestatic Display.getDisplay(MIDlet)
    Buffer.from([0xbb]),
    u2(canvasClass), // new DrawCanvas
    Buffer.from([0x59]), // dup
    Buffer.from([0xb7]),
    u2(canvasInit), // invokespecial <init>
    Buffer.from([0xb6]),
    u2(setCurrent), // invokevirtual setCurrent(Displayable)
    Buffer.from([0xb1]), // return
  ]);

  return classFile(cp, "DrawMIDlet", MIDLET, [method(cp, "<init>", "()V", 1, 1, init), method(cp, "startApp", "()V", 3, 1, startApp)]);
};

const MANIFEST = ["Manifest-Version: 1.0", "MIDlet-Name: DrawFixture", "MIDlet-1: DrawFixture, , DrawMIDlet", ""].join("\n");

// ── Minimal STORED zip (no timestamps — the output stays byte-stable) ────────
// NOTE zip is little-endian; the class file above is big-endian. Separate helpers.
const l2 = (v) => Buffer.from([v & 0xff, (v >> 8) & 0xff]);
const l4 = (v) => Buffer.from([v & 0xff, (v >>> 8) & 0xff, (v >>> 16) & 0xff, (v >>> 24) & 0xff]);

const zip = (entries) => {
  const locals = [];
  const central = [];
  let offset = 0;
  for (const [name, data] of entries) {
    const n = Buffer.from(name, "utf8");
    const crc = crc32(data);
    const head = Buffer.concat([
      Buffer.from("PK\x03\x04", "latin1"),
      l2(20),
      l2(0),
      l2(0),
      l2(0),
      l2(0), // version, flags, method=stored, time, date
      l4(crc),
      l4(data.length),
      l4(data.length),
      l2(n.length),
      l2(0),
      n,
    ]);
    locals.push(head, data);
    central.push(
      Buffer.concat([
        Buffer.from("PK\x01\x02", "latin1"),
        l2(20),
        l2(20),
        l2(0),
        l2(0),
        l2(0),
        l2(0),
        l4(crc),
        l4(data.length),
        l4(data.length),
        l2(n.length),
        l2(0),
        l2(0),
        l2(0),
        l2(0),
        l4(0),
        l4(offset),
        n,
      ]),
    );
    offset += head.length + data.length;
  }
  const cd = Buffer.concat(central);
  const eocd = Buffer.concat([
    Buffer.from("PK\x05\x06", "latin1"),
    l2(0),
    l2(0),
    l2(entries.length),
    l2(entries.length),
    l4(cd.length),
    l4(offset),
    l2(0),
  ]);
  return Buffer.concat([...locals, cd, eocd]);
};

// The jar is BUILT, never committed: `.jar` is git-ignored and
// scripts/audit-no-leak.sh fails on any tracked *.jar (Constraint 9). The
// round-trip imports `drawFixtureJar()` and serves the bytes from memory.
export const drawFixtureJar = () =>
  zip([
    ["META-INF/MANIFEST.MF", Buffer.from(MANIFEST, "utf8")],
    ["DrawMIDlet.class", drawMidlet()],
    ["DrawCanvas.class", drawCanvas()],
  ]);

// Run directly to drop the jar on disk (handy for `wie_validate <jar>`).
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "test_data", "draw_j2me.jar");
  writeFileSync(out, drawFixtureJar());
  console.log(`wrote ${out}`);
}
