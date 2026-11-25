use wie_util::Result;

use crate::{Allocator, ArmCore, context::ArmCoreContext};

// fork: 1 MB, as before the 2026-11-23 upstream merge (upstream: 64 KB). The stack is a heap block, so
// a guest that recurses past it writes into its neighbours rather than faulting.
const STACK_SIZE: u32 = 0x100000; // 1 MB

pub struct ThreadState {
    core: ArmCore,
    pub context: ArmCoreContext,
    pub stack_base: usize,
    pub stack_size: usize,
    /// Sliced out by the instruction budget and not polled since. What a thread waiting for a
    /// holder reads (`ArmCore::hold_others`): it clears the moment the thread wakes, so two holders
    /// never wait for each other.
    pub preempted: bool,
    /// In the middle of guest code but not running it: sliced out between two guest instructions
    /// rather than parked in a host call, and — unlike `preempted` — still so while it is awake
    /// waiting for a holder. See [`ArmCore::others_preempted`].
    pub mid_code: bool,
}

impl ThreadState {
    pub fn new(mut core: ArmCore) -> Result<Self> {
        let stack_base = Allocator::alloc(&mut core, STACK_SIZE)?;
        let context = ArmCoreContext {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            sb: 0,
            sl: 0,
            fp: 0,
            ip: 0,
            sp: stack_base + STACK_SIZE,
            lr: 0,
            pc: 0,
            cpsr: 0x10, // USR32
        };

        Ok(Self {
            core,
            context,
            stack_base: stack_base as _,
            stack_size: STACK_SIZE as _,
            preempted: false,
            mid_code: false,
        })
    }
}

impl Drop for ThreadState {
    fn drop(&mut self) {
        if let Err(err) = Allocator::free(&mut self.core, self.stack_base as _, self.stack_size as u32) {
            tracing::error!("Failed to free thread stack: {err}");
        }
    }
}
