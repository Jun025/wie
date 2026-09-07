#![no_std]
extern crate alloc;

pub mod api;
mod context;
mod method;

pub use self::context::{WIPICContext, WIPICResult};
pub use self::method::MethodImpl;

use alloc::boxed::Box;

use wie_util::WieError;

use crate::method::MethodBody;

pub type WIPICMethodBody = Box<dyn MethodBody<WieError>>;

/// Pins the `WIPICError` vocabulary this crate's return codes were judged against.
///
/// `docs/wipi-c-abi-error-codes.md` decides four out-of-vocabulary return codes, and two of those
/// four ride on "the vocabulary has no variant for this" — `net::socket_close` (no generic failure)
/// and `database::stream_read` (no EOF). Those are claims about an UPSTREAM enum: `wipi_types` is a
/// git dependency with **no `rev` in `Cargo.toml`**, so what holds its revision is the lock file
/// alone. Measured 2026-09-07 over the commits that actually changed that lock line: **31 of them,
/// spanning 2025-07-02 to 2026-04-11** (283 days, one every 9.4 on average) — and then **nothing
/// for 149 days**, which is where the pin still sits. **19 of the 31 were dependabot's own
/// `Bump wipi_types from X to Y` PRs** (`.github/dependabot.yml` still schedules cargo `daily`);
/// the other 12 rode in on unrelated feature commits.
///
/// This is a test rather than a checklist entry because of that 12, not because of the cadence.
/// A checklist can only be read by a round that knows it is bumping the pin, which covers the 19
/// and misses the 12; a test fires on all 31 — and on the dependabot PRs too, since they run CI.
/// The cost side is what makes the frequency irrelevant: no workflow step, no dependency, no
/// `~/.cargo` path, just one more case in a suite that already runs on every commit. So it stays
/// on whether the pin moves weekly or sits for five months, which is what it has actually done.
///
/// It locks two things and neither needs a script, a workflow step, or a `~/.cargo` path:
///   * **additions** — the match below is exhaustive over a non-`#[non_exhaustive]` enum, so a new
///     upstream variant is a COMPILE error here. That is the case that would quietly invalidate the
///     "no variant for this" half of the doc (an added `Eof = -23` makes `stream_read`'s disposition
///     wrong, and nothing else in this workspace would notice).
///   * **renumbering / removal** — the asserts pin each discriminant. Removing one that a
///     transmute-path site returns is how a currently-sound return becomes UB.
///
/// **What it does NOT lock, stated so nobody reads more into it than is there:** which host
/// functions reach `WIPICError::from_raw` at all. That lives in `wipic_sys`, which this workspace
/// does not depend on (it is the guest-side SDK; it appears in this crate only inside comments), so
/// the census in that doc still has to be re-run by hand when the answer matters. This test covers
/// the half that is silent; the other half stays a documented reversal condition.
#[cfg(test)]
mod wipi_vocabulary_lock {
    use wipi_types::wipic::WIPICError;

    #[test]
    fn wipic_error_vocabulary_is_unchanged_test() {
        // Exhaustive on purpose: adding a variant upstream must not compile until the four
        // dispositions in docs/wipi-c-abi-error-codes.md have been re-read.
        fn _exhaustive(e: &WIPICError) {
            match e {
                WIPICError::ImageDone
                | WIPICError::Success
                | WIPICError::Invalid
                | WIPICError::NoSuchEntry
                | WIPICError::InsufficientBufferSize
                | WIPICError::BadRecordId
                | WIPICError::InvalidHandle => {}
            }
        }

        assert_eq!(WIPICError::ImageDone as i32, 1);
        assert_eq!(WIPICError::Success as i32, 0);
        assert_eq!(WIPICError::Invalid as i32, -9);
        assert_eq!(WIPICError::NoSuchEntry as i32, -12);
        assert_eq!(WIPICError::InsufficientBufferSize as i32, -18);
        assert_eq!(WIPICError::BadRecordId as i32, -22);
        assert_eq!(WIPICError::InvalidHandle as i32, -25);
    }
}
