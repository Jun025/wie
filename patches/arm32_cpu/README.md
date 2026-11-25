# arm32_cpu 0.1.2 — wie patch

Vendored copy of crates.io [`arm32_cpu` 0.1.2](https://crates.io/crates/arm32_cpu/0.1.2)
(dlunch/arm32_cpu @ `7488118a`, per the tarball's `.cargo_vcs_info.json`; the source here is the
published tarball, the one `Cargo.lock` pinned before this patch). Applied by the workspace root's
`[patch.crates-io]` and listed in its `members`, so `cargo test --all` / clippy / fmt cover it.
`LICENSE` is the crate's MIT notice.

**The change**: `src/thumb.rs` `Instruction::decode` is one lookup in a 256-entry table on the top
byte, instead of matching `INST_MATCH_ORDER`'s 21 rows in order (7.25 rows per instruction on
average; that decode was 24.8–26.3% of the ARM run loop — `docs/report/0575` §2–§3). Every row but
`Nop` masks only the top byte, and `Nop` is the single word `0xbf00`. The old walk stays as the
test oracle `decode_linear`; `decode_table_matches_linear_match_for_every_word` compares the two on
all 65,536 Thumb words, so equal decode is proven exhaustively, not sampled.

**Carried along, not behaviour**:
- The optional `serde-serialize` / `advanced_disasm` features and their `cfg` blocks are removed.
  Nothing here enables them, but as a workspace member their dependencies (`capstone` 0.6 with its
  C library, `serde-big-array` 0.3) would enter `Cargo.lock` — `cargo audit` scans them and
  `coverage.yml`'s `--all-features` would build them.
- One `#![allow(...)]` in `src/lib.rs` names the tarball's own clippy/rustc findings. A registry
  crate is never linted (cargo caps dependency lints); a member is, with `-D warnings`.
- `pub const WIE_PATCH` in `src/lib.rs` — the guard below.

Diff against the tarball: `diff -r ~/.cargo/registry/src/*/arm32_cpu-0.1.2/src patches/arm32_cpu/src`.

**The guard.** A `[patch.crates-io]` entry is used only while its version satisfies the dependents'
requirement (`wie-core-arm/Cargo.toml`: `^0.1.2`). If an upstream merge raises that to a version
this directory does not have, cargo prints one `patch ... was not used` warning and builds the
registry crate — green, and without the table. `wie-core-arm/src/engine/arm32_cpu.rs` names
`arm32_cpu::WIE_PATCH`, which only this copy exports, so that fallback is a compile error instead.
When it fires: re-vendor the new release here and re-apply the change (the table, the oracle test,
the marker), or drop the patch as below.

**Drop the patch** when a released `arm32_cpu` decodes Thumb without the linear walk — delete this
directory, its `members` entry, the `[patch.crates-io]` line and the `WIE_PATCH` line in
`wie-core-arm`. Upstream 0.1.3 (2026-11-24) does not: it adds Thumb-2 `B.W` and still walks the
table. Moving to 0.1.3 is a behaviour change (a word that was `Undefined` now branches) and a
separate round from this one. Offering the change upstream is not this repo's to push.
