#!/usr/bin/env python3
"""ktf-image-sweep.py — read a KTF client image: which interface slots does it
call, with what arguments, and what does the code around one call look like.

── Why this is in the repo at all ───────────────────────────────────────────
It was not. Until 2026-09-20 these three sweeps lived only under
`~/orchestrator/reports/evidence/…-p0/` (`slots.py`, `refs2.py`, `trace.py`) —
outside this repository, where nothing in `scripts/` or `git grep` can reach
them. ★This lineage was rejected once for exactly that shape: a `-fix` round
concluded "the classification rule is written down nowhere" when the rule was in
the ledger the repo cannot see.

★The cost is measured, not hypothetical, and the measurement is embarrassing:
the round that adopted this proposal's sibling (`#p0`, 2026-09-20) *did* find
the evidence directory, copied `slots.py` out of it — and then **re-implemented
`trace.py` from scratch** under another name because it did not notice that one
was already there. One round, one tool re-derived. That is the whole argument.

── What it is NOT ───────────────────────────────────────────────────────────
★Not a check. It has no failing state on findings, and CI cannot run it: it
needs a KTF client image, and those come out of `game_lab/`, which is git-ignored
real game bytes (Constraint 9). Same class as `scripts/smoke_gate.sh` and
`scripts/game-lab-recensus.sh` — local-only and structurally so, and therefore
expected to show up in `scripts/checker-census.mjs` with **zero callers**. A zero
there is a question, not a defect, and for this file the answer is this
paragraph.
★Exit 2 means "could not measure" (no capstone, unreadable image), never
"found nothing".

── Dependency, and how to run it when the import fails ──────────────────────
Python + `capstone`. ★Measured 2026-09-20: `capstone` is **not installed** for
any `python3` on this machine, so the obvious invocation fails at the import.
It is still the right dependency — a synchronised Thumb decoder is not something
to hand-roll — so the tool tells you the one-liner instead of dying obscurely:

    uv run --with capstone python3 scripts/ktf-image-sweep.py <sub> …

── The three sweeps, and what each is for ───────────────────────────────────
  slots  <img> <SL> [global|-] [win]   every indirect call through an interface
                                       table, for ALL slot offsets, with the
                                       global the table came from + arg summary
  refs   <img> <SL> <off,off,…>        who reads/writes given sl-relative globals
  window <img> <SL> <addr> [back]      synchronised Thumb window ENDING at addr,
                                       with pc-relative literals resolved

★`SL` is the image's sl base; find it in the fault dump (`[SP+0] == SL`).

── Limits. Read these before trusting a number ──────────────────────────────
★**Synchronised decode is the whole trick.** Thumb `BL` is a 4-byte pair, so
starting at an arbitrary even address lands inside one and every later
instruction is garbage. `preceding()` retries start offsets until the window
ends exactly on the target — do not "simplify" it into a single disasm call.

★**The `slots` argument-summary column is best effort, and it is WRONG in at
least one known place.** It walks a straight-line window with no register
liveness, so a later re-load of an argument register is missed. Measured
counterexample (image `01031C0A`, 2026-09-20): at the `slot 0` call `0x128f4a`
the column reports `r0=[glob 0x13ef54]` (the token `"ga"`), but reading the
window by hand shows `0x128f1e` loads `r5 = *(sl+0x6d8)` (`"/ga/aysis.dat"`) and
`0x128f42`'s `adds r0, r5, #0` overwrites the `r0` set at `0x128f36`. The real
argument is the path, not the token. ⇒ ★**quote the column as circumstance;
confirm anything load-bearing with `window`.**

★**Only two images have ever been swept** (`0103451A`, `01031C0A`). The veneer
detection, the `table = *(*global)` double-deref idiom and the sl-relative
back-tracking are all patterns read off those two. A third image may break any
of them, and the failure mode is a *plausible wrong answer*, not a crash.

★**Relocation is NOT handled here and must not be.** sl-relative pointer tables
are relocated by a per-image delta that this tool does not know; it prints the
static word. Derive the delta per image and cross-check it against an
independent value (the prior round derived `0103451A +0xE40` and confirmed it
against `R5` in the fault dump). ★Do not reuse a delta across images.
"""
import sys

