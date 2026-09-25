use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext, RUN_FUNCTION_LR};
use wie_util::{Result, WieError, read_generic, write_generic};

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;
const FRAME_WORDS: u32 = 18;

// Host-side ledger for the LGT Java SVC handlers; the guest never reads it. The fixed word holds
// the head of a list with one record per emulated thread, because a try block on one thread must
// not push onto, pop, or unwind into another thread's frames (a shared chain did exactly that when
// threads interleaved inside try blocks).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaSupportContext {
    ptr_first_thread_state: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ThreadExceptionState {
    thread_key: u32,
    ptr_current_exception_frame: u32,
    ptr_pending_exception: u32,
    ptr_next: u32,
    // sp of the frame the last unwind consumed, until a push or pop settles it; 0 = none.
    consumed_frame_sp: u32,
}

pub fn init(core: &mut ArmCore) -> Result<()> {
    write_generic(core, SUPPORT_CONTEXT_BASE, JavaSupportContext::zeroed())
}

// 0 is guest code run outside any emulated thread; thread ids start at 1.
fn current_thread_key(core: &ArmCore) -> u32 {
    core.current_thread_id().map_or(0, |thread_id| thread_id as u32)
}

fn find_thread_state(core: &ArmCore) -> Result<Option<(u32, ThreadExceptionState)>> {
    let thread_key = current_thread_key(core);
    let support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let mut ptr_state = support_context.ptr_first_thread_state;
    while ptr_state != 0 {
        let state: ThreadExceptionState = read_generic(core, ptr_state)?;
        if state.thread_key == thread_key {
            return Ok(Some((ptr_state, state)));
        }
        ptr_state = state.ptr_next;
    }

    Ok(None)
}

fn find_or_create_thread_state(core: &mut ArmCore) -> Result<(u32, ThreadExceptionState)> {
    if let Some(found) = find_thread_state(core)? {
        return Ok(found);
    }

    let mut support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let state = ThreadExceptionState {
        thread_key: current_thread_key(core),
        ptr_next: support_context.ptr_first_thread_state,
        ..ThreadExceptionState::zeroed()
    };
    let ptr_state = Allocator::alloc(core, size_of::<ThreadExceptionState>() as u32)?;
    write_generic(core, ptr_state, state)?;
    support_context.ptr_first_thread_state = ptr_state;
    write_generic(core, SUPPORT_CONTEXT_BASE, support_context)?;

    Ok((ptr_state, state))
}

pub fn push(core: &mut ArmCore) -> Result<()> {
    let context = core.save_context();
    let (ptr_state, mut state) = find_or_create_thread_state(core)?;
    let frame = [
        state.ptr_current_exception_frame,
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
    ];
    let ptr_frame = Allocator::alloc(core, FRAME_WORDS * size_of::<u32>() as u32)?;
    write_generic(core, ptr_frame, frame)?;
    state.ptr_current_exception_frame = ptr_frame;
    state.ptr_pending_exception = 0;
    state.consumed_frame_sp = 0;
    write_generic(core, ptr_state, state)
}

pub fn pop(core: &mut ArmCore) -> Result<()> {
    let sp = core.save_context().sp;
    if let Some((ptr_state, mut state)) = find_thread_state(core)?
        && state.consumed_frame_sp != 0
    {
        let consumed_here = state.consumed_frame_sp == sp;
        state.consumed_frame_sp = 0;
        if consumed_here {
            // The catch pops the frame unwind already consumed (the Thumb titles' convention).
            state.ptr_pending_exception = 0;
            return write_generic(core, ptr_state, state);
        }
        write_generic(core, ptr_state, state)?;
    }

    let (ptr_state, mut state) = find_thread_state(core)?
        .filter(|(_, state)| state.ptr_current_exception_frame != 0)
        .ok_or_else(|| WieError::FatalError("LGT exception frame pop without a pushed frame on this thread".into()))?;
    let ptr_frame = state.ptr_current_exception_frame;
    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
    state.ptr_current_exception_frame = frame[0];
    state.ptr_pending_exception = 0;
    write_generic(core, ptr_state, state)?;
    Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)
}

