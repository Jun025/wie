use alloc::{format, string::String, vec::Vec};
use core::{
    cmp::{Ordering, min},
    iter::from_fn,
};
use jiff::{Timestamp, tz::Offset};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId, stdlib};
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};
use wie_wipi_c::api::kernel;

use crate::runtime::{SVC_CATEGORY_STDLIB, globals::STRTOK_STATE, svc_ids::StdlibSvcId};

pub fn register_stdlib_svc_handler(core: &mut ArmCore, system: &System) -> Result<()> {
    async fn handle_stdlib_svc(core: &mut ArmCore, system: &mut System, id: SvcId) -> Result<()> {
        let (_, lr) = core.read_pc_lr()?;

        match id.0 {
            x if x == StdlibSvcId::Unk2 as u32 => EmulatedFunction::call(&unk2, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Sprintf as u32 => EmulatedFunction::call(&sprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Vsprintf as u32 => EmulatedFunction::call(&vsprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atoi as u32 => EmulatedFunction::call(&atoi, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strtol as u32 => EmulatedFunction::call(&strtol, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Rand as u32 => EmulatedFunction::call(&rand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Srand as u32 => EmulatedFunction::call(&srand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Strcpy as u32 => EmulatedFunction::call(&stdlib::strcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncpy as u32 => EmulatedFunction::call(&strncpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcat as u32 => EmulatedFunction::call(&strcat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncat as u32 => EmulatedFunction::call(&strncat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcmp as u32 => EmulatedFunction::call(&strcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncmp as u32 => EmulatedFunction::call(&strncmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strchr as u32 => EmulatedFunction::call(&strchr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strstr as u32 => EmulatedFunction::call(&strstr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strlen as u32 => EmulatedFunction::call(&stdlib::strlen, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strtok as u32 => EmulatedFunction::call(&strtok, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcpy as u32 => EmulatedFunction::call(&stdlib::memcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memmove as u32 => EmulatedFunction::call(&stdlib::memmove, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcmp as u32 => EmulatedFunction::call(&memcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memchr as u32 => EmulatedFunction::call(&memchr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memset as u32 => EmulatedFunction::call(&stdlib::memset, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Time as u32 => EmulatedFunction::call(&time, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Localtime as u32 => EmulatedFunction::call(&localtime, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atexit as u32 => EmulatedFunction::call(&atexit, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Abort as u32 => EmulatedFunction::call(&abort, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Malloc as u32 => EmulatedFunction::call(&malloc, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Realloc as u32 => EmulatedFunction::call(&realloc, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Free as u32 => EmulatedFunction::call(&free, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk450 as u32 => EmulatedFunction::call(&unk450, core, &mut ()).await?.write(core, lr),
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
    let result = kernel::sprintf(core, &format, [a0, a1, a2, a3, a4, a5].into_iter().map(Ok))?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

// `ap` is the address of the first variadic word; a word is read only when the format consumes it.
async fn vsprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, mut ptr_args: u32) -> Result<u32> {
    tracing::debug!("vsprintf({ptr_dst:#x}, {ptr_format:#x}, {ptr_args:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let args = from_fn(|| {
        let value = read_generic(core, ptr_args);
        ptr_args = ptr_args.wrapping_add(4);
        Some(value)
    });
    let result = kernel::sprintf(core, &format, args)?;
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

    // src need not be NUL-terminated within n bytes, so read no further than n.
    let mut src = Vec::new();
    for i in 0..size {
        let byte: u8 = read_generic(core, ptr_src + i)?;
        if byte == 0 {
            break;
        }
        src.push(byte);
    }
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    write_null_terminated_string_bytes(core, ptr_dst + dst.len() as u32, &src)?;

    Ok(())
}

async fn strcmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32) -> Result<u32> {
    tracing::debug!("strcmp({ptr_str1:#x}, {ptr_str2:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

const COPY_CHUNK: usize = 4096;

async fn strtok(core: &mut ArmCore, _: &mut (), ptr_str: u32, ptr_delimiters: u32) -> Result<u32> {
    let mut cursor = if ptr_str == 0 { read_generic(core, STRTOK_STATE)? } else { ptr_str };
    let delimiters = read_null_terminated_string_bytes(core, ptr_delimiters)?;
    let mut token = 0;
    while cursor != 0 {
        let byte: u8 = read_generic(core, cursor)?;
        if byte == 0 {
            cursor = 0;
            break;
        }
        if delimiters.contains(&byte) {
            if token != 0 {
                write_generic(core, cursor, 0u8)?;
                cursor = cursor.wrapping_add(1);
                break;
            }
        } else if token == 0 {
            token = cursor;
        }
        cursor = cursor.wrapping_add(1);
    }
    write_generic(core, STRTOK_STATE, cursor)?;
    Ok(token)
}

async fn memcmp(core: &mut ArmCore, _: &mut (), ptr_first: u32, ptr_second: u32, len: u32) -> Result<u32> {
    let mut first = [0; COPY_CHUNK];
    let mut second = [0; COPY_CHUNK];
    let mut offset = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.read_bytes(ptr_first.wrapping_add(offset), &mut first[..chunk])?;
        core.read_bytes(ptr_second.wrapping_add(offset), &mut second[..chunk])?;
        let result = first[..chunk].cmp(&second[..chunk]);
        if result != Ordering::Equal {
            return Ok(result as u32);
        }
        offset += chunk as u32;
    }
    Ok(0)
}

async fn memchr(core: &mut ArmCore, _: &mut (), ptr: u32, value: u32, len: u32) -> Result<u32> {
    let mut buf = [0; COPY_CHUNK];
    let mut offset = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.read_bytes(ptr.wrapping_add(offset), &mut buf[..chunk])?;
        if let Some(index) = buf[..chunk].iter().position(|byte| *byte == value as u8) {
            return Ok(ptr.wrapping_add(offset).wrapping_add(index as u32));
        }
        offset += chunk as u32;
    }
    Ok(0)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn strtol(core: &mut ArmCore, _: &mut (), ptr_str: u32, ptr_end: u32, base: u32) -> Result<u32> {
    tracing::debug!("strtol({ptr_str:#x}, {ptr_end:#x}, {base})");

    let s = read_null_terminated_string_bytes(core, ptr_str)?;
    let (value, consumed) = parse_long(&s, base);
    if ptr_end != 0 {
        write_generic(core, ptr_end, ptr_str + consumed as u32)?;
    }

    Ok(value as u32)
}

/// C `strtol` over bytes: returns the value and how many bytes it consumed (0 when no digits).
fn parse_long(s: &[u8], base: u32) -> (i32, usize) {
    let mut i = s.iter().take_while(|&&b| b == b' ' || (b'\t'..=b'\r').contains(&b)).count();
    let negative = s.get(i) == Some(&b'-');
    if matches!(s.get(i), Some(b'-' | b'+')) {
        i += 1;
    }
    let hex_prefix = s.get(i) == Some(&b'0') && matches!(s.get(i + 1), Some(b'x' | b'X')) && s.get(i + 2).is_some_and(|b| b.is_ascii_hexdigit());
    let base = match base {
        0 if hex_prefix => 16,
        0 if s.get(i) == Some(&b'0') => 8,
        0 => 10,
        x => x,
    };
    if base == 16 && hex_prefix {
        i += 2;
    }
    if !(2..=36).contains(&base) {
        return (0, 0);
    }

    let start = i;
    let mut magnitude: i64 = 0;
    while let Some(digit) = s.get(i).and_then(|&b| (b as char).to_digit(base)) {
        magnitude = (magnitude * base as i64 + digit as i64).min(1 << 32); // saturate; clamped below
        i += 1;
    }
    if i == start {
        return (0, 0);
    }
    let value = if negative { -magnitude } else { magnitude };

    (value.clamp(i32::MIN as i64, i32::MAX as i64) as i32, i)
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
    let kst = Offset::constant(9);
    let dt = kst.to_datetime(Timestamp::from_second(time.into()).unwrap());

    // TODO tm struct
    write_generic(core, result, dt.second() as u32)?;
    write_generic(core, result + 0x04, dt.minute() as u32)?;
    write_generic(core, result + 0x08, dt.hour() as u32)?;
    write_generic(core, result + 0x0c, dt.day() as u32)?;
    write_generic(core, result + 0x10, (dt.month() as u32) - 1)?; // months since January
    write_generic(core, result + 0x14, (dt.year() as u32) - 1900)?; // years since 1900
    write_generic(core, result + 0x18, dt.weekday().to_sunday_zero_offset() as u32)?; // days since Sunday
    write_generic(core, result + 0x1c, dt.day_of_year() as u32)?; // days since January 1
    write_generic(core, result + 0x20, 0u32)?; // DST flag
    write_generic(core, result + 0x24, kst.seconds() as u32)?; // timezone offset in seconds
    write_generic(core, result + 0x28, 0u32)?; // timezone abbreviation ptr

    Ok(result)
}

async fn unk2(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk2({a0:#x})");

    // error exit?

    Ok(())
}

// upstream #1502. Until the 2026-11-23 merge this slot (then `unk3`, unchanged base code) ran the
// callback at once — at registration.
async fn atexit(_core: &mut ArmCore, _: &mut (), callback: u32) -> Result<u32> {
    tracing::warn!("stub atexit({callback:#x})");

    Ok(0)
}

async fn abort(_core: &mut ArmCore, _: &mut ()) -> Result<()> {
    Err(WieError::FatalError("Guest called abort()".into()))
}

async fn unk450(_core: &mut ArmCore, _: &mut ()) -> Result<f64> {
    tracing::warn!("stub lgt stdlib unk450");
    Ok(0.0)
}

async fn strncmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32, size: u32) -> Result<u32> {
    tracing::debug!("strncmp({ptr_str1:#x}, {ptr_str2:#x}, {size:#x})");

    // At most `size` bytes of each, as C reads them: a buffer with no NUL inside `size` is legal.
    let read = |ptr: u32| -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut byte = [0];
        while bytes.len() < size as usize {
            core.read_bytes(ptr + bytes.len() as u32, &mut byte)?;
            if byte[0] == 0 {
                break;
            }
            bytes.push(byte[0]);
        }
        Ok(bytes)
    };

    Ok(read(ptr_str1)?.cmp(&read(ptr_str2)?) as u32)
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

// upstream #1481's `realloc`, on our block header (`MALLOC_HEADER`): a failed allocation answers NULL
// and leaves the old block alone, as C does.
async fn realloc(core: &mut ArmCore, context: &mut (), ptr: u32, size: u32) -> Result<u32> {
    if size == 0 {
        free(core, context, ptr).await?;
        return Ok(0);
    }
    let new_ptr = malloc(core, context, size).await?;
    if ptr != 0 && new_ptr != 0 {
        let old_total: u32 = read_generic(core, ptr - MALLOC_HEADER)?;
        stdlib::memcpy(core, context, new_ptr, ptr, size.min(old_total - MALLOC_HEADER)).await?;
        free(core, context, ptr).await?;
    }
    Ok(new_ptr)
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

async fn strchr(core: &mut ArmCore, _: &mut (), ptr_str: u32, c: u32) -> Result<u32> {
    tracing::debug!("strchr({ptr_str:#x}, {c:#x})");

    // C: `c` is converted to char, and the terminator itself can be found.
    let c = c as u8;
    let s = read_null_terminated_string_bytes(core, ptr_str)?;
    let position = if c == 0 { Some(s.len()) } else { s.iter().position(|&b| b == c) };

    Ok(position.map_or(0, |position| ptr_str + position as u32))
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use futures::FutureExt;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

    use super::{STRTOK_STATE, free, localtime, malloc, parse_long, rand, register_stdlib_svc_handler, srand};
    use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

    /// `localtime` fills the guest's `tm` in KST. The expected fields are calendar facts, not this
    /// code's output: the date library under it was swapped (chrono → jiff, with RustJava 0.5.1).
    #[test]
    fn localtime_fills_tm_in_kst() -> Result<()> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        Allocator::init(&mut core)?;
        let ptr_time = Allocator::alloc(&mut core, 4)?;

        // sec, min, hour, mday, mon (0-based), year - 1900, wday (0 = Sunday), day of year (1-based, as before), dst, offset
        for (time, expected) in [
            (0u32, [0u32, 0, 9, 1, 0, 70, 4, 1, 0, 32_400]),             // 1970-01-01 09:00:00 KST, a Thursday
            (1_000_000_000, [40, 46, 10, 9, 8, 101, 0, 252, 0, 32_400]), // 2001-09-09 10:46:40 KST, a Sunday
        ] {
            write_generic(&mut core, ptr_time, time)?;
            let tm = localtime(&mut core, &mut (), ptr_time).now_or_never().unwrap()?;
            for (i, want) in expected.into_iter().enumerate() {
                let got: u32 = read_generic(&core, tm + 4 * i as u32)?;
                assert_eq!(got, want, "time {time}, tm word {i}");
            }
        }

        Ok(())
    }

    #[test]
    fn random_state_is_shared_by_system_clones_and_process_local() -> Result<()> {
        let mut first = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        let mut second = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
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
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
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
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
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
            // src need not be terminated within n: "gh" ends a mapped page, the next one is not mapped.
            core.map(0x6000_0000, 0x1_0000)?; // one 64 KiB page
            core.write_bytes(0x6000_fffe, b"gh")?;
            let _: () = core.run_function(strncat_stub, &[out, 0x6000_fffe, 2]).await?;
            core.read_bytes(out, &mut joined)?;
            assert_eq!(&joined, b"abcdgh");

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

    /// Import 0x3ff is strtol: acc4215b7ec0 died on `Unknown lgt stdlib import: 0x3ff` in a menu, at
    /// `f("0xFFAE08", &end, 16)` (see `StdlibSvcId::Strtol`). Goes through the SVC table, like the tests above.
    #[test]
    fn stdlib_import_0x3ff_is_strtol_through_the_svc_table() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
            Allocator::init(&mut core)?;
            let stack = Allocator::alloc(&mut core, 0x1000)?;
            let mut context = core.save_context();
            context.sp = stack + 0x1000;
            core.restore_context(&context);
            register_stdlib_svc_handler(&mut core, &system_clone)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Strtol)?;
            assert_eq!(StdlibSvcId::Strtol as u32, 0x3ff);

            let text = Allocator::alloc(&mut core, 16)?;
            core.write_bytes(text, b"0xFFAE08|\0")?;
            let end = Allocator::alloc(&mut core, 4)?;
            let value: u32 = core.run_function(stub, &[text, end, 16]).await?;
            let mut end_ptr = [0u8; 4];
            core.read_bytes(end, &mut end_ptr)?;
            assert_eq!((value, u32::from_le_bytes(end_ptr)), (0xffae08, text + 8));

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// Import 0x40a is strncmp: caf9d76ffd13 compares the `PHONENUMBER` it saved at login with the
    /// handset's, 12 bytes, and a no-op here left the first pointer in r0 — «인증에 실패» on every
    /// launch after the first (see `StdlibSvcId::Strncmp`). Goes through the SVC table.
    #[test]
    fn stdlib_import_0x40a_is_strncmp_through_the_svc_table() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
            Allocator::init(&mut core)?;
            let stack = Allocator::alloc(&mut core, 0x1000)?;
            let mut context = core.save_context();
            context.sp = stack + 0x1000;
            core.restore_context(&context);
            register_stdlib_svc_handler(&mut core, &system_clone)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Strncmp)?;
            assert_eq!(StdlibSvcId::Strncmp as u32, 0x40a);

            let a = Allocator::alloc(&mut core, 12)?;
            let b = Allocator::alloc(&mut core, 12)?;
            core.write_bytes(a, b"01000000000\0")?;
            core.write_bytes(b, b"01000000000\0")?;
            let same: u32 = core.run_function(stub, &[a, b, 12]).await?;
            core.write_bytes(b, b"01000000001\0")?;
            let less: u32 = core.run_function(stub, &[a, b, 12]).await?;
            let prefix: u32 = core.run_function(stub, &[a, b, 10]).await?;
            assert_eq!((same, less as i32, prefix), (0, -1, 0));

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// Import 0x40b is strchr: 01f05f8231f4 died on `Unknown lgt stdlib import: 0x40b` at its first
    /// pitch, finding the extension of `"ui/slt_pitch.raw"` (see `StdlibSvcId::Strchr`).
    #[test]
    fn stdlib_import_0x40b_is_strchr_through_the_svc_table() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(wie_backend::Options {
                enable_gdbserver: false,
                aot: None,
                profile: None,
            })?;
            Allocator::init(&mut core)?;
            let stack = Allocator::alloc(&mut core, 0x1000)?;
            let mut context = core.save_context();
            context.sp = stack + 0x1000;
            core.restore_context(&context);
            register_stdlib_svc_handler(&mut core, &system_clone)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_STDLIB, StdlibSvcId::Strchr)?;
            assert_eq!(StdlibSvcId::Strchr as u32, 0x40b);

            let text = Allocator::alloc(&mut core, 16)?;
            core.write_bytes(text, b"a/b.c.raw\0")?;
            let mut got = [0; 3];
            for (found, c) in got.iter_mut().zip([b'.', b'x', 0]) {
                *found = core.run_function::<u32>(stub, &[text, c as u32]).await?;
            }
            assert_eq!(got, [text + 3, 0, text + 9]);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    #[test]
    fn parse_long_follows_c_strtol() {
        assert_eq!(parse_long(b"0xFFAE08", 16), (0xffae08, 8));
        assert_eq!(parse_long(b"FFFFFF", 16), (0xffffff, 6));
        assert_eq!(parse_long(b"  -42abc", 10), (-42, 5));
        assert_eq!(parse_long(b"0x", 16), (0, 1)); // "0" parsed, "x" left
        assert_eq!(parse_long(b"017", 0), (0o17, 3));
        assert_eq!(parse_long(b"0x1f", 0), (0x1f, 4));
        assert_eq!(parse_long(b"zz", 10), (0, 0));
        assert_eq!(parse_long(b"99999999999", 10), (i32::MAX, 11));
        assert_eq!(parse_long(b"-99999999999", 10), (i32::MIN, 12));
    }

    // ── upstream's tests for the slots the 2026-11-23 merge brought in (strtok · memcmp · memchr ·
    // realloc · atexit · abort) and for its lazy `vsprintf` argument reads. ──

    fn stdlib_core() -> Result<ArmCore> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        Allocator::init(&mut core)?;
        core.map(0x10000, 0x10000)?;
        let mut context = core.save_context();
        context.sp = 0x18000;
        core.restore_context(&context);
        let system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        register_stdlib_svc_handler(&mut core, &system)?;
        Ok(core)
    }

    #[test]
    fn tokenization_resumes_in_guest_memory_and_accepts_changing_delimiters() -> Result<()> {
        let mut core = stdlib_core()?;
        let tokenize = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x413u32)?;
        core.write_bytes(0x10000, b",one;;two:three,,\0")?;
        core.write_bytes(0x11000, b",;\0;:\0,\0\0")?;
        assert_eq!(core.run_function::<u32>(tokenize, &[0x10000, 0x11000]).now_or_never().unwrap()?, 0x10001);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10001)?, b"one");
        assert_eq!(read_generic::<u32, _>(&core, STRTOK_STATE)?, 0x10005);

        let mut resumed = core.clone();
        assert_eq!(resumed.run_function::<u32>(tokenize, &[0, 0x11003]).now_or_never().unwrap()?, 0x10006);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10006)?, b"two");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0x1000a);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x1000a)?, b"three");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0);
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0);

        core.write_bytes(0x10000, b"whole,string\0")?;
        assert_eq!(core.run_function::<u32>(tokenize, &[0x10000, 0x11008]).now_or_never().unwrap()?, 0x10000);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"whole,string");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11008]).now_or_never().unwrap()?, 0);
        Ok(())
    }

    #[test]
    fn variadic_formatting_reads_only_consumed_guest_arguments() -> Result<()> {
        let mut core = stdlib_core()?;
        let format = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x3f9u32)?;
        core.write_bytes(0x11000, b"%u %u %u %u %u %u %u %u\0")?;
        for index in 0..8 {
            write_generic(&mut core, 0x1ffe0 + index * 4, index + 1)?;
        }
        assert_eq!(
            core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1ffe0]).now_or_never().unwrap()?,
            15
        );
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"1 2 3 4 5 6 7 8");

        core.write_bytes(0x11000, b"literal %%\0")?;
        assert_eq!(core.run_function::<u32>(format, &[0x10000, 0x11000, 0x20000]).now_or_never().unwrap()?, 9);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"literal %");

        core.write_bytes(0x11000, b"%s:%llx\0")?;
        core.write_bytes(0x12000, b"value\0")?;
        write_generic(&mut core, 0x1fff4, 0x12000u32)?;
        write_generic(&mut core, 0x1fff8, 0x89abcdefu32)?;
        write_generic(&mut core, 0x1fffc, 0x01234567u32)?;
        core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1fff4]).now_or_never().unwrap()?;
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"value:123456789abcdef");

        core.write_bytes(0x11000, b"%u %u\0")?;
        assert!(matches!(
            core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1fffc]).now_or_never().unwrap(),
            Err(WieError::InvalidMemoryAccess(0x20000))
        ));
        Ok(())
    }

    #[test]
    fn memcmp_and_memchr_imports_scan_guest_bytes_across_chunks() -> Result<()> {
        let mut core = stdlib_core()?;
        core.map(0x20000, 0x10000)?;
        let source = (0..5001).map(|index| (index % 251 + 1) as u8).collect::<alloc::vec::Vec<_>>();
        core.write_bytes(0x10000, &source)?;
        core.write_bytes(0x20000, &source)?;

        let compare = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x416u32)?;
        assert_eq!(core.run_function::<u32>(compare, &[0x10000, 0x20000, 5001]).now_or_never().unwrap()?, 0);
        core.write_bytes(0x11000, b"\0\x80")?;
        core.write_bytes(0x21000, b"\0\x7f")?;
        for (first, second, sign) in [(0x10000, 0x20000, 1), (0x20000, 0x10000, -1)] {
            let result = core.run_function::<u32>(compare, &[first, second, 5001]).now_or_never().unwrap()? as i32;
            assert_eq!(result.signum(), sign);
        }
        assert_eq!(core.run_function::<u32>(compare, &[0x30000, 0x30000, 0]).now_or_never().unwrap()?, 0);

        let find = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x417u32)?;
        core.write_bytes(0x2ec78, &[1; 5000])?;
        core.write_bytes(0x2fffe, b"\0\x80")?;
        for (value, len, expected) in [(0x180, 5000, 0x2ffff), (0, 5000, 0x2fffe), (1, 5000, 0x2ec78), (0x80, 4999, 0)] {
            assert_eq!(core.run_function::<u32>(find, &[0x2ec78, value, len]).now_or_never().unwrap()?, expected);
        }
        assert_eq!(core.run_function::<u32>(find, &[0x30000, 0, 0]).now_or_never().unwrap()?, 0);
        Ok(())
    }

    /// `realloc` (0x427) on our block header: the bytes move, the old block goes back, and a failed
    /// allocation answers NULL with the old block untouched — `malloc`'s NULL contract, which upstream's
    /// own `realloc` does not have (it fails the run). `atexit` (0x424) registers without running the
    /// callback; `abort` (0x425) ends the run.
    #[test]
    fn realloc_atexit_and_abort_imports_answer() -> Result<()> {
        let mut core = stdlib_core()?;
        let allocate = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x426u32)?;
        let resize = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x427u32)?;
        let release = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x428u32)?;

        let mut ptr = core.run_function::<u32>(allocate, &[16]).now_or_never().unwrap()?;
        core.write_bytes(ptr, b"0123456789abcdef")?;
        for size in [5000, 8] {
            ptr = core.run_function::<u32>(resize, &[ptr, size]).now_or_never().unwrap()?;
            assert_ne!(ptr, 0);
            let mut bytes = [0; 8];
            core.read_bytes(ptr, &mut bytes)?;
            assert_eq!(&bytes, b"01234567");
        }
        assert_eq!(core.run_function::<u32>(resize, &[ptr, 0x8000_0000]).now_or_never().unwrap()?, 0);
        let mut bytes = [0; 8];
        core.read_bytes(ptr, &mut bytes)?;
        assert_eq!(&bytes, b"01234567", "a failed realloc leaves the old block");
        assert_eq!(core.run_function::<u32>(resize, &[ptr, 0]).now_or_never().unwrap()?, 0);
        let fresh = core.run_function::<u32>(resize, &[0, 32]).now_or_never().unwrap()?;
        assert_ne!(fresh, 0);
        core.run_function::<u32>(release, &[fresh]).now_or_never().unwrap()?;

        // 0x30000 is unmapped: running the callback would fault.
        let register = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x424u32)?;
        assert_eq!(core.run_function::<u32>(register, &[0x30001]).now_or_never().unwrap()?, 0);

        let abort = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x425u32)?;
        assert!(matches!(
            core.run_function::<u32>(abort, &[]).now_or_never().unwrap(),
            Err(WieError::FatalError(message)) if message == "Guest called abort()"
        ));
        Ok(())
    }
}
