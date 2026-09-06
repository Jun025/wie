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
// And it OPENS A RESOURCE BY NAME: startApp() calls Image.createImage(String)
// on a bundled PNG and paints a rect of the dimensions the host reported. Same
// trick, different question — the pixel count says what the class loader found,
// not merely that nothing threw (Scenario C-img). See IMG_RESOURCE_NAME below
// for why that one call site is worth a fixture.
//
// Usage: imported by scripts/contract-roundtrip.mjs; run it directly to write
// test_data/draw_j2me.jar for `wie_validate` (the jar itself is never committed
// — *.jar is git-ignored and the leak audit rejects tracked ones).

import { writeFileSync } from "node:fs";
import { crc32, deflateSync } from "node:zlib";
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
  string(s) {
    const u = this.utf8(s);
    return this.#add(`s:${s}`, Buffer.concat([Buffer.from([8]), u2(u)]));
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
//
// `handlers` is the Code attribute's exception_table: one {startPc, endPc,
// handlerPc, catchType} per entry, catchType being a CONSTANT_Class index (0
// would mean "any", which we never want — see the catch-narrowly note in
// drawMidlet). Offsets are byte offsets into `code`; the caller computes them
// from the length of the buffers it concatenated, so a re-ordered instruction
// cannot silently point the handler at the wrong pc.
const method = (cp, name, desc, maxStack, maxLocals, code, handlers = []) => {
  const table = Buffer.concat(handlers.map((h) => Buffer.concat([u2(h.startPc), u2(h.endPc), u2(h.handlerPc), u2(h.catchType)])));
  const body = Buffer.concat([u2(maxStack), u2(maxLocals), u4(code.length), code, u2(handlers.length), table, u2(0)]);
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

// ── The bundled image resource (Scenario C-img) ──────────────────────────────
// startApp() opens this BY NAME through Image.createImage(String) and stores the
// dimensions the host reported; paint() then fills a rect of exactly those
// dimensions. So the bar's pixel count IS the decoded image's pixel count —
// "the guest asked for a resource by name and got back a 16x8 image" becomes a
// number the round-trip can assert, not a "no exception was thrown".
//
// Why that call and not any other: createImage(String) is the ONE site of the
// six in the get_system_class_loader migration whose behaviour actually CHANGED
// (docs/upstream-realign-verdict.md §8-4(3)-b). Through current_class_loader the
// calling class while the Rust proto runs is `Image` itself, defined by
// RustJarClassLoader, whose findResource is the base ClassLoader's `Ok(None)`
// and whose parent is None — it could never resolve a guest resource, so the
// call threw IOException every time. The system URLClassLoader (parent =
// RustJar + the guest jar URLs) resolves it. Before this fixture nothing in the
// repo exercised either half: a planted panic!() at image.rs left the suite and
// all five fixtures green (measured 2026-09-05).
export const IMG_RESOURCE_NAME = "/wie-img.png"; // leading slash = MIDP convention; the jar branch of URLClassLoader::findResource trims it
// The two failure branches. `MISSING` is deliberately NOT a jar entry; `BROKEN`
// IS one, holding bytes that are not any image format — so the two exercise the
// two different arms (resource lookup vs. decode) and cannot be confused.
export const IMG_MISSING_NAME = "/wie-absent.png";
export const IMG_BROKEN_NAME = "/wie-broken.png";
export const IMG_ERR_MISSING = "imgerr:missing";
export const IMG_ERR_BROKEN = "imgerr:broken";
export const IMG_W = 16;
export const IMG_H = 8;
export const IMG_BAR_Y = 48; // below the key bar (32..40), so no rect ever overlaps another
export const IMG_RECT_PX = IMG_W * IMG_H;

// Total non-black pixels once the image bar is up and one key of `midpCode` has
// been delivered. Exported so the expectation lives in ONE place — the fixture
// that draws it (scripts/contract-roundtrip.mjs imports this, never restates it).
export const keyBarPixels = (midpCode) => BASE_RECT_PX + IMG_RECT_PX + midpCode * KEY_BAR_H;

// Minimal opaque-white RGB PNG, emitted byte-wise for the same reason the class
// files are: the repo (and CI) has no image tooling, and a committed binary
// would be one more untracked-by-anything blob. `decode_image` guesses the
// format, so any real PNG works; this is the smallest one that is real.
const pngChunk = (type, data) => {
  const body = Buffer.concat([Buffer.from(type, "latin1"), data]);
  return Buffer.concat([u4(data.length), body, u4(crc32(body) >>> 0)]);
};
const png = (w, h) => {
  const ihdr = Buffer.concat([u4(w), u4(h), Buffer.from([8, 2, 0, 0, 0])]); // 8-bit, colour type 2 (RGB), no interlace
  const scanline = Buffer.concat([Buffer.from([0]), Buffer.alloc(w * 3, 0xff)]); // filter 0 + white pixels
  const raw = Buffer.concat(Array.from({ length: h }, () => scanline));
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
};

const CANVAS = "javax/microedition/lcdui/Canvas";
const GRAPHICS = "javax/microedition/lcdui/Graphics";
const DISPLAY = "javax/microedition/lcdui/Display";
const DISPLAYABLE = "javax/microedition/lcdui/Displayable";
const MIDLET = "javax/microedition/midlet/MIDlet";
const IMAGE = "javax/microedition/lcdui/Image";

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
  const imgW = cp.field("DrawCanvas", "imgW", "I");
  const imgH = cp.field("DrawCanvas", "imgH", "I");
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

  // if (imgW != 0) g.fillRect(0, IMG_BAR_Y, imgW, imgH)
  // The width AND height come from the Image the host handed back, so this rect
  // can only have IMG_W*IMG_H pixels if createImage(String) actually resolved
  // the jar entry and decode_image produced those dimensions.
  const imgBar = Buffer.concat([
    Buffer.from([0x2b, 0x03]), // aload_1 (Graphics), iconst_0 (x = 0)
    Buffer.from([0x10, IMG_BAR_Y]), // bipush  (y)
    Buffer.from([0xb2]),
    u2(imgW), // getstatic imgW  (width  = what getWidth() reported)
    Buffer.from([0xb2]),
    u2(imgH), // getstatic imgH  (height = what getHeight() reported)
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
    Buffer.from([0xb2]),
    u2(imgW), // getstatic imgW
    Buffer.from([0x99]),
    u2(3 + imgBar.length), // ifeq → skip the image bar (still 0 if startApp has not stored yet)
    imgBar,
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
    [staticIntField(cp, "keyHit"), staticIntField(cp, "imgW"), staticIntField(cp, "imgH")],
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
  const resourceName = cp.string(IMG_RESOURCE_NAME);
  const createImage = cp.method(IMAGE, "createImage", `(Ljava/lang/String;)L${IMAGE};`);
  const getWidth = cp.method(IMAGE, "getWidth", "()I");
  const getHeight = cp.method(IMAGE, "getHeight", "()I");
  const imgW = cp.field("DrawCanvas", "imgW", "I");
  const imgH = cp.field("DrawCanvas", "imgH", "I");

  const init = Buffer.concat([Buffer.from([0x2a]), Buffer.from([0xb7]), u2(superInit), Buffer.from([0xb1])]);

  // Open the bundled resource BY NAME and remember the dimensions the host
  // reported. The exception table below guards only the two failure-branch
  // calls, never THIS one, so a throw here propagates out of startApp.
  // MEASURED (2026-09-06), because
  // an earlier version of this comment guessed and guessed wrong: it does NOT
  // degrade to "the base rect only". net/wie/Launcher.startMIDlet does not
  // swallow it, so the boot aborts and wie_validate reports
  //   FAIL · paints 0 · content false
  // with the Java stack naming Image.createImage(String) ← DrawMIDlet.startApp.
  // That is the pre-migration behaviour reproduced exactly: reverting image.rs
  // to jvm.current_class_loader() yields
  //   java.io.IOException: Resource not found: /wie-img.png
  // Loud is the right failure mode here; the point of the fixture is that this
  // call site used to be unobservable.
  const loadImage = Buffer.concat([
    Buffer.from([0x13]),
    u2(resourceName), // ldc_w "/wie-img.png"
    Buffer.from([0xb8]),
    u2(createImage), // invokestatic Image.createImage(String)
    Buffer.from([0x59]), // dup
    Buffer.from([0xb6]),
    u2(getWidth), // invokevirtual getWidth()I
    Buffer.from([0xb3]),
    u2(imgW), // putstatic DrawCanvas.imgW
    Buffer.from([0xb6]),
    u2(getHeight), // invokevirtual getHeight()I
    Buffer.from([0xb3]),
    u2(imgH), // putstatic DrawCanvas.imgH
  ]);

  // ── The two FAILURE branches of the same call ─────────────────────────────
  // The success path above locks "the host found the resource". These lock what
  // the guest RECEIVES when it does not — which no fixture said anything about
  // until now, so either branch could change type or stop throwing and every
  // check stayed green.
  //
  // The two branches throw DIFFERENT types, read from the host source, not
  // guessed (wie_midp/.../lcdui/image.rs):
  //   • name not in the jar  -> java/io/IOException "Resource not found: {name}"
  //     (create_image_from_name, the `None =>` arm)
  //   • bytes are not an image -> java/lang/IllegalArgumentException
  //     "Failed to decode image" (create_image_from_data, the decode_image err arm)
  //
  // CATCH NARROWLY, on purpose. Each handler names its exact class rather than
  // java/lang/Throwable (catch_type 0 = "any" is never used here), so the TYPE
  // is part of what the fixture locks: if a branch starts throwing something
  // else, the handler no longer catches, the throw leaves startApp, and the boot
  // aborts — a loud red, not a marker that silently keeps meaning "fine".
  // Symmetrically, if a branch stops throwing at all, control falls through the
  // `goto` and the marker is never printed — also red, via the stdout assertion.
  const ioException = cp.class_("java/io/IOException");
  const illegalArgument = cp.class_("java/lang/IllegalArgumentException");
  const sysOut = cp.field("java/lang/System", "out", "Ljava/io/PrintStream;");
  const println = cp.method("java/io/PrintStream", "println", "(Ljava/lang/String;)V");
  const missingName = cp.string(IMG_MISSING_NAME);
  const brokenName = cp.string(IMG_BROKEN_NAME);
  const missingMark = cp.string(IMG_ERR_MISSING);
  const brokenMark = cp.string(IMG_ERR_BROKEN);

  // try { createImage(name); } catch (<type>) { System.out.println(mark); }
  // 7 bytes of guarded body, 3 of goto, 10 of handler = 20 per branch.
  const attempt = (nameIdx) =>
    Buffer.concat([
      Buffer.from([0x13]),
      u2(nameIdx), // ldc_w <resource name>
      Buffer.from([0xb8]),
      u2(createImage), // invokestatic Image.createImage(String)
      Buffer.from([0x57]), // pop — a returned Image is not the point here
    ]);
  const report = (markIdx) =>
    Buffer.concat([
      Buffer.from([0x4c]), // astore_1 — the caught exception (handler entry stack = [exc])
      Buffer.from([0xb2]),
      u2(sysOut), // getstatic System.out
      Buffer.from([0x13]),
      u2(markIdx), // ldc_w "imgerr:…"
      Buffer.from([0xb6]),
      u2(println), // invokevirtual println(String)
    ]);
  const GUARDED = 7; // attempt()
  const HANDLER = 10; // report()
  const SKIP = 3; // goto
  const branch = (nameIdx, markIdx) => Buffer.concat([attempt(nameIdx), Buffer.from([0xa7]), u2(GUARDED + SKIP + HANDLER - GUARDED), report(markIdx)]);

  const prologue = Buffer.concat([
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
    loadImage,
  ]);

  // Offsets are DERIVED from the buffers, never written by hand: a handler that
  // points at the wrong pc is the classic way an exception table goes silently
  // wrong (it catches, jumps into the middle of an instruction, and the JVM
  // reports something unrelated).
  const b1 = prologue.length;
  const b2 = b1 + GUARDED + SKIP + HANDLER;
  const startApp = Buffer.concat([
    prologue,
    branch(missingName, missingMark),
    branch(brokenName, brokenMark),
    Buffer.from([0xb1]), // return
  ]);
  const handlers = [
    { startPc: b1, endPc: b1 + GUARDED, handlerPc: b1 + GUARDED + SKIP, catchType: ioException },
    { startPc: b2, endPc: b2 + GUARDED, handlerPc: b2 + GUARDED + SKIP, catchType: illegalArgument },
  ];

  // max_locals 2: the handlers astore_1 the caught exception (local 0 is `this`).
  return classFile(cp, "DrawMIDlet", MIDLET, [method(cp, "<init>", "()V", 1, 1, init), method(cp, "startApp", "()V", 3, 2, startApp, handlers)]);
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
    // The resource startApp() opens by name. Entry name has no leading slash;
    // the jar branch of URLClassLoader::findResource trims the one the guest
    // passes (`name_str.trim_start_matches('/')` at the pinned rev).
    [IMG_RESOURCE_NAME.replace(/^\//, ""), png(IMG_W, IMG_H)],
    // Present but undecodable — the decode arm's input. NOT a truncated PNG: a
    // valid signature with a broken body could plausibly be handled by some
    // future partial decoder, and then this branch would quietly stop testing
    // anything. Bytes that match no format at all keep the branch honest.
    // (IMG_MISSING_NAME is absent from this list on purpose — that IS its test.)
    [IMG_BROKEN_NAME.replace(/^\//, ""), Buffer.from("not an image", "utf8")],
  ]);

// Run directly to drop the jar on disk (handy for `wie_validate <jar>`).
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "test_data", "draw_j2me.jar");
  writeFileSync(out, drawFixtureJar());
  console.log(`wrote ${out}`);
}
