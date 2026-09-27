# classfile 0.1.1 — wie patch

Vendored copy of crates.io [`classfile` 0.1.1](https://crates.io/crates/classfile/0.1.1)
(dlunch/RustJava, path `classfile`; the crate's `.cargo_vcs_info.json` names `78ffcd7d`, which is
not on the public repo — the source here is the published tarball). Applied by the workspace
root's `[patch.crates-io]`. `LICENSE` is RustJava's MIT notice.

**One change**: `src/constant_pool.rs` `parse_utf8` decodes `CONSTANT_Utf8` as *modified* UTF-8
(JVMS §4.4.7) instead of `String::from_utf8`. Without it a class holding `"\0"` (C0 80) or a
supplementary character (a surrogate pair, six bytes) in any constant fails with
`ClassFormatError: Invalid class file` — measured on the first Pebble Snake build
(`docs/report/0327`). Everything else is byte-identical to the tarball; `diff -r` against
`~/.cargo/registry/src/*/classfile-0.1.1/src` shows only that function, its imports and its tests.

The tarball's own `tests/test.rs` is not carried: it reads `../../test-data/*.class`, which exists
only in the RustJava repo.

**Drop the patch** when a released `classfile` passes the four tests at the bottom of
`constant_pool.rs` — delete this directory, its `members` entry and the `[patch.crates-io]` line.
Upstream `main` (`a8bc80e2`, 2026-09-28) still uses `String::from_utf8`; the fix is small enough to
offer upstream as a PR, which is not this repo's to push.
