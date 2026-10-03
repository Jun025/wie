mod audio;
mod event_queue;
mod file_system;

use alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc};
use core::sync::atomic::{AtomicBool, Ordering};

use spin::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};

use wie_util::Result;

use crate::{
    AsyncCallable,
    canvas::{Image, ImageBuffer},
    executor::{Executor, TICK_BUDGET_MS},
    pacing::Pacing,
    platform::Platform,
    task::{SleepFuture, YieldFuture},
    task_runner::TaskRunner,
    time::Instant,
};

use self::{audio::Audio, event_queue::EventQueue};

pub use self::{
    event_queue::{Event, KeyCode},
    file_system::FilesystemOverlay,
};

#[derive(Clone)]
pub struct System {
    pid: String,
    aid: String,
    executor: Executor,
    platform: Arc<Box<dyn Platform>>,
    filesystem: FilesystemOverlay,
    event_queue: Arc<RwLock<EventQueue>>,
    audio: Arc<RwLock<Audio>>,
    task_runner: Arc<dyn TaskRunner>,
    random_state: Arc<RwLock<u32>>,
    redraw_pending: Arc<AtomicBool>,
    // (task, yields since that task last slept)
    yield_streak: Arc<Mutex<(u64, u32)>>,
    pacing: Arc<Mutex<Pacing>>,
    screen_compositor: Arc<Mutex<Option<ScreenCompositor>>>,
}

/// Draws a guest-native screen onto the Java screen image before the host presents it: the image
/// as it stands, and a writer onto the same pixels. Returns whether any native pixel went onto it.
pub type ScreenCompositor = Box<dyn FnMut(&dyn Image, &mut dyn ImageBuffer) -> bool + Send>;

impl System {
    pub fn new<T>(platform: Box<dyn Platform>, pid: &str, aid: &str, task_runner: T) -> Self
    where
        T: TaskRunner + 'static,
    {
        let audio_sink = platform.audio_sink();
        let platform = Arc::new(platform);

        Self {
            pid: pid.to_owned(),
            aid: aid.to_owned(), // TODO create metadata dictionary or something
            executor: Executor::new(),
            filesystem: FilesystemOverlay::new(platform.clone(), aid),
            platform,
            event_queue: Arc::new(RwLock::new(EventQueue::new())),
            audio: Arc::new(RwLock::new(Audio::new(audio_sink))),
            task_runner: Arc::new(task_runner),
            random_state: Arc::new(RwLock::new(1)),
            redraw_pending: Arc::new(AtomicBool::new(false)),
            yield_streak: Arc::new(Mutex::new((0, 0))),
            pacing: Arc::new(Mutex::new(Pacing::default())),
            screen_compositor: Arc::new(Mutex::new(None)),
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.tick_for(TICK_BUDGET_MS)
    }

    pub fn tick_for(&mut self, budget_ms: u64) -> Result<()> {
        let platform = self.platform.clone();
        let result = self.executor.tick_for(move || platform.now(), budget_ms);
        self.flush_redraw();
        self.pacing.lock().tick_ended();
        result
    }

    /// A guest repaint. Its Redraw reaches the event queue at the end of the tick — when the host
    /// used to deliver it — unless a guest thread is spinning on Thread.yield first: a second yield
    /// from one thread with no sleep of its own between is a thread waiting for something, and
    /// 배틀몬스터's game thread waits for its paint that way (the tick end cost it a host frame per
    /// game frame: 13.9 -> 20fps of its 20). A single yield is left alone — 메이플스토리2007 yields
    /// once between repaint and sleep, and a paint started there delayed its sleep past a host frame.
    pub fn request_redraw(&self) {
        self.redraw_pending.store(true, Ordering::Release);
    }

    pub fn guest_yielded(&self) {
        self.pacing.lock().guest_yielded();
        let task = self.current_task_id();
        let spinning = {
            let mut streak = self.yield_streak.lock();
            *streak = if streak.0 == task { (task, streak.1 + 1) } else { (task, 1) };
            streak.1 >= 2
        };
        if spinning {
            self.flush_redraw();
        }
    }

    pub fn guest_slept(&self) {
        let task = self.current_task_id();
        let mut streak = self.yield_streak.lock();
        if streak.0 == task {
            streak.1 = 0;
        }
    }

    pub fn flush_redraw(&self) {
        if self.redraw_pending.swap(false, Ordering::AcqRel) {
            self.event_queue().push(Event::Redraw);
        }
    }

    /// A guest `Thread.sleep`: records how late it woke (`Pacing`).
    pub async fn guest_sleep(&self, timeout: u64) {
        self.guest_slept();
        let due = self.platform.now() + timeout;
        self.sleep(timeout).await;
        let late = self.platform.now().raw().saturating_sub(due.raw());
        self.pacing.lock().guest_slept(timeout, late);
    }

    pub fn pacing(&self) -> MutexGuard<'_, Pacing> {
        self.pacing.lock()
    }

    pub fn spawn<C>(&self, callable: C)
    where
        C: AsyncCallable<Result<()>> + 'static + Send,
    {
        let runner_clone = self.task_runner.clone();
        self.executor.spawn(async move || runner_clone.run(Box::pin(callable.call())).await);
    }

    pub fn sleep(&self, timeout: u64) -> SleepFuture {
        SleepFuture::new(timeout, &self.executor)
    }

    /// `sleep(timeout)` on behalf of a wake at `pace`: the tick stays alive for `pace` as it would
    /// for a sleep that long, so a thread that polls every millisecond for a timer due at `pace`
    /// still runs it on time rather than on the host's next frame.
    pub fn sleep_toward(&self, timeout: u64, pace: Instant) -> SleepFuture {
        SleepFuture::toward(timeout, pace, &self.executor)
    }

    pub fn current_task_id(&self) -> u64 {
        self.executor.current_task_id()
    }

    pub fn yield_now(&self) -> YieldFuture {
        YieldFuture::new()
    }

    /// Unified filesystem view. Reads consult the persistent platform
    /// backend first and fall back to the in-memory virtual layer loaded
    /// from archives; writes always hit the platform backend.
    pub fn filesystem(&self) -> &FilesystemOverlay {
        &self.filesystem
    }

    pub fn pid(&self) -> &str {
        &self.pid
    }

    pub fn aid(&self) -> &str {
        &self.aid
    }

    pub fn random_state(&self) -> u32 {
        *self.random_state.read()
    }

    pub fn set_random_state(&self, random_state: u32) {
        *self.random_state.write() = random_state;
    }

    pub fn platform(&self) -> &dyn Platform {
        self.platform.as_ref().as_ref()
    }

    pub fn audio(&self) -> RwLockWriteGuard<'_, Audio> {
        self.audio.as_ref().write()
    }

    pub fn event_queue(&self) -> RwLockWriteGuard<'_, EventQueue> {
        self.event_queue.write()
    }

    /// Break the reference cycles through this system so the emulator owning it can be freed:
    /// its tasks and the screen compositor capture clones of the system and the core.
    pub fn teardown(&self) {
        self.executor.clear();
        let compositor = self.screen_compositor.lock().take();
        drop(compositor);
    }

    pub fn set_screen_compositor(&self, compositor: ScreenCompositor) {
        *self.screen_compositor.lock() = Some(compositor);
    }

    pub fn has_screen_compositor(&self) -> bool {
        self.screen_compositor.lock().is_some()
    }

    pub fn compose_screen(&self, current: &dyn Image, target: &mut dyn ImageBuffer) -> bool {
        self.screen_compositor
            .lock()
            .as_mut()
            .is_some_and(|compositor| compositor(current, target))
    }
}
