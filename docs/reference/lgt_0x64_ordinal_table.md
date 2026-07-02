# LGT ez-i `0x64` java-interface — ordinal → native (status)

Source: reverse-engineering of the AromaWIPI WIPI SDK 1.1.1.8 (`WIPIEmul.exe`,
`WIPIstub4java.obj`) cross-checked against wie's own LGT ARM traces
(`wie_lgt/src/runtime/java/interface.rs`, `docs/lgt_abi.md` §6, and the game-side
arg-shape triangulation). Facts only — no vendor source is reproduced.

**Bottom line:** the AromaWIPI SDK confirms the *ABI shape* and the *name surface* of the
ez-i java-interface, but its native table is **not co-numbered** with LGT's `0x64` ordinal.
The ordinal→native mapping for the render-relevant PENDING indices therefore remains
**unresolved**; no new ordinal reached HIGH confidence and none was bound.

## What the SDK confirms (ABI shape)

- The AOT-Java client is handed a **single flat function-pointer table `nEnv`** plus a
  `newArrayTypeClass` helper (`WIPIstub4java.obj: __setnEnv(nEnv, newArrayTypeClass)`).
  This matches wie's `get_import_function(0x64, index)` → flat ordinal model exactly.
- In `WIPIEmul.exe` the table is a ~236-entry `.text` pointer array in `.data`
  (accessor returns base `0x519b5c`). Entries were disassembled and characterised
  (object allocation, class resolution, `Shared.shBuf` accessors, `getProperty`, UTF-16
  string routines). Details: `game_lab/reports/EZI_REGISTRY_DUMP_2026-07-02.md`.

## What the SDK does NOT give (numbering)

AromaWIPI's `nEnv` indices do not equal LGT's `0x64` indices. Proof (base-independent):
LGT calls index `0x0e` as `(1|2, 0, size)` — a numeric typed size-allocation — while
AromaWIPI's index `0x0e` is `findClass(char* name)` (dereferences a name string, tests the
`'['` array prefix). Different primitives, incompatible argument types. See EZI_REGISTRY_DUMP §5.

Reason: AromaWIPI is SKT-general WIPI; LGT ships ez-i (Xceed AOT toolchain). Same standard
surface, vendor-specific ordinal assignment.

## Ordinal status table (unchanged confirmations + this session's cross-check)

| idx | wie status (LGT ARM RE) | AromaWIPI cross-check | confidence |
|---|---|---|---|
| `0x03 0x06 0x07 0x14 0x82 0x83` | BOOT (implemented, §3) | — | confirmed (wie) |
| `0x09` | String factory `(ctx,utf16,count,out)` (cp10) | AromaWIPI string routines exist but at different indices (~`0x78+`) | confirmed (wie); numbering not shared |
| `0x0c` | getInstance(handle) (cp20) | AromaWIPI `0x0c` = `newArray` | confirmed (wie); numbering not shared |
| `0x0f` | native `new` (cp8) | AromaWIPI `0x0f` = `newObject` — **same primitive, same index** | confirmed (wie), corroborated in shape |
| `0x54` | method-entry/safepoint helper, no-op | — | confirmed (wie) |
| `0x0b 0x0d` | exception push/pop, no-op-safe | AromaWIPI has GC/alloc helpers nearby | confirmed (wie) |
| `0x0e` | PENDING `(1\|2,0,size)→handle` (render-relevant) | AromaWIPI `0x0e` = `findClass(name)` — **diverges** | still PENDING |
| `0x10` | PENDING `(0,idx,n)→field` | AromaWIPI `0x10` = table accessor — diverges | still PENDING |
| `0x12` | PENDING query w/ out-buffer | — | still PENDING |
| `0x1f` | PENDING hot per-frame native (16385×) | AromaWIPI `0x1f` = `Shared.shBuf` accessor (inconclusive) | still PENDING |
| `0x21` | pushCard-family / render-driver object register (§7) | — | render-driver: unresolved |
| `0x22` | PENDING font/image resource | — | still PENDING |
| `0x55 0x56 0x57` | carried-code / callback registration (render driver, §7) | not present as such in AromaWIPI nEnv | unresolved |

## Static confirmation from the app binaries (2026-07-02)

The `0x64` import *surface* is now recovered directly and authoritatively from the LGT
`binary.mod` AOT thunks — no firmware, SDK, or runtime trace needed. Each import is a fixed
16-byte thunk `str lr,[sp,#-4]!; bl <dispatcher>; .word table; .word index`; scanning for it
yields `(table, index)` statically (`game_lab/reports/AOT_THUNK_EXTRACTION_2026-07-02.md`).

- **24 of 102 LGT titles are AOT-Java** (carry table `0x64`); the other 78 are WIPI-C clets
  (table `0x1fb`, driven fine by wie's `SVC_CATEGORY_WIPIC`). So `0x64` only matters for those 24.
- **Index set, frequency across the 24 AOT titles** — core (24/24):
  `03 06 07 09 0b 0c 0d 0e 0f 10 12 13 14 1f 20 21 22 23 54 55 61 82 83 e1 fa`;
  near-universal: `11 25 e2`(23) `40 64`(22); optional: `26`(9) `38`(10) `56 57`(14) `5b fd`(5).
- **Method validated:** the static set is a strict superset of the runtime trace
  (`ORDINAL_TRIANGULATION_2026-07-01`) with **zero contradictions** — every dynamically
  observed index (`0b 0d 0e 10 11 12 1f 21 22 23 55 e2 fa`) is present statically.
- **The render-relevant PENDING indices `0e/10/12/1f/22` are universal** (24/24 or 23/24),
  i.e. core runtime primitives — not per-game quirks that could be no-op'd away.

This confirms index *distribution* and arg-shape but still not the index↔native *name*: the
registration-order table remains the one missing artifact, and the app binaries are now
exhausted for names (naming from index-value/arg-shape/frequency alone is forbidden).

## Next primary source needed

The app binaries are now exhausted: the `binary.mod` thunks give the full `0x64` *index*
surface (above) but no names. The only remaining authoritative *naming* source is what a real
device does with each index — the ez-i **runtime resolver** `get_import_function(0x64, index)`
in LGT firmware, the Xceed AOT link/symbol map, or a device trace logging `0x64(index)→callee`.
AromaWIPI x86 runs interpreted clets, so it never indexes `nEnv` observably, and its numbering
differs regardless. Until such a source exists, the render-relevant PENDING indices stay no-op
in `wie_lgt` (binding a guessed handle would crash — guardrail).
