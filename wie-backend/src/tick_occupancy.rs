//! Where a browser host's frame goes — emulating, waiting inside a tick for a guest's paced wake, or
//! handed back to the host — on a simulated vsync and clock, so the split is a ratio no machine's
//! load moves (docs/report/0556). `cargo test -p wie-backend tick_occupancy -- --nocapture` prints
//! the table. Its tests also hold the host-wake path (`Platform::wake_margin_ms`) to the paced
//! sleep's timing.

extern crate std;

use alloc::{sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};
use std::{println, string::String};

use spin::Mutex;

use crate::{
    executor::Executor,
    frame_pacer::FramePacer,
    speed_clock::{AchievedSpeed, SpeedClock},
    task::YieldFuture,
};

// Wall µs a clock read costs; waiting for a wake is nothing but reads.
const READ_US: u64 = 1;
// Wall µs of emulation between two yields of a busy guest.
const SLICE_US: u64 = 500;
// Wall µs of emulation in one game frame of a sleeping guest (update + paint): 2-4ms, varying, so
// a sleep does not lock onto the display's phase — a fixed one made sleep(60) at 2x land in the
// same gap after every budget, and that is a property of the simulation, not of a game.
const FRAME_SLICES: (u64, u64) = (4, 8);
// Wall µs a frame callback runs after its vsync, at most.
const CALLBACK_LATE_US: u64 = 1_000;
// Wall µs of one poll.
const POLL_WORK_US: u64 = 50;
const SECONDS: u64 = 10;

#[derive(Clone, Copy, Debug)]
enum Shape {
    // Never sleeps: a title that wants more CPU than a frame has.
    Busy,
    // `FRAME_SLICES` of work then `sleep(n)`, for ever (메이플스토리2007's sleep(60)).
    Sleep(u64),
    // An event thread polling every 1ms for a key that never comes (`POLL_SLEEP_MS`).
    Menu,
}

// The host's wake timer: fires this many µs after it was due, uniformly up to the bound.
struct Jitter(u64, u64);

impl Jitter {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if self.1 == 0 { 0 } else { (self.0 >> 33) % (self.1 + 1) }
    }
}

#[derive(Default, Debug)]
struct Split {
    wall_us: u64,
    emulate_us: u64,
    wait_us: u64,
    ticks: u64,
    // How late each `sleep(n)` woke, guest ms past `n`.
    late: Vec<u64>,
    // `AchievedSpeed` at the end, fed every tick as the browser host feeds it.
    achieved: f64,
}

impl Split {
    fn busy(&self) -> f64 {
        (self.emulate_us + self.wait_us) as f64 / self.wall_us as f64
    }

    fn pct(&self, us: u64) -> f64 {
        us as f64 * 100.0 / self.wall_us as f64
    }

    fn late_max(&self) -> u64 {
        self.late.iter().copied().max().unwrap_or(0)
    }

    fn late_p50(&self) -> u64 {
        let mut late = self.late.clone();
        late.sort_unstable();
        late.get(late.len() / 2).copied().unwrap_or(0)
    }
}

