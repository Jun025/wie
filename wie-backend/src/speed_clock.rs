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

    /// The wall time of guest time `guest`, seen from wall time `wall_ms`: guest time runs `speed`
    /// times as fast, so guest work due 100ms ahead at 2x is 50 wall ms ahead.
    pub fn wall_at(&self, wall_ms: f64, guest: Instant) -> f64 {
        wall_ms + (guest.raw() as f64 - self.now(wall_ms).raw() as f64) / self.speed()
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
/// behind: the guest clock itself never stops — it is the wall clock times `speed` — so measured on
/// it the ratio is `speed` by construction. Of each tick-to-tick interval, what counts runs from the
/// tick's start
///
/// - to the end of its budget (`budget_ms`), if it ran out of budget with the guest still wanting CPU
///   (`starved`). A tick runs past its budget only by a step the executor cannot cut, and a starved
///   guest's step that ran on far past it is the host's CPU falling short as well (the heavy game at
///   CPU/60: a 14ms budget, ticks of 1.7s);
/// - to `GRID_MS` past the guest's next due work (`due_ms`: a paced sleep's wake, a pending timer —
///   `Pacing::due`) if it was waiting on a time — wherever that falls, the tick itself included. A
///   game frame that ran past the game's own next frame, or a host that came back after it, is the
///   game running late: the light game on a slow device, which never starves (its frame is one step
///   the budget cannot cut) and so read as on schedule (`docs/report/0532`);
/// - to the next tick if nothing waits on a time (idle until a key): asleep on nothing is on schedule.
///
/// A gap longer than `HOST_PAUSE_MS` that is also `PAUSE_RATIO` times every interval of the last
/// `ACHIEVED_WINDOW_MS` (pauses included, so a host that hitches now and then reads its next hitch
/// as a frame) is the host not ticking — a background tab (rAF stops but the wall clock runs on),
/// the shell's pause, an app switch — not a frame: that interval is left out and the window kept,
/// so a pause holds the figure. A host that is slow *every* frame is not paused by this, however
/// slow: its frames read low.
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
    // (begin, end, until when the gap after it counts) of the last tick
    last: Option<(f64, f64, f64)>,
    // (start of the next tick, interval) of every interval of the last `ACHIEVED_WINDOW_MS`, pauses
    // included: what a gap is judged against
    recent: VecDeque<(f64, f64)>,
}

pub const ACHIEVED_WINDOW_MS: f64 = 2000.0;

// The frame grid of the 60Hz host this is measured against: a wake that waits for the next frame
// by less than a frame is how every host runs a sleep, not a slow one (`Executor::tick_for`).
const GRID_MS: f64 = 1000.0 / 60.0;

// A gap between ticks is a pause only past both: longer than this, and this many times the longest
// interval of the last `ACHIEVED_WINDOW_MS`. Under CPU throttle (docs/report/0532) a slow host's
// frames run 233ms with a longest of 1.2s, 567ms with a longest of 19s, or 40ms with 0.9s hitches
// between — the absolute bound alone read every frame of the second as a pause, and the interval
// before alone every hitch of the third, so the figure stayed near `speed`.
const HOST_PAUSE_MS: f64 = 500.0;
const PAUSE_RATIO: f64 = 8.0;

