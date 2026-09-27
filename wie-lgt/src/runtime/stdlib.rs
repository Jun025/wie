use alloc::{format, string::String, vec::Vec};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use core::cmp::min;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId, stdlib};
use wie_util::{ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};
use wie_wipi_c::api::kernel;

use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

pub fn register_stdlib_svc_handler(core: &mut ArmCore, system: &System) -> Result<()> {
    async fn handle_stdlib_svc(core: &mut ArmCore, system: &mut System, id: SvcId) -> Result<()> {
        let (_, lr) = core.read_pc_lr()?;

        match id.0 {
            x if x == StdlibSvcId::Unk2 as u32 => EmulatedFunction::call(&unk2, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Sprintf as u32 => EmulatedFunction::call(&sprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Vsprintf as u32 => EmulatedFunction::call(&vsprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atoi as u32 => EmulatedFunction::call(&atoi, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Rand as u32 => EmulatedFunction::call(&rand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Srand as u32 => EmulatedFunction::call(&srand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Strcpy as u32 => EmulatedFunction::call(&stdlib::strcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncpy as u32 => EmulatedFunction::call(&strncpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcat as u32 => EmulatedFunction::call(&strcat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncat as u32 => EmulatedFunction::call(&strncat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcmp as u32 => EmulatedFunction::call(&strcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk4 as u32 => EmulatedFunction::call(&unk4, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strstr as u32 => EmulatedFunction::call(&strstr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strlen as u32 => EmulatedFunction::call(&stdlib::strlen, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcpy as u32 => EmulatedFunction::call(&stdlib::memcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memmove as u32 => EmulatedFunction::call(&stdlib::memmove, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memset as u32 => EmulatedFunction::call(&stdlib::memset, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Time as u32 => EmulatedFunction::call(&time, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Localtime as u32 => EmulatedFunction::call(&localtime, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk3 as u32 => EmulatedFunction::call(&unk3, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Malloc as u32 => EmulatedFunction::call(&malloc, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Free as u32 => EmulatedFunction::call(&free, core, &mut ()).await?.write(core, lr),
            _ => Err(WieError::FatalError(format!("Unknown lgt stdlib import: {:#x}", id.0))),
        }
    }

    core.register_svc_handler(SVC_CATEGORY_STDLIB, handle_stdlib_svc, system)
}

async fn srand(_core: &mut ArmCore, system: &mut System, seed: u32) -> Result<()> {
    tracing::debug!("srand({seed:#x})");

    system.set_random_state(seed);
    Ok(())
}

async fn rand(_core: &mut ArmCore, system: &mut System) -> Result<u32> {
    tracing::debug!("rand()");

    let state = system.random_state();
    let state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    system.set_random_state(state);

    Ok((state >> 16) & 0x7fff)
}

#[allow(clippy::too_many_arguments)]
async fn sprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> Result<u32> {
    tracing::debug!("sprintf({ptr_dst:#x}, {ptr_format:#x}, {a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}, {a4:#x}, {a5:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let result = kernel::sprintf(core, &format, &[a0, a1, a2, a3, a4, a5])?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

// `ap` is the address of the first variadic word. Only as many words as the format can consume
// are read (two per conversion covers `%lld`), and reading stops at the first unmapped one.
async fn vsprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, ap: u32) -> Result<u32> {
    tracing::debug!("vsprintf({ptr_dst:#x}, {ptr_format:#x}, {ap:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let wanted = format.iter().filter(|&&x| x == b'%').count() * 2;
    let mut args = Vec::with_capacity(wanted);
    for i in 0..wanted as u32 {
        match read_generic::<u32, _>(core, ap + i * 4) {
            Ok(word) => args.push(word),
            Err(_) => break,
        }
    }
    let result = kernel::sprintf(core, &format, &args)?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

async fn strncpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncpy({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;

    let size_to_copy = min(size, src.len() as u32);
    let bytes = &src[..size_to_copy as usize];

    core.write_bytes(ptr_dst, bytes)?;

    Ok(())
}

async fn strcat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    tracing::debug!("strcat({ptr_dst:#x}, {ptr_src:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    let offset = dst.len();
    write_null_terminated_string_bytes(core, ptr_dst + offset as u32, &src)?;

    Ok(())
}

async fn strncat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncat({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    let src = &src[..min(size as usize, src.len())];
    write_null_terminated_string_bytes(core, ptr_dst + dst.len() as u32, src)?;

    Ok(())
}

async fn strcmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32) -> Result<u32> {
    tracing::debug!("strcmp({ptr_str1:#x}, {ptr_str2:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn time(core: &mut ArmCore, system: &mut System, ptr_time: u32) -> Result<u32> {
    let epoch_seconds = (system.platform().now().raw() / 1000) as u32;
    tracing::debug!("time({ptr_time:#x}) -> {epoch_seconds}");

    if ptr_time != 0 {
        write_generic(core, ptr_time, epoch_seconds)?;
    }

    Ok(epoch_seconds)
}

// TODO is this method better suit on wie_backend?
async fn localtime(core: &mut ArmCore, _: &mut (), ptr_time: u32) -> Result<u32> {
    tracing::debug!("localtime({ptr_time:#x})");

    // TODO we need static buffer
    let result = Allocator::alloc(core, 0x2c)?;
    let time: u32 = read_generic(core, ptr_time)?;

    // TODO kst only for now
    let kst = FixedOffset::east_opt(9 * 3600).unwrap();
    let dt: DateTime<FixedOffset> = kst.timestamp_opt(time as _, 0).unwrap();

    // TODO tm struct
    write_generic(core, result, dt.second() as u32)?;
    write_generic(core, result + 0x04, dt.minute() as u32)?;
    write_generic(core, result + 0x08, dt.hour() as u32)?;
    write_generic(core, result + 0x0c, dt.day() as u32)?;
    write_generic(core, result + 0x10, (dt.month() as u32) - 1)?; // months since January
    write_generic(core, result + 0x14, (dt.year() as u32) - 1900)?; // years since 1900
    write_generic(core, result + 0x18, dt.weekday().num_days_from_sunday() as u32)?; // days since Sunday
    write_generic(core, result + 0x1c, dt.ordinal() as u32)?; // days since January 1
    write_generic(core, result + 0x20, 0u32)?; // DST flag
    write_generic(core, result + 0x24, kst.local_minus_utc() as u32)?; // timezone offset in seconds
    write_generic(core, result + 0x28, 0u32)?; // timezone abbreviation ptr

    Ok(result)
}

async fn unk2(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk2({a0:#x})");

    // error exit?

    Ok(())
}

async fn unk3(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk3({a0:#x})");

    let _: () = core.run_function(a0, &[]).await?;

    Ok(())
}

async fn unk4(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("unk4({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(())
}

// The heap's free needs the size back, and C's free does not pass it: keep it in a word in front.
const MALLOC_HEADER: u32 = 4;

async fn malloc(core: &mut ArmCore, _: &mut (), size: u32) -> Result<u32> {
    tracing::debug!("malloc({size:#x})");

    let Some(total) = size.checked_add(MALLOC_HEADER) else {
        return Ok(0);
    };
    let Ok(block) = Allocator::alloc(core, total) else {
        // C's contract is a null return; the guest's operator new turns that into its own exception.
        return Ok(0);
    };
    write_generic(core, block, total)?;

    Ok(block + MALLOC_HEADER)
}

async fn free(core: &mut ArmCore, _: &mut (), ptr: u32) -> Result<()> {
    tracing::debug!("free({ptr:#x})");

    if ptr == 0 {
        return Ok(());
    }
    let block = ptr - MALLOC_HEADER;
    let total: u32 = read_generic(core, block)?;

    Allocator::free(core, block, total)
}

async fn strstr(core: &mut ArmCore, _: &mut (), ptr_haystack: u32, ptr_needle: u32) -> Result<u32> {
    tracing::debug!("strstr({ptr_haystack:#x}, {ptr_needle:#x})");

    let haystack = read_null_terminated_string_bytes(core, ptr_haystack)?;
    let needle = read_null_terminated_string_bytes(core, ptr_needle)?;
    let position = if needle.is_empty() {
        Some(0)
    } else {
        haystack.windows(needle.len()).position(|window| window == needle)
    };

    Ok(position.map_or(0, |position| ptr_haystack + position as u32))
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use futures::FutureExt;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteRead, ByteWrite, Result};

    use super::{free, malloc, rand, register_stdlib_svc_handler, srand};
    use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

    #[test]
    fn random_state_is_shared_by_system_clones_and_process_local() -> Result<()> {
        let mut first = ArmCore::new(false, None)?;
        let mut second = ArmCore::new(false, None)?;
        let mut first_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut second_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut first_system_clone = first_system.clone();

        srand(&mut first, &mut first_system_clone, 1).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 16_838);
        assert_eq!(first_system.random_state(), 1_103_527_590);

        assert_eq!(rand(&mut second, &mut second_system).now_or_never().unwrap()?, 16_838);
        srand(&mut second, &mut second_system, 7).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 5_758);

        Ok(())
    }

    /// Import `0x415` reaches `memmove` through the stdlib table, and an overlapping copy comes out right.
    ///
    /// The row existed before the base swap (`d70b93f8`) and #161 dropped it without a trace: the
    /// implementation in `wie-core-arm` stayed, only the dispatch line went, and nothing failed until
    /// (LGT)알바타이쿤2 and 데몬헌터 died on `Unknown lgt stdlib import: 0x415`. A test that calls
    /// `memmove` directly would have stayed green through that, so this one goes through the SVC stub.
    /// The overlap is the part that separates memmove from memcpy: dst = src + 2 over the same buffer.
    #[test]
    fn stdlib_import_0x415_is_memmove_through_the_svc_table() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(false, None)?;
            Allocator::init(&mut core)?;
            let stack = Allocator::alloc(&mut core, 0x1000)?;
            let mut context = core.save_context();
            context.sp = stack + 0x1000;
            core.restore_context(&context);
            register_stdlib_svc_handler(&mut core, &system_clone)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Memmove)?;
            assert_eq!(StdlibSvcId::Memmove as u32, 0x415);

            let buffer = Allocator::alloc(&mut core, 16)?;
            core.write_bytes(buffer, b"abcdefgh")?;
            let _: () = core.run_function(stub, &[buffer + 2, buffer, 6]).await?;

            let mut out = [0u8; 8];
            core.read_bytes(buffer, &mut out)?;
            assert_eq!(&out, b"ababcdef");

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// Imports 0x426/0x428 are malloc/free (b7699c10dfd1 died on `Unknown lgt stdlib import: 0x426`
    /// at boot), 0x3f9 is vsprintf (2dbde9acca99, on its first keys) and 0x408 strncat (4fdbd64c9fbd,
    /// in a key handler). Goes through the SVC table like the memmove test above, then checks the block is
    /// really handed back: free must find its own size again.
    #[test]
    fn stdlib_imports_0x3f9_0x408_0x426_0x428_are_vsprintf_strncat_malloc_free_through_the_svc_table() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(false, None)?;
            Allocator::init(&mut core)?;
            let stack = Allocator::alloc(&mut core, 0x1000)?;
            let mut context = core.save_context();
            context.sp = stack + 0x1000;
            core.restore_context(&context);
            register_stdlib_svc_handler(&mut core, &system_clone)?;
            let malloc_stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Malloc)?;
            let free_stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Free)?;
            assert_eq!((StdlibSvcId::Malloc as u32, StdlibSvcId::Free as u32), (0x426, 0x428));

            // 0x3f9 = vsprintf: the arguments come from memory at `ap`, not from registers.
            let vsprintf_stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Vsprintf)?;
            assert_eq!(StdlibSvcId::Vsprintf as u32, 0x3f9);
            let text = Allocator::alloc(&mut core, 64)?;
            core.write_bytes(text, b"%s/%d.Dat\0save\0")?;
            let ap = Allocator::alloc(&mut core, 8)?;
            core.write_bytes(ap, &[(text + 10).to_le_bytes(), 7u32.to_le_bytes()].concat())?;
            let out = Allocator::alloc(&mut core, 32)?;
            let written: u32 = core.run_function(vsprintf_stub, &[out, text, ap]).await?;
            let mut formatted = [0u8; 11];
            core.read_bytes(out, &mut formatted)?;
            assert_eq!((written, &formatted), (10, b"save/7.Dat\0"));

            // 0x408 = strncat: at most n bytes of src, then a terminator.
            let strncat_stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Strncat)?;
            assert_eq!(StdlibSvcId::Strncat as u32, 0x408);
            core.write_bytes(out, b"ab\0")?;
            core.write_bytes(text, b"cdef\0")?;
            let _: () = core.run_function(strncat_stub, &[out, text, 2]).await?;
            let mut joined = [0u8; 6];
            core.read_bytes(out, &mut joined)?;
            assert_eq!(&joined[..5], b"abcd\0");

            let ptr: u32 = core.run_function(malloc_stub, &[24]).await?;
            assert_ne!(ptr, 0);
            core.write_bytes(ptr, &[0xab; 24])?;
            assert!(Allocator::is_allocated(&core, ptr - 4, 28)?);
            let _: () = core.run_function(free_stub, &[ptr]).await?;
            assert!(!Allocator::is_allocated(&core, ptr - 4, 28)?);
            let _: () = core.run_function(free_stub, &[0]).await?;

            // Direct calls as well, for the null contract on an impossible size.
            assert_eq!(malloc(&mut core, &mut (), u32::MAX).now_or_never().unwrap()?, 0);
            free(&mut core, &mut (), 0).now_or_never().unwrap()?;

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