/// Runs `shape` at `speed` on a display of `period_us`, ticking once per vsync with the budget
/// `FramePacer` gives that display. `margin`: `None` is today's host; `Some(ms)` a host that ends a
/// tick on a wake further off than `ms` and comes back for it on a timer late by up to `jitter_us`.
fn run(shape: Shape, speed: f64, period_us: u64, margin: Option<u64>, jitter_us: u64) -> Split {
    let wall = Arc::new(AtomicU64::new(1_000_000));
    let emulated = Arc::new(AtomicU64::new(0));
    let late = Arc::new(Mutex::new(Vec::new()));
    let clock = Arc::new(SpeedClock::new());
    clock.set_speed(1_000.0, speed);
    let mut executor = Executor::new();

    let work = {
        let (wall, emulated) = (wall.clone(), emulated.clone());
        move |us: u64| {
            wall.fetch_add(us, Ordering::SeqCst);
            emulated.fetch_add(us, Ordering::SeqCst);
        }
    };
    let guest_now = {
        let (wall, clock) = (wall.clone(), clock.clone());
        move || clock.now(wall.load(Ordering::SeqCst) as f64 / 1000.0)
    };
    let (task_executor, task_late, task_now) = (executor.clone(), late.clone(), guest_now.clone());
    let mut frame_work = Jitter(3, FRAME_SLICES.1 - FRAME_SLICES.0);
    executor.spawn(move || async move {
        // Not `loop`: a task that never returns has type `!`, which `spawn` does not take.
        for _ in 0.. {
            match shape {
                Shape::Busy => {
                    work(SLICE_US);
                    YieldFuture::new().await;
                }
                Shape::Sleep(ms) => {
                    for _ in 0..FRAME_SLICES.0 + frame_work.next() {
                        work(SLICE_US);
                        YieldFuture::new().await;
                    }
                    let due = task_now() + ms;
                    task_executor.sleep(ms);
                    Pending(false).await;
                    task_late.lock().push(task_now().raw().saturating_sub(due.raw()));
                }
                Shape::Menu => {
                    work(POLL_WORK_US);
                    task_executor.sleep(1);
                    Pending(false).await;
                }
            }
        }
    });

    let budget = FramePacer::budget_for_period_us(period_us);
    let max_wait = margin.map(|ms| clock.budget(ms));
    let mut jitter = Jitter(7, jitter_us);
    let mut callback_late = Jitter(11, CALLBACK_LATE_US);
    let mut split = Split::default();
    let start = wall.load(Ordering::SeqCst);
    let mut achieved = AchievedSpeed::default();
    let mut tick = |executor: &mut Executor, split: &mut Split, budget: u64| {
        let (w0, e0) = (wall.load(Ordering::SeqCst), emulated.load(Ordering::SeqCst));
        let read = {
            let (wall, clock) = (wall.clone(), clock.clone());
            move || clock.now(wall.fetch_add(READ_US, Ordering::SeqCst) as f64 / 1000.0)
        };
        executor.tick_for(read, clock.budget(budget), max_wait).unwrap();
        let (w1, e1) = (wall.load(Ordering::SeqCst), emulated.load(Ordering::SeqCst));
        split.emulate_us += e1 - e0;
        split.wait_us += (w1 - w0) - (e1 - e0);
        split.ticks += 1;
        let (begin, end) = (w0 as f64 / 1000.0, w1 as f64 / 1000.0);
        let due = executor.next_due().map(|due| clock.wall_at(end, due));
        achieved.tick(begin, end, budget as f64, executor.wants_cpu(), due);
    };
    for frame in 1..=SECONDS * 1_000_000 / period_us {
        let vsync = start + frame * period_us;
        // A tick that ran past this vsync missed it; the host's next frame is the one after.
        if wall.load(Ordering::SeqCst) >= vsync {
            continue;
        }
        wall.store(vsync + callback_late.next(), Ordering::SeqCst);
        tick(&mut executor, &mut split, budget);
        // The host-wake loop (crate doc of `wie_featurephone`, `wake_in_ms`): after a tick, a timer
        // for the next paced wake, unless the guest still wants CPU or the next frame comes first.
        while let Some(ms) = margin {
            let next = executor.next_due().filter(|_| !executor.wants_cpu());
            let Some(due) = next else { break };
            let now_us = wall.load(Ordering::SeqCst);
            let fire = now_us + (clock.wake_delay(now_us as f64 / 1000.0, due, ms) * 1000.0) as u64 + jitter.next();
            // The next frame callback: the first vsync after this tick ended.
            if fire >= start + ((now_us - start) / period_us + 1) * period_us {
                break;
            }
            wall.store(fire.max(now_us), Ordering::SeqCst);
            // The margin on top: the wait before the wake is not the guest's slice (`wake_tick`).
            tick(&mut executor, &mut split, budget + ms);
        }
    }
    split.wall_us = wall.load(Ordering::SeqCst) - start;
    split.achieved = achieved.get(speed);
    split.late = core::mem::take(&mut *late.lock());
    executor.clear();
    split
}

// A future pending on its first poll: what `SleepFuture` is once `Executor::sleep` registered it.
struct Pending(bool);