impl AchievedSpeed {
    /// A tick of `budget_ms` ran from `begin_ms` to `end_ms` on the wall clock; `starved` if it ended
    /// on its budget with the guest still wanting CPU; `due_ms` the wall time of its next due work if
    /// it was waiting on a time (`Pacing::due`).
    pub fn tick(&mut self, begin_ms: f64, end_ms: f64, budget_ms: f64, starved: bool, due_ms: Option<f64>) {
        if let Some((b, e, counts_until)) = self.last
            && begin_ms > b
        {
            let interval = begin_ms - b;
            let tick = (e - b).clamp(0.0, interval);
            let gap = interval - tick;
            while let Some(&(at, _)) = self.recent.front()
                && at < begin_ms - ACHIEVED_WINDOW_MS
            {
                self.recent.pop_front();
            }
            let longest = self.recent.iter().map(|x| x.1).fold(0.0, f64::max);
            let paused = gap > HOST_PAUSE_MS && gap > PAUSE_RATIO * longest;
            self.recent.push_back((begin_ms, interval));
            if !paused {
                let counted = (counts_until.min(begin_ms) - b).clamp(0.0, interval);
                self.intervals.push_back((interval, counted));
                self.total += interval;
                self.counted += counted;
            }
            while let Some(&(t, c)) = self.intervals.front()
                && self.total - t >= ACHIEVED_WINDOW_MS
            {
                self.intervals.pop_front();
                self.total -= t;
                self.counted -= c;
            }
        }
        let counts_until = match (starved, due_ms) {
            (true, _) => end_ms.min(begin_ms + budget_ms),
            (false, Some(due)) => due + GRID_MS,
            (false, None) => f64::INFINITY,
        };
        self.last = Some((begin_ms, end_ms, counts_until));
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

    #[test]
    fn a_guest_due_is_nearer_on_the_wall_clock_at_2x() {
        // `wie_featurephone`'s `tick` converts `Pacing::due` with this; at 2x a due 100 guest ms
        // ahead is 50 wall ms ahead, at 1x the same 100.
        let clock = SpeedClock::new();
        clock.set_speed(1000.0, 2.0);
        assert_eq!(clock.wall_at(1500.0, Instant::from_epoch_millis(2100)), 1550.0);
        clock.set_speed(1500.0, 1.0);
        assert_eq!(clock.wall_at(1500.0, Instant::from_epoch_millis(2100)), 1600.0);
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

    // How a guest shaped like `guest` plays at `speed` on a host ticking every `frame_ms` with the
    // 14ms budget, the wall clock advancing 0.1ms per read: the achieved speed after 180 ticks, as
    // the browser host feeds it (`wie_featurephone`'s `tick`).
    fn achieved_on<F, Fut>(speed: f64, frame_ms: f64, guest: F) -> f64
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
            let end = wall.load(Ordering::SeqCst) as f64 / 10.0;
            let due = executor.next_due().map(|due| clock.wall_at(end, due));
            achieved.tick(frame as f64 / 10.0, end, 14.0, executor.wants_cpu(), due);
            frame += (frame_ms * 10.0) as u64;
        }
        achieved.get(speed)
    }

