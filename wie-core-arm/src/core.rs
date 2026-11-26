use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, sync::Arc, vec, vec::Vec};
use core::{
    mem::size_of,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use spin::Mutex;

use wie_backend::{Options, ProfileCallback, ProfileSample, YieldFuture};
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use crate::{
    EmulatedFunction, ResultWriter, ThreadId,
    context::ArmCoreContext,
    engine::{Arm32CpuEngine, ArmEngine, ArmRegister, DebugInner, DebuggedArm32CpuEngine, EngineStopReason, MemoryPermission},
    function::{RegisteredFunction, RegisteredFunctionHolder},
    thread::ThreadState,
    thread_wrapper::ArmCoreThreadWrapper,
};

const GLOBAL_DATA_BASE: u32 = 0x7fff0000;
const FUNCTIONS_BASE: u32 = 0x71000000;
const FUNCTIONS_SIZE: usize = 0x10000;
const SVC_STUB_SIZE: u32 = 16;
/// How many SVC stubs fit in the stub region — `make_svc_stub` fails past this.
pub const SVC_STUB_CAPACITY: u32 = FUNCTIONS_SIZE as u32 / SVC_STUB_SIZE;

/// Most SVC stubs any `ArmCore` in this process has bound. Process-wide on purpose: it is read
/// by `wie_validate` (one emulator per process) for its `svc_stub_slots` field, which has no
/// handle on the core. A counter rather than a trace event because enabling TRACE on any target
/// raises `tracing`'s global level hint, and that alone halved keydraw paints (measured).
static SVC_STUBS_HIGH_WATER: AtomicU32 = AtomicU32::new(0);

/// See [`SVC_STUBS_HIGH_WATER`].
pub fn svc_stub_high_water() -> u32 {
    SVC_STUBS_HIGH_WATER.load(Ordering::Relaxed)
}
const INSTRUCTIONS_PER_YIELD: u32 = 10_000;
/// Most rounds a thread waits for a sliced-out holder (`hold_others`) before running anyway.
const HOLD_ROUNDS: u32 = 200;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const HEAP_SIZE: u32 = 0x10000000;

/// Limit on stack frames recorded per sample. Bounds memory and protects
/// against runaway loops if the R7 chain forms a cycle.
const PROFILE_MAX_STACK: usize = 32;
/// Flush the per-stack counter map every this many samples taken.
const PROFILE_FLUSH_INTERVAL: u32 = 1000;

struct ProfileState {
    samples: BTreeMap<Vec<u32>, u64>,
    counter: u32,
    callback: ProfileCallback,
}

pub(crate) struct ArmCoreInner {
    pub(crate) engine: Box<dyn ArmEngine>,
    instructions_remaining: u32,
    last_thread_id: ThreadId,
    current_thread_id: Option<ThreadId>,
    svc_handlers: BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>,
    next_stub_address: u32,
    shared_stubs: BTreeMap<(u32, u32), u32>,
    profile: Option<ProfileState>,
    teardown_hooks: Vec<Box<dyn FnOnce() + Send>>,
}

/// `ArmCore`s alive in this process. Read by tests and `wie_validate` to show an emulator is freed.
static LIVE_CORES: AtomicU32 = AtomicU32::new(0);

/// See [`LIVE_CORES`].
pub fn live_cores() -> u32 {
    LIVE_CORES.load(Ordering::Relaxed)
}

impl Drop for ArmCoreInner {
    fn drop(&mut self) {
        LIVE_CORES.fetch_sub(1, Ordering::Relaxed);
        if let Some(mut profile) = self.profile.take() {
            let batch = drain_samples(&mut profile.samples);
            if !batch.is_empty() {
                (profile.callback)(batch);
            }
        }
    }
}

fn drain_samples(samples: &mut BTreeMap<Vec<u32>, u64>) -> Vec<ProfileSample> {
    core::mem::take(samples)
        .into_iter()
        .map(|(stack, count)| ProfileSample { stack, count })
        .collect()
}

#[derive(Clone)]
pub struct ArmCore {
    pub(crate) inner: Arc<Mutex<ArmCoreInner>>, // TODO can we change it to another lock like async-lock?
    threads: Arc<Mutex<BTreeMap<ThreadId, ThreadState>>>,
    // The guest registers each in-flight `run_function` put aside, keyed by call. While host code
    // runs a nested guest call, these are the only copy of what the outer guest code held — the
    // arguments of the host method it called included. See `guest_root_words`.
    saved_contexts: Arc<Mutex<BTreeMap<u64, ArmCoreContext>>>,
    // The threads holding the others. See `hold_others`.
    holders: Arc<Mutex<Vec<ThreadId>>>,
}

static NEXT_SAVED_CONTEXT: AtomicU64 = AtomicU64::new(0);

struct SavedContextGuard {
    saved_contexts: Arc<Mutex<BTreeMap<u64, ArmCoreContext>>>,
    key: u64,
}

impl Drop for SavedContextGuard {
    fn drop(&mut self) {
        self.saved_contexts.lock().remove(&self.key);
    }
}

impl ArmCore {
    pub fn new(options: Options) -> Result<Self> {
        let Options {
            enable_gdbserver,
            aot,
            profile,
        } = options;
        let mut engine = if enable_gdbserver {
            Box::new(DebuggedArm32CpuEngine::new()) as Box<dyn ArmEngine>
        } else {
            Box::new(Arm32CpuEngine::with_backend(aot))
        };

        engine.mem_map(FUNCTIONS_BASE, FUNCTIONS_SIZE, MemoryPermission::ReadExecute);
        engine.mem_map(GLOBAL_DATA_BASE, 0x4000, MemoryPermission::ReadWriteExecute);

        let profile = profile.map(|callback| ProfileState {
            samples: BTreeMap::new(),
            counter: 0,
            callback,
        });

        let inner = ArmCoreInner {
            engine,
            instructions_remaining: INSTRUCTIONS_PER_YIELD,
            last_thread_id: 0,
            current_thread_id: None,
            svc_handlers: BTreeMap::new(),
            next_stub_address: FUNCTIONS_BASE,
            shared_stubs: BTreeMap::new(),
            profile,
            teardown_hooks: Vec::new(),
        };

        LIVE_CORES.fetch_add(1, Ordering::Relaxed);
        let result = Self {
            inner: Arc::new(Mutex::new(inner)),
            threads: Arc::new(Mutex::new(BTreeMap::new())),
            saved_contexts: Arc::new(Mutex::new(BTreeMap::new())),
            holders: Arc::new(Mutex::new(Vec::new())),
        };

        if enable_gdbserver {
            crate::gdb::start(result.clone())?;
        }

        Ok(result)
    }

    /// Identity of this emulated machine, stable while any clone of it is alive. For host-side
    /// state that must be kept per core (two emulators can hand out the same guest addresses).
    pub fn id(&self) -> usize {
        Arc::as_ptr(&self.inner) as *const () as usize
    }

    /// Break the reference cycles through this core so it can be freed: SVC handler contexts hold
    /// the JVM and the system, which hold clones of this core. Call once the emulator is done —
    /// the core cannot run guest code that reaches a handler afterwards. Dropped outside the
    /// locks, since freeing a JVM or a thread stack locks the core again.
    pub fn teardown(&self) {
        let hooks = core::mem::take(&mut self.inner.lock().teardown_hooks);
        hooks.into_iter().for_each(|hook| hook());
        let handlers = core::mem::take(&mut self.inner.lock().svc_handlers);
        drop(handlers);
        let threads = core::mem::take(&mut *self.threads.lock());
        drop(threads);
    }

    /// Run `hook` in [`Self::teardown`]: for a cycle through this core that the SVC handlers do
    /// not own, e.g. a function table the JVM also holds.
    pub fn on_teardown(&self, hook: impl FnOnce() + Send + 'static) {
        self.inner.lock().teardown_hooks.push(Box::new(hook));
    }

    pub(crate) fn debug_inner(&self) -> Option<Arc<DebugInner>> {
        let inner = self.inner.lock();

        inner
            .engine
            .as_any()
            .downcast_ref::<DebuggedArm32CpuEngine>()
            .map(|engine| engine.debug_inner())
    }

    pub fn load(&mut self, data: &[u8], address: u32, map_size: usize) -> Result<()> {
        let mut inner = self.inner.lock();

        inner
            .engine
            .mem_map(address, map_size.next_multiple_of(0x1000), MemoryPermission::ReadWriteExecute);
        inner.engine.mem_write(address, data)?;
        inner.engine.record_image(address, data.len());

        Ok(())
    }

    pub async fn prepare_execution(&mut self) -> Result<()> {
        let preparation = self.inner.lock().engine.begin_preparation()?;
        if let Some(preparation) = preparation {
            let result = preparation.await;
            self.inner.lock().engine.finish_preparation(result);
        }
        Ok(())
    }

    pub fn is_preparing(&self) -> bool {
        self.inner.lock().engine.is_preparing()
    }

    pub fn run_in_thread<F, Fut>(&self, entry: F) -> Result<ArmCoreThreadWrapper>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let state = ThreadState::new(self.clone())?;

        let thread_id = {
            let mut inner = self.inner.lock();

            let thread_id = inner.last_thread_id + 1;
            inner.last_thread_id += 1;

            thread_id
        };
        self.threads.lock().insert(thread_id, state);

        tracing::info!("Create thread: {thread_id}");

        ArmCoreThreadWrapper::new(self.clone(), thread_id, entry)
    }

    pub fn delete_thread_context(&self, thread_id: ThreadId) {
        tracing::info!("Terminate thread: {thread_id}");

        self.holders.lock().retain(|id| *id != thread_id);
        // Release the thread lock before freeing its stack.
        let _thread_state = self.threads.lock().remove(&thread_id);
    }

    pub fn enter_thread_context(&self, thread_id: ThreadId) -> ThreadContextGuard {
        ThreadContextGuard::new(self.clone(), thread_id)
    }

    pub fn read_thread_context(&self, thread_id: ThreadId) -> Result<ArmCoreContext> {
        let context = self.threads.lock().get(&thread_id).unwrap().context;

        Ok(context)
    }

    pub fn write_thread_context(&mut self, thread_id: ThreadId, context: &ArmCoreContext) {
        self.threads.lock().get_mut(&thread_id).unwrap().context = *context;
    }

    /// The emulated thread whose context is currently entered, or `None` when host code runs guest
    /// code outside any `run_in_thread` thread (boot, tests).
    pub fn current_thread_id(&self) -> Option<ThreadId> {
        self.inner.lock().current_thread_id
    }

    /// The lowest address of the current thread's stack, or `None` outside any `run_in_thread` thread.
    pub fn current_stack_base(&self) -> Option<u32> {
        let id = self.current_thread_id()?;
        self.threads.lock().get(&id).map(|thread| thread.stack_base as u32)
    }

    /// Whether a thread other than the current one is suspended in the middle of guest code — sliced
    /// out by the instruction budget, not parked in a host call (sleep, wait, I/O). Host code that
    /// hands the guest an event can wait for this to clear: a handset does not deliver a key into
    /// the middle of another thread's frame (docs/report/0430).
    ///
    /// A thread that woke from its slice and is waiting for a holder is still in the middle
    /// (docs/report/0602), so this reads a mark of its own, not the one that wait reads.
    pub fn others_preempted(&self) -> bool {
        let current = self.current_thread_id();
        self.threads.lock().iter().any(|(id, state)| Some(*id) != current && state.mid_code)
    }

    /// While `on`, the current thread holds the others: whenever it is sliced out mid-code, no
    /// other thread resumes guest code until it runs again, so its guest code runs as one piece up
    /// to its next blocking host call. A key handler that frees what another thread's frame draws
    /// needs this (docs/report/0430). Capped at [`HOLD_ROUNDS`] rounds per wait.
    ///
    /// The hold is the thread's own: taking or dropping it leaves every other thread's as it was,
    /// and two holders keep out of each other's sliced code. It was one slot once, and a spawned
    /// callback took a key handler's hold and cleared it on the way out (docs/report/0601).
    ///
    /// Returns whether the current thread already held them, so a nested holder (a paint run from
    /// inside a key handler) can leave the outer hold in place.
    pub fn hold_others(&self, on: bool) -> bool {
        let Some(current) = self.current_thread_id() else {
            return false;
        };
        let mut holders = self.holders.lock();
        let held = holders.contains(&current);
        holders.retain(|id| *id != current);
        if on {
            holders.push(current);
        }
        held
    }

    async fn wait_for_holder(&self) {
        for _ in 0..HOLD_ROUNDS {
            if self.holders.lock().is_empty() {
                return;
            }
            let current = self.current_thread_id();
            let sliced = |id: &ThreadId| Some(*id) != current && self.threads.lock().get(id).is_some_and(|state| state.preempted);
            if current.is_none() || !self.holders.lock().iter().any(sliced) {
                return;
            }
            YieldFuture::new().await;
        }
    }

    fn mark_sliced(&self, thread_id: Option<ThreadId>, preempted: bool, mid_code: bool) {
        if let Some(id) = thread_id
            && let Some(state) = self.threads.lock().get_mut(&id)
        {
            state.preempted = preempted;
            state.mid_code = mid_code;
        }
    }

    pub fn get_thread_ids(&self) -> Vec<ThreadId> {
        self.threads.lock().keys().cloned().collect()
    }

    /// Every word guest code may be holding a pointer in, for a conservative GC root scan: the
    /// registers — live, each thread's saved context, and each context `run_function` put aside
    /// for a nested call — and each thread's stack from the lowest of those SPs inside it to its
    /// top. Words that only look like pointers are included; that is the point of conservative.
    pub fn guest_root_words(&self) -> Result<Vec<u32>> {
        let mut contexts = vec![self.save_context()];
        contexts.extend(self.saved_contexts.lock().values().copied());

        // threads before inner, as delete_thread_context takes them (freeing a stack locks inner).
        let threads = self.threads.lock();
        contexts.extend(threads.values().map(|thread| thread.context));

        let mut words = contexts.iter().flat_map(ArmCoreContext::words).collect::<Vec<_>>();
        for thread in threads.values() {
            let (base, top) = (thread.stack_base as u32, (thread.stack_base + thread.stack_size) as u32);
            let low = contexts
                .iter()
                .map(|context| context.sp & !3)
                .filter(|sp| (base..=top).contains(sp))
                .min()
                .unwrap_or(base);
            let mut stack = vec![0; (top - low) as usize];
            self.read_bytes(low, &mut stack)?;
            words.extend(stack.as_chunks::<4>().0.iter().map(|word| u32::from_le_bytes(*word)));
        }

        Ok(words)
    }

    fn sample_profile(&self) {
        let mut inner = self.inner.lock();
        if inner.profile.is_none() {
            return;
        }
        let pc = inner.engine.reg_read(ArmRegister::PC);
        let mut stack = vec![pc];
        let mut r7 = inner.engine.reg_read(ArmRegister::R7);
        for _ in 0..PROFILE_MAX_STACK {
            // Thumb frame: [saved R7 | saved LR] at [r7].
            let mut buf = [0u8; 8];
            if inner.engine.mem_read(r7, 8, &mut buf).is_err() {
                break;
            }
            let prev_r7 = u32::from_le_bytes(buf[0..4].try_into().unwrap());
            let lr = u32::from_le_bytes(buf[4..8].try_into().unwrap());
            // Heuristic stop: zero/null frame or non-Thumb LR (we only ever
            // call into Thumb code from the guest).
            if prev_r7 == 0 || lr == 0 || lr & 1 == 0 {
                break;
            }
            stack.push(lr);
            if prev_r7 <= r7 {
                // R7 chain must walk upward; bail on any inversion to avoid loops.
                break;
            }
            r7 = prev_r7;
        }
        let profile = inner.profile.as_mut().unwrap();
        *profile.samples.entry(stack).or_insert(0) += 1;
        profile.counter = profile.counter.wrapping_add(1);
        if profile.counter >= PROFILE_FLUSH_INTERVAL {
            profile.counter = 0;
            let batch = drain_samples(&mut profile.samples);
            (profile.callback)(batch);
        }
    }

    pub async fn run_function<R>(&mut self, address: u32, params: &[u32]) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        // we don't need to save r0-r3, but to make it simple, we save all registers
        let previous_context = self.save_context();
        let key = NEXT_SAVED_CONTEXT.fetch_add(1, Ordering::Relaxed);
        self.saved_contexts.lock().insert(key, previous_context);
        let _saved = SavedContextGuard {
            saved_contexts: self.saved_contexts.clone(),
            key,
        };
        let result = self.run_function_inner(address, params).await;
        self.restore_context(&previous_context);
        result
    }

    /// Runs without yielding to application tasks or dispatching registered async handlers.
    /// The SVC callback owns result/return-PC writes using `ResultWriter`.
    /// CPU context is restored on success and error; guest memory writes are retained.
    pub fn run_function_sync<R>(&mut self, address: u32, params: &[u32], mut svc: impl FnMut(&mut ArmCore, u32) -> Result<()>) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        let _context_guard = ContextGuard {
            core: self.clone(),
            context: self.save_context(),
        };
        self.prepare_function_call(address, params)?;

        loop {
            let result = self.inner.lock().engine.run(RUN_FUNCTION_LR, INSTRUCTIONS_PER_YIELD)?;
            match result.stop_reason {
                EngineStopReason::End => break,
                EngineStopReason::Yield if result.budget_consumed == INSTRUCTIONS_PER_YIELD => continue,
                EngineStopReason::Yield => {
                    return Err(WieError::FatalError(
                        "Debugger requested a thread switch during synchronous ARM execution".into(),
                    ));
                }
                EngineStopReason::Svc { category, lr, spsr } => {
                    {
                        let mut inner = self.inner.lock();
                        inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                        inner.engine.reg_write(ArmRegister::PC, lr);
                    }
                    let svc_step = self
                        .debug_inner()
                        .and_then(|debug| debug.begin_svc_step().map(|thread_id| (debug, thread_id)));
                    let result = svc(self, category);
                    if let Some((debug, thread_id)) = svc_step {
                        debug.end_svc_step(thread_id, result.is_ok());
                    }
                    result?;
                }
            }
        }

        Ok(R::get(self))
    }

    fn prepare_function_call(&mut self, address: u32, params: &[u32]) -> Result<()> {
        let mut inner = self.inner.lock();

        if !params.is_empty() {
            inner.engine.reg_write(ArmRegister::R0, params[0]);
        }
        if params.len() > 1 {
            inner.engine.reg_write(ArmRegister::R1, params[1]);
        }
        if params.len() > 2 {
            inner.engine.reg_write(ArmRegister::R2, params[2]);
        }
        if params.len() > 3 {
            inner.engine.reg_write(ArmRegister::R3, params[3]);
        }
        if params.len() > 4 {
            for param in params[4..].iter().rev() {
                let sp: u32 = inner.engine.reg_read(ArmRegister::SP) - 4;

                inner.engine.mem_write(sp, &param.to_le_bytes())?;
                inner.engine.reg_write(ArmRegister::SP, sp);
            }
        }

        inner.engine.reg_write(ArmRegister::PC, address);
        inner.engine.reg_write(ArmRegister::LR, RUN_FUNCTION_LR);

        let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
        let new_cpsr = (cpsr & !0x3f) | 0x1f | ((address & 1) << 5);
        inner.engine.reg_write(ArmRegister::Cpsr, new_cpsr);

        Ok(())
    }

    async fn run_function_inner<R>(&mut self, address: u32, params: &[u32]) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        self.prepare_function_call(address, params)?;

        // The thread, from a slice that cut it between two instructions until it runs again.
        let mut mid_code = None;
        loop {
            self.wait_for_holder().await;
            self.mark_sliced(mid_code.take(), false, false);
            let (result, should_yield) = {
                let mut inner = self.inner.lock();
                let budget = inner.instructions_remaining;
                let result = inner.engine.run(RUN_FUNCTION_LR, budget)?;
                inner.instructions_remaining -= result.budget_consumed;
                let exhausted = inner.instructions_remaining == 0;
                if exhausted {
                    inner.instructions_remaining = INSTRUCTIONS_PER_YIELD;
                }
                let should_yield = exhausted || matches!(result.stop_reason, EngineStopReason::Yield);
                (result.stop_reason, should_yield)
            };

            self.sample_profile();

            let svc_step = if let EngineStopReason::Svc { lr, spsr, .. } = result {
                // Leave exception mode before yielding: thread contexts do not save banked SVC registers.
                {
                    let mut inner = self.inner.lock();
                    inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                    inner.engine.reg_write(ArmRegister::PC, lr);
                }
                // Nested ARM calls remain part of this handler's single step.
                self.debug_inner()
                    .and_then(|debug| debug.begin_svc_step().map(|thread_id| (debug, thread_id)))
            } else {
                None
            };

            if should_yield {
                // A budget that runs out on an `svc` slices the thread out just as mid-code: the host
                // call has not started yet, and the frame around it is half done. Left unmarked, a
                // paint read the thread as parked and freed what its frame was filling
                // (3ccc6cf147d2, docs/report/0506).
                let sliced = if matches!(result, EngineStopReason::Yield | EngineStopReason::Svc { .. }) {
                    self.current_thread_id()
                } else {
                    None
                };
                self.mark_sliced(sliced, true, true);
                YieldFuture::new().await;
                // Awake. On an `svc` the host call starts now; between two instructions the thread
                // stays mid-code until the wait at the top of the loop lets it run. `preempted`
                // clears here either way: kept through that wait, two holders wait for each other.
                mid_code = sliced.filter(|_| matches!(result, EngineStopReason::Yield));
                self.mark_sliced(sliced, false, mid_code.is_some());
            }

            match result {
                EngineStopReason::End => break,
                EngineStopReason::Yield => continue,
                EngineStopReason::Svc { category, .. } => {
                    let function = {
                        let inner = self.inner.lock();
                        inner
                            .svc_handlers
                            .get(&category)
                            .cloned()
                            .ok_or_else(|| WieError::FatalError(format!("Unknown SVC handler category: {category}")))?
                    };

                    let mut self1 = self.clone();
                    let result = function.call(&mut self1).await;
                    if let Some((debug, thread_id)) = svc_step {
                        debug.end_svc_step(thread_id, result.is_ok());
                    }
                    result?;
                }
            }
        }

        Ok(R::get(self))
    }

    pub fn register_svc_handler<F, C, R, P>(&mut self, category: u32, handler: F, context: &C) -> Result<()>
    where
        F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
        C: Clone + 'static + Sync + Send,
        R: ResultWriter<R> + Sync + Send + 'static,
        P: Sync + Send + 'static,
    {
        let mut inner = self.inner.lock();

        if inner.svc_handlers.contains_key(&category) {
            return Err(WieError::FatalError(format!("SVC handler already registered for {category}")));
        }

        inner
            .svc_handlers
            .insert(category, Arc::new(Box::new(RegisteredFunctionHolder::new(handler, context))));

        Ok(())
    }

    /// Like [`Self::make_svc_stub`], but hands back the same stub for the same `(category, id)`.
    ///
    /// A stub is a pure function of those two values, so callers that only need "something
    /// that raises SVC `category` with `id`" can share one. The stub space holds 4,096 stubs in
    /// total, and a caller that fills tables with placeholder stubs would otherwise spend one per
    /// slot per table per rewrite.
    pub fn make_shared_svc_stub(&mut self, category: u32, id: impl Into<u32>) -> Result<u32> {
        let id = id.into();
        if let Some(address) = self.inner.lock().shared_stubs.get(&(category, id)) {
            return Ok(*address);
        }

        let address = self.make_svc_stub(category, id)?;
        self.inner.lock().shared_stubs.insert((category, id), address);

        Ok(address)
    }

    pub fn make_svc_stub(&mut self, category: u32, id: impl Into<u32>) -> Result<u32> {
        let mut inner = self.inner.lock();
        let id = id.into();

        if !inner.svc_handlers.contains_key(&category) {
            return Err(WieError::FatalError(format!("Unknown SVC handler category: {category}")));
        }

        let address = inner.next_stub_address;
        if address + SVC_STUB_SIZE > FUNCTIONS_BASE + FUNCTIONS_SIZE as u32 {
            // Read by `wie_validate`, which writes its result line here: on some runtimes this
            // error ends in a stack-overflow abort that no later code survives to report.
            tracing::info!("SVC stub space exhausted");
            return Err(WieError::FatalError("SVC stub space exhausted".into()));
        }
        inner.next_stub_address += SVC_STUB_SIZE;
        SVC_STUBS_HIGH_WATER.fetch_max((inner.next_stub_address - FUNCTIONS_BASE) / SVC_STUB_SIZE, Ordering::Relaxed);

        let stub = [
            0x10,
            0xb4, // push {r4}
            0x02,
            0x4c, // ldr r4, [pc, #8]
            0xa4,
            0x46, // mov r12, r4
            0x10,
            0xbc, // pop {r4}
            category as u8,
            0xdf, // svc #category
            0x70,
            0x47, // bx lr
        ]
        .into_iter()
        .chain(id.to_le_bytes())
        .collect::<Vec<_>>();
        inner.engine.mem_write(address, &stub)?;

        tracing::trace!("Register SVC stub at {address:#x}, category={category}, id={id}");

        Ok(address + 1)
    }

    /// `write_bytes` for guest CODE rewritten by the host (binary patches, hooks): also tells the
    /// AOT backend, which no plain store reaches (`ArmEngine::code_written`).
    pub(crate) fn write_code(&mut self, address: u32, data: &[u8]) -> Result<()> {
        let mut inner = self.inner.lock();
        inner.engine.mem_write(address, data)?;
        inner.engine.code_written(address, data.len());
        Ok(())
    }

    pub fn map(&mut self, address: u32, size: u32) -> Result<()> {
        tracing::trace!("Map address: {address:#x}, size: {size:#x}");

        let mut inner = self.inner.lock();

        inner.engine.mem_map(address, size as usize, MemoryPermission::ReadWrite);

        Ok(())
    }

    pub fn dump_reg_stack(&self, image_base: u32) -> String {
        format!(
            "\n{}\nPossible call stack:\n{}\nStack:\n{}",
            self.dump_regs(),
            self.dump_call_stack(image_base).unwrap(),
            self.dump_stack().unwrap()
        )
    }

    pub fn restore_context(&mut self, context: &ArmCoreContext) {
        let mut inner = self.inner.lock();

        inner.engine.reg_write(ArmRegister::R0, context.r0);
        inner.engine.reg_write(ArmRegister::R1, context.r1);
        inner.engine.reg_write(ArmRegister::R2, context.r2);
        inner.engine.reg_write(ArmRegister::R3, context.r3);
        inner.engine.reg_write(ArmRegister::R4, context.r4);
        inner.engine.reg_write(ArmRegister::R5, context.r5);
        inner.engine.reg_write(ArmRegister::R6, context.r6);
        inner.engine.reg_write(ArmRegister::R7, context.r7);
        inner.engine.reg_write(ArmRegister::R8, context.r8);
        inner.engine.reg_write(ArmRegister::SB, context.sb);
        inner.engine.reg_write(ArmRegister::SL, context.sl);
        inner.engine.reg_write(ArmRegister::FP, context.fp);
        inner.engine.reg_write(ArmRegister::IP, context.ip);
        inner.engine.reg_write(ArmRegister::SP, context.sp);
        inner.engine.reg_write(ArmRegister::LR, context.lr);
        inner.engine.reg_write(ArmRegister::PC, context.pc);
        inner.engine.reg_write(ArmRegister::Cpsr, context.cpsr);
    }

    pub fn save_context(&self) -> ArmCoreContext {
        let inner = self.inner.lock();

        ArmCoreContext {
            r0: inner.engine.reg_read(ArmRegister::R0),
            r1: inner.engine.reg_read(ArmRegister::R1),
            r2: inner.engine.reg_read(ArmRegister::R2),
            r3: inner.engine.reg_read(ArmRegister::R3),
            r4: inner.engine.reg_read(ArmRegister::R4),
            r5: inner.engine.reg_read(ArmRegister::R5),
            r6: inner.engine.reg_read(ArmRegister::R6),
            r7: inner.engine.reg_read(ArmRegister::R7),
            r8: inner.engine.reg_read(ArmRegister::R8),
            sb: inner.engine.reg_read(ArmRegister::SB),
            sl: inner.engine.reg_read(ArmRegister::SL),
            fp: inner.engine.reg_read(ArmRegister::FP),
            ip: inner.engine.reg_read(ArmRegister::IP),
            sp: inner.engine.reg_read(ArmRegister::SP),
            lr: inner.engine.reg_read(ArmRegister::LR),
            pc: inner.engine.reg_read(ArmRegister::PC),
            cpsr: inner.engine.reg_read(ArmRegister::Cpsr),
        }
    }

    pub fn read_pc_lr(&self) -> Result<(u32, u32)> {
        let inner = self.inner.lock();

        let lr = inner.engine.reg_read(ArmRegister::LR);
        let pc = inner.engine.reg_read(ArmRegister::PC);

        Ok((pc, lr))
    }

    pub fn write_return_value(&mut self, result: &[u32]) -> Result<()> {
        let mut inner = self.inner.lock();

        if !result.is_empty() {
            inner.engine.reg_write(ArmRegister::R0, result[0]);
        }
        if result.len() > 1 {
            inner.engine.reg_write(ArmRegister::R1, result[1]);
        }
        if result.len() > 2 {
            todo!() // TODO
        }

        Ok(())
    }

    pub fn set_next_pc(&mut self, pc: u32) -> Result<()> {
        let mut inner = self.inner.lock();

        inner.engine.reg_write(ArmRegister::PC, pc);

        let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
        let new_cpsr = if pc & 1 == 1 { cpsr | 0x20 } else { cpsr & !0x20 };
        inner.engine.reg_write(ArmRegister::Cpsr, new_cpsr);

        Ok(())
    }

    pub fn read_param(&self, pos: usize) -> Result<u32> {
        let inner = self.inner.lock();

        let result = if pos == 0 {
            inner.engine.reg_read(ArmRegister::R0)
        } else if pos == 1 {
            inner.engine.reg_read(ArmRegister::R1)
        } else if pos == 2 {
            inner.engine.reg_read(ArmRegister::R2)
        } else if pos == 3 {
            inner.engine.reg_read(ArmRegister::R3)
        } else {
            let sp = inner.engine.reg_read(ArmRegister::SP);

            drop(inner);

            read_generic(self, sp + 4 * (pos as u32 - 4))?
        };

        Ok(result)
    }

    pub(crate) fn dump_regs_inner(engine: &dyn ArmEngine) -> String {
        [
            format!(
                "R0: {:#x} R1: {:#x} R2: {:#x} R3: {:#x} R4: {:#x} R5: {:#x} R6: {:#x} R7: {:#x} R8: {:#x}",
                engine.reg_read(ArmRegister::R0),
                engine.reg_read(ArmRegister::R1),
                engine.reg_read(ArmRegister::R2),
                engine.reg_read(ArmRegister::R3),
                engine.reg_read(ArmRegister::R4),
                engine.reg_read(ArmRegister::R5),
                engine.reg_read(ArmRegister::R6),
                engine.reg_read(ArmRegister::R7),
                engine.reg_read(ArmRegister::R8),
            ),
            format!(
                "SB: {:#x} SL: {:#x} FP: {:#x} IP: {:#x} SP: {:#x} LR: {:#x} PC: {:#x}",
                engine.reg_read(ArmRegister::SB),
                engine.reg_read(ArmRegister::SL),
                engine.reg_read(ArmRegister::FP),
                engine.reg_read(ArmRegister::IP),
                engine.reg_read(ArmRegister::SP),
                engine.reg_read(ArmRegister::LR),
                engine.reg_read(ArmRegister::PC),
            ),
            format!("CPSR: {:032b}\n", engine.reg_read(ArmRegister::Cpsr)),
        ]
        .join("\n")
    }

    fn is_code_address(address: u32, image_base: u32) -> bool {
        // TODO image size temp

        (address % 2 == 1 && (image_base..image_base + 0x100000).contains(&address))
            || (FUNCTIONS_BASE..FUNCTIONS_BASE + FUNCTIONS_SIZE as u32).contains(&address)
    }

    fn dump_regs(&self) -> String {
        let inner = self.inner.lock();

        Self::dump_regs_inner(&*inner.engine)
    }

    fn format_callstack_address(address: u32, image_base: u32) -> String {
        let description = if (image_base..image_base + 0x100000).contains(&address) {
            format!("<Base>+{:#x}", address - image_base)
        } else if (FUNCTIONS_BASE..FUNCTIONS_BASE + FUNCTIONS_SIZE as u32).contains(&address) {
            "<Native function>".to_owned()
        } else {
            "<Unknown>".to_owned()
        };

        format!("{address:#x}: {description}\n")
    }

    fn dump_call_stack(&self, image_base: u32) -> Result<String> {
        let mut inner = self.inner.lock();

        let sp = inner.engine.reg_read(ArmRegister::SP);
        let pc = inner.engine.reg_read(ArmRegister::PC);
        let lr = inner.engine.reg_read(ArmRegister::LR);

        let mut call_stack = Self::format_callstack_address(pc, image_base);
        if lr != RUN_FUNCTION_LR && lr != 0 {
            call_stack += &Self::format_callstack_address(lr - 5, image_base);
        }

        for i in 0..128 {
            let address = sp + (i * 4);
            if !inner.engine.is_mapped(address, size_of::<u32>()) {
                break;
            }

            let mut value = [0; size_of::<u32>()];
            inner.engine.mem_read(address, size_of::<u32>(), &mut value)?;
            let value_u32 = u32::from_le_bytes(value);

            if value_u32 > 5 && Self::is_code_address(value_u32 - 4, image_base) {
                call_stack += &Self::format_callstack_address(value_u32 - 5, image_base);
            }
        }

        Ok(call_stack)
    }

    fn dump_stack(&self) -> Result<String> {
        let mut inner = self.inner.lock();

        let sp = inner.engine.reg_read(ArmRegister::SP);

        let mut result = String::new();
        for i in 0..16 {
            let address = sp + (i * 4);

            if !inner.engine.is_mapped(address, size_of::<u32>()) {
                break;
            }

            let mut value = [0; size_of::<u32>()];
            inner.engine.mem_read(address, size_of::<u32>(), &mut value)?;
            let value_u32 = u32::from_le_bytes(value);

            result += &format!("SP+{:#x}: {value_u32:#x}\n", i * 4);
        }

        Ok(result)
    }
}

