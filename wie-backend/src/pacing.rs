//! Delays the engine itself adds between what a guest asked for and what it got, on the engine
//! clock (`Platform::now`). Both speed regressions this fork shipped came in silently and were
//! found by a player first (a KTF clet 35 -> 17fps, #291; an LGT title 20 -> 12fps, #338); each
//! is a number here, not an fps figure that load moves:
//!
//! - `sleep_ms`    guest `Thread.sleep(n)` arguments: one value is a fixed-sleep loop, a spread a
//!   deadline loop (`sleep(next - now)`); `yields` counts `Thread.yield` (a spin shows up here).
//! - `sleep_late`  guest `Thread.sleep(n)`: woke this many ms after `n` was up (host tick grid).
//! - `timer_late`  WIPI `MC_knlSetTimer`: callback ran this many ms after it fell due (#291's wait).
//! - `redraw`      guest `Display.repaint` -> `Display.handlePaintEvent` start (#338's tick-end delivery),
//!   and `redraw_cross_tick`, how many of those paints started in a later host tick than their
//!   request. That count is host-speed independent: #338's shape is every paint crossing a tick.
//! - `gcs` per `paints`: a full collection per paint was the other half of #338.

use alloc::{format, string::String, vec::Vec};

use crate::time::Instant;

// ponytail: samples past this are counted but not kept; a window longer than ~10 minutes of a busy
// guest reads its percentiles from the first part. Take the stats per window if that matters.
const MAX_SAMPLES: usize = 1 << 16;

#[derive(Default)]
struct Samples {
    count: u64,
    sum: u64,
    kept: Vec<u32>,
}

impl Samples {
    fn push(&mut self, value: u64) {
        self.count += 1;
        self.sum += value;
        if self.kept.len() < MAX_SAMPLES {
            self.kept.push(value.min(u32::MAX as u64) as u32);
        }
    }

    fn pct(&self, p: f64) -> Option<u32> {
        let mut sorted = self.kept.clone();
        sorted.sort_unstable();
        sorted
            .get(((sorted.len() as f64 * p) as usize).min(sorted.len().wrapping_sub(1)))
            .copied()
    }
}

#[derive(Default)]
pub struct Pacing {
    ticks: u64,
    yields: u64,
    sleep_ms: Samples,
    sleep_late: Samples,
    timer_late: Samples,
    redraw: Samples,
    redraw_cross_tick: u64,
    // (requested at, tick it was requested in) of the oldest repaint not yet painted
    redraw_requested: Option<(Instant, u64)>,
    paints: u64,
    gcs: u64,
    gc_ms: u64,
}

impl Pacing {
    pub(crate) fn tick_ended(&mut self) {
        self.ticks += 1;
    }

    pub fn guest_yielded(&mut self) {
        self.yields += 1;
    }

    pub fn guest_slept(&mut self, requested_ms: u64, late_ms: u64) {
        self.sleep_ms.push(requested_ms);
        self.sleep_late.push(late_ms);
    }

    pub fn timer_fired(&mut self, late_ms: u64) {
        self.timer_late.push(late_ms);
    }

    pub fn redraw_requested(&mut self, now: Instant) {
        if self.redraw_requested.is_none() {
            self.redraw_requested = Some((now, self.ticks));
        }
    }

    pub fn paint_started(&mut self, now: Instant) {
        self.paints += 1;
        if let Some((at, tick)) = self.redraw_requested.take() {
            self.redraw.push(now.raw().saturating_sub(at.raw()));
            if self.ticks != tick {
                self.redraw_cross_tick += 1;
            }
        }
    }

    pub fn collected_garbage(&mut self, ms: u64) {
        self.gcs += 1;
        self.gc_ms += ms;
    }

    /// Everything since the last `take`, and start over — a measurement window.
    pub fn take(&mut self) -> Self {
        let pending = self.redraw_requested;
        let ticks = self.ticks;
        let taken = core::mem::take(self);
        // A request made before the window is still owed its paint; ticks keep counting so its
        // cross-tick test still compares like with like.
        self.redraw_requested = pending;
        self.ticks = ticks;
        taken
    }

    /// One JSON object. Percentiles are `null` when nothing was sampled.
    pub fn summary_json(&self) -> String {
        let p = |s: &Samples, q: f64| s.pct(q).map_or(String::from("null"), |v| format!("{v}"));
        format!(
            "{{\"ticks\":{},\"yields\":{},\"sleeps\":{},\"sleep_ms_p05\":{},\"sleep_ms_p50\":{},\"sleep_ms_p95\":{},\
             \"sleep_late_p50\":{},\"sleep_late_p95\":{},\"sleep_late_sum\":{},\
             \"timers\":{},\"timer_late_p50\":{},\"timer_late_p95\":{},\"timer_late_sum\":{},\
             \"redraws\":{},\"redraw_p50\":{},\"redraw_p95\":{},\"redraw_sum\":{},\"redraw_cross_tick\":{},\
             \"paints\":{},\"gcs\":{},\"gc_ms\":{}}}",
            self.ticks,
            self.yields,
            self.sleep_ms.count,
            p(&self.sleep_ms, 0.05),
            p(&self.sleep_ms, 0.5),
            p(&self.sleep_ms, 0.95),
            p(&self.sleep_late, 0.5),
            p(&self.sleep_late, 0.95),
            self.sleep_late.sum,
            self.timer_late.count,
            p(&self.timer_late, 0.5),
            p(&self.timer_late, 0.95),
            self.timer_late.sum,
            self.redraw.count,
            p(&self.redraw, 0.5),
            p(&self.redraw, 0.95),
            self.redraw.sum,
            self.redraw_cross_tick,
            self.paints,
            self.gcs,
            self.gc_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Pacing;
    use crate::time::Instant;

    #[test]
    fn a_paint_is_timed_from_the_first_unpainted_repaint_and_counted_across_ticks() {
        let mut pacing = Pacing::default();
        pacing.redraw_requested(Instant::from_epoch_millis(10));
        pacing.redraw_requested(Instant::from_epoch_millis(12)); // coalesced into the first
        pacing.paint_started(Instant::from_epoch_millis(13));
        pacing.redraw_requested(Instant::from_epoch_millis(20));
        pacing.tick_ended();
        pacing.paint_started(Instant::from_epoch_millis(36));
        pacing.paint_started(Instant::from_epoch_millis(40)); // no request: a paint, not a redraw

        let json = pacing.summary_json();
        assert!(
            json.contains("\"redraws\":2,\"redraw_p50\":16,\"redraw_p95\":16,\"redraw_sum\":19,\"redraw_cross_tick\":1,\"paints\":3"),
            "{json}"
        );
    }

    #[test]
    fn take_starts_a_window_but_keeps_the_owed_paint() {
        let mut pacing = Pacing::default();
        pacing.guest_slept(50, 3);
        pacing.redraw_requested(Instant::from_epoch_millis(5));
        let first = pacing.take();
        assert!(
            first
                .summary_json()
                .contains("\"sleep_ms_p50\":50,\"sleep_ms_p95\":50,\"sleep_late_p50\":3,\"sleep_late_p95\":3,\"sleep_late_sum\":3")
        );

        pacing.paint_started(Instant::from_epoch_millis(9));
        let json = pacing.summary_json();
        assert!(json.contains("\"sleeps\":0,\"sleep_ms_p05\":null"), "{json}");
        assert!(json.contains("\"redraws\":1,\"redraw_p50\":4"), "{json}");
    }
}