pub fn pending(core: &ArmCore) -> Result<u32> {
    Ok(find_thread_state(core)?.map_or(0, |(_, state)| state.ptr_pending_exception))
}

pub fn unwind(core: &mut ArmCore, ptr_exception: u32) -> Result<Option<u32>> {
    // Rust callers must handle the exception before an enclosing guest catch can run.
    if core.read_pc_lr()?.1 == RUN_FUNCTION_LR {
        return Ok(None);
    }

    let Some((ptr_state, mut state)) = find_thread_state(core)? else {
        return Ok(None);
    };
    if state.ptr_current_exception_frame == 0 {
        return Ok(None);
    }

    let ptr_frame = state.ptr_current_exception_frame;
    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
    // Entering a catch consumes its frame. Two compiled conventions are measured, and both need
    // it: 배틀몬스터 (ARM, 51 of 51 catch blocks) never pops from a catch, so a frame left on the
    // chain outlives its stack and a later throw resumes into dead registers (fp = sp = pc = 0);
    // 현영맞고2006 and 놈3 (Thumb) do pop from the catch, at the frame's sp. consumed_frame_sp
    // turns that pop into a no-op instead of letting it free the enclosing try's frame. A pop at
    // any other sp, or a push, settles it: that pop belongs to a frame still on the chain.
    // ponytail: sp is the only discriminator — an ARM function with nested tries at one sp whose
    // inner catch returns into the outer try would no-op the outer pop (main leaked the same frame
    // there too); widen the key (e.g. with the pop's return address) if a title shows that shape.
    state.ptr_current_exception_frame = frame[0];
    state.ptr_pending_exception = ptr_exception;
    state.consumed_frame_sp = frame[14];
    write_generic(core, ptr_state, state)?;
    Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)?;

    let context = ArmCoreContext {
        r0: frame[1],
        r1: frame[2],
        r2: frame[3],
        r3: frame[4],
        r4: frame[5],
        r5: frame[6],
        r6: frame[7],
        r7: frame[8],
        r8: frame[9],
        sb: frame[10],
        sl: frame[11],
        fp: frame[12],
        ip: frame[13],
        sp: frame[14],
        lr: frame[15],
        pc: frame[16],
        cpsr: frame[17],
    };
    core.restore_context(&context);
    core.set_next_pc(context.lr)?;

    Ok(Some(context.lr))
}

#[cfg(test)]
mod tests {
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use super::{init, pending, pop, push, unwind};

    #[test]
    fn exception_frame_restores_guest_context() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        let mut context = core.save_context();
        context.r4 = 0x44;
        context.sp = 0x12000;
        context.lr = 0x4001;
        core.restore_context(&context);
        push(&mut core)?;

        context.r4 = 0;
        context.lr = 0;
        core.restore_context(&context);
        assert_eq!(unwind(&mut core, 0x1234)?, Some(0x4001));

        let restored = core.save_context();
        assert_eq!(restored.r4, 0x44);
        assert_eq!(restored.sp, 0x12000);
        assert_eq!(restored.pc, 0x4000);
        assert_eq!(pending(&core)?, 0x1234);

        pop(&mut core)?;
        assert_eq!(pending(&core)?, 0);
        assert_eq!(unwind(&mut core, 0x5678)?, None);
        assert_eq!(pending(&core)?, 0);