IMAGE_BASE = 0x100000

try:
    from capstone import Cs, CS_ARCH_ARM, CS_MODE_THUMB
except ImportError:
    sys.stderr.write(
        "ktf-image-sweep: capstone is not importable for this python.\n"
        "  This is expected on this machine (measured 2026-09-20: no python3 here has it).\n"
        "  Run it with uv instead, which needs no install step:\n"
        "    uv run --with capstone python3 scripts/ktf-image-sweep.py " +
        " ".join(sys.argv[1:] or ["<sub>", "…"]) + "\n")
    raise SystemExit(2)


def die(msg):
    sys.stderr.write(f"ktf-image-sweep: {msg}\n")
    raise SystemExit(2)


def load(path):
    try:
        return open(path, "rb").read()
    except OSError as e:
        die(f"cannot read image {path}: {e}")


class Img:
    """Shared decode surface. ★The per-sweep string helpers are deliberately NOT
    merged into one: the three original sweeps read C strings with three
    different rules (bytes vs str, printable-required vs raw, truncate vs None),
    and flattening them would silently change their output."""

    def __init__(self, path):
        self.d = load(path)
        self.end = IMAGE_BASE + len(self.d)
        self.md = Cs(CS_ARCH_ARM, CS_MODE_THUMB)
        self.md.detail = True

    def u16(self, a):
        o = a - IMAGE_BASE
        return int.from_bytes(self.d[o:o + 2], "little") if 0 <= o <= len(self.d) - 2 else None

    def u32(self, a):
        o = a - IMAGE_BASE
        return int.from_bytes(self.d[o:o + 4], "little") if 0 <= o <= len(self.d) - 4 else None

    def cstr_raw(self, a, n, pad):
        """Bytes up to NUL, or None. `pad` = on no NUL, return the n-byte slice
        (trace.py's rule) rather than None (slots.py/refs2.py's rule)."""
        o = a - IMAGE_BASE
        if not (0 <= o < len(self.d)):
            return None
        e = self.d.find(b"\0", o, o + n)
        if e < 0:
            if not pad:
                return None
            e = o + n
        return self.d[o:e]

    def preceding(self, end, back):
        """★Synchronised: retry start offsets until the window ends exactly on
        `end`. Anything else desyncs on the 4-byte BL pair."""
        for start in range(max(IMAGE_BASE, end - back * 2), end, 2):
            got = list(self.md.disasm(self.d[start - IMAGE_BASE:end - IMAGE_BASE], start))
            if got and got[-1].address + got[-1].size == end:
                return got
        return []


def printable(s):
    return bool(s) and all(32 <= c < 127 for c in s)


