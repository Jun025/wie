use alloc::collections::VecDeque;

use spin::Mutex;

use crate::time::Instant;

/// The play-speed clock: the guest's `now()` advances `speed` times as fast as the host's wall
/// clock, for a user-chosen `speed` in `[MIN_SPEED, MAX_SPEED]`.
///
/// Every guest-visible timer (sleep, WIPI timers, MIDP `currentTimeMillis`, the executor's own
/// tick budget) reads `Platform::now()`, so a host that returns this clock there speeds all of
/// them up together. A speed change re-anchors at the moment it happens, so guest time neither
/// jumps nor runs backwards across it; a backwards step of the wall clock is held, not followed.
///
/// At speed 1.0 with no change ever made, `now` is the wall clock unchanged.
pub struct SpeedClock {
    state: Mutex<State>,
}

struct State {
    // Wall and guest time at the last speed change (or the first read).
    anchor: Option<(f64, f64)>,
    speed: f64,
    last: u64,
}

pub const MIN_SPEED: f64 = 1.0;
pub const MAX_SPEED: f64 = 2.0;

impl Default for SpeedClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedClock {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                anchor: None,
                speed: MIN_SPEED,
                last: 0,
            }),
        }
    }

    /// `speed` clamped to `[MIN_SPEED, MAX_SPEED]`; NaN and infinities are 1.0. Not quantized.
    pub fn clamp(speed: f64) -> f64 {
        if speed.is_finite() {
            speed.clamp(MIN_SPEED, MAX_SPEED)
        } else {
            MIN_SPEED
        }
    }

    pub fn speed(&self) -> f64 {
        self.state.lock().speed
    }

    /// Guest time at host wall time `wall_ms`.
    pub fn now(&self, wall_ms: f64) -> Instant {
        let mut state = self.state.lock();
        Instant::from_epoch_millis(state.read(wall_ms))
    }

    /// Changes the speed from `wall_ms` on; returns the speed actually applied.
    pub fn set_speed(&self, wall_ms: f64, speed: f64) -> f64 {
        let mut state = self.state.lock();
        let guest = state.guest(wall_ms);
        state.anchor = Some((wall_ms, guest));
        state.speed = Self::clamp(speed);
        state.speed
    }

    /// A tick budget of `wall_ms` wall-clock milliseconds, in guest milliseconds: the executor
    /// measures its budget on the guest clock, so without this a 2x tick would end after half
    /// of the host frame and the game would not get faster.
    pub fn budget(&self, wall_ms: u64) -> u64 {
        (wall_ms as f64 * self.speed()) as u64
    }
}

/// How many times one guest timer may fire in host tick `tick` at play `speed` (WIPI
/// `MC_knlSetTimer`'s once-per-host-frame rule, `docs/report/0384`): `speed` per tick on average,
/// as `floor(speed * (tick + 1)) - floor(speed * tick)` — 1 at 1x, 1 and 2 alternating at 1.5x,
/// 2 at 2x. Each tick stands alone: a tick that fired less banks nothing for the next.
pub fn timer_fires_per_tick(speed: f64, tick: u64) -> u32 {
    let speed = SpeedClock::clamp(speed);
    ((speed * (tick + 1) as f64).floor() - (speed * tick as f64).floor()) as u32
}

/// The play speed a host actually got over the last `ACHIEVED_WINDOW_MS` of wall time.
///
/// Defined as guest time ÷ wall time on a clock that, unlike `SpeedClock`, stops while the guest is
/// starved: from the end of a tick that ran out of budget with the guest still wanting CPU
/// (`Pacing::starved_ticks`) to the start of the next. The guest clock itself never stops — it is
/// the wall clock times `speed` — so measured on it the ratio is `speed` by construction. A gap
/// after a tick in which every task slept is counted in full: a guest asleep is on schedule, so
/// sleeping neither lowers nor inflates the figure. The tick itself always counts.
///
/// ponytail: a starved guest reads at least `speed × budget ÷ frame` (≈0.84 × `speed` at 60Hz)
/// however much more CPU it wanted — the executor sees *that* it wanted more, not how much. A
/// paint-rate comparison against 1x (`scripts/audio-probe.mjs --paint-window`) is the deep measure.
#[derive(Default)]
pub struct AchievedSpeed {
    // (wall interval, counted part of it) per tick-to-tick interval, oldest first
    intervals: VecDeque<(f64, f64)>,
    total: f64,
    counted: f64,
    // (begin, end, starved) of the last tick
    last: Option<(f64, f64, bool)>,
}

pub const ACHIEVED_WINDOW_MS: f64 = 2000.0;

impl AchievedSpeed {
    /// A tick ran from `begin_ms` to `end_ms` on the wall clock; `starved` if it ended on its
    /// budget with the guest still wanting CPU.
    pub fn tick(&mut self, begin_ms: f64, end_ms: f64, starved: bool) {
        if let Some((b, e, was_starved)) = self.last
            && begin_ms > b
        {
            let total = begin_ms - b;
            let counted = if was_starved { (e - b).clamp(0.0, total) } else { total };
            self.intervals.push_back((total, counted));
            self.total += total;
            self.counted += counted;
            while let Some(&(t, c)) = self.intervals.front()
                && self.total - t >= ACHIEVED_WINDOW_MS
            {
                self.intervals.pop_front();
                self.total -= t;
                self.counted -= c;
            }
        }
        self.last = Some((begin_ms, end_ms, starved));
    }

