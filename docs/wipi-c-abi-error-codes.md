# `wie_wipi_c` return codes that are outside the `WIPICError` vocabulary

Four host functions return an `i32` that is not a `WIPICError` variant. **All four are deliberate
and stay as they are.** This file is the census and the reasoning; the four sites carry a one-line
comment pointing here. Read this before "cleaning up" any of them.

## The vocabulary, and why "outside it" is not automatically a defect

At the pinned rev (`Cargo.lock` → `git+https://github.com/dlunch/wipi.git#1e0c634e…`,
`wipi_types/src/wipic.rs`):

```rust
#[repr(i32)]
pub enum WIPICError { ImageDone = 1, Success = 0, Invalid = -9, NoSuchEntry = -12,
                      InsufficientBufferSize = -18, BadRecordId = -22, InvalidHandle = -25 }
impl WIPICError { pub fn from_raw(raw: i32) -> Self { unsafe { core::mem::transmute(raw) } } }
```

`from_raw` is a `transmute`, so handing it a value that is not a variant is an invalid discriminant
— UB. **That is what makes a code "outside the vocabulary" dangerous, and it only applies on calls
that actually pass through it.** Measured at that rev, `WIPICError::from_raw` has exactly five
call sites in `wipic_sys`, covering **two** host functions:

| host fn | wrappers | reachable via `from_raw` |
|---|---|---|
| `kernel::get_resource` | ktf · lgt · simulation | **yes** |
| `graphics::create_image` | ktf · lgt (simulation builds the enum by match) | **yes** |
| everything else | — | no |

`get_resource` returned `-1` until 2026-09-07 and was changed to `-18` for exactly this reason
(`docs/report/0069--…`). `create_image` only ever returns `1`, which is in the vocabulary. **So the
transmute path is clean, and the four sites below are off it.**

## The four sites, and the disposition of each

Each was measured 2026-09-07: how the value reaches the guest, whether it is on the transmute path,
and whether anything inside `wie_wipi_c` branches on it. **Internal branch readers: 0 for all four**
(the only in-crate references are the definitions themselves, a `tracing::debug!` string, and one
test that asserts the success path).

| # | site | value | reaches the guest as | disposition |
|---|---|---|---|---|
| 1 | `net::socket_close` | `-1` (`M_E_ERROR`) | raw `i32` — dispatched by ktf + lgt, no `wipic_sys` wrapper | keep |
| 2 | `database::seek_record_single`, unknown `origin` | `-1` | raw `i32` — lgt only (`WIPICSvcId::Unk12`) | keep |
| 3 | `database::stream_read`, past EOF | `-23` (`M_E_EOF`) | raw `i32` — ktf + lgt | keep |
| 4 | `media::clip_put_data`, null clip | `-1` | `wipic_sys` wrapper: `if result < 0 { Err(MediaError::Platform(result)) }` | keep |

**Why keep, per site — the reasons are different, so do not collapse them into one rule.**

1. **`socket_close` is a stub** (it logs `stub MC_netSocketClose` and does nothing). `-1` means "this
   call failed"; the vocabulary has no generic-failure variant, and every specific one would be a
   *claim* the stub cannot honour — `Invalid` would blame the caller's arguments, `InvalidHandle`
   would blame the fd. Picking one to satisfy a lint would make the code say something false.
2. **`seek_record_single`'s unknown-`origin` arm is the one case where the vocabulary does fit**:
   `origin ∉ {0,1,2}` is precisely `Invalid = -9`, and the same function already returns `-25` for a
   bad handle while `list_record_info` next door returns `-22`. It is still left alone, and that is
   the deliberate part: changing it is a **behaviour change whose compatibility cannot be checked
   here** (no commercial corpus — Constraint 9 — and no WIPI error-code spec in this repo or the
   pinned wipi repo; `M_E_*` has zero definitions in either, the names in these files are bare
   comments). 2026-09-07 paid that unmeasurable price once, at `get_resource`, **to remove a definite
   UB**. There is no UB here, so there is nothing to buy with it — consistency is not a defect. Note
   also that success returns a clamped, non-negative position, so `-1` is unambiguous as an error.
3. **`stream_read`'s EOF has no vocabulary equivalent at all.** `NoSuchEntry`/`Invalid` would both be
   wrong — the handle is valid and the record exists, the cursor is simply past the end. Widening the
   enum (adding an `Eof = -23` variant) is the only *correct*-shaped fix and it needs evidence that a
   real device defines it; that evidence is zero. The site's own comment records that titles are
   observed to call past EOF with a NULL buffer, so this path is exercised in practice — which makes
   changing the number riskier here than anywhere else, not safer.
4. **`clip_put_data`'s value is not interpreted as an enum by anything.** Its `wipic_sys` wrapper
   reads `if result < 0` and carries the number through as data (`MediaError::Platform(i32)`), so the
   SDK's own contract for this call is "negative means platform error", not "must be a `WIPICError`".
   `-1` satisfies that contract exactly.

## What would change this

- **A site moves onto the transmute path** — i.e. a pin bump makes `wipic_sys` expose one of these
  four as `-> WIPICError`. Then it becomes UB and the `get_resource` precedent applies directly.
  Re-run the census above when the RustJava/wipi pins move.
- **A WIPI error-code spec turns up.** If it defines `-1` or `-23` for these conditions, sites 1/3
  are vindicated as-is; if it defines something else, that is the evidence sites 2 and 3 currently
  lack.
- **A title is observed to break on one of these codes.** That is the only compatibility signal this
  repo can receive, and it can only come from someone running a game.
