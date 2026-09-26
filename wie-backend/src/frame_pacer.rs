use alloc::{collections::VecDeque, vec::Vec};

use crate::executor::TICK_BUDGET_MS;

// Wall-clock the host keeps for itself each frame (blit, compositor, JS). The budget is
// `floor(period - HOST_HEADROOM_US)`: 16.67ms (60Hz) -> 14 = `TICK_BUDGET_MS`, 8.33ms (120Hz) -> 5,
// 6.94ms (144Hz) -> 4. Not 2ms: the executor's clock is whole milliseconds, so a tick runs up to 1ms
// past its budget before it even notices, plus the poll in flight — at 120Hz a budget of 6 left
// ~0.7ms of the frame, and a budget of 7 overran it (measured 2026-09-26 on a simulated vsync).
const HOST_HEADROOM_US: u64 = 2_500;

// Budget of a probe tick. A tick overruns its budget by up to ~1.6ms (measured 2026-09-25: p95 15.6
// at budget 14), so 4 still fits a 160Hz frame.
const PROBE_BUDGET_MS: u64 = 4;
// A full probe runs every tick at `PROBE_BUDGET_MS` until this many samples are in.
const PROBE_SAMPLES: usize = 8;
// A host that never waits between ticks gives no usable sample; stop probing and use the default.
const PROBE_GIVE_UP_TICKS: u32 = 120;
// Between full probes, one tick in this many samples is a spot probe: it runs at `PROBE_BUDGET_MS`,
// and if the interval after it is well under the current estimate the display is faster
// than the pacer thinks, so a full probe follows. Without it the pacer cannot leave a too-long
// estimate: a busy guest overruns every frame of a fast display, so the host only ever reports
// multi-frame intervals, and the budget that caused them is the one they keep selecting. That is
// not hypothetical — a full probe at boot, where KTF ticks are long, settled 영웅서기 제로 on "60Hz"
// for good on a simulated 120Hz display (2026-09-26, budget 14 on all 568 ticks). At 60Hz the cost
// is one 4ms tick every ~2s.
const SPOT_EVERY: u32 = 120;

// A window estimate this far above what the last full probe settled on means the display slowed
// down or most recent frames overran; either way a full probe re-measures. The budget itself comes
// from the settled period, not the window: on a loaded host the window reads longer than the
// display (late callbacks, overruns), and a budget that follows it feeds itself — overruns lengthen
// the intervals, a longer estimate raises the budget, a larger budget overruns more. On a
// simulated 120Hz display that took 놈3 from 5 back to 14 within seconds (2026-09-26), and a later
// version that re-probed on drift but still budgeted from the window spent 60-70% of its ticks
// probing.
const DRIFT_US: u64 = 1_500;

// Intervals are kept over this many accepted frames. The refresh period is the mean of the samples
// within 1ms of their median: the median ignores missed frames while they are a minority (a
// majority is what `DRIFT_US` exists for), and the mean of the rest recovers the fraction a
// whole-ms clock loses (a 120Hz display reads as 8, 8, 9, ...; a 60Hz one as 16, 17, 17, ...).
// Not the minimum: a frame callback that runs late is followed by one that runs early, so the
// shortest interval in a window undershoots the display (measured 2026-09-26: a 60Hz rAF host got
// 13ms ticks instead of 14 and 영웅서기4 fell 36.5 -> 33fps).
const WINDOW: usize = 64;

// A tick that starts less than this after the previous one ended was not waiting for a frame
// (a burst of calls at boot, or a host loop without a frame clock). Its interval says nothing about
// the display, so it is not a sample.
const MIN_GAP_MS: u64 = 1;

enum Probe {
    Idle { since_spot: u32 },
    Spot,
    Full { ticks: u32 },
}

/// Derives an `Emulator::tick_for` budget from the interval at which the host calls it.
///
/// Call `begin` right before `tick_for` and `end` right after, with the host's wall clock in
/// milliseconds. The budget never exceeds `TICK_BUDGET_MS`, so a host at 60Hz or slower behaves
/// as with plain `tick` (bar one short probe tick every ~2s); only faster frame clocks get a
/// smaller budget.
pub struct FramePacer {
    last_begin: Option<u64>,
    last_end: Option<u64>,
    samples: VecDeque<u64>,
    probe: Probe,
    settled_us: Option<u64>,
}

impl Default for FramePacer {
    fn default() -> Self {
        Self::new()
    }
}

impl FramePacer {
    pub fn new() -> Self {
        Self {
            last_begin: None,
            last_end: None,
            samples: VecDeque::with_capacity(WINDOW),
            probe: Probe::Full { ticks: 0 },
            settled_us: None,
        }
    }