# ── slots ────────────────────────────────────────────────────────────────────
def sub_slots(argv):
    if len(argv) < 2:
        die("slots <client.bin> <SL> [global_filter_hex|-] [window]")
    img = Img(argv[0])
    SL = int(argv[1], 0)
    GFILTER = int(argv[2], 0) if len(argv) > 2 and argv[2] != "-" else None
    WIN = int(argv[3]) if len(argv) > 3 else 3
    d, END, md = img.d, img.end, img.md
    u16, u32 = img.u16, img.u32

    def cstr(a, n=64):
        s = img.cstr_raw(a, n, pad=False)
        return s if printable(s) else None

    def preceding(end, back=40):
        return img.preceding(end, back)

    # Every 2-byte `bx rN` in the image is a possible ARM/ADS call veneer.
    veneers = {}
    for a in range(IMAGE_BASE, END - 1, 2):
        h = u16(a)
        if h is not None and (h & 0xFF87) == 0x4700:
            veneers[a] = (h >> 3) & 0xF
            veneers[a | 1] = (h >> 3) & 0xF

    def mem(ins):
        for o in ins.operands:
            if o.type == 3:
                return (ins.reg_name(o.mem.base) if o.mem.base else None, o.mem.disp, o.mem.index)
        return (None, 0, 0)

    def litval(ins):
        """Value of a `ldr rX,[pc,#k]` literal, or None."""
        b, disp, idx = mem(ins)
        if b != "pc" or idx:
            return None
        return u32(((ins.address + 4) & ~3) + disp)

    def resolve_global(win, rn):
        """Which global did the table pointer in `rn` come from?  Straight-line
        symbolic model over the window, same three shapes the prior sweep found."""
        val = {}      # reg -> literal constant
        isaddr = {}   # reg -> resolved absolute global address
        prov = {}     # reg -> origin global it was (transitively) loaded from
        for i in win:
            m = i.mnemonic
            ops = [x.strip() for x in i.op_str.split(",")]
            dst = ops[0] if ops else ""
            if m == "ldr" and litval(i) is not None:
                val[dst] = litval(i)
                isaddr.pop(dst, None)
            elif m in ("add", "adds") and len(ops) >= 2 and ops[-1] == "sl" and dst in val:
                isaddr[dst] = (val[dst] + SL) & 0xFFFFFFFF
            elif m == "mov" and len(ops) == 2 and ops[1] == "sl":
                val[dst] = 0
                isaddr[dst] = SL
            elif m in ("add", "adds") and len(ops) == 2 and ops[1] in val and dst in isaddr:
                isaddr[dst] = (isaddr[dst] + val[ops[1]]) & 0xFFFFFFFF
            elif m == "ldr":
                b, disp, idx = mem(i)
                if b and not idx and b in isaddr and disp == 0:
                    # first deref of an sl-relative global
                    prov[dst] = isaddr[b]
                    isaddr.pop(dst, None)
                    val.pop(dst, None)
                elif b and not idx and b in prov:
                    # chained deref: keep the ORIGIN global (the KTF idiom is
                    # table = *(*global) -- the global holds an object whose first
                    # word is the interface table, so one deref is one short)
                    prov[dst] = prov[b]
                    isaddr.pop(dst, None)
                    val.pop(dst, None)
                else:
                    isaddr.pop(dst, None)
                    val.pop(dst, None)
                    prov.pop(dst, None)
            else:
                for r in i.regs_access()[1]:
                    n = i.reg_name(r)
                    val.pop(n, None)
                    isaddr.pop(n, None)
                    prov.pop(n, None)
        return prov.get(rn)

    def arg_notes(win):
        """★Best effort: what r0/r1/r2 were set from, in the straight-line
        window. See the header's measured counterexample before quoting it."""
        out = {}
        val, isaddr = {}, {}
        for i in win:
            m = i.mnemonic
            ops = [x.strip() for x in i.op_str.split(",")]
            dst = ops[0] if ops else ""
            if m == "ldr" and litval(i) is not None:
                val[dst] = litval(i)
                isaddr.pop(dst, None)
            elif m in ("add", "adds") and len(ops) >= 2 and ops[-1] == "sl" and dst in val:
                isaddr[dst] = (val[dst] + SL) & 0xFFFFFFFF
            elif m in ("movs", "mov") and len(ops) == 2 and ops[1].startswith("#"):
                if dst in ("r0", "r1", "r2", "r3"):
                    out[dst] = f"#{int(ops[1][1:], 0)}"
                val.pop(dst, None)
                isaddr.pop(dst, None)
            elif m == "ldr":
                b, disp, idx = mem(i)
                if dst in ("r0", "r1", "r2", "r3") and b in isaddr and not idx and disp == 0:
                    g = isaddr[b]
                    v = u32(g)
                    s = cstr(v + IMAGE_BASE) if v is not None and v < 0x100000 else (cstr(v) if v else None)
                    out[dst] = f"[glob {g:#x}]" + (f' (static -> {v:#x}{"" if not s else chr(32) + repr(s.decode())})' if v is not None else "")
                elif dst in ("r0", "r1", "r2", "r3"):
                    out[dst] = f"[{b}{'' if not disp else f',#{disp:#x}'}]"
                val.pop(dst, None)
                isaddr.pop(dst, None)
        return out

    hits = []
    # ── ★The global filter must not swallow "could not measure" ─────────────
    # `g is None` means `resolve_global` walked off the end of its straight-line model,
    # NOT that the table came from a different global. Dropping those under a filter turns
    # "unmeasured" into "absent", which is the one thing this repo keeps having to relearn.
    # ★It already cost a real finding: 2026-09-20, `01031C0A`'s Open at `0x128f78` vanished
    # under `filter=0x13eb30` and only came back with the filter off, where it reads
    # `via global UNRESOLVED`. A gate② reviewer found it by hand.
    # ★The two reasons are counted SEPARATELY on purpose — one is a verdict, the other is a
    # gap — and neither is mixed into `hits`, so anything parsing the `slot +…` lines is
    # untouched (`docs/report/0204`).
    dropped_other = 0
    dropped_unresolved = 0
    for a in range(IMAGE_BASE, END - 1, 2):
        h = u16(a)
        if h is None or (h & 0xF800) != 0x6800:
            continue
        imm5 = (h >> 6) & 0x1F
        if imm5 > 16:
            continue
        rn = (h >> 3) & 7
        rt = h & 7
        win = preceding(a + 2, 40)
        if not win or win[-1].address != a or win[-1].mnemonic != "ldr":
            continue
        # forward: is rT called within WIN instructions, without being rewritten?
        fwd = list(md.disasm(d[a + 2 - IMAGE_BASE: a + 2 + 2 * WIN * 2 - IMAGE_BASE], a + 2))
        tname = f"r{rt}"
        called = None
        for j in fwd[:WIN]:
            if j.mnemonic in ("blx", "bx") and j.op_str.strip() == tname:
                called = j.address
                break
            if j.mnemonic == "bl":
                tgt = int(j.op_str.replace("#", ""), 0)
                if veneers.get(tgt) == rt or veneers.get(tgt | 1) == rt:
                    called = j.address
                    break
            if tname in [j.reg_name(r) for r in j.regs_access()[1]]:
                break
        if called is None:
            continue
        g = resolve_global(win[:-1], f"r{rn}")
        if GFILTER is not None and g != GFILTER:
            if g is None:
                dropped_unresolved += 1
            else:
                dropped_other += 1
            continue
        hits.append((a, called, imm5 * 4, rn, rt, g, arg_notes(win[:-1])))

    print(f"# {argv[0]}  SL={SL:#x}  veneers={len(veneers) // 2}  hits={len(hits)}  filter={GFILTER and hex(GFILTER)}")
    if GFILTER is not None:
        print(f"#   filter dropped: {dropped_other} with a DIFFERENT global · {dropped_unresolved} UNRESOLVED")
        if dropped_unresolved:
            # ★Said, not swallowed — and with the one command that gets them back. The count
            # alone is the whole prescription: 331 of 501 hits are UNRESOLVED on this image
            # (279/364 on the other), so printing them all would bury the filtered list under
            # the noise the filter exists to remove.
            print(
                f"#   ★those {dropped_unresolved} are NOT 'a different global' — they are 'could not resolve'."
                f"  Re-run with '-' as the filter and grep UNRESOLVED to see them."
            )
    for a, called, off, rn, rt, g, notes in sorted(hits, key=lambda x: (x[5] or 0, x[2], x[0])):
        gs = f"{g:#x}" if g is not None else "UNRESOLVED"
        args = "  ".join(f"{k}={v}" for k, v in sorted(notes.items()))
        print(f"  slot +{off:#04x} (={off // 4:2d})  ldr@{a:#x} call@{called:#x}  base=r{rn} via global {gs}   {args}")
    return 0


