use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use spin::Mutex;

use wie_util::{Result, WieError};

use crate::time::Instant;

type Task = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

// Wall-clock slice one `tick` may spend polling tasks. Hosts call `tick` once per display frame
// (the browser shell once per `requestAnimationFrame`, ~16.7ms at 60Hz), and the ARM core yields
// every `INSTRUCTIONS_PER_YIELD` instructions, so a CPU-bound guest gets exactly this share of
// each frame. At 8ms a guest frame costing 8-16ms of emulation spilled into a second host frame
// and ran at half its native rate; 14ms leaves ~2.7ms of a 60Hz frame to the host.
// This is the default for hosts that do not know their frame interval; a host that does passes
// its own budget to `tick_for` (see `FramePacer`, which derives it and never exceeds this value).
pub const TICK_BUDGET_MS: u64 = 14;

pub struct ExecutorInner {
    current_task_id: Option<usize>,
    // BTreeMap, not HashMap: task ids are monotonic, so iteration follows spawn
    // order. Hash-order polling made scheduling differ per build artifact and
    // per run, flipping boot-order-sensitive titles between PASS and blank.
    tasks: BTreeMap<usize, Task>,
    // (wake, the wake a tick stays alive for — `None` for a poll, see `POLL_SLEEP_MS`)
    sleeping_tasks: BTreeMap<usize, (Instant, Option<Instant>)>,
    last_task_id: usize,
    last_now: Instant,
}

pub trait AsyncCallable<R>: Send
where
    R: Send,
{
    fn call(self) -> impl Future<Output = R> + Send;
}

impl<F, R, Fut> AsyncCallable<R> for F
where
    F: FnOnce() -> Fut + 'static + Send,
    R: AsyncCallableResult,
    Fut: Future<Output = R> + 'static + Send,
{
    async fn call(self) -> R {
        self().await
    }
}

pub trait AsyncCallableResult: Send {
    fn err(self) -> Option<WieError>;
}

impl<R> AsyncCallableResult for core::result::Result<R, WieError>
where
    R: Send,
{
    fn err(self) -> Option<WieError> {
        self.err()
    }
}

impl AsyncCallableResult for () {
    fn err(self) -> Option<WieError> {
        None
    }
}