    /// Budget in milliseconds for the tick that starts at `now_ms`.
    pub fn begin(&mut self, now_ms: u64) -> u64 {
        let sample = match (self.last_begin, self.last_end) {
            (Some(begin), Some(end)) if now_ms.saturating_sub(end) >= MIN_GAP_MS => Some(now_ms - begin),
            _ => None,
        };
        self.last_begin = Some(now_ms);

        // The estimate before this sample, so a spot sample is compared against what it tests.
        let before = self.period_us();
        if let Some(sample) = sample {
            if self.samples.len() == WINDOW {
                self.samples.pop_front();
            }
            self.samples.push_back(sample);
        }

        match &mut self.probe {
            Probe::Full { ticks } => {
                *ticks += 1;
                if self.samples.len() >= PROBE_SAMPLES {
                    self.probe = Probe::Idle { since_spot: 0 };
                    self.settled_us = self.period_us();
                } else if *ticks > PROBE_GIVE_UP_TICKS {
                    self.probe = Probe::Idle { since_spot: 0 };
                    self.settled_us = None;
                } else {
                    return PROBE_BUDGET_MS;
                }
            }
            Probe::Spot => {
                // Under two thirds of the estimate: 120Hz reads as 1/2 of 60Hz and 90Hz as 2/3, while
                // a 60Hz callback that runs early after a late one reads as ~3/4 (12.7 of 16.7ms).
                let faster = matches!((sample, before), (Some(s), Some(p)) if s * 3_000 < p * 2);
                if faster {
                    self.samples.clear();
                    self.samples.extend(sample);
                    self.probe = Probe::Full { ticks: 1 };
                    return PROBE_BUDGET_MS;
                }
                self.probe = Probe::Idle { since_spot: 0 };
            }
            Probe::Idle { since_spot } => {
                if sample.is_some() {
                    *since_spot += 1;
                }
                if *since_spot >= SPOT_EVERY {
                    self.probe = Probe::Spot;
                    return PROBE_BUDGET_MS;
                }
                // Not on the few samples a probe leaves behind: their median swings on one frame.
                if let (Some(settled), Some(period)) = (self.settled_us, self.period_us())
                    && self.samples.len() >= WINDOW / 2
                    && period > settled + DRIFT_US
                {
                    self.samples.clear();
                    self.probe = Probe::Full { ticks: 1 };
                    return PROBE_BUDGET_MS;
                }
            }
        }

        match self.settled_us.or_else(|| self.period_us()) {
            Some(period) => Self::budget_for_period_us(period),
            None => TICK_BUDGET_MS,
        }
    }

    /// Budget for a host that already knows its frame period, in microseconds
    /// (60Hz = 16_667 -> 14, 120Hz = 8_333 -> 5).
    pub fn budget_for_period_us(period_us: u64) -> u64 {
        (period_us.saturating_sub(HOST_HEADROOM_US) / 1000).clamp(1, TICK_BUDGET_MS)
    }

    pub fn end(&mut self, now_ms: u64) {
        self.last_end = Some(now_ms);
    }

    fn period_us(&self) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted: Vec<u64> = self.samples.iter().copied().collect();
        sorted.sort_unstable();
        let median = sorted[sorted.len() / 2];
        let core = sorted.iter().filter(|&&s| s.abs_diff(median) <= 1);
        let (sum, count) = core.fold((0, 0), |(sum, count), &s| (sum + s, count + 1));
        Some(sum * 1000 / count)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{FramePacer, PROBE_BUDGET_MS, SPOT_EVERY};

    // Simulated host: a vsync every `period_us`; each tick runs for its budget plus `overrun_us`
    // (a busy guest uses all of it), and the next tick starts at the first vsync after it ends.
    // Returns the budgets handed out, in order.
    fn run(pacer: &mut FramePacer, clock_us: &mut u64, period_us: u64, overrun_us: u64, frames: usize) -> Vec<u64> {
        let mut budgets = Vec::new();
        for _ in 0..frames {
            let budget = pacer.begin(*clock_us / 1000);
            budgets.push(budget);
            let end = *clock_us + budget * 1000 + overrun_us;
            pacer.end(end / 1000);
            *clock_us = end.div_ceil(period_us) * period_us;
        }
        budgets
    }

    // Steady state: every budget is `want`, except the spot probes (at most one per SPOT_EVERY).
    fn steady(budgets: &[u64], want: u64) -> bool {
        let spots = budgets.iter().filter(|&&b| b == PROBE_BUDGET_MS).count();
        budgets.iter().all(|&b| b == want || b == PROBE_BUDGET_MS) && spots <= budgets.len() / SPOT_EVERY as usize + 1
    }

