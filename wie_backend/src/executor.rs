use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use spin::Mutex;

use wie_util::{Result, WieError};

use crate::time::Instant;

type Task = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub struct ExecutorInner {
    current_task_id: Option<usize>,
    // BTreeMap, not HashMap: task ids are monotonic, so iteration follows spawn
    // order. Hash-order polling made scheduling differ per build artifact and
    // per run, flipping boot-order-sensitive titles between PASS and blank.
    tasks: BTreeMap<usize, Task>,
    sleeping_tasks: BTreeMap<usize, Instant>,
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

        Self { inner }
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

    // TODO we need to remove error handling from here. we need to JoinHandle like on spawn..
    pub fn tick<T>(&mut self, now: T) -> Result<()>
    where
        T: Fn() -> Instant,
    {
        let end = now() + 8; // TODO hardcoded

        // Conservative boot-flip mitigation. The 8ms wall-clock budget alone ties
        // the amount of work done per tick to real elapsed time, so under host
        // load or coarse/jittery `Platform::now()` (e.g. browser `Date.now()`) a
        // frame can advance the emulator by wildly different amounts — even zero
        // steps if two `now()` reads straddle the 8ms window. That makes
        // boot-order-sensitive titles flip between booting and a blank screen.
        //
        // Guaranteeing a small floor of executor steps whenever there is runnable
        // work makes boot progress consistently regardless of real-time jitter,
        // while the idle break below (all tasks sleeping) still yields immediately
        // in the steady frame loop — so gameplay pace is unchanged. MAX caps the
        // loop so a permanently-runnable task can never freeze the caller (the rAF
        // frame / CLI loop); the wall-clock budget still governs work above the
        // floor. This is a mitigation, not a determinism fix — the real cure is a
        // virtual/fixed-timestep clock decoupled from wall time.
        const MIN_STEPS_PER_TICK: u32 = 64;
        const MAX_STEPS_PER_TICK: u32 = 65_536;
        let mut steps: u32 = 0;

        loop {
            if steps >= MAX_STEPS_PER_TICK {
                break;
            }

            let now = now();

            // Honour the wall-clock budget only once the deterministic step floor
            // is met, so a slow/jittery host frame can't starve boot to a blank.
            if steps >= MIN_STEPS_PER_TICK && now > end {
                break;
            }

            {
                let inner = self.inner.lock();
                let running_task_count = inner.tasks.len() - inner.sleeping_tasks.len();
                if running_task_count == 0 && !inner.sleeping_tasks.is_empty() {
                    let next_wakeup = *inner.sleeping_tasks.values().min().unwrap();
                    if now < next_wakeup {
                        // Nothing runnable this instant — yield regardless of the
                        // step floor so idle frames don't spin (keeps real-time pace).
                        break;
                    }
                }
            }

            self.step(now)?;
            steps += 1;
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
        for (task_id, mut task) in tasks.into_iter() {
            let item = sleeping_tasks.get(&task_id);
            if let Some(item) = item {
                if *item <= now {
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
                Poll::Ready(x) => {
                    x?;
                }
                Poll::Pending => {
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.lock().current_task_id = None;
        }

        self.inner.lock().sleeping_tasks.extend(sleeping_tasks);
        self.inner.lock().tasks.extend(next_tasks);

        Ok(())
    }

    pub(crate) fn sleep(&self, timeout: u64) {
        let task_id = self.inner.lock().current_task_id.unwrap();

        let until = self.inner.lock().last_now + timeout;
        self.inner.lock().sleeping_tasks.insert(task_id, until);
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