#[derive(Clone)]
pub struct Executor {
    inner: Arc<Mutex<ExecutorInner>>,
    // Set by `halt`: no task is polled again. See there. Outside the lock because it is read once
    // per `tick_for` iteration and once per task in `step`, and it only ever goes false -> true:
    // taking the lock for it cost +3~4% CPU per tick (docs/report/0500).
    halted: Arc<AtomicBool>,
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(ExecutorInner {
            current_task_id: None,
            tasks: BTreeMap::new(),
            sleeping_tasks: BTreeMap::new(),
            last_task_id: 0,
            last_now: Instant::from_epoch_millis(0),
        }));

        Self {
            inner,
            halted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn<C, R>(&self, callable: C) -> usize
    where
        C: AsyncCallable<R> + 'static,
        R: AsyncCallableResult,
    {
        let fut = async move {
            let result = callable.call().await;
            if let Some(err) = result.err() {
                return Err(err);
            }

            Ok(())
        };

        let task_id = {
            let mut inner = self.inner.lock();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.lock().tasks.insert(task_id, Box::pin(fut));

        task_id
    }

    /// Drop every task. Tasks hold clones of what owns this executor (the system, the core, the
    /// JVM), so an emulator whose tasks are never dropped is never freed. Dropped outside the
    /// lock: a task's drop may reach back into the executor.
    pub fn clear(&self) {
        let tasks = {
            let mut inner = self.inner.lock();
            inner.sleeping_tasks.clear();
            core::mem::take(&mut inner.tasks)
        };
        drop(tasks);
    }

    /// Stop polling tasks, from the task that is running now on. A program that exits is gone on a
    /// handset — no thread of it runs again — but here every other task went on being polled for
    /// the rest of the tick. The task that asked is not stopped by this; see
    /// `System::exit_from_guest`. Tasks are kept, not dropped: dropping them is `clear`'s, the host's.
    pub fn halt(&self) {
        self.halted.store(true, Ordering::Relaxed);
    }

    // TODO we need to remove error handling from here. we need to JoinHandle like on spawn..
    pub fn tick<T>(&mut self, now: T) -> Result<()>
    where
        T: Fn() -> Instant,
    {
        self.tick_for(now, TICK_BUDGET_MS)
    }

    /// `tick` with a host-chosen wall-clock budget in milliseconds instead of `TICK_BUDGET_MS`.
    pub fn tick_for<T>(&mut self, now: T, budget_ms: u64) -> Result<()>
    where
        T: Fn() -> Instant,
    {
        let end = now() + budget_ms;
        loop {
            if self.halted.load(Ordering::Relaxed) {
                break;
            }
            let mut current = now();

            if current > end {
                break;
            }

            let next_wakeup = {
                let inner = self.inner.lock();
                let running_task_count = inner.tasks.len() - inner.sleeping_tasks.len();
                if running_task_count == 0 && !inner.sleeping_tasks.is_empty() {
                    let next = inner.sleeping_tasks.values().map(|x| x.0).min().unwrap();
                    let paced = inner.sleeping_tasks.values().filter_map(|x| x.1).min();
                    Some((next, paced))
                } else {
                    None
                }
            };

            // Every task is asleep. Ending the tick here used to put each wake on the host's frame
            // grid: 메이플스토리2007's sleep(60) woke 66.7 or 83.3ms later, never 60. A paced wake
            // that falls inside the budget is waited for instead — spending CPU the budget already
            // allowed — and one past it still ends the tick.
            if let Some((next_wakeup, paced)) = next_wakeup
                && current < next_wakeup
            {
                if paced.is_none_or(|paced| paced > end) {
                    break;
                }
                match wait_until(&now, current, next_wakeup) {
                    Some(woke) => current = woke,
                    None => break,
                }
            }

            self.step(current)?;
        }

        Ok(())
    }

    pub fn current_task_id(&self) -> u64 {
        self.inner.lock().current_task_id.unwrap() as _
    }

    fn step(&mut self, now: Instant) -> Result<()> {
        self.inner.lock().last_now = now;

        let mut next_tasks = BTreeMap::new();
        let tasks = core::mem::take(&mut self.inner.lock().tasks);
        let mut sleeping_tasks = core::mem::take(&mut self.inner.lock().sleeping_tasks);

        // ascending task id == spawn order; keeps dispatch deterministic
        let mut first_error = None;

        for (task_id, mut task) in tasks.into_iter() {
            if self.halted.load(Ordering::Relaxed) {
                next_tasks.insert(task_id, task);
                continue;
            }
            let item = sleeping_tasks.get(&task_id);
            if let Some(item) = item {
                if item.0 <= now {
                    sleeping_tasks.remove(&task_id);
                } else {
                    next_tasks.insert(task_id, task);
                    continue;
                }
            }

            let waker = self.create_waker();
            let mut context = Context::from_waker(&waker);
            self.inner.lock().current_task_id = Some(task_id);

            match task.as_mut().poll(&mut context) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
                Poll::Pending => {
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.lock().current_task_id = None;
        }

        self.inner.lock().sleeping_tasks.extend(sleeping_tasks);
        self.inner.lock().tasks.extend(next_tasks);

        if let Some(err) = first_error { Err(err) } else { Ok(()) }
    }

    pub(crate) fn sleep(&self, timeout: u64) {
        self.sleep_toward(timeout, None);
    }

    // A sleep of `timeout` that keeps a tick alive for `pace` instead of its own wake: a thread
    // polling every millisecond for a WIPI timer is waiting for the timer, not for the poll.
    pub(crate) fn sleep_toward(&self, timeout: u64, pace: Option<Instant>) {
        let task_id = self.inner.lock().current_task_id.unwrap();

        let until = self.inner.lock().last_now + timeout;
        let pace = pace.or((timeout > POLL_SLEEP_MS).then_some(until));
        self.inner.lock().sleeping_tasks.insert(task_id, (until, pace));
    }

    fn create_waker(&self) -> Waker {
        unsafe fn noop_clone(_data: *const ()) -> RawWaker {
            noop_raw_waker()
        }

        unsafe fn noop(_data: *const ()) {}

        const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

        const fn noop_raw_waker() -> RawWaker {
            RawWaker::new(core::ptr::null(), &NOOP_WAKER_VTABLE)
        }

        unsafe { Waker::from_raw(noop_raw_waker()) }
    }
}

// A sleep this short is a poll, not a pace: KTF 영웅서기4 re-arms MC_knlSetTimer(1) every frame to
// mean "as soon as you can", and the MIDP event thread checks its queue every 1ms. Only a longer
// sleep keeps a tick alive; waiting on these too turned 영웅서기4 from 36 into 93 frames/s — the
// game itself 2.6x faster — and spun every idle tick to its budget.
const POLL_SLEEP_MS: u64 = 1;

// A clock read this many times in a row without moving is not going to move: a test's frozen
// clock (`TestClock`), where waiting for a wake would never return. A real millisecond clock
// moves long before this — a read costs tens of nanoseconds on both hosts.
// ponytail: read-count heuristic; a host whose clock read takes ~1ns would end ticks early.
const FROZEN_CLOCK_READS: u32 = 1_000_000;

// Reads `now` until it reaches `until`; `None` if the clock stops moving first.
fn wait_until<T>(now: &T, mut last: Instant, until: Instant) -> Option<Instant>
where
    T: Fn() -> Instant,
{
    let mut unchanged = 0;
    while last < until {
        let read = now();
        if read == last {
            unchanged += 1;
            if unchanged >= FROZEN_CLOCK_READS {
                return None;
            }
        } else {
            unchanged = 0;
            last = read;
        }
    }
    Some(last)
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use core::{
        cell::Cell,
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicBool, AtomicU64, Ordering},
        task::{Context, Poll},
    };

    use wie_util::WieError;

    use super::Executor;
    use crate::time::Instant;

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

    fn advancing_clock(start: u64) -> impl Fn() -> Instant {
        let time = Cell::new(start);
        move || {
            let now = time.get();
            time.set(now + 1);
            Instant::from_epoch_millis(now)
        }
    }

    #[test]
    fn test_tick_gives_a_yielding_task_most_of_a_60hz_frame() {
        let mut executor = Executor::new();

        let polls = Arc::new(AtomicU64::new(0));
        let polls_clone = polls.clone();
        executor.spawn(move || async move {
            // Far more work than one slice can hold.
            for _ in 0..1000 {
                polls_clone.fetch_add(1, Ordering::Relaxed);
                YieldOnce(false).await;
            }
        });

        // The clock advances 1ms per read and the executor reads it once per step, so the task is
        // polled once per millisecond of the slice. A literal, not `TICK_BUDGET_MS`: comparing the
        // constant to itself passes at any value, and a merge that restores the old 8ms budget
        // (17→35fps on KTF 영웅서기4) must turn this red.
        executor.tick(advancing_clock(0)).unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 14);
    }

    #[test]
    fn test_halt_stops_every_task_including_later_ones_in_the_same_step() {
        // One thread exits; another — spawned later, so polled after it in the same step — must not
        // run again, in this tick or any later one.
        let mut executor = Executor::new();
        let after_exit = Arc::new(AtomicU64::new(0));

        let halter = executor.clone();
        executor.spawn(move || async move {
            halter.halt();
            YieldOnce(false).await;
        });
        let after_exit_clone = after_exit.clone();
        executor.spawn(move || async move {
            for _ in 0..1000 {
                after_exit_clone.fetch_add(1, Ordering::Relaxed);
                YieldOnce(false).await;
            }
        });

        executor.tick(advancing_clock(0)).unwrap();
        executor.tick(advancing_clock(100)).unwrap();
        assert_eq!(after_exit.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_tick_for_spends_the_host_budget_not_the_default() {
        let mut executor = Executor::new();

        let polls = Arc::new(AtomicU64::new(0));
        let polls_clone = polls.clone();
        executor.spawn(move || async move {
            for _ in 0..1000 {
                polls_clone.fetch_add(1, Ordering::Relaxed);
                YieldOnce(false).await;
            }
        });

        // A 120Hz host's budget (`FramePacer::budget_for_period_us(8_333)`); ignoring it and
        // spending the 14ms default is exactly the overrun of an 8.3ms frame this API exists to prevent.
        executor.tick_for(advancing_clock(0), 5).unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 5);
    }

    fn sleeper(executor: &mut Executor, millis: u64) -> Arc<AtomicBool> {
        let woke = Arc::new(AtomicBool::new(false));
        let woke_clone = woke.clone();
        let executor_clone = executor.clone();
        executor.spawn(move || async move {
            executor_clone.sleep(millis);
            YieldOnce(false).await;
            woke_clone.store(true, Ordering::Relaxed);
        });
        woke
    }

    #[test]
    fn test_tick_waits_for_a_wake_inside_its_budget() {
        let mut executor = Executor::new();
        // 10ms sits inside the 14ms budget: the same tick must wake it, not leave it to the next
        // host frame. Ending the tick when every task sleeps is what stretched sleep(60) to 66.7/83.3ms.
        let woke = sleeper(&mut executor, 10);
        executor.tick(advancing_clock(0)).unwrap();
        assert!(woke.load(Ordering::Relaxed));
    }

    #[test]
    fn test_tick_ends_on_a_wake_past_its_budget() {
        let mut executor = Executor::new();
        let woke = sleeper(&mut executor, 30);
        executor.tick(advancing_clock(0)).unwrap();
        assert!(!woke.load(Ordering::Relaxed));
        executor.tick(advancing_clock(40)).unwrap();
        assert!(woke.load(Ordering::Relaxed));
    }

    #[test]
    fn test_tick_does_not_wait_for_a_1ms_poll() {
        let mut executor = Executor::new();
        // MC_knlSetTimer(1) every frame (KTF 영웅서기4): waiting on it lets such a loop run as fast
        // as the emulator instead of once per host frame.
        let woke = sleeper(&mut executor, 1);
        // 1ms every 4 reads, so the wake is still ahead when the task goes to sleep.
        let reads = Cell::new(0u64);
        let slow = || {
            reads.set(reads.get() + 1);
            Instant::from_epoch_millis(reads.get() / 4)
        };
        executor.tick(slow).unwrap();
        assert!(!woke.load(Ordering::Relaxed));
    }

    #[test]
    fn test_tick_waits_for_the_pace_of_a_poll() {
        let mut executor = Executor::new();
        // net.wie.EventQueue waiting for a WIPI timer due at 10ms: it polls every 1ms, and the tick
        // must stay alive for the timer as it would for a sleep(10). Without the pace it is the
        // bare 1ms poll above, and the timer fires on the host's next frame.
        let woke = Arc::new(AtomicBool::new(false));
        let (woke_task, executor_task) = (woke.clone(), executor.clone());
        executor.spawn(move || async move {
            executor_task.sleep_toward(1, Some(Instant::from_epoch_millis(10)));
            YieldOnce(false).await;
            woke_task.store(true, Ordering::Relaxed);
        });
        let reads = Cell::new(0u64);
        let slow = || {
            reads.set(reads.get() + 1);
            Instant::from_epoch_millis(reads.get() / 4)
        };
        executor.tick(slow).unwrap();
        assert!(woke.load(Ordering::Relaxed));
    }

    #[test]
    fn test_tick_ends_when_the_clock_is_frozen() {
        let mut executor = Executor::new();
        let woke = sleeper(&mut executor, 10);
        // A `TestClock` that nobody advances. Waiting for the wake would never return; the read cap
        // turns that hang into a failure instead of a stuck test run.
        let reads = Cell::new(0u64);
        let frozen = || {
            reads.set(reads.get() + 1);
            assert!(reads.get() < 10_000_000, "tick kept waiting on a clock that never moves");
            Instant::from_epoch_millis(5)
        };
        executor.tick(frozen).unwrap();
        assert!(!woke.load(Ordering::Relaxed));
    }

    #[test]
    fn test_failed_task_preserves_others() {
        let mut executor = Executor::new();

        executor.spawn(|| async { Err::<(), _>(WieError::FatalError("test error".into())) });

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();
        executor.spawn(move || async move {
            YieldOnce(false).await;
            completed_clone.store(true, Ordering::Relaxed);
        });

        assert!(executor.tick(advancing_clock(0)).is_err());
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(100)).unwrap();
        assert!(completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_failed_task_preserves_sleeping_tasks() {
        let mut executor = Executor::new();

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();
        let executor_clone = executor.clone();
        executor.spawn(move || async move {
            executor_clone.sleep(100);
            YieldOnce(false).await;
            completed_clone.store(true, Ordering::Relaxed);
        });

        executor.spawn(|| async { Err::<(), _>(WieError::FatalError("test error".into())) });

        assert!(executor.tick(advancing_clock(0)).is_err());
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(50)).unwrap();
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(200)).unwrap();
        assert!(completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_sleep_wakes_on_the_first_step_at_or_after_its_deadline() {
        let mut executor = Executor::new();

        // One clock for the executor (1ms per read) and the task (reads without stepping).
        let clock = Arc::new(AtomicU64::new(0));
        let slept = Arc::new(AtomicU64::new(u64::MAX));
        let woke = Arc::new(AtomicU64::new(u64::MAX));
        let (clock_task, slept_task, woke_task, executor_task) = (clock.clone(), slept.clone(), woke.clone(), executor.clone());
        executor.spawn(move || async move {
            slept_task.store(clock_task.load(Ordering::SeqCst), Ordering::SeqCst);
            executor_task.sleep(20);
            YieldOnce(false).await;
            woke_task.store(clock_task.load(Ordering::SeqCst), Ordering::SeqCst);
        });

        // A host that ticks back to back, as wie_validate does: each tick ends when every task
        // sleeps, and the next one starts at once. A wake held past its deadline — rounded to a
        // host frame, or waiting out a fixed slice as #291's getNextEvent did — shows up here.
        while woke.load(Ordering::SeqCst) == u64::MAX {
            let clock = clock.clone();
            executor
                .tick(move || Instant::from_epoch_millis(clock.fetch_add(1, Ordering::SeqCst)))
                .unwrap();
        }
        let late = woke.load(Ordering::SeqCst) - slept.load(Ordering::SeqCst);
        assert!((20..=22).contains(&late), "sleep(20) woke after {late}ms");
    }

    #[test]
    fn test_all_ok_tasks_complete() {
        let mut executor = Executor::new();

        let completed_a = Arc::new(AtomicBool::new(false));
        let completed_a_clone = completed_a.clone();
        executor.spawn(move || async move {
            completed_a_clone.store(true, Ordering::Relaxed);
        });

        let completed_b = Arc::new(AtomicBool::new(false));
        let completed_b_clone = completed_b.clone();
        executor.spawn(move || async move {
            YieldOnce(false).await;
            completed_b_clone.store(true, Ordering::Relaxed);
        });

        executor.tick(advancing_clock(0)).unwrap();

        assert!(completed_a.load(Ordering::Relaxed));
        assert!(completed_b.load(Ordering::Relaxed));
    }
}
