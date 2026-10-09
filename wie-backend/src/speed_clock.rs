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
pub const MAX_SPEED: f64 = 3.0;

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
    /// measures its budget on the guest clock, so without this a 3x tick would end after a third
    /// of the host frame and the game would not get faster.
    pub fn budget(&self, wall_ms: u64) -> u64 {
        (wall_ms as f64 * self.speed()) as u64
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

    use super::SpeedClock;
    use crate::executor::Executor;

    #[test]
    fn speed_is_clamped_and_not_quantized() {
        let clock = SpeedClock::new();
        assert_eq!(clock.set_speed(0.0, 0.5), 1.0);
        assert_eq!(clock.set_speed(0.0, 3.7), 3.0);
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
        for (i, speed) in [2.0, 1.0, 3.0, 1.5, 1.0, 2.75].into_iter().enumerate() {
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
        clock.set_speed(1200.0, 3.0); // +200 at 2x
        assert_eq!(clock.now(1300.0).raw(), 1000 + 100 + 200 + 300);
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
        let three = sleep_100_wakes_after(3.0);
        assert!((99.0..=102.0).contains(&one), "1x: {one}ms");
        assert!((49.0..=52.0).contains(&two), "2x: {two}ms");
        assert!((33.0..=35.0).contains(&three), "3x: {three}ms");
    }
}
