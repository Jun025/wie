mod sprintf;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::iter;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use crate::{WIPICResult, context::WIPICContext, method::MethodBody};

pub use self::sprintf::sprintf;

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICTimer {
    fn_callback: WIPICWord,
}

pub async fn current_time(context: &mut dyn WIPICContext) -> Result<u64> {
    tracing::debug!("MC_knlCurrentTime()");

    Ok(context.system().platform().now().raw())
}

pub async fn get_system_property(context: &mut dyn WIPICContext, ptr_id: WIPICWord, p_out: WIPICWord, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetSystemProperty({ptr_id:#x}, {p_out:#x}, {buf_size:#x})");

    let id_bytes = read_null_terminated_string_bytes(context, ptr_id)?;
    let id = encoding_rs::EUC_KR.decode(&id_bytes).0;

    let value = match id.as_ref() {
        "RSSILEVEL" => "30",
        "BATTERYLEVEL" => "100",
        "PHONEMODEL" => "Emulator",
        // Was "" ("putting this cause some game to fail authentication" — no title named). An LGT
        // title copies `strlen(number) - 4` bytes of it: "" became a 4 GB memcpy and a host stack
        // overflow (a23f3c9fc2cb). SKT's MIN value; the titles that read this were re-run with it.
        "PHONENUMBER" => "01000000000",
        "MIN" => "01000000000",
        "ANNUN_CALL" => "0",
        "ANNUN_SMS" => "0",
        "ANNUN_SILENT" => "0",
        "ANNUN_ALARM" => "0",
        "ANNUN_SECURITY" => "0",
        "CURRENTCH" => "0",
        "AIRPLANE_MODE" => "0",
        "ROAMING_AREA" => "0",
        "DS_LOCK" => "0",
        // Titles look for "Yamaha_MA3" here — the SMAF chip, the one format `clip_put_data`
        // plays — and create no clip without it: M_E_INVALID left 3 LGT titles silent
        // (2026-09-27 silent-104 census, 0 → 2..8 plays in 90 s).
        "MEDIADEVICES" => "Yamaha_MA3",
        _ => {
            tracing::warn!("unknown system property id: {id}");
            return Ok(-9); // M_E_INVALID
        }
    };

    let bytes = value.as_bytes();
    if bytes.len() + 1 > buf_size as usize {
        return Ok(-18); // M_E_SHORTBUF
    }

    write_null_terminated_string_bytes(context, p_out, value.as_bytes())?;

    Ok(0)
}

pub async fn set_system_property(_context: &mut dyn WIPICContext, ptr_id: WIPICWord, ptr_value: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlSetSystemProperty({ptr_id:#x}, {ptr_value:#x})");

    Ok(())
}

pub async fn def_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord, fn_callback: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlDefTimer({ptr_timer:#x}, {fn_callback:#x})");

    let timer = WIPICTimer { fn_callback };

    write_generic(context, ptr_timer, timer)?;

    Ok(())
}

pub async fn set_timer(
    context: &mut dyn WIPICContext,
    ptr_timer: WIPICWord,
    timeout_low: WIPICWord,
    timeout_high: WIPICWord,
    param: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_knlSetTimer({ptr_timer:#x}, {timeout_low:#x}, {timeout_high:#x}, {param:#x})");

    struct TimerCallback {
        ptr_timer: WIPICWord,
        fn_callback: WIPICWord,
        param: WIPICWord,
        arming: u64,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for TimerCallback {
        #[tracing::instrument(name = "timer", skip_all)]
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            // Unset since. With Unset a no-op, a title that re-arms with Unset + Set kept the old
            // chain alive beside the new one, and its game loop ran two to four times per period
            // (2d5cada03004: 37 callbacks/s of a 62ms timer; 3cc7a9b4cb15: 40/s of a 37ms one).
            if !context.system().event_queue().is_timer_armed(self.ptr_timer, self.arming) {
                return Ok(WIPICResult { results: Vec::new() });
            }
            let tick = context.system().pacing().ticks();
            context.system().event_queue().timer_fired(self.ptr_timer, tick);
            context.call_function(self.fn_callback, &[self.ptr_timer, self.param]).await?;

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    let now = context.system().platform().now();
    let timeout = (((timeout_high as u64) << 32) | (timeout_low as u64)) as _;
    let timer: WIPICTimer = read_generic(context, ptr_timer)?;
    let arming = context.system().event_queue().arm_timer(ptr_timer);
    // When a tick may be kept alive until this timer is due (instead of the host's next frame, the
    // 16.7ms grid every timer used to land on): never for 1ms, which means "as soon as you can" (KTF
    // 49ade89578c5 re-arms MC_knlSetTimer(1) each frame; waiting on it ran the game at 44fps, not its 39),
    // and not in the tick this timer already fired in — a second fire there is a timer faster than
    // the host's frames (a ~10ms one: 62 -> 94fps).
    let tick = context.system().pacing().ticks();
    let pace_from = if timeout <= 1 {
        u64::MAX
    } else if context.system().event_queue().timer_fired_in(ptr_timer, tick) {
        tick + 1
    } else {
        0
    };

    context.set_timer(
        now + timeout,
        pace_from,
        Box::new(TimerCallback {
            ptr_timer,
            fn_callback: timer.fn_callback,
            param,
            arming,
        }),
    );

    Ok(())
}

pub async fn unset_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlUnsetTimer({ptr_timer:#x})");

    context.system().event_queue().cancel_timer(ptr_timer);

    Ok(())
}

pub async fn alloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlAlloc({size:#x})");

    if size == 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    alloc_or_null(context, size)
}

pub async fn calloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlCalloc({size:#x})");

    if size == 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    let memory = alloc_or_null(context, size)?;
    if memory.0 == 0 {
        return Ok(memory);
    }

    let zero = iter::repeat_n(0, size as _).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

/// Allocate `size` bytes, returning a NULL handle (0) when the heap can't satisfy
/// the request — the C/WIPI-C `malloc`/`calloc` contract — instead of propagating
/// `AllocationFailure` as a fatal VM error.
///
/// LGT clets have been seen requesting wildly oversized buffers from a corrupted
/// size argument (놈ZERO's "처음부터 시작" asks for `calloc(0x72657473)` ≈ 1.9 GB —
/// the size is ASCII "ster"/resource garbage, not a real length). That exceeds the
/// 256 MB heap, so the allocator correctly fails; turning that into a fatal abort
/// killed the whole VM. A real device's `malloc` returns NULL here, letting the app
/// notice the failed allocation and continue. Successful allocations are unchanged,
/// and any non-OOM allocator error still propagates — so no normal game's behaviour
/// changes.
fn alloc_or_null(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    match context.alloc(size) {
        Ok(memory) => Ok(memory),
        Err(WieError::AllocationFailure) => {
            tracing::warn!("allocation of {size:#x} bytes failed (out of memory); returning NULL");
            Ok(WIPICIndirectPtr(0))
        }
        Err(e) => Err(e),
    }
}

pub async fn free(context: &mut dyn WIPICContext, memory: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlFree({:#x})", memory.0);

    if memory.0 == 0 {
        return Ok(memory);
    }

    context.free(memory)?;

    Ok(memory)
}

pub async fn get_resource_id(context: &mut dyn WIPICContext, ptr_name: WIPICWord, ptr_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResourceID({ptr_name:#x}, {ptr_size:#x})");

    let raw_name = read_null_terminated_string_bytes(context, ptr_name)?;
    let name = encoding_rs::EUC_KR.decode(&raw_name).0;
    tracing::debug!("  resource name: {name}");

    let size = context.get_resource_size(&name).await?;

    if size.is_none() {
        if ptr_size != 0 {
            write_generic(context, ptr_size, 0u32)?;
        }
        return Ok(-12); // M_E_NOENT
    }

    let size = size.unwrap();

    let name_bytes = name.as_bytes();
    let handle_size = name_bytes
        .len()
        .checked_add(1)
        .ok_or_else(|| WieError::FatalError("Resource name too long".to_string()))?;
    let ptr_handle = context.alloc_raw(handle_size as _)?;
    write_null_terminated_string_bytes(context, ptr_handle, name_bytes)?;
    write_generic(context, ptr_size, size as u32)?;

    Ok(ptr_handle as _)
}

pub async fn get_resource(context: &mut dyn WIPICContext, id: i32, buf: WIPICIndirectPtr, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResource({id}, {:#x}, {buf_size})", buf.0);

    if id < 0 {
        return Ok(-9); // M_E_INVALID
    }

    let name_bytes = read_null_terminated_string_bytes(context, id as _)?;
    // the handle was written by get_resource_id as utf-8, not guest-encoded
    let name = String::from_utf8_lossy(&name_bytes);

    let data = context.read_resource(&name).await?;

    // -18, not the -1 this returned until 2026-09-07. -1 is not a `WIPICError` variant at the
    // pinned rev (`{1, 0, -9, -12, -18, -22, -25}` in wipi_types/src/wipic.rs) and
    // `wipic_sys::kernel::get_resource` is typed `-> WIPICError` via `from_raw`, which is a
    // `transmute` — so handing a guest -1 was an invalid discriminant, i.e. UB. Measured at that
    // rev: `get_resource` and `graphics::create_image` are the *only* host functions reachable
    // through that transmute (6 wrappers, ktf/lgt/simulation), and create_image only ever returns
    // 1, so this was the one out-of-vocabulary value on a transmute path.
    //
    // Whether a real title branches on -1 CANNOT be checked here: there is no commercial corpus
    // (Constraint 9) and no WIPI error-code spec in this repo or the pinned wipi repo — `M_E_*` has
    // zero definitions in either, the names in this file are bare comments. So this trades an
    // unmeasurable compatibility risk for removing a definite UB. The two sibling sites that
    // already return -18 for the same condition (`get_system_property`, `get_program_name`) are
    // deliberately untouched; this makes all three agree.
    if data.len() as u32 > buf_size {
        return Ok(-18); // M_E_SHORTBUF
    }

    context.write_bytes(context.data_ptr(buf)?, &data)?;

    Ok(0)
}

pub async fn printk(context: &mut dyn WIPICContext, ptr_format: WIPICWord, a0: WIPICWord, a1: WIPICWord, a2: WIPICWord, a3: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlPrintk({ptr_format:#x}, {a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let result = sprintf(context, &format_string, &[a0, a1, a2, a3])?;
    let result = encoding_rs::EUC_KR.decode(&result).0;

    context.system().platform().write_stdout(result.as_bytes());

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn sprintk(
    context: &mut dyn WIPICContext,
    dest: WIPICWord,
    ptr_format: WIPICWord,
    a0: WIPICWord,
    a1: WIPICWord,
    a2: WIPICWord,
    a3: WIPICWord,
    a4: WIPICWord,
    a5: WIPICWord,
) -> Result<WIPICWord> {
    tracing::debug!("MC_knlSprintk({dest:#x}, {ptr_format:#x}, {a1}, {a2}, {a3}, {a4}, {a5})",);

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let result = sprintf(context, &format_string, &[a0, a1, a2, a3, a4, a5])?;

    write_null_terminated_string_bytes(context, dest, &result)?;

    Ok(result.len() as _)
}

pub async fn get_total_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetTotalMemory()");

    Ok(0x100000) // TODO hardcoded
}

pub async fn get_free_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetFreeMemory()");

    Ok(0x100000) // TODO hardcoded
}

pub async fn exit(context: &mut dyn WIPICContext, code: i32) -> Result<()> {
    tracing::debug!("MC_knlExit({code})");

    context.system().exit_from_guest().await
}

pub async fn get_cur_program_id(_context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_knlGetCurProgramID()");

    Ok(1)
}

pub async fn get_program_name(context: &mut dyn WIPICContext, name_buf: WIPICWord, buf_size: i32) -> Result<i32> {
    tracing::debug!("MC_knlGetProgramName({name_buf:#x}, {buf_size})");

    let aid = context.system().aid().to_string();

    if buf_size < aid.len() as i32 + 1 {
        return Ok(-18); // M_E_SHORTBUF
    }

    let aid_bytes = aid.as_bytes();
    context.write_bytes(name_buf, aid_bytes)?;
    context.write_bytes(name_buf + aid_bytes.len() as u32, &[0])?;

    Ok(0)
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, string::String};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite, Result, read_null_terminated_string_bytes, write_null_terminated_string_bytes};

    use crate::{WIPICContext, context::test::TestContext, method::MethodImpl};

    use super::{
        alloc, calloc, def_timer, free, get_program_name, get_resource, get_resource_id, get_system_property, set_timer, sprintk, unset_timer,
    };

    #[futures_test::test]
    async fn test_sprintk() -> Result<()> {
        let mut context = TestContext::new();

        let sprintk = sprintk.into_body();

        let format = context.alloc_raw(10).unwrap();
        let dest = context.alloc_raw(10).unwrap();

        write_null_terminated_string_bytes(&mut context, format, "%d".as_bytes()).unwrap();
        sprintk
            .call(&mut context, Box::new([dest, format, 1234, 0, 0, 0, 0, 0, 0, 0]))
            .await
            .unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "1234");

        write_null_terminated_string_bytes(&mut context, format, "test %02d".as_bytes()).unwrap();
        sprintk
            .call(&mut context, Box::new([dest, format, 1, 0, 0, 0, 0, 0, 0, 0]))
            .await
            .unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "test 01");

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_system_property_min() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        write_null_terminated_string_bytes(&mut context, id, b"MIN").unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), 0);
        let result = read_null_terminated_string_bytes(&context, out).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "01000000000");

        Ok(())
    }

    // An LGT title copies `strlen(PHONENUMBER) - 4` bytes; "" underflowed into a 4 GB memcpy.
    #[futures_test::test]
    async fn test_get_system_property_phone_number_is_not_empty() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        write_null_terminated_string_bytes(&mut context, id, b"PHONENUMBER").unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), 0);
        assert!(read_null_terminated_string_bytes(&context, out).unwrap().len() > 4);

        Ok(())
    }

    // Titles pick their sound path off MEDIADEVICES; M_E_INVALID left them silent.
    #[futures_test::test]
    async fn test_get_system_property_media_devices() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        write_null_terminated_string_bytes(&mut context, id, b"MEDIADEVICES").unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), 0);
        let result = read_null_terminated_string_bytes(&context, out).unwrap();
        assert!(String::from_utf8(result).unwrap().contains("Yamaha_MA3"));

        Ok(())
    }

    // Arms `timer` per `ops` (a timeout to MC_knlSetTimer it, `None` to MC_knlUnsetTimer it), then
    // lets every timer that was set fall due. Returns the guest callbacks that ran.
    async fn fire_after(ops: &[Option<u32>]) -> Result<alloc::vec::Vec<u32>> {
        let system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        let timer = context.alloc_raw(4).unwrap();
        def_timer(&mut context, timer, 0x1234).await?;
        for op in ops {
            match op {
                Some(timeout) => set_timer(&mut context, timer, *timeout, 0, 0).await?,
                None => unset_timer(&mut context, timer).await?,
            }
        }
        for (_, _, callback) in core::mem::take(&mut context.timers) {
            callback.call(&mut context, Box::new([])).await?;
        }
        Ok(core::mem::take(&mut context.calls))
    }

    // Unset was a no-op, so each Unset + Set left one more timer chain running for good.
    #[futures_test::test]
    async fn test_unset_timer_cancels_a_pending_timer() -> Result<()> {
        assert_eq!(fire_after(&[Some(10), None]).await?, []);
        assert_eq!(fire_after(&[Some(10), None, Some(10)]).await?, [0x1234]);
        Ok(())
    }

    #[futures_test::test]
    async fn test_when_a_timer_may_keep_a_tick_alive() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut context = TestContext::with_system(system.clone());
        let timer = context.alloc_raw(4).unwrap();
        def_timer(&mut context, timer, 0x1234).await?;
        let pace_from = async |context: &mut TestContext, timeout| {
            set_timer(context, timer, timeout, 0, 0).await.unwrap();
            context.timers.last().unwrap().1
        };
        let tick = system.pacing().ticks();

        assert_eq!(pace_from(&mut context, 1).await, u64::MAX, "MC_knlSetTimer(1): as soon as you can");
        assert_eq!(pace_from(&mut context, 10).await, 0, "a 10ms period keeps the tick alive");
        let (_, _, callback) = context.timers.pop().unwrap();
        callback.call(&mut context, Box::new([])).await?;
        // Re-armed from its own callback — the usual loop. Only this tick's second fire is withheld.
        assert_eq!(pace_from(&mut context, 10).await, tick + 1);
        system.tick()?;
        assert_eq!(pace_from(&mut context, 10).await, 0);

        Ok(())
    }

    #[futures_test::test]
    async fn test_zero_size_memory_returns_null() -> Result<()> {
        let mut context = TestContext::new();

        assert_eq!(alloc(&mut context, 0).await.unwrap().0, 0);
        assert_eq!(calloc(&mut context, 0).await.unwrap().0, 0);
        assert_eq!(free(&mut context, wipi_types::wipic::WIPICIndirectPtr(0)).await.unwrap().0, 0);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_system_property_non_utf8() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        // EUC-KR "한글"
        write_null_terminated_string_bytes(&mut context, id, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), -9);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_resource_id_non_utf8() -> Result<()> {
        let mut context = TestContext::new();
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        assert_eq!(get_resource_id(&mut context, name, size).await.unwrap(), -12);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_resource_euc_kr_roundtrip() -> Result<()> {
        let data = [1u8, 2, 3, 4];
        let mut context = TestContext::new().with_resource("한글", &data);
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        let handle = get_resource_id(&mut context, name, size).await.unwrap();
        assert!(handle >= 0);

        let mut size_bytes = [0; 4];
        context.read_bytes(size, &mut size_bytes).unwrap();
        assert_eq!(u32::from_le_bytes(size_bytes), 4);

        let buf = context.alloc(4).unwrap();
        assert_eq!(get_resource(&mut context, handle, buf, 4).await.unwrap(), 0);

        let mut result = [0; 4];
        context.read_bytes(context.data_ptr(buf).unwrap(), &mut result).unwrap();
        assert_eq!(result, data);

        Ok(())
    }

    #[futures_test::test]
    async fn test_missing_resource_clears_size() -> Result<()> {
        let mut context = TestContext::new();
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, b"missing").unwrap();
        context.write_bytes(size, &[0xff; 4]).unwrap();

        assert_eq!(get_resource_id(&mut context, name, size).await.unwrap(), -12);
        let mut result = [0; 4];
        context.read_bytes(size, &mut result).unwrap();
        assert_eq!(u32::from_le_bytes(result), 0);

        Ok(())
    }

    /// `MC_knlGetResource` must refuse to write past a caller buffer that is too small.
    ///
    /// The sibling failure branch (`get_resource_id` -> -12) has had a test since it was
    /// written; this one had none — measured 2026-09-06 with a planted `panic!()` at the
    /// branch, which left `cargo test --all` at 157 passed / 0 failed while the same drill
    /// at the -12 branch killed `test_missing_resource_clears_size`.
    ///
    /// This is a HOST-side test on purpose. The guest route the obvious way — have the
    /// fixture call `wipic_sys::kernel::get_resource` with a short buffer and print the
    /// code — is what made the old -1 unobservable: that function is typed `-> WIPICError`,
    /// whose variants at the pinned rev are {1, 0, -9, -12, -18, -22, -25}, and `from_raw`
    /// is a `transmute`, so -1 was an invalid discriminant. That is why the code is now -18
    /// (`InsufficientBufferSize`); the reasoning lives at the branch itself.
    ///
    /// **-18 is the assertion that matters here.** It is not incidental: reverting the branch
    /// to -1 fails this test, which is the only machine tie holding the value inside the ABI
    /// vocabulary — nothing else in the workspace checks it.
    ///
    /// Both directions are asserted: a buffer one byte short fails, the exact size passes.
    /// Asserting only the failure would also pass if the function returned -18 always.
    #[futures_test::test]
    async fn test_resource_larger_than_buffer_is_rejected() -> Result<()> {
        const PAYLOAD: &[u8] = b"0123456789";

        let mut context = TestContext::new().with_resource("big.bin", PAYLOAD);
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();
        write_null_terminated_string_bytes(&mut context, name, b"big.bin").unwrap();

        let id = get_resource_id(&mut context, name, size).await.unwrap();
        assert!(id > 0, "setup: expected a handle, got {id}");
        let mut reported = [0; 4];
        context.read_bytes(size, &mut reported).unwrap();
        assert_eq!(u32::from_le_bytes(reported) as usize, PAYLOAD.len());

        let short = context.alloc(PAYLOAD.len() as u32 - 1).unwrap();
        assert_eq!(get_resource(&mut context, id, short, PAYLOAD.len() as u32 - 1).await.unwrap(), -18);

        let exact = context.alloc(PAYLOAD.len() as u32).unwrap();
        assert_eq!(get_resource(&mut context, id, exact, PAYLOAD.len() as u32).await.unwrap(), 0);
        let mut got = [0; PAYLOAD.len()];
        context.read_bytes(context.data_ptr(exact).unwrap(), &mut got).unwrap();
        assert_eq!(&got, PAYLOAD);

        Ok(())
    }

    // The two siblings below already returned -18 before `get_resource` was fixed to match them,
    // and precisely because they were already right, nothing asserted it: mutating either constant
    // broke no test. These pin the value the same way the resource test above does — as the NUMBER
    // the guest receives, not a constant name, because `M_E_SHORTBUF` has no definition anywhere in
    // this repo or the pinned runtime (it lives only in comments).
    #[futures_test::test]
    async fn test_system_property_larger_than_buffer_is_rejected() -> Result<()> {
        // "PHONEMODEL" -> "Emulator": 8 bytes plus the NUL, so 9 fits and 8 cannot.
        const VALUE: &[u8] = b"Emulator";

        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();
        write_null_terminated_string_bytes(&mut context, id, b"PHONEMODEL").unwrap();

        assert_eq!(get_system_property(&mut context, id, out, VALUE.len() as u32).await.unwrap(), -18);

        assert_eq!(get_system_property(&mut context, id, out, VALUE.len() as u32 + 1).await.unwrap(), 0);
        let got = read_null_terminated_string_bytes(&context, out).unwrap();
        assert_eq!(got, VALUE);

        Ok(())
    }

    #[futures_test::test]
    async fn test_program_name_larger_than_buffer_is_rejected() -> Result<()> {
        // `System::new`'s aid below is the string this writes, so its length drives the boundary.
        const AID: &[u8] = b"test-aid";

        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        let out = context.alloc_raw(16).unwrap();

        assert_eq!(get_program_name(&mut context, out, AID.len() as i32).await.unwrap(), -18);

        assert_eq!(get_program_name(&mut context, out, AID.len() as i32 + 1).await.unwrap(), 0);
        let got = read_null_terminated_string_bytes(&context, out).unwrap();
        assert_eq!(got, AID);

        Ok(())
    }
}