impl ByteRead for ArmCore {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> wie_util::Result<usize> {
        let mut inner = self.inner.lock();

        let read = inner.engine.mem_read(address, result.len(), result)?;

        Ok(read)
    }
}

impl ByteWrite for ArmCore {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        let mut inner = self.inner.lock();

        inner.engine.mem_write(address, data)?;

        Ok(())
    }
}

pub trait RunFunctionResult<R> {
    fn get(core: &ArmCore) -> R;
}

impl RunFunctionResult<u32> for u32 {
    fn get(core: &ArmCore) -> u32 {
        core.read_param(0).unwrap()
    }
}

// r0:r1 — a 64-bit (`J`/`D`) return, low word first.
impl RunFunctionResult<(u32, u32)> for (u32, u32) {
    fn get(core: &ArmCore) -> (u32, u32) {
        (core.read_param(0).unwrap(), core.read_param(1).unwrap())
    }
}

impl RunFunctionResult<()> for () {
    fn get(_: &ArmCore) {}
}

struct ContextGuard {
    core: ArmCore,
    context: ArmCoreContext,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        self.core.restore_context(&self.context);
    }
}

pub struct ThreadContextGuard {
    core: ArmCore,
    thread_id: ThreadId,
    previous_thread_id: Option<ThreadId>,
}