impl Future for Pending {
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

const SHAPES: [Shape; 4] = [Shape::Busy, Shape::Sleep(60), Shape::Sleep(20), Shape::Menu];
const SPEEDS: [f64; 3] = [1.0, 1.5, 2.0];
const PERIODS: [u64; 2] = [16_667, 8_333];
// The margin the browser host is told to use: WebKit's setTimeout, armed from a frame callback,
// fired at most 3ms late (300 samples × 2 runs, headless, 2026-11-11 — docs/report/0556).
const MARGIN_MS: u64 = 3;
const WEBKIT_JITTER_US: u64 = 3_000;

fn row(label: &str, shape: Shape, speed: f64, period_us: u64, s: &Split) -> String {
    std::format!(
        "| {label} | {shape:?} | {speed}x | {}Hz | {:.1} | {:.1} | {:.1} | {:.1} | {} | {} | {} | {:.3} |",
        (1e6 / period_us as f64).round(),
        s.pct(s.emulate_us),
        s.pct(s.wait_us),
        s.pct(s.wall_us - s.emulate_us - s.wait_us),
        s.busy() * 100.0,
        s.late_p50(),
        s.late_max(),
        s.late.len(),
        s.achieved,
    )
}

#[test]
fn tick_occupancy_table() {
    println!("| host | shape | speed | display | emulate % | wait % | host % | busy % | late p50 | late max | frames | achieved |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    for shape in SHAPES {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                println!("{}", row("before", shape, speed, period, &before));
                println!("{}", row("after", shape, speed, period, &after));
                // The busy guest is the one the margin must not touch: it never sleeps.
                if let Shape::Busy = shape {
                    assert_eq!(before.busy(), after.busy(), "{shape:?} {speed}x {period}");
                }
            }
        }
    }
    println!();
    println!("| margin ms | timer jitter ms | Sleep(60) 1x 60Hz wait % | late p50 | late max |");
    println!("|---|---|---|---|---|");
    for jitter in [3_000, 10_000] {
        for margin in [0, 1, 2, 3, 4, 6] {
            let s = run(Shape::Sleep(60), 1.0, 16_667, Some(margin), jitter);
            println!(
                "| {margin} | {} | {:.1} | {} | {} |",
                jitter / 1000,
                s.pct(s.wait_us),
                s.late_p50(),
                s.late_max()
            );
        }
    }
}

const SLEEPERS: [Shape; 2] = [Shape::Sleep(60), Shape::Sleep(20)];

#[test]
fn a_paced_sleep_keeps_its_timing_when_the_host_wakes_it() {
    for shape in SLEEPERS {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                let case = std::format!("{shape:?} {speed}x {period}us");
                assert!(before.late.len() > 100, "{case}: {}", before.late.len());
                // As many game frames (a frame either way at the end of the run), none of them later.
                assert!(
                    after.late.len() * 100 >= before.late.len() * 99,
                    "{case}: {} vs {}",
                    after.late.len(),
                    before.late.len()
                );
                assert!(
                    after.late_max() <= before.late_max(),
                    "{case}: late max {} > {}",
                    after.late_max(),
                    before.late_max()
                );
                assert!(
                    after.late_p50() <= before.late_p50(),
                    "{case}: late p50 {} > {}",
                    after.late_p50(),
                    before.late_p50()
                );
                // Extra ticks between frames are not a slower game: `achieved_speed` stays at most
                // `speed` and within 1% of today's (measured when written: -0.7% to +1.1%).
                assert!(
                    after.achieved <= speed && after.achieved >= before.achieved * 0.99,
                    "{case}: achieved {} vs {}",
                    after.achieved,
                    before.achieved
                );
            }
        }
    }
}

#[test]
fn the_host_wake_hands_back_the_wait() {
    let (mut before_us, mut after_us) = (0, 0);
    for shape in SLEEPERS {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                // Per game frame, the wait is the margin (plus the clock's last ms), not what is left of the budget.
                let bound = after.late.len() as u64 * (MARGIN_MS + 1) * 1_000;
                assert!(after.wait_us <= bound, "{shape:?} {speed}x {period}us: wait {} > {bound}", after.wait_us);
                before_us += before.wait_us;
                after_us += after.wait_us;
            }
        }
    }
    // Measured when written: 13.1s -> 6.7s of 120s. Not lower: where a sleep keeps landing just past
    // the display's vsync, today's tick waits less than the margin does (`tick_occupancy_table`).
    assert!(after_us * 10 < before_us * 6, "wait {after_us}us vs {before_us}us");
}

#[test]
fn without_the_margin_a_late_timer_makes_the_wake_late() {
    // The margin absorbs the timer's lateness: with none, the wake is as late as its timer.
    let tight = run(Shape::Sleep(60), 1.0, 16_667, Some(0), WEBKIT_JITTER_US);
    let before = run(Shape::Sleep(60), 1.0, 16_667, None, 0);
    assert!(tight.late_p50() > before.late_p50(), "{} vs {}", tight.late_p50(), before.late_p50());
}