    /// Forget the window — after a speed change, so the figure is about the new speed only.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// The speed achieved at play speed `speed`; `speed` itself until two ticks have run.
    pub fn get(&self, speed: f64) -> f64 {
        if self.total > 0.0 { speed * self.counted / self.total } else { speed }
    }
}

impl State {
    fn guest(&mut self, wall_ms: f64) -> f64 {
        let (wall, guest) = *self.anchor.get_or_insert((wall_ms, wall_ms));
        guest + (wall_ms - wall) * self.speed
    }

    fn read(&mut self, wall_ms: f64) -> u64 {
        let guest = self.guest(wall_ms).max(0.0) as u64;
        self.last = self.last.max(guest);
        self.last
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use core::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicBool, AtomicU64, Ordering},
        task::{Context, Poll},
    };

    use super::{AchievedSpeed, SpeedClock};
    use crate::{executor::Executor, task::YieldFuture};

    #[test]
    fn timer_fires_per_tick_is_speed_on_average_and_one_at_1x() {
        use super::timer_fires_per_tick;
        let window = |speed| (1000..1060).map(|tick| timer_fires_per_tick(speed, tick)).collect::<alloc::vec::Vec<_>>();

        assert!(window(1.0).iter().all(|&x| x == 1), "1x stays once per host tick");
        assert!(window(2.0).iter().all(|&x| x == 2));
        let half = window(1.5);
        assert!(half.iter().all(|&x| x == 1 || x == 2));
        assert_eq!(half.iter().sum::<u32>(), 90, "1.5x: 90 fires in 60 ticks");
        assert_eq!(window(1.25).iter().sum::<u32>(), 75);
        // Far into a run, where `speed * tick` is large.
        assert_eq!(timer_fires_per_tick(1.0, u32::MAX as u64 * 7), 1);
        assert_eq!(timer_fires_per_tick(f64::NAN, 3), 1);
    }

    #[test]
    fn speed_is_clamped_and_not_quantized() {
        let clock = SpeedClock::new();
        assert_eq!(clock.set_speed(0.0, 0.5), 1.0);
        assert_eq!(clock.set_speed(0.0, 2.7), 2.0);
        assert_eq!(clock.set_speed(0.0, 3.0), 2.0);
        assert_eq!(clock.set_speed(0.0, 2.0), 2.0);
        assert_eq!(clock.set_speed(0.0, f64::NAN), 1.0);
        assert_eq!(clock.set_speed(0.0, f64::INFINITY), 1.0);
        assert_eq!(clock.set_speed(0.0, 1.37), 1.37);
        assert_eq!(clock.speed(), 1.37);
    }

    #[test]
    fn untouched_clock_is_the_wall_clock() {
        let clock = SpeedClock::new();
        for wall in [1_700_000_000_000u64, 1_700_000_000_016, 1_700_000_005_000] {
            assert_eq!(clock.now(wall as f64).raw(), wall);
        }
    }

    #[test]
    fn guest_time_is_monotonic_across_speed_changes() {
        let clock = SpeedClock::new();
        let mut last = 0;
        let mut wall = 1_000_000.0;
        for (i, speed) in [2.0, 1.0, 1.75, 1.5, 1.0, 1.25].into_iter().enumerate() {
            for _ in 0..50 {
                wall += 0.7;
                let now = clock.now(wall).raw();
                assert!(now >= last, "{now} < {last}");
                last = now;
            }
            clock.set_speed(wall, speed);
            assert!(clock.now(wall).raw() >= last, "speed change {i} stepped back");
        }
        // The wall clock stepping back is held, not followed.
        assert!(clock.now(wall - 10_000.0).raw() >= last);
    }

    #[test]
    fn segments_accumulate_at_their_own_speed() {
        let clock = SpeedClock::new();
        assert_eq!(clock.now(1000.0).raw(), 1000);
        clock.set_speed(1100.0, 2.0); // +100 at 1x
        clock.set_speed(1200.0, 1.5); // +200 at 2x
        assert_eq!(clock.now(1300.0).raw(), 1000 + 100 + 200 + 150);
    }

    struct YieldOnce(bool);

    impl Future for YieldOnce {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                Poll::Pending
            }
        }
    }

    // A guest sleep(100) on a host that ticks every 16ms with the 14ms budget, the wall clock
    // advancing 0.1ms per read. Returns the wall time the sleep took.
    fn sleep_100_wakes_after(speed: f64) -> f64 {
        let clock = Arc::new(SpeedClock::new());
        let wall = Arc::new(AtomicU64::new(10_000_000)); // tenths of a ms
        clock.set_speed(wall.load(Ordering::SeqCst) as f64 / 10.0, speed);

        let mut executor = Executor::new();
        let (slept, woke) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicBool::new(false)));
        let (slept_task, woke_task, wall_task, executor_task) = (slept.clone(), woke.clone(), wall.clone(), executor.clone());
        executor.spawn(move || async move {
            slept_task.store(wall_task.load(Ordering::SeqCst), Ordering::SeqCst);
            executor_task.sleep(100);
            YieldOnce(false).await;
            slept_task.store(wall_task.load(Ordering::SeqCst) - slept_task.load(Ordering::SeqCst), Ordering::SeqCst);
            woke_task.store(true, Ordering::SeqCst);
        });

        let mut frame = wall.load(Ordering::SeqCst);
        while !woke.load(Ordering::SeqCst) {
            wall.store(frame, Ordering::SeqCst);
            let (c, w) = (clock.clone(), wall.clone());
            executor
                .tick_for(move || c.now(w.fetch_add(1, Ordering::SeqCst) as f64 / 10.0), clock.budget(14))
                .unwrap();
            frame += 160;
        }
        slept.load(Ordering::SeqCst) as f64 / 10.0
    }

    #[test]
    fn a_2x_sleep_100_wakes_after_about_50_wall_ms() {
        let one = sleep_100_wakes_after(1.0);
        let two = sleep_100_wakes_after(2.0);
        let three = sleep_100_wakes_after(3.0); // clamped to 2x
        assert!((99.0..=102.0).contains(&one), "1x: {one}ms");
        assert!((49.0..=52.0).contains(&two), "2x: {two}ms");
        assert!((49.0..=52.0).contains(&three), "3x clamped to 2x: {three}ms");
    }

    // How a guest shaped like `guest` plays at `speed` on a 60Hz host with the 14ms budget, the wall
    // clock advancing 0.1ms per read: the achieved speed after 3s of wall time.
    fn achieved<F, Fut>(speed: f64, guest: F) -> f64
    where
        F: FnOnce(Executor) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let clock = Arc::new(SpeedClock::new());
        let wall = Arc::new(AtomicU64::new(10_000_000)); // tenths of a ms
        clock.set_speed(1_000_000.0, speed);
        let mut executor = Executor::new();
        let task_executor = executor.clone();
        executor.spawn(move || async move {
            guest(task_executor).await;
            Ok::<(), wie_util::WieError>(())
        });

        let mut achieved = AchievedSpeed::default();
        let mut frame = 10_000_000u64;
        for _ in 0..180 {
            wall.store(frame, Ordering::SeqCst);
            let (c, w) = (clock.clone(), wall.clone());
            executor
                .tick_for(move || c.now(w.fetch_add(1, Ordering::SeqCst) as f64 / 10.0), clock.budget(14))
                .unwrap();
            achieved.tick(frame as f64 / 10.0, wall.load(Ordering::SeqCst) as f64 / 10.0, executor.wants_cpu());
            frame += 167; // 16.7ms
        }
        achieved.get(speed)
    }

    // Never finishes and never asks to be polled again: a JVM monitor `wait` with nobody to notify.
    struct Parked;

    impl Future for Parked {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Pending
        }
    }

    #[test]
    fn a_guest_with_cpu_to_spare_achieves_its_speed() {
        for speed in [1.0, 2.0] {
            let got = achieved(speed, |executor| async move {
                loop {
                    executor.sleep(10);
                    YieldOnce(false).await;
                }
            });
            assert_eq!(got, speed);
        }
    }

    #[test]
    fn a_guest_parked_on_a_monitor_is_not_starved() {
        // It spins every tick to its budget, but nothing in it wants the CPU.
        assert_eq!(achieved(2.0, |_| Parked), 2.0);
    }

    #[test]
    fn a_guest_that_fills_its_budget_achieves_less_than_its_speed() {
        // The ARM core and the JVM interpreter yield this way between slices of work.
        let got = achieved(2.0, |_| async {
            loop {
                YieldFuture::new().await;
            }
        });
        // 2 × 14ms of every 16.7ms frame = 1.68, plus what the tick overruns its budget by
        // (1.72 when written: the guest clock is whole milliseconds).
        assert!((1.6..1.8).contains(&got), "{got}");
    }

    #[test]
    fn a_speed_change_starts_a_new_window() {
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..200 {
            achieved.tick(t, t + 14.0, true);
            t += 16.0;
        }
        assert!(achieved.get(2.0) < 1.8);
        achieved.reset();
        assert_eq!(achieved.get(1.5), 1.5, "nothing measured yet at the new speed");
        for _ in 0..200 {
            achieved.tick(t, t + 5.0, false);
            t += 16.0;
        }
        assert_eq!(achieved.get(1.5), 1.5, "the starved ticks before the change are gone");
    }

    #[test]
    fn the_window_is_the_last_two_seconds() {
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..1000 {
            achieved.tick(t, t + 8.0, true); // half of every frame counted
            t += 16.0;
        }
        for _ in 0..200 {
            achieved.tick(t, t + 1.0, false); // 3.2s of on-schedule frames
            t += 16.0;
        }
        assert_eq!(achieved.get(1.0), 1.0);
    }
}