impl ThreadContextGuard {
    pub fn new(mut core: ArmCore, thread_id: ThreadId) -> Self {
        let context = core.threads.lock().get(&thread_id).unwrap().context;
        core.restore_context(&context);
        let previous_thread_id = core.inner.lock().current_thread_id.replace(thread_id);

        if let Some(debug) = core.debug_inner() {
            debug.on_thread_entered(thread_id);
        }

        Self {
            core,
            thread_id,
            previous_thread_id,
        }
    }
}

impl Drop for ThreadContextGuard {
    fn drop(&mut self) {
        let context = self.core.save_context();

        self.core.threads.lock().get_mut(&self.thread_id).unwrap().context = context;
        self.core.inner.lock().current_thread_id = self.previous_thread_id;

        if let Some(debug) = self.core.debug_inner() {
            debug.on_thread_exited(self.thread_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{
        future::poll_fn,
        pin::pin,
        sync::atomic::{AtomicU32, Ordering},
        task::{Context, Poll, Waker},
    };

    use bytemuck::cast_slice;

    use crate::{Allocator, EmulatedFunctionParam, SvcId, function::JumpTo};
    use wie_util::{read_null_terminated_string_bytes, write_generic};

    use super::*;

    #[test]
    fn current_thread_id_follows_the_entered_context() -> Result<()> {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        crate::Allocator::init(&mut core)?;
        let _thread1 = core.run_in_thread(|| async { Ok(()) })?;
        let _thread2 = core.run_in_thread(|| async { Ok(()) })?;
        assert_eq!(core.current_thread_id(), None);
        {
            let _outer = core.enter_thread_context(1);
            assert_eq!(core.current_thread_id(), Some(1));
            {
                let _inner = core.enter_thread_context(2);
                assert_eq!(core.current_thread_id(), Some(2));
            }
            assert_eq!(core.current_thread_id(), Some(1));
        }
        assert_eq!(core.current_thread_id(), None);

        Ok(())
    }

    fn context_registers(context: &ArmCoreContext) -> [u32; 17] {
        [
            context.r0,
            context.r1,
            context.r2,
            context.r3,
            context.r4,
            context.r5,
            context.r6,
            context.r7,
            context.r8,
            context.sb,
            context.sl,
            context.fp,
            context.ip,
            context.sp,
            context.lr,
            context.pc,
            context.cpsr,
        ]
    }

    #[test]
    fn synchronous_calls_restore_context_without_consuming_the_outer_budget() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: Some(Box::new(|_| panic!("synchronous calls must not sample the profiler"))),
        })
        .unwrap();
        let mut code = vec![0xe2800001u32; 10_001]; // add r0, r0, #1
        code.push(0xe12fff1e); // bx lr
        core.load(cast_slice(&code), 0x1000, code.len() * 4).unwrap();
        core.map(0x10000, 0x10000).unwrap();
        let context = ArmCoreContext {
            r0: 10,
            r1: 11,
            r2: 12,
            r3: 13,
            r4: 14,
            r5: 15,
            r6: 16,
            r7: 17,
            r8: 18,
            sb: 19,
            sl: 20,
            fp: 21,
            ip: 22,
            sp: 0x20000,
            lr: 0x3001,
            pc: 0x4000,
            cpsr: 0xa000003f,
        };
        core.restore_context(&context);
        core.inner.lock().instructions_remaining = 5;
        let value = core.run_function_sync::<u32>(0x1000, &[0], |_, _| panic!("unexpected SVC")).unwrap();
        assert_eq!(value, 10_001);
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));

        let async_calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count_inline_svc, &async_calls).unwrap();
        let target = core.make_svc_stub(1, 42u32).unwrap();
        let mut arguments = Vec::new();
        let value = core
            .run_function_sync::<u32>(target, &[1, 2, 3, 4, 5, 6], |core, category| {
                assert_eq!(category, 1);
                let id = SvcId::get(core, 0);
                assert_eq!(id.0, 42);
                assert_eq!(core.save_context().cpsr & 0x3f, 0x3f);
                for index in 0..6 {
                    arguments.push(core.read_param(index)?);
                }
                core.write_bytes(0x10000, &123u32.to_le_bytes())?;
                let (_, lr) = core.read_pc_lr()?;
                99u32.write(core, lr)
            })
            .unwrap();
        assert_eq!(value, 99);
        assert_eq!(arguments, [1, 2, 3, 4, 5, 6]);
        assert_eq!(read_generic::<u32, _>(&core, 0x10000).unwrap(), 123);
        assert_eq!(async_calls.load(Ordering::Relaxed), 0);
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));
        let inner = core.inner.lock();
        assert_eq!(inner.instructions_remaining, 5);
        let profile = inner.profile.as_ref().unwrap();
        assert_eq!(profile.counter, 0);
        assert!(profile.samples.is_empty());
    }

    #[test]
    fn synchronous_call_errors_restore_context_including_partial_argument_setup() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        core.map(0x10000, 0x10000).unwrap();
        let async_calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count_inline_svc, &async_calls).unwrap();
        let target = core.make_svc_stub(1, 0u32).unwrap();
        let mut context = core.save_context();
        context.r0 = 42;
        context.r1 = 43;
        context.sp = 0x20000;
        context.lr = 0x4001;
        context.cpsr = 0x6000001f;
        core.restore_context(&context);
        core.inner.lock().instructions_remaining = 5;

        let result = core.run_function_sync::<()>(target, &[1, 2, 3, 4, 5, 6], |core, category| {
            core.write_return_value(&[99, 98])?;
            Err(WieError::FatalError(format!("Unexpected synchronous SVC: {category}")))
        });
        assert!(matches!(result, Err(WieError::FatalError(message)) if message == "Unexpected synchronous SVC: 1"));
        assert_eq!(async_calls.load(Ordering::Relaxed), 0);
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));

        let result = core.run_function_sync::<()>(0x20001, &[1, 2, 3, 4, 5, 6], |_, _| panic!("unexpected SVC"));
        assert!(matches!(result, Err(WieError::InvalidMemoryAccess(0x20000))));
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));

        context.sp = 0x10004;
        core.restore_context(&context);
        let result = core.run_function_sync::<()>(target, &[1, 2, 3, 4, 5, 6], |_, _| panic!("argument setup must fail"));
        assert!(matches!(result, Err(WieError::InvalidMemoryAccess(0xfffc))));
        assert_eq!(read_generic::<u32, _>(&core, 0x10000).unwrap(), 6);
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));
        assert_eq!(core.inner.lock().instructions_remaining, 5);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn synchronous_call_rejects_debugger_thread_switching() {
        extern crate std;

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        core.inner.lock().instructions_remaining = 5;
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        let context = core.save_context();
        debug.on_thread_entered(1);
        debug.resume(Vec::new(), Some(vec![2]));
        let (tx, rx) = crossbeam::channel::bounded(1);
        let runner = std::thread::spawn(move || {
            let result = core.run_function_sync::<()>(0x1001, &[1], |_, _| panic!("unexpected SVC"));
            tx.send((result, core)).unwrap();
        });
        let finished = rx.recv_timeout(std::time::Duration::from_secs(1));
        debug.resume(Vec::new(), None);
        runner.join().unwrap();
        let (result, core) = finished.unwrap();
        assert!(matches!(result, Err(WieError::FatalError(message)) if message.contains("thread switch")));
        assert_eq!(context_registers(&core.save_context()), context_registers(&context));
        assert_eq!(core.inner.lock().instructions_remaining, 5);
    }

    #[test]
    fn interpreter_executes_without_preparation() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let _context = core.enter_thread_context(1);
        core.load(&[42, 0x20, 0x70, 0x47], 0x1000, 0x1000).unwrap();
        assert!(!core.is_preparing());
        assert!(core.inner.lock().engine.begin_preparation().unwrap().is_none());
        futures::executor::block_on(core.prepare_execution()).unwrap();
        assert_eq!(futures::executor::block_on(core.run_function::<u32>(0x1001, &[])).unwrap(), 42);
    }

    #[test]
    fn terminated_string_reads_use_normal_and_debug_engines() {
        for debug in [false, true] {
            let mut core = ArmCore::new(Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })
            .unwrap();
            if debug {
                core.inner.lock().engine = Box::new(DebuggedArm32CpuEngine::new());
            }
            core.load(&[], 0x10000, 0x10000).unwrap();
            assert!(core.inner.lock().engine.begin_preparation().unwrap().is_none());
            core.write_bytes(0x1fffc, b"end\0").unwrap();
            let reader: &dyn ByteRead = &core;
            assert_eq!(read_null_terminated_string_bytes(reader, 0x1fffc).unwrap(), b"end");
            assert!(read_null_terminated_string_bytes(reader, 0x1ffff).unwrap().is_empty());
            assert!(matches!(
                read_null_terminated_string_bytes(reader, 0),
                Err(WieError::InvalidMemoryAccess(0))
            ));
            core.write_bytes(0x1ffff, b"x").unwrap();
            assert!(matches!(
                read_null_terminated_string_bytes(&core, 0x1fffc),
                Err(WieError::InvalidMemoryAccess(0x20000))
            ));
            core.load(b"page\0", 0x20000, 5).unwrap();
            assert_eq!(read_null_terminated_string_bytes(&core, 0x1fffc).unwrap(), b"endxpage");
            core.load(b"raw\0", 0, 4).unwrap();
            assert!(matches!(
                read_null_terminated_string_bytes(&core, 0),
                Err(WieError::InvalidMemoryAccess(0))
            ));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn saved_thread_contexts_are_accessible_while_the_engine_is_locked() {
        extern crate std;

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let mut context = core.read_thread_context(1).unwrap();
        context.r0 = 42;

        let mut observer = core.clone();
        let (tx, rx) = crossbeam::channel::bounded(1);
        let engine_guard = core.inner.lock();
        let reader = std::thread::spawn(move || {
            observer.write_thread_context(1, &context);
            tx.send((observer.read_thread_context(1).unwrap().r0, observer.get_thread_ids())).unwrap();
        });
        let result = rx.recv_timeout(std::time::Duration::from_secs(1));
        drop(engine_guard);
        reader.join().unwrap();
        assert_eq!(result.unwrap(), (42, vec![1]));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn scheduler_locking_yields_without_resetting_the_instruction_budget() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let _context = core.enter_thread_context(1);
        core.inner.lock().instructions_remaining = 5;
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        debug.on_thread_entered(1);
        debug.resume(Vec::new(), Some(vec![2]));

        let observer = core.clone();
        let mut run = pin!(core.run_function::<()>(0x1001, &[]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.save_context().pc, 0x1000);
        assert_eq!(observer.inner.lock().instructions_remaining, 5);
        debug.resume(Vec::new(), None);
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert_eq!(observer.inner.lock().instructions_remaining, 4);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn scheduler_locking_keeps_other_thread_futures_stopped() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let mut task = pin!(
            core.run_in_thread(move || async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .unwrap()
        );
        let _other = core.run_in_thread(|| async { Ok(()) }).unwrap();
        debug.resume(Vec::new(), Some(vec![2]));
        let (waker, wake_count) = futures_test::task::new_count_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(wake_count, 0);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        debug.resume(Vec::new(), None);
        assert!(matches!(task.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert_eq!(observed.load(Ordering::Relaxed), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn interrupt_stops_a_thread_waiting_in_host_code() {
        extern crate std;

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let mut task = Box::pin(
            core.run_in_thread(move || async move {
                loop {
                    calls.fetch_add(1, Ordering::Relaxed);
                    YieldFuture::new().await;
                }
            })
            .unwrap(),
        );
        let _other = core.run_in_thread(|| async { Ok(()) }).unwrap();
        debug.resume(Vec::new(), None);
        assert!(task.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_pending());

        debug.interrupt();
        let runner = std::thread::spawn(move || {
            assert!(task.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_pending());
            task
        });
        let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
        let mut context = debug.read_registers();
        context.r0 = 123;
        debug.write_registers(&context);
        debug.resume(Vec::new(), Some(vec![2]));
        let _task = runner.join().unwrap();
        assert!(matches!(
            stopped,
            Ok(crate::engine::DebugStopReason::Signal(crate::engine::DebugSignal::Int, 1))
        ));
        assert_eq!(observed.load(Ordering::Relaxed), 1);
        assert_eq!(core.read_thread_context(1).unwrap().r0, 123);
    }

    async fn test_svc_handler(_core: &mut ArmCore, seen_id: &mut Option<u32>, id: crate::SvcId) -> Result<()> {
        *seen_id = Some(id.0);

        Ok(())
    }

    async fn count_inline_svc(core: &mut ArmCore, calls: &mut Arc<AtomicU32>) -> Result<JumpTo> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(JumpTo(core.read_pc_lr()?.0 | 1))
    }

    async fn call_nested_arm(core: &mut ArmCore, result: &mut Arc<AtomicU32>) -> Result<JumpTo> {
        let pc = core.read_pc_lr()?.0;
        let value = core.run_function(0x10001, &[0]).await?;
        result.store(value, Ordering::Relaxed);
        Ok(JumpTo(pc | 1))
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stepping_svc_waits_for_nested_handlers_and_preserves_breakpoints() {
        extern crate std;

        use crate::engine::{DebugBreakpointKind, DebugStopReason};

        async fn call_nested_sync(core: &mut ArmCore, context: &mut (Arc<AtomicU32>, Arc<AtomicU32>)) -> Result<JumpTo> {
            let pc = core.read_pc_lr()?.0;
            let value = core.run_function_sync::<u32>(0x10001, &[0], |core, category| {
                assert_eq!(category, 2);
                context.1.fetch_add(1, Ordering::Relaxed);
                JumpTo(core.read_pc_lr()?.0 | 1).write(core, 0)
            })?;
            context.0.store(value, Ordering::Relaxed);
            Ok(JumpTo(pc | 1))
        }

        for (synchronous, breakpoint) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut core = ArmCore::new(Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })
            .unwrap();
            let engine = DebuggedArm32CpuEngine::new();
            let debug = engine.debug_inner();
            core.inner.lock().engine = Box::new(engine);
            crate::Allocator::init(&mut core).unwrap();
            let result = Arc::new(AtomicU32::new(0));
            let calls = Arc::new(AtomicU32::new(0));
            if synchronous {
                core.register_svc_handler(1, call_nested_sync, &(result.clone(), calls.clone())).unwrap();
            } else {
                core.register_svc_handler(1, call_nested_arm, &result).unwrap();
            }
            core.register_svc_handler(2, count_inline_svc, &calls).unwrap();
            core.load(&[0x01, 0xdf, 0x01, 0x30, 0x70, 0x47], 0x1000, 6).unwrap(); // svc #1; add r0, #1; bx lr
            core.load(&[0x02, 0xdf, 0x01, 0x30, 0x70, 0x47], 0x10000, 6).unwrap(); // svc #2; add r0, #1; bx lr
            if breakpoint {
                debug.add_breakpoint(0x10000, DebugBreakpointKind::Thumb16).unwrap();
            }
            let mut running = core.clone();
            let task = core
                .run_in_thread(move || async move { running.run_function::<()>(0x1001, &[0]).await })
                .unwrap();
            debug.resume(vec![1], None);
            let (tx, rx) = crossbeam::channel::bounded(1);
            let runner = std::thread::spawn(move || tx.send(futures::executor::block_on(task)).unwrap());
            let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
            let context = debug.read_registers();
            let completed = (result.load(Ordering::Relaxed), calls.load(Ordering::Relaxed));
            let nested_stop = if breakpoint {
                debug.resume(vec![1], None);
                let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
                Some((stopped, debug.read_registers(), calls.load(Ordering::Relaxed)))
            } else {
                None
            };
            debug.resume(Vec::new(), None);
            let finished = rx.recv_timeout(std::time::Duration::from_secs(1));
            debug.resume(Vec::new(), None);
            runner.join().unwrap();
            finished.unwrap().unwrap();

            if let Some((nested_stop, nested_context, nested_calls)) = nested_stop {
                assert!(matches!(stopped, Ok(DebugStopReason::SwBreak(1))));
                assert_eq!(context.pc, 0x10000);
                assert_eq!(completed, (0, 0));
                assert!(matches!(nested_stop, Ok(DebugStopReason::DoneStep(1))));
                assert_eq!(nested_context.pc, 0x10002);
                assert_eq!(nested_calls, 1);
            } else {
                assert!(matches!(stopped, Ok(DebugStopReason::DoneStep(1))));
                assert_eq!(context.pc, 0x1002);
                assert_eq!(context.r0, 0);
                assert_eq!(completed, (1, 1));
            }
            assert_eq!(result.load(Ordering::Relaxed), 1);
        }
    }

    async fn scan_roots(core: &mut ArmCore, words: &mut Arc<Mutex<Vec<u32>>>) -> Result<JumpTo> {
        *words.lock() = core.guest_root_words()?;
        Ok(JumpTo(core.read_pc_lr()?.0 | 1))
    }

    // The outer call's r0 is overwritten by the nested call's argument, so only the context
    // run_function put aside still holds it — as a host method's arguments are held while it
    // runs guest code that collects.
    #[test]
    fn guest_root_words_see_registers_a_nested_call_put_aside() {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        let words = Arc::new(Mutex::new(Vec::new()));
        core.register_svc_handler(1, call_nested_arm, &Arc::new(AtomicU32::new(0))).unwrap();
        core.register_svc_handler(2, scan_roots, &words).unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x1000, 4).unwrap(); // svc #1; bx lr
        core.load(&[0x02, 0xdf, 0x70, 0x47], 0x10000, 4).unwrap(); // svc #2; bx lr

        let observer = core.clone();
        let mut run = pin!(core.run_function::<u32>(0x1001, &[0x4abc_0000]));
        assert!(matches!(run.as_mut().poll(&mut Context::from_waker(Waker::noop())), Poll::Ready(Ok(_))));
        assert!(words.lock().contains(&0x4abc_0000));
        assert!(observer.saved_contexts.lock().is_empty(), "a finished call leaves nothing put aside");
    }

    #[test]
    fn guest_root_words_see_a_thread_stack_from_its_sp_up() -> Result<()> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        crate::Allocator::init(&mut core)?;
        let _thread = core.run_in_thread(|| async { Ok(()) })?;
        let mut context = core.read_thread_context(1)?;
        context.sp -= 8;
        core.write_thread_context(1, &context);
        core.write_bytes(context.sp + 4, &0x4123_4560u32.to_le_bytes())?;
        core.write_bytes(context.sp - 4, &0x4333_3330u32.to_le_bytes())?;

        let words = core.guest_root_words()?;
        assert!(words.contains(&0x4123_4560));
        assert!(!words.contains(&0x4333_3330), "below SP is dead");

        Ok(())
    }

    // The eleven budget and hold tests from here to `under_aot` run twice: on the interpreter as written, and
    // again in that module with the native AOT backend attached. This is the engine the current run builds.
    extern crate std;
    std::thread_local!(static UNDER_AOT: core::cell::Cell<bool> = const { core::cell::Cell::new(false) });

    fn engine_under_test() -> Option<Box<dyn wie_core_arm_aot::CompiledExecutor>> {
        UNDER_AOT.get().then(|| Box::new(test_utils::TestNativeExecutor::new()) as _)
    }

    /// Call once the guest code is loaded. On the interpreter there is nothing to prepare.
    fn compile(core: &mut ArmCore) {
        futures::executor::block_on(core.prepare_execution()).unwrap();
    }

    #[test]
    fn yields_every_ten_thousand_instructions_across_svc() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let _context = core.enter_thread_context(1);
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count_inline_svc, &calls).unwrap();
        let mut code = [0x01, 0x30, 0x01, 0xdf].repeat(10_000); // add r0, #1; svc #1
        code.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&code, 0x1000, code.len()).unwrap();
        compile(&mut core);

        let observer = core.clone();
        let mut run = pin!(core.run_function::<u32>(0x1001, &[0]));
        let mut cx = Context::from_waker(Waker::noop());
        for completed in [5_000, 10_000] {
            assert!(run.as_mut().poll(&mut cx).is_pending());
            assert_eq!(observer.read_param(0).unwrap(), completed);
            assert_eq!(calls.load(Ordering::Relaxed), completed - 1);
            assert_eq!(observer.save_context().cpsr & 0x3f, 0x3f);
        }
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(10_000))));
        assert_eq!(calls.load(Ordering::Relaxed), 10_000);
    }

    /// Thread 1 holds the others and is sliced out mid-code; thread 2 does `first` and then calls
    /// guest code. Returns the guest calls thread 2 got in while thread 1 was sliced out.
    fn guest_calls_into_a_sliced_holder(first: fn(&ArmCore)) -> u32 {
        async fn count(_: &mut ArmCore, calls: &mut Arc<AtomicU32>) -> Result<()> {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count, &calls).unwrap();
        let mut spin = [0xc0, 0x46].repeat(15_000); // nop
        spin.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&spin, 0x1000, spin.len()).unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x20000, 4).unwrap(); // svc #1; bx lr
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let mut runner = core.clone();
        let mut holder = pin!(
            core.run_in_thread(move || async move {
                runner.hold_others(true);
                runner.run_function::<()>(0x1001, &[]).await?;
                assert!(runner.hold_others(false), "the hold is still this thread's when it lets go");
                Ok(())
            })
            .unwrap()
        );
        let mut runner = core.clone();
        let mut other = pin!(
            core.run_in_thread(move || async move {
                first(&runner);
                runner.run_function::<()>(0x20001, &[]).await
            })
            .unwrap()
        );

        assert!(holder.as_mut().poll(&mut cx).is_pending());
        assert!(other.as_mut().poll(&mut cx).is_pending());
        let calls_while_sliced = calls.load(Ordering::Relaxed);
        assert!(matches!(holder.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        while other.as_mut().poll(&mut cx).is_pending() {}
        assert_eq!(calls.load(Ordering::Relaxed), 1, "and runs once the holder lets go");

        calls_while_sliced
    }

    #[test]
    fn a_holder_sliced_out_keeps_the_others_off_guest_code() {
        assert_eq!(guest_calls_into_a_sliced_holder(|_| {}), 0, "the holder is sliced out mid-code");
    }

    // 155972cac664: a spawned callback that had waited took the hold of a handler parked in a host
    // call, and the rest of that handler ran unheld (docs/report/0601).
    #[test]
    fn another_thread_taking_a_hold_leaves_the_holders_in_place() {
        let calls = guest_calls_into_a_sliced_holder(|core| assert!(!core.hold_others(true), "not this thread's yet"));
        assert_eq!(calls, 0, "a second holder stays out of the first one's sliced code too");
    }

    // MC_netConnect's sleeping thread cleared a key handler's hold on its way out (docs/report/0601).
    #[test]
    fn another_thread_letting_go_leaves_the_holders_in_place() {
        let calls = guest_calls_into_a_sliced_holder(|core| assert!(!core.hold_others(false), "it held nothing"));
        assert_eq!(calls, 0);
    }

    #[test]
    fn a_holder_parked_in_a_host_call_keeps_nobody_off() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let runner = core.clone();
        let mut holder = pin!(
            core.run_in_thread(move || async move {
                runner.hold_others(true);
                YieldFuture::new().await;
                Ok(())
            })
            .unwrap()
        );
        let mut runner = core.clone();
        let mut other = pin!(
            core.run_in_thread(move || async move { runner.run_function::<()>(0x1001, &[]).await })
                .unwrap()
        );

        assert!(holder.as_mut().poll(&mut cx).is_pending());
        assert!(
            matches!(other.as_mut().poll(&mut cx), Poll::Ready(Ok(()))),
            "the hold covers sliced guest code only"
        );
    }

    #[test]
    fn a_nested_hold_reports_the_outer_one() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let runner = core.clone();
        let mut thread = pin!(
            core.run_in_thread(move || async move {
                assert!(!runner.hold_others(true), "first hold");
                assert!(runner.hold_others(true), "a paint inside a key handler sees the handler's hold");
                assert!(runner.hold_others(false));
                assert!(!runner.hold_others(false));
                Ok(())
            })
            .unwrap()
        );
        assert!(matches!(
            thread.as_mut().poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(()))
        ));
    }

    #[test]
    fn only_a_thread_sliced_out_mid_code_counts_as_preempted() {
        async fn park(_: &mut ArmCore, _: &mut ()) -> Result<()> {
            YieldFuture::new().await;
            Ok(())
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.register_svc_handler(1, park, &()).unwrap();
        let mut spin = [0xc0, 0x46].repeat(20_000); // nop
        spin.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&spin, 0x1000, spin.len()).unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x20000, 4).unwrap(); // svc #1; bx lr
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let mut runner = core.clone();
        let mut sliced = pin!(
            core.run_in_thread(move || async move { runner.run_function::<()>(0x1001, &[]).await })
                .unwrap()
        );
        assert!(sliced.as_mut().poll(&mut cx).is_pending());
        assert!(core.others_preempted(), "out of budget between two guest instructions");
        while sliced.as_mut().poll(&mut cx).is_pending() {}
        assert!(!core.others_preempted(), "a finished thread is not");

        let mut runner = core.clone();
        let mut parked = pin!(
            core.run_in_thread(move || async move { runner.run_function::<()>(0x20001, &[]).await })
                .unwrap()
        );
        assert!(parked.as_mut().poll(&mut cx).is_pending());
        assert!(!core.others_preempted(), "waiting in a host call");
        assert!(matches!(parked.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
    }

    #[test]
    fn a_budget_that_runs_out_on_an_svc_counts_as_preempted() {
        async fn host_call(_: &mut ArmCore, _: &mut ()) -> Result<()> {
            YieldFuture::new().await;
            Ok(())
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.register_svc_handler(1, host_call, &()).unwrap();
        // The budget's last instruction is the `svc`, so the engine stops on it with nothing left.
        let mut code = [0xc0, 0x46].repeat(INSTRUCTIONS_PER_YIELD as usize - 1); // nop
        code.extend_from_slice(&[0x01, 0xdf, 0x70, 0x47]); // svc #1; bx lr
        core.load(&code, 0x1000, code.len()).unwrap();
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let mut runner = core.clone();
        let mut sliced = pin!(
            core.run_in_thread(move || async move { runner.run_function::<()>(0x1001, &[]).await })
                .unwrap()
        );
        assert!(sliced.as_mut().poll(&mut cx).is_pending());
        assert!(core.others_preempted(), "out of budget on the svc, before the host call runs");
        assert!(sliced.as_mut().poll(&mut cx).is_pending());
        assert!(!core.others_preempted(), "parked in the host call now");
        while sliced.as_mut().poll(&mut cx).is_pending() {}
        assert!(!core.others_preempted());
    }

    /// `nop` x 15,000, then `tail`, at `address`: one slice and a half.
    fn load_spin(core: &mut ArmCore, address: u32, tail: &[u8]) {
        let mut spin = [0xc0, 0x46].repeat(15_000); // nop
        spin.extend_from_slice(tail);
        core.load(&spin, address, spin.len()).unwrap();
    }

    // A thread that wakes from a slice to find a sliced-out holder waits for it, still mid-code. It
    // read as parked, and a callback that asked once the holder had reached a host call ran beside it.
    #[test]
    fn a_thread_kept_off_by_a_holder_still_counts_as_preempted() {
        async fn park(_: &mut ArmCore, _: &mut ()) -> Result<()> {
            YieldFuture::new().await;
            Ok(())
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.register_svc_handler(1, park, &()).unwrap();
        load_spin(&mut core, 0x1000, &[0x70, 0x47]); // bx lr
        load_spin(&mut core, 0x20000, &[0x01, 0xdf, 0x70, 0x47]); // svc #1; bx lr
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let mut runner = core.clone();
        let mut kept_off = pin!(
            core.run_in_thread(move || async move { runner.run_function::<()>(0x1001, &[]).await })
                .unwrap()
        );
        let mut runner = core.clone();
        let mut holder = pin!(
            core.run_in_thread(move || async move {
                runner.hold_others(true);
                runner.run_function::<()>(0x20001, &[]).await?;
                runner.hold_others(false);
                Ok(())
            })
            .unwrap()
        );

        assert!(kept_off.as_mut().poll(&mut cx).is_pending(), "sliced out");
        assert!(holder.as_mut().poll(&mut cx).is_pending(), "sliced out holding");
        assert!(kept_off.as_mut().poll(&mut cx).is_pending(), "awake, waiting for the holder");
        assert!(holder.as_mut().poll(&mut cx).is_pending(), "the holder is parked in its host call");
        assert!(core.others_preempted(), "5,000 instructions short of its end and not in a host call");

        assert!(kept_off.as_mut().poll(&mut cx).is_pending(), "runs now, to the end of the budget");
        assert!(matches!(kept_off.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert!(!core.others_preempted());
        assert!(matches!(holder.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
    }

    // Two holders can be sliced out at once: the second waited its `HOLD_ROUNDS` and ran anyway. The
    // one that wakes first waits for the other — which must not wait back, or both sit out the cap.
    #[test]
    fn two_sliced_holders_do_not_wait_for_each_other() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        load_spin(&mut core, 0x1000, &[0x70, 0x47]); // bx lr
        compile(&mut core);

        let mut cx = Context::from_waker(Waker::noop());
        let holder = || {
            let mut runner = core.clone();
            Box::pin(
                core.run_in_thread(move || async move {
                    runner.hold_others(true);
                    runner.run_function::<()>(0x1001, &[]).await
                })
                .unwrap(),
            )
        };
        let (mut first, mut second) = (holder(), holder());

        assert!(first.as_mut().poll(&mut cx).is_pending(), "sliced out holding");
        for _ in 0..HOLD_ROUNDS {
            assert!(second.as_mut().poll(&mut cx).is_pending(), "waits for the first");
        }
        assert!(
            second.as_mut().poll(&mut cx).is_pending(),
            "past the cap: ran, and is sliced out holding too"
        );

        assert!(first.as_mut().poll(&mut cx).is_pending(), "awake, waiting for the second");
        assert!(
            matches!(second.as_mut().poll(&mut cx), Poll::Ready(Ok(()))),
            "the second runs: the first is awake, not sliced out"
        );
        while first.as_mut().poll(&mut cx).is_pending() {}
    }

    // A hold left behind by a thread that ended would keep `wait_for_holder` off its fast path for good.
    #[test]
    fn a_thread_that_ends_holding_leaves_no_hold_behind() {
        let core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: engine_under_test(),
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core.clone()).unwrap();
        let runner = core.clone();
        let mut thread = Box::pin(
            core.run_in_thread(move || async move {
                runner.hold_others(true);
                Ok(())
            })
            .unwrap(),
        );
        assert!(matches!(
            thread.as_mut().poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(()))
        ));
        assert_eq!(core.holders.lock().len(), 1);
        drop(thread);
        assert!(core.holders.lock().is_empty());
    }

    /// The eleven above with the native AOT backend attached (docs/report/0607 §6 ⑶). Compiled code does not
    /// count its instructions against the budget, so no thread is ever sliced out inside it, and the eight
    /// marked rows fail at the first place they expect one to be. `should_panic` keeps that count in the
    /// gates: a marked row that starts passing reddens, and the round that makes compiled code yield takes
    /// its mark off.
    mod under_aot {
        struct Reset;

        impl Drop for Reset {
            fn drop(&mut self) {
                super::UNDER_AOT.set(false); // the harness may run the next test on this thread
            }
        }

        macro_rules! rerun {
            ($($(#[$unmet:meta])? $name:ident)*) => {$(
                #[test]
                $(#[$unmet])?
                fn $name() {
                    super::UNDER_AOT.set(true);
                    let _reset = Reset;
                    super::$name()
                }
            )*};
        }

        rerun! {
            // The first slice ends after 10,000 `svc`s, not 5,000: the `add`s between them are not counted.
            #[should_panic(expected = "left: 10000")]
            yields_every_ten_thousand_instructions_across_svc
            #[should_panic(expected = "holder.as_mut().poll(&mut cx).is_pending()")]
            a_holder_sliced_out_keeps_the_others_off_guest_code
            #[should_panic(expected = "holder.as_mut().poll(&mut cx).is_pending()")]
            another_thread_taking_a_hold_leaves_the_holders_in_place
            #[should_panic(expected = "holder.as_mut().poll(&mut cx).is_pending()")]
            another_thread_letting_go_leaves_the_holders_in_place
            a_holder_parked_in_a_host_call_keeps_nobody_off
            a_nested_hold_reports_the_outer_one
            #[should_panic(expected = "sliced.as_mut().poll(&mut cx).is_pending()")]
            only_a_thread_sliced_out_mid_code_counts_as_preempted
            #[should_panic(expected = "out of budget on the svc")]
            a_budget_that_runs_out_on_an_svc_counts_as_preempted
            #[should_panic(expected = "sliced out")]
            a_thread_kept_off_by_a_holder_still_counts_as_preempted
            #[should_panic(expected = "sliced out holding")]
            two_sliced_holders_do_not_wait_for_each_other
            a_thread_that_ends_holding_leaves_no_hold_behind
        }
    }

    #[test]
    fn nested_arm_calls_share_the_budget_and_keep_suspended_roots_visible() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let _context = core.enter_thread_context(1);
        let suspended = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let mut suspended_context = core.read_thread_context(2).unwrap();
        suspended_context.r4 = 0x12345678;
        suspended_context.sp -= 4;
        write_generic(&mut core, suspended_context.sp, 0xabcdef01u32).unwrap();
        core.write_thread_context(2, &suspended_context);
        let result = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, call_nested_arm, &result).unwrap();
        let mut outer = [0xc0, 0x46].repeat(5_000); // nop
        outer.extend_from_slice(&[0x01, 0xdf, 0x70, 0x47]); // svc #1; bx lr
        core.load(&outer, 0x1000, outer.len()).unwrap();
        let mut nested = [0x01, 0x30].repeat(5_000); // add r0, #1
        nested.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&nested, 0x10000, nested.len()).unwrap();

        let observer = core.clone();
        let mut run = pin!(core.run_function::<u32>(0x1001, &[17]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.read_param(0).unwrap(), 4_999);
        assert_eq!(result.load(Ordering::Relaxed), 0);
        let words = observer.guest_root_words().unwrap();
        assert!(words.contains(&17), "the caller's suspended R0 must remain visible");
        assert!(words.contains(&0x12345678));
        assert!(words.contains(&0xabcdef01));
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(17))));
        assert_eq!(result.load(Ordering::Relaxed), 5_000);
        drop(suspended);
        let words = observer.guest_root_words().unwrap();
        assert!(!words.contains(&17));
        assert!(!words.contains(&0x12345678));
        assert!(!words.contains(&0xabcdef01));
    }

    // fork: `run_function` hands the caller its registers and stack back on an error as well — upstream
    // #1534 leaves the fault's there instead (`failed_arm_calls_preserve_fault_registers_and_stack`).
    #[test]
    fn failed_arm_calls_restore_caller_registers_and_stack() {
        async fn throwing_handler(core: &mut ArmCore, _: &mut ()) -> Result<()> {
            core.write_return_value(&[99, 98])?;
            Err(WieError::JavaException(0x1234))
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        core.map(0x2000, 0x1000).unwrap();
        core.register_svc_handler(1, throwing_handler, &()).unwrap();
        let target = core.make_svc_stub(1, 0u32).unwrap();
        let mut context = core.save_context();
        context.r0 = 42;
        context.r1 = 43;
        context.sp = 0x3000;
        context.lr = 0x4001;
        core.restore_context(&context);
        let registers = core.dump_regs();

        for address in [target, 0x1001] {
            let result = futures::executor::block_on(core.run_function::<()>(address, &[1, 2, 3, 4, 5]));
            if address == target {
                assert!(matches!(result, Err(WieError::JavaException(0x1234))));
            } else {
                assert!(matches!(result, Err(WieError::InvalidMemoryAccess(_))));
            }
            assert_eq!(core.dump_regs(), registers);
        }
    }

    #[test]
    fn returning_arm_calls_do_not_reset_the_instruction_budget() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let _context = core.enter_thread_context(1);
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        let completed = Arc::new(AtomicU32::new(0));
        let observed = completed.clone();
        let mut run = pin!(async move {
            for _ in 0..10_001 {
                core.run_function::<()>(0x1001, &[]).await.unwrap();
                completed.fetch_add(1, Ordering::Relaxed);
            }
        });
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observed.load(Ordering::Relaxed), 9_999);
        assert!(run.as_mut().poll(&mut cx).is_ready());
        assert_eq!(observed.load(Ordering::Relaxed), 10_001);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn ready_arm_threads_yield_and_allow_host_tasks_to_run() {
        use test_utils::{TestClock, TestPlatform};
        use wie_backend::{DefaultTaskRunner, System};

        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "test", "test", DefaultTaskRunner);
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.load(&[0x01, 0x30, 0xfd, 0xe7], 0x1000, 4).unwrap(); // add r0, #1; b 0x1000
        let polls = Arc::new(AtomicU32::new(0));
        for _ in 0..2 {
            let mut running = core.clone();
            let task = core
                .run_in_thread(move || async move { running.run_function::<()>(0x1001, &[0]).await })
                .unwrap();
            let clock = clock.clone();
            let polls = polls.clone();
            system.spawn(move || async move {
                let mut task = pin!(task);
                poll_fn(|cx| {
                    let result = task.as_mut().poll(cx);
                    assert!(result.is_pending());
                    polls.fetch_add(1, Ordering::Relaxed);
                    // Limit the executor to one step.
                    clock.advance(9);
                    result
                })
                .await
            });
        }
        let host_calls = Arc::new(AtomicU32::new(0));
        let observed = host_calls.clone();
        system.spawn(move || async move {
            host_calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });

        system.tick().unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 2);
        assert_eq!(observed.load(Ordering::Relaxed), 1);
        let first = [1, 2].map(|thread| core.read_thread_context(thread).unwrap().r0);
        assert_eq!(first, [5_000, 5_000]);
        assert_eq!(core.inner.lock().instructions_remaining, 10_000);
        system.tick().unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 4);
        for (thread, previous) in [1, 2].into_iter().zip(first) {
            assert_eq!(core.read_thread_context(thread).unwrap().r0 - previous, 5_000);
        }
    }

    #[test]
    fn test_thumb_svc_stub_dispatch() {
        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        core.map(0x1000, 0x1000).unwrap();

        let mut context = core.save_context();
        context.sp = 0x2000;
        core.restore_context(&context);

        core.register_svc_handler(1, test_svc_handler, &None).unwrap();
        let first = core.make_svc_stub(1, 0u32).unwrap();
        let second = core.make_svc_stub(1, 1u32).unwrap();
        assert_eq!(first, FUNCTIONS_BASE + 1);
        assert_eq!(second, FUNCTIONS_BASE + SVC_STUB_SIZE + 1);

        let result = {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::Cpsr, 0x3f);
            inner.engine.reg_write(ArmRegister::PC, second);
            inner.engine.reg_write(ArmRegister::LR, RUN_FUNCTION_LR);
            inner.engine.run(RUN_FUNCTION_LR, 10).unwrap()
        };

        assert_eq!(result.budget_consumed, 5);
        match result.stop_reason {
            EngineStopReason::Svc { category, lr, spsr } => {
                assert_eq!(category, 1);
                assert_eq!(lr, FUNCTIONS_BASE + SVC_STUB_SIZE + 10);
                assert_ne!(spsr & 0x20, 0);
            }
            EngineStopReason::End => panic!("expected SVC, got end"),
            EngineStopReason::Yield => panic!("expected SVC, got yield"),
        }
    }

    #[test]
    fn double_result_reaches_guest_registers_in_low_high_word_order() {
        async fn return_double(_core: &mut ArmCore, value: &mut f64) -> Result<f64> {
            Ok(*value)
        }

        let mut core = ArmCore::new(Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })
        .unwrap();
        // push {r4, lr}; mov r4, r3; blx r2; stmia r4, {r0, r1}; pop {r4, pc}
        let code = [0xe92d4010u32, 0xe1a04003, 0xe12fff32, 0xe8840003, 0xe8bd8010];
        core.load(cast_slice(&code), 0x1000, 0x1000).unwrap();
        let mut context = core.save_context();
        context.sp = 0x2000;
        core.restore_context(&context);
        core.register_svc_handler(1, return_double, &-1.1).unwrap();
        let function = core.make_svc_stub(1, 0u32).unwrap();
        futures::executor::block_on(core.run_function::<()>(0x1000, &[0, 0, function, 0x1800])).unwrap();
        assert_eq!(read_generic::<u32, _>(&core, 0x1800).unwrap(), 0x9999999a);
        assert_eq!(read_generic::<u32, _>(&core, 0x1804).unwrap(), 0xbff19999);
    }
}