    fn achieved<F, Fut>(speed: f64, guest: F) -> f64
    where
        F: FnOnce(Executor) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        achieved_on(speed, 16.7, guest)
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
        // 2 × 14ms of every 16.7ms frame = 1.68: what the tick overruns its budget by does not count.
        assert!((1.6..1.8).contains(&got), "{got}");
    }

    // A game loop that sleeps 10ms a frame: never starved, whatever the host.
    async fn sleeps_10(executor: Executor) {
        loop {
            executor.sleep(10);
            YieldOnce(false).await;
        }
    }

    #[test]
    fn a_guest_with_cpu_to_spare_on_a_slow_host_achieves_less() {
        // 4 frames a second (CPU throttle, docs/report/0532): the game wanted a frame every 10ms
        // and got one every 250ms. Before `due` it read exactly `speed` — it never starved.
        for speed in [1.0, 2.0] {
            let got = achieved_on(speed, 250.0, sleeps_10);
            assert!(got < 0.5 * speed, "speed {speed}: {got}");
        }
        // 30Hz: within a frame of the 60Hz grid's lateness, so not reported as slow.
        let got = achieved_on(2.0, 33.3, sleeps_10);
        assert!(got > 0.9 * 2.0, "30Hz: {got}");
    }

    #[test]
    fn a_guest_waiting_on_nothing_is_on_schedule_on_a_slow_host() {
        // Parked until a key: nothing is due, so a late host keeps it waiting for nothing.
        assert_eq!(achieved_on(2.0, 250.0, |_| Parked), 2.0);
    }

    #[test]
    fn frames_slower_than_the_pause_bound_still_read_low() {
        // 600ms a frame, every frame: a slow host, not a paused one. Before, each gap past 500ms
        // emptied the window and the figure stayed at `speed`.
        assert!(achieved_on(2.0, 600.0, sleeps_10) < 1.0);
        let starved = achieved_on(2.0, 600.0, |_| async {
            loop {
                YieldFuture::new().await;
            }
        });
        assert!(starved < 0.2, "{starved}");

        // A starved guest whose step runs on far past the budget: 14ms of every 1.7s frame.
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..20 {
            achieved.tick(t, t + 1700.0, 14.0, true, None);
            t += 1710.0;
        }
        let got = achieved.get(2.0);
        assert!((got - 2.0 * 14.0 / 1710.0).abs() < 1e-9, "{got}");

        // 40ms frames with a 900ms hitch every fourth: the hitches are the host, not pauses.
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for i in 0..200 {
            achieved.tick(t, t + 14.0, 14.0, true, None);
            t += if i % 4 == 3 { 900.0 } else { 40.0 };
        }
        let got = achieved.get(2.0);
        assert!(got < 0.5, "{got}");
    }

    #[test]
    fn work_that_runs_past_its_due_counts_only_to_the_grid() {
        // The light game at CPU/20 (docs/report/0532): one guest step of 155ms the 14ms budget
        // cannot cut, its next frame due 25ms in, the next tick 176ms after this one. Not starved —
        // it never asked for more — so before, the whole tick counted and it read `speed`.
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..40 {
            achieved.tick(t, t + 155.0, 14.0, false, Some(t + 25.0));
            t += 176.0;
        }
        let got = achieved.get(1.0);
        assert!((got - (25.0 + 1000.0 / 60.0) / 176.0).abs() < 1e-9, "{got}");
    }

    #[test]
    fn a_speed_change_starts_a_new_window() {
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..200 {
            achieved.tick(t, t + 14.0, 14.0, true, None);
            t += 16.0;
        }
        assert!(achieved.get(2.0) < 1.8);
        achieved.reset();
        assert_eq!(achieved.get(1.5), 1.5, "nothing measured yet at the new speed");
        for _ in 0..200 {
            achieved.tick(t, t + 5.0, 14.0, false, None);
            t += 16.0;
        }
        assert_eq!(achieved.get(1.5), 1.5, "the starved ticks before the change are gone");
    }

    #[test]
    fn the_window_is_the_last_two_seconds() {
        let mut achieved = AchievedSpeed::default();
        let mut t = 0.0;
        for _ in 0..1000 {
            achieved.tick(t, t + 8.0, 14.0, true, None); // half of every frame counted
            t += 16.0;
        }
        for _ in 0..200 {
            achieved.tick(t, t + 1.0, 14.0, false, None); // 3.2s of on-schedule frames
            t += 16.0;
        }
        assert_eq!(achieved.get(1.0), 1.0);
    }

    #[test]
    fn a_host_pause_holds_the_figure() {
        // A tab in the background, or the shell's pause: no ticks for 30s, on a device that starves
        // the guest (2 × 14ms of every 16.7ms ≈ 1.68) and on one too slow for 60Hz (a 16ms tick every
        // 600ms, the guest due 10ms after each). Before the gap the last tick was or was not starved.
        for (frame, starved, due) in [(16.7, true, None), (16.7, false, Some(0.0)), (600.0, false, Some(10.0))] {
            let tick = if starved { 14.0 } else { 16.0 };
            let mut achieved = AchievedSpeed::default();
            let mut t = 0.0;
            let feed = |achieved: &mut AchievedSpeed, t: f64| achieved.tick(t, t + tick, 14.0, starved, due.map(|d: f64| t + tick + d));
            for _ in 0..200 {
                feed(&mut achieved, t);
                t += frame;
            }
            let before = achieved.get(2.0);
            // Below `speed` where it can be, so a reset (which reads `speed`) would show; on schedule,
            // a 30s gap counted as a late frame would read ≈0.
            assert_eq!(before < 1.9, starved || frame > 100.0, "frame {frame}: {before}");
            t += 30_000.0;
            feed(&mut achieved, t);
            assert_eq!(achieved.get(2.0), before, "frame {frame}: the gap is left out, the window kept");
            for _ in 0..30 {
                t += frame;
                feed(&mut achieved, t);
            }
            assert!((achieved.get(2.0) - before).abs() < 1e-9, "frame {frame}: still the device's figure");
        }
    }
}