# ── refs ─────────────────────────────────────────────────────────────────────
def sub_refs(argv):
    """Sweep MANY sl-offsets in one pass, and also catch `str rY,[rX,#disp]`
    writes where the base was `sl + L` (so the reported global is L+disp, not L)."""
    if len(argv) < 3:
        die("refs <client.bin> <SL> <off,off,…>")
    img = Img(argv[0])
    SL = int(argv[1], 0)
    TARGETS = {int(x, 0) for x in argv[2].split(",")}
    FWD = 8
    d, END, md = img.d, img.end, img.md
    u16, u32 = img.u16, img.u32

    MINT, MAXT = min(TARGETS), max(TARGETS)
    out = []
    for a in range(IMAGE_BASE, END - 1, 2):
        h = u16(a)
        if h is None or (h & 0xF800) != 0x4800:
            continue
        rt = (h >> 8) & 7
        lit = ((a + 4) & ~3) + ((h & 0xFF) * 4)
        L = u32(lit)
        if L is None or not (MINT - 0x80 <= L <= MAXT):
            continue
        win = img.preceding(a + 2, 24)
        if not win or win[-1].address != a or win[-1].mnemonic != "ldr":
            continue
        fwd = list(md.disasm(d[a + 2 - IMAGE_BASE: a + 2 + FWD * 4 - IMAGE_BASE], a + 2))[:FWD]
        reg = f"r{rt}"
        saw_add = False
        for j in fwd:
            ops = [x.strip() for x in j.op_str.split(",")]
            if j.mnemonic in ("add", "adds") and ops and ops[0] == reg and ops[-1] == "sl":
                saw_add = True
                continue
            if not saw_add:
                if ops and ops[0] == reg:
                    break
                continue
            if j.mnemonic.startswith(("str", "ldr")) and len(ops) >= 2:
                memops = [o for o in j.operands if o.type == 3]
                if memops and j.reg_name(memops[0].mem.base) == reg and not memops[0].mem.index:
                    g = L + memops[0].mem.disp
                    if g in TARGETS:
                        out.append((a, "WRITE" if j.mnemonic.startswith("str") else "read", g, j.address, j.mnemonic, j.op_str))
                    break
                if ops[0] == reg:
                    break
            elif ops and ops[0] == reg:
                break

    print(f"# {argv[0]} SL={SL:#x} targets={[hex(t) for t in sorted(TARGETS)]}")
    for a, kind, g, ja, m, ops in sorted(out, key=lambda x: (x[2], x[0])):
        print(f"  SL+{g:#05x} ({SL + g:#x})  {kind:<5} ldr@{a:#x}  -> {ja:#x}: {m} {ops}")
    print(f"# {len(out)} reference(s)")
    return 0