    #[test]
    fn a_60hz_host_gets_the_default_budget() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let budgets = run(&mut pacer, &mut clock, 16_667, 1_000, 600);
        assert!(budgets[..8].iter().all(|&b| b == PROBE_BUDGET_MS));
        assert!(steady(&budgets[20..], 14), "{budgets:?}");
    }

    #[test]
    fn a_busy_guest_on_a_120hz_host_gets_a_budget_that_fits_the_frame() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        // 1.6ms overrun is the measured p95 excess of a tick over its budget.
        let budgets = run(&mut pacer, &mut clock, 8_333, 1_600, 600);
        assert!(steady(&budgets[20..], 5), "{budgets:?}");
        assert!(5 * 1000 + 1_600 < 8_333);
    }

    #[test]
    fn a_120hz_host_is_found_after_a_boot_whose_ticks_miss_every_frame() {
        // 영웅서기 제로 on a simulated 120Hz display: boot ticks run long, so the first probe reads
        // a slow display and hands the guest 14ms, which a busy guest then overruns every frame.
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        run(&mut pacer, &mut clock, 8_333, 20_000, 40);
        let budgets = run(&mut pacer, &mut clock, 8_333, 1_600, 400);
        assert!(steady(&budgets[200..], 5), "{budgets:?}");
    }

    #[test]
    fn a_120hz_host_recovers_after_a_long_run_of_overrunning_frames() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        run(&mut pacer, &mut clock, 8_333, 1_600, 100);
        // Long enough that every sample in the window is a two-frame interval, and longer than any
        // probe: the window then reads "60Hz", and at 14ms every later frame overruns.
        run(&mut pacer, &mut clock, 8_333, 9_000, 300);
        let budgets = run(&mut pacer, &mut clock, 8_333, 1_600, 400);
        assert!(steady(&budgets[200..], 5), "{budgets:?}");
    }

    #[test]
    fn a_laggy_120hz_host_does_not_drift_back_to_the_default_budget() {
        // What a loaded browser looks like: callbacks land up to 2ms after the vsync, and a busy
        // guest's tick ends 0-4ms past its budget. At a small budget most frames fit; each budget
        // step up makes more of them overrun, which lengthens the intervals, which raises the budget.
        let mut pacer = FramePacer::new();
        let mut clock: u64 = 1_000_000;
        let mut seed: u64 = 1;
        let mut rand = move |n: u64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) % n
        };
        let mut budgets = Vec::new();
        for _ in 0..3000 {
            let start = clock + rand(2_000);
            let budget = pacer.begin(start / 1000);
            budgets.push(budget);
            let end = start + budget * 1000 + rand(4_000);
            pacer.end(end / 1000);
            clock = end.div_ceil(8_333) * 8_333;
        }
        let late = &budgets[1000..];
        let at_default = late.iter().filter(|&&b| b == 14).count();
        let probing = late.iter().filter(|&&b| b == PROBE_BUDGET_MS).count();
        // Measured when written, of 2000: at the default 0 and probing 253 (13%). Taking the budget
        // from the window instead of the last probe: 20 at the default but probing 34%. Neither
        // re-probe: 1516 at the default.
        assert!(at_default * 10 < late.len(), "{at_default} of {} ticks at the default budget", late.len());
        assert!(probing * 5 < late.len(), "{probing} of {} ticks spent probing", late.len());
    }

    #[test]
    fn frame_callback_jitter_does_not_shrink_a_60hz_budget() {
        // A callback that starts 4ms late is followed by one that starts on time, so one interval
        // in four is 20.7ms and the next 12.7ms. The display is still 60Hz.
        let mut pacer = FramePacer::new();
        let mut budgets = Vec::new();
        for frame in 0..600u64 {
            let late = if frame % 4 == 0 { 4_000 } else { 0 };
            let start = 1_000_000 + frame * 16_667 + late;
            let budget = pacer.begin(start / 1000);
            budgets.push(budget);
            pacer.end((start + budget * 1000 + 1_000) / 1000);
        }
        assert!(steady(&budgets[20..], 14), "{budgets:?}");
    }

    #[test]
    fn a_30hz_host_keeps_the_default_budget() {
        // Never more than plain `tick` gives: a slow host would otherwise hand a guest 31ms slices.
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let budgets = run(&mut pacer, &mut clock, 33_333, 1_000, 400);
        assert!(steady(&budgets[20..], 14), "{budgets:?}");
    }

    #[test]
    fn back_to_back_calls_are_not_frame_samples() {
        let mut pacer = FramePacer::new();
        // A host that ticks six times in a row before its first frame (web/src/lib/emulator.ts
        // does this at boot) must not teach the pacer a 4ms display.
        let mut t = 1000;
        for _ in 0..6 {
            let b = pacer.begin(t);
            t += b;
            pacer.end(t);
        }
        let mut clock = 2_000_000;
        let budgets = run(&mut pacer, &mut clock, 16_667, 1_000, 400);
        assert!(steady(&budgets[20..], 14), "{budgets:?}");
    }

    #[test]
    fn moving_to_a_slower_display_raises_the_budget() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        run(&mut pacer, &mut clock, 8_333, 1_000, 100);
        // Once the window holds only 60Hz intervals its median is the new period.
        let budgets = run(&mut pacer, &mut clock, 16_667, 1_000, 400);
        assert!(steady(&budgets[150..], 14), "{budgets:?}");
    }

    #[test]
    fn a_host_without_a_frame_clock_falls_back_to_the_default() {
        let mut pacer = FramePacer::new();
        let mut t = 0;
        let mut last = 0;
        for _ in 0..200 {
            last = pacer.begin(t);
            t += last;
            pacer.end(t);
        }
        assert_eq!(last, 14);
    }
}