        Ok(())
    }

    fn push_at(core: &mut ArmCore, sp: u32, lr: u32) -> Result<()> {
        let mut context = core.save_context();
        context.sp = sp;
        context.lr = lr;
        core.restore_context(&context);
        push(core)
    }

    fn throw_at(core: &mut ArmCore, sp: u32, ptr_exception: u32) -> Result<Option<u32>> {
        let mut context = core.save_context();
        context.sp = sp;
        context.lr = 0;
        core.restore_context(&context);
        unwind(core, ptr_exception)
    }

    // 현영맞고2006 / 놈3 (Thumb): the catch pops its own frame. That pop must not free the
    // enclosing try's frame (the review trace: POP released the outer 0x4a85c380).
    #[test]
    fn catch_that_pops_leaves_the_enclosing_frame_on_the_chain() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        push_at(&mut core, 0x12000, 0x7cb7)?;
        push_at(&mut core, 0x11f00, 0x7ccb)?;
        assert_eq!(throw_at(&mut core, 0x11e00, 0x1234)?, Some(0x7ccb));
        assert_eq!(core.save_context().sp, 0x11f00);
        pop(&mut core)?;
        assert_eq!(pending(&core)?, 0);

        assert_eq!(throw_at(&mut core, 0x11f00, 0x5678)?, Some(0x7cb7));
        pop(&mut core)?;
        assert_eq!(throw_at(&mut core, 0x12000, 0x9abc)?, None);

        Ok(())
    }

    // 배틀몬스터 (ARM): the catch never pops and returns. The enclosing function's own pop at the
    // end of its try must still release its frame, and nothing stale may be left to resume into.
    #[test]
    fn catch_that_returns_without_pop_leaves_no_dead_frame() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        push_at(&mut core, 0x12000, 0x4001)?;
        push_at(&mut core, 0x11f00, 0x5001)?;
        assert_eq!(throw_at(&mut core, 0x11e00, 0x1234)?, Some(0x5001));
        // The catch returns; the caller finishes its try body and pops at its own sp.
        let mut context = core.save_context();
        context.sp = 0x12000;
        core.restore_context(&context);
        pop(&mut core)?;
        assert_eq!(throw_at(&mut core, 0x12000, 0x5678)?, None);

        // A catch at the top level that returns without pop leaves nothing behind either.
        push_at(&mut core, 0x12000, 0x6001)?;
        assert_eq!(throw_at(&mut core, 0x11e00, 0x1234)?, Some(0x6001));
        assert_eq!(throw_at(&mut core, 0x12100, 0x5678)?, None);

        Ok(())
    }

    #[test]
    fn rethrow_from_catch_reaches_the_enclosing_frame() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        push_at(&mut core, 0x12000, 0x4001)?;
        push_at(&mut core, 0x11f00, 0x5001)?;
        assert_eq!(throw_at(&mut core, 0x11e00, 0x1234)?, Some(0x5001));
        assert_eq!(throw_at(&mut core, 0x11f00, 0x1234)?, Some(0x4001));
        assert_eq!(core.save_context().sp, 0x12000);
        assert_eq!(throw_at(&mut core, 0x12000, 0x1234)?, None);

        Ok(())
    }

    // Mirrors the interleave measured on 놈3: thread 2 pushes while thread 1 holds a frame, then
    // thread 1 pops. With one shared chain thread 1 freed thread 2's frame and thread 2's catch
    // restored thread 1's registers.
    #[test]
    fn exception_frames_are_per_thread() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        let _thread1 = core.run_in_thread(|| async { Ok(()) })?;
        let _thread2 = core.run_in_thread(|| async { Ok(()) })?;
        let push_on = |core: &mut ArmCore, thread_id, r4, lr| -> Result<()> {
            let _guard = core.enter_thread_context(thread_id);
            let mut context = core.save_context();
            context.r4 = r4;
            context.lr = lr;
            core.restore_context(&context);
            push(core)
        };

        push_on(&mut core, 1, 0x11, 0x1001)?;
        push_on(&mut core, 2, 0x22, 0x2001)?;

        {
            let _guard = core.enter_thread_context(1);
            pop(&mut core)?;
            assert_eq!(unwind(&mut core, 0x1234)?, None);
        }
        {
            let _guard = core.enter_thread_context(2);
            assert_eq!(unwind(&mut core, 0x5678)?, Some(0x2001));
            assert_eq!(core.save_context().r4, 0x22);
            assert_eq!(pending(&core)?, 0x5678);
            pop(&mut core)?;
            assert_eq!(unwind(&mut core, 0x5678)?, None);
        }
        assert_eq!(pending(&core)?, 0);

        Ok(())
    }
}