# ── window ───────────────────────────────────────────────────────────────────
def sub_window(argv):
    """Print a synchronised Thumb window ENDING at a given address, resolving
    pc-relative literal loads and dereferencing guest addresses."""
    if len(argv) < 3:
        die("window <client.bin> <SL> <addr> [back_instrs]")
    img = Img(argv[0])
    SL = int(argv[1], 0)
    END = int(argv[2], 0)
    back = int(argv[3]) if len(argv) > 3 else 32

    for i in img.preceding(END, back):
        note = ""
        for o in i.operands:
            if o.type == 3 and o.mem.base and i.reg_name(o.mem.base) == "pc":
                lit = (i.address + 4) & ~3
                lit += o.mem.disp
                v = img.u32(lit)
                note = f"   ; literal@{lit:#x} = {v:#x}"
                if v is not None:
                    s = img.cstr_raw(v, 40, pad=True)
                    if printable(s):
                        note += f' -> "{s.decode()}"'
                    note += f"  (+SL = {(v + SL) & 0xFFFFFFFF:#x})"
        print(f"  {i.address:#08x}: {i.mnemonic:<8} {i.op_str}{note}")
    return 0


SUBS = {"slots": sub_slots, "refs": sub_refs, "window": sub_window}


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        sys.stdout.write(__doc__)
        return 0
    sub = SUBS.get(argv[0])
    if sub is None:
        die(f"unknown sub-command {argv[0]!r} — one of: {', '.join(SUBS)}")
    return sub(argv[1:])


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
