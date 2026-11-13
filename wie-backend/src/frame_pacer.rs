use alloc::{collections::VecDeque, vec::Vec};

use crate::executor::TICK_BUDGET_MS;

// Wall-clock the host keeps for itself each frame (blit, compositor, JS). The budget is
// `floor(period - HOST_HEADROOM_US)`: 16.67ms (60Hz) -> 14 = `TICK_BUDGET_MS`, 8.33ms (120Hz) -> 5,
// 6.94ms (144Hz) -> 4. Not 2ms: the executor's clock is whole milliseconds, so a tick runs up to 1ms
// past its budget before it even notices, plus the poll in flight — at 120Hz a budget of 6 left
// ~0.7ms of the frame, and a budget of 7 overran it (measured 2026-09-26 on a simulated vsync).
const HOST_HEADROOM_US: u64 = 2_500;

// The longest budget a slow frame clock gets: a 30Hz frame's worth (iOS Low Power Mode and Android
// battery savers hold `requestAnimationFrame` at 30Hz). Capped at the 60Hz `TICK_BUDGET_MS` until
// 2026-11-13, which left a 30Hz host idle for 19 of every 33ms while a heavy scene starved — LGT
// 3cc7a9b4cb15's dusk village drew 7.5/s there against 15 at 60Hz (docs/report 0559). Not uncapped:
// a 15Hz host would hold the page for 64ms at a time, past the 50ms long-task line.
const MAX_BUDGET_MS: u64 = 30;
// The shortest probed period that earns more than `TICK_BUDGET_MS`. A 60Hz host probed while its
// ticks overran (a boot's long guest steps) reads anywhere above 16.7ms, and a budget raised from
// that keeps every later interval as long: a busy browser calls back right after a long tick
// instead of on the next vsync. Measured 2026-11-13 in Chromium: a 24.5ms probe, a 22ms budget and
// 49 frames a second for 95s. The spot probe's two-thirds rule tells a 16.7ms frame only from an
// estimate above 25ms — so this is 25 with jitter room. A 30Hz display (33.3ms) qualifies; between
// 60 and ~36Hz the budget stays the 60Hz one, as it always was.
const RAISE_FROM_US: u64 = 28_000;

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
// down or most recent frames overran; either way a full probe re-measures. This far below means the
// probe misread a faster display — a boot's long ticks read a 60Hz page as 30Hz, and a light guest
// that ends its ticks early then shows the real 16.7ms frames (measured 2026-11-13 in Chromium:
// 15s of 18 ticks a second past 16ms on a 60Hz page). The spot probe cannot see that one: it
// compares against the window, which already reads 16.7ms. The budget itself comes
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
/// milliseconds. A 60Hz host gets `TICK_BUDGET_MS`, as with plain `tick` (bar one short probe tick
/// every ~2s); a faster frame clock gets less and a slower one more, up to `MAX_BUDGET_MS`.
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
        let raised = self.settled_us.is_some_and(|settled| settled >= RAISE_FROM_US);
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
                // A raised budget counts ticks, not samples. A busy browser calls back right after
                // a tick that ran past its frame, which is no sample (`MIN_GAP_MS`), so counting
                // samples stops the spot probe in exactly the case a raise must be able to leave —
                // measured 2026-11-13: a 60Hz Chromium page held a 25ms budget for 70s.
                if sample.is_some() || raised {
                    *since_spot += 1;
                }
                if *since_spot >= SPOT_EVERY {
                    self.probe = Probe::Spot;
                    return PROBE_BUDGET_MS;
                }
                // Not on the few samples a probe leaves behind: their median swings on one frame.
                if let (Some(settled), Some(period)) = (self.settled_us, self.period_us())
                    && self.samples.len() >= WINDOW / 2
                    && period.abs_diff(settled) > DRIFT_US
                {
                    self.samples.clear();
                    self.probe = Probe::Full { ticks: 1 };
                    return PROBE_BUDGET_MS;
                }
            }
        }

        // Only a probe's period may raise the budget past `TICK_BUDGET_MS`, and only one the spot
        // probe can still correct (`RAISE_FROM_US`). The window never may: it reads the intervals
        // this budget made (see `DRIFT_US`), so a budget that grows with it lengthens what it reads.
        match (self.settled_us, self.period_us()) {
            (Some(settled), _) if settled >= RAISE_FROM_US => Self::budget_for_period_us(settled),
            (Some(period), _) | (None, Some(period)) => Self::budget_for_period_us(period).min(TICK_BUDGET_MS),
            (None, None) => TICK_BUDGET_MS,
        }
    }

    /// Budget for a host that already knows its frame period, in microseconds
    /// (60Hz = 16_667 -> 14, 120Hz = 8_333 -> 5, 30Hz = 33_333 -> 30).
    pub fn budget_for_period_us(period_us: u64) -> u64 {
        (period_us.saturating_sub(HOST_HEADROOM_US) / 1000).clamp(1, MAX_BUDGET_MS)
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

    use super::{FramePacer, MAX_BUDGET_MS, PROBE_BUDGET_MS, PROBE_SAMPLES, SPOT_EVERY, WINDOW};
    use crate::executor::TICK_BUDGET_MS;

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

    // A frame clock whose phase follows the page, as Chromium's does under load (measured
    // 2026-11-13): the next callback comes one period after this one began, or right after the
    // tick returns if it ran past that (22.7ms ticks, 22.9ms intervals), `gap_us` later. The pacer
    // reads whole milliseconds, so a 200us gap counts as a sample about one time in five.
    fn run_following(pacer: &mut FramePacer, clock_us: &mut u64, period_us: u64, overrun_us: u64, gap_us: u64, frames: usize) -> Vec<u64> {
        let mut budgets = Vec::new();
        for _ in 0..frames {
            let budget = pacer.begin(*clock_us / 1000);
            budgets.push(budget);
            let end = *clock_us + budget * 1000 + overrun_us;
            pacer.end(end / 1000);
            *clock_us = (*clock_us + period_us).max(end + gap_us);
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
    fn a_30hz_host_gets_a_30hz_frame_budget() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let budgets = run(&mut pacer, &mut clock, 33_333, 1_600, 400);
        // A literal: comparing to `MAX_BUDGET_MS` would pass with the cap back at 14.
        assert!(steady(&budgets[20..], 30), "{budgets:?}");
        // The measured p95 overrun still lands inside the frame, so the budget does not cost a frame.
        assert!(30 * 1000 + 1_600 < 33_333);
    }

    #[test]
    fn a_host_slower_than_30hz_is_not_given_more_than_a_30hz_budget() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let budgets = run(&mut pacer, &mut clock, 66_667, 1_000, 200);
        assert!(steady(&budgets[20..], MAX_BUDGET_MS), "{budgets:?}");
    }

    // The self-amplifying case a budget above `TICK_BUDGET_MS` opens: a 60Hz host whose boot ticks
    // miss every other frame is probed as 30Hz, and a 30ms budget then misses every other frame too,
    // so every later interval agrees with the mistake. Only the spot probe can see past it.
    #[test]
    fn a_60hz_host_probed_as_30hz_at_boot_returns_to_the_60hz_budget() {
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let boot = run(&mut pacer, &mut clock, 16_667, 20_000, 40);
        assert_eq!(boot.last(), Some(&MAX_BUDGET_MS), "the boot is misread as 30Hz: {boot:?}");
        let budgets = run(&mut pacer, &mut clock, 16_667, 1_000, 600);
        // Back within one spot interval (SPOT_EVERY samples) and a full probe.
        assert!(steady(&budgets[(SPOT_EVERY as usize + 20)..], TICK_BUDGET_MS), "{budgets:?}");
    }

    #[test]
    fn a_60hz_host_probed_slow_while_its_frames_followed_the_page_keeps_the_60hz_budget() {
        // The measured case: a boot whose ticks overran by ~20ms probed a 60Hz Chromium page at
        // ~24ms. A budget raised from that (21) reads back as 21ms intervals, and the spot probe's
        // 16.7ms sample is not under two thirds of 24 — the page stayed at 49 frames a second.
        // A whole-millisecond gap, so every interval is a sample. Never raised; the misread itself
        // is re-measured once the window shows 16.7ms (`DRIFT_US`).
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        run_following(&mut pacer, &mut clock, 16_667, 20_000, 1_000, 40);
        let budgets = run_following(&mut pacer, &mut clock, 16_667, 1_000, 1_000, 600);
        assert!(budgets.iter().all(|&b| b <= TICK_BUDGET_MS), "{budgets:?}");
        assert!(steady(&budgets[(WINDOW / 2 + PROBE_SAMPLES + 20)..], TICK_BUDGET_MS), "{budgets:?}");
    }

    #[test]
    fn a_60hz_host_probed_as_30hz_while_its_frames_followed_the_page_returns_to_the_60hz_budget() {
        // Overran by more, the boot reads as 30Hz and earns the raise; the spot probe undoes it.
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let boot = run_following(&mut pacer, &mut clock, 16_667, 26_000, 1_000, 40);
        assert!(boot.iter().any(|&b| b > TICK_BUDGET_MS), "the boot earns the raise: {boot:?}");
        let budgets = run_following(&mut pacer, &mut clock, 16_667, 1_000, 1_000, 600);
        assert!(steady(&budgets[(SPOT_EVERY as usize + 20)..], TICK_BUDGET_MS), "{budgets:?}");
    }

    #[test]
    fn a_60hz_page_that_calls_back_at_once_after_a_raised_tick_still_gets_its_spot_probe() {
        // The second measured case: the browser called back 0.2ms after each 25ms tick, so four
        // intervals in five were no sample, the spot probe never came, and the raise held for 70s.
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_200;
        let boot = run_following(&mut pacer, &mut clock, 16_667, 26_000, 200, 60);
        assert!(boot.iter().any(|&b| b > TICK_BUDGET_MS), "the boot earns the raise: {boot:?}");
        let budgets = run_following(&mut pacer, &mut clock, 16_667, 1_000, 200, 600);
        assert!(steady(&budgets[(SPOT_EVERY as usize + 20)..], TICK_BUDGET_MS), "{budgets:?}");
    }

    #[test]
    fn a_30hz_page_keeps_its_budget_when_some_ticks_run_past_the_frame() {
        // Spot probes by tick must not eat a real 30Hz page: ticks end 0-4ms past a 30ms budget, so
        // some run past 33.3ms and are called back at once, and a spot probe still reads 33.3ms.
        let mut pacer = FramePacer::new();
        let mut clock: u64 = 1_000_000;
        let mut seed: u64 = 11;
        let mut budgets = Vec::new();
        for _ in 0..3000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            budgets.extend(run_following(&mut pacer, &mut clock, 33_333, (seed >> 33) % 4_000, 200, 1));
        }
        let late = &budgets[1000..];
        assert!(late.iter().all(|&b| b <= MAX_BUDGET_MS), "{late:?}");
        let full = late.iter().filter(|&&b| b == MAX_BUDGET_MS).count();
        assert!(full * 10 >= late.len() * 7, "{full} of {} ticks at the 30Hz budget", late.len());
    }

    #[test]
    fn a_60hz_host_probed_as_30hz_drops_the_raise_once_a_light_guest_shows_its_real_frames() {
        // The third measured case: the raise held through a title screen whose ticks ended early, so
        // the page kept 60 frames a second and only its game-frame ticks ran 20-30ms.
        let mut pacer = FramePacer::new();
        let mut clock = 1_000_000;
        let boot = run(&mut pacer, &mut clock, 16_667, 20_000, 40);
        assert!(boot.iter().any(|&b| b > TICK_BUDGET_MS), "the boot earns the raise: {boot:?}");
        let mut budgets = Vec::new();
        for _ in 0..300 {
            budgets.push(pacer.begin(clock / 1000));
            let end = clock + 3_000; // the guest sleeps: the tick ends long before its budget
            pacer.end(end / 1000);
            clock = end.div_ceil(16_667) * 16_667;
        }
        // One window's worth of samples to see it (`WINDOW / 2`), then a full probe.
        assert!(steady(&budgets[(WINDOW / 2 + PROBE_SAMPLES + 4)..], TICK_BUDGET_MS), "{budgets:?}");
    }

    #[test]
    fn a_laggy_30hz_host_never_drifts_past_its_budget_and_does_not_live_in_probes() {
        // Callbacks land up to 3ms late and a busy guest's tick ends 0-4ms past its budget, so some
        // 30ms ticks miss their frame and read as 15Hz. The cap holds the budget, and the drift
        // re-probe brings a run of misses back to 30 rather than parking on the probe budget.
        let mut pacer = FramePacer::new();
        let mut clock: u64 = 1_000_000;
        let mut seed: u64 = 7;
        let mut rand = move |n: u64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) % n
        };
        let mut budgets = Vec::new();
        for _ in 0..3000 {
            let start = clock + rand(3_000);
            let budget = pacer.begin(start / 1000);
            budgets.push(budget);
            let end = start + budget * 1000 + rand(4_000);
            pacer.end(end / 1000);
            clock = end.div_ceil(33_333) * 33_333;
        }
        let late = &budgets[1000..];
        assert!(late.iter().all(|&b| b <= MAX_BUDGET_MS), "{late:?}");
        let full = late.iter().filter(|&&b| b == MAX_BUDGET_MS).count();
        assert!(full * 10 >= late.len() * 8, "{full} of {} ticks at the 30Hz budget", late.len());
    }

    #[test]
    fn a_host_the_probe_gave_up_on_is_not_raised_past_the_default_by_its_window() {
        // No settled probe: the budget falls back to the window, which a budget past the default
        // would lengthen — so the fallback stays at `TICK_BUDGET_MS` however slow the window reads.
        let mut pacer = FramePacer::new();
        let mut t = 0;
        for _ in 0..130 {
            t += pacer.begin(t);
            pacer.end(t);
        }
        let mut clock = t * 1000 + 1_000_000;
        let budgets = run(&mut pacer, &mut clock, 33_333, 1_000, 200);
        assert!(budgets[20..].iter().all(|&b| b <= TICK_BUDGET_MS), "{budgets:?}");
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
