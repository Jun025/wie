//! Shadow comparison — the check a generic hook has to pass to stay in `binary_patches.toml`.
//!
//! Off unless a host calls [`enable`] (`wie_validate` does under `WIE_HOOK_MODE=shadow`). Then, at
//! every hook fire: put the two displaced bytes back and run the title's OWN code on the same
//! registers and memory up to where the hook would resume it; record what that left; undo it; run
//! the hook to the same point; compare. A hook that hands the call to the title's own routine is
//! followed through it, so that path is compared too (and counted as `own`).
//!
//! Compared, per fire:
//!   - memory: the destination range with 32 bytes either side (`mem_mismatch`). For the library
//!     kinds the 128 bytes below SP are left out — the routine's own pushes, dead on return.
//!   - what the code hands back (`result_mismatch`): `r0` for the library kinds; the three frame
//!     slots for `inline_copy`; the three loop registers for `reg_inline_copy`.
//!   - what it must not touch (`callee_saved_mismatch`): `r4`-`r11` and `sp` for the library kinds,
//!     `r8`-`r12` and `sp` for the loops.
//!
//! NOT compared: caller-saved scratch (`r1`-`r3`, `ip`, flags) after a library call, and the loop
//! body's temporaries after a loop — the title's code and the engine leave different values there.
//!
//! `skipped` fires were not compared (range unreadable, or over 8 MiB); `own_fault` is the title's
//! own code not reaching the resume point, and `hook_fault` is the hook's side not reaching it — the
//! worst a hook can do, and a mismatch like the three above. A run proves nothing about a site that
//! never fired: read `fires` before `mismatch`. Sweep a corpus with `scripts/hook-shadow-sweep.sh`.
use alloc::{format, string::String, vec, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

use spin::Mutex;
use wie_util::{ByteRead, ByteWrite, Result};

use super::{HookKind, decode_exit_b, dispatch};
use crate::{
    ArmCore,
    engine::{ArmRegister, EngineStopReason},
    function::JumpTo,
    stdlib,
};

const KINDS: [&str; 6] = ["memcpy", "memset", "strcpy", "strlen", "inline_copy", "reg_inline_copy"];
const STATS: [&str; 9] = [
    "fires",
    "compared",
    "mem_mismatch",
    "result_mismatch",
    "callee_saved_mismatch",
    "own",
    "skipped",
    "own_fault",
    "hook_fault",
];
const FIRES: usize = 0;
const COMPARED: usize = 1;
const MEM: usize = 2;
const RESULT: usize = 3;
const CALLEE: usize = 4;
const OWN: usize = 5;
const SKIPPED: usize = 6;
const OWN_FAULT: usize = 7;
const HOOK_FAULT: usize = 8;

static ENABLED: AtomicBool = AtomicBool::new(false);
static COUNTS: [[AtomicU64; 9]; 6] = [const { [const { AtomicU64::new(0) }; 9] }; 6];
static NOTES: Mutex<Vec<String>> = Mutex::new(Vec::new());

const MARGIN: u32 = 32;
const MAX_LEN: u32 = 8 << 20;
const BUDGET: u32 = 400_000_000;

pub fn enable() {
    ENABLED.store(true, Relaxed);
}

pub(super) fn enabled() -> bool {
    ENABLED.load(Relaxed)
}

/// The counters as one JSON object: `{"<kind>":{"fires":…,…},…,"notes":[…]}`. Kinds that never
/// fired are left out; `notes` holds the first 60 distinct mismatch lines (site, length, where).
pub fn report() -> String {
    let mut fields = Vec::new();
    for (name, counts) in KINDS.iter().zip(&COUNTS) {
        if counts[FIRES].load(Relaxed) != 0 {
            let stats = STATS.iter().zip(counts).map(|(stat, n)| format!("\"{stat}\":{}", n.load(Relaxed)));
            fields.push(format!("\"{name}\":{{{}}}", stats.collect::<Vec<_>>().join(",")));
        }
    }
    let notes = NOTES.lock().iter().map(|note| format!("\"{note}\"")).collect::<Vec<_>>();
    fields.push(format!("\"notes\":[{}]", notes.join(",")));
    format!("{{{}}}", fields.join(","))
}

fn note(line: String) {
    let mut notes = NOTES.lock();
    if notes.len() < 60 && !notes.contains(&line) {
        notes.push(line);
    }
}

fn read(core: &ArmCore, addr: u32, len: usize) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; len];
    core.read_bytes(addr, &mut buf).ok().map(|_| buf)
}

/// Runs the guest from where it stands until it reaches `end`.
fn run_to(core: &mut ArmCore, end: u32) -> bool {
    matches!(core.inner.lock().engine.run(end, BUDGET), Ok(r) if matches!(r.stop_reason, EngineStopReason::End))
}

/// What a run left behind: registers, the destination window, and (`inline_copy`) the frame slots.
type State = ([u32; 17], Option<Vec<u8>>, Option<Vec<u8>>);

pub(super) async fn fire(core: &mut ArmCore, hook_pc: u32, kind: HookKind, displaced: [u8; 2], lr: u32) -> Result<JumpTo> {
    let site = hook_pc & !1;
    let entry = core.save_context();
    let reg = |r: ArmRegister| core.inner.lock().engine.reg_read(r);
    let mut slots = None;
    // (index into KINDS, registers handed back as a bit mask, dst, bytes written, where the hook resumes the title)
    let (k, handed_back, dst, len, end) = match kind {
        HookKind::Memcpy => (0, 1, entry.r0, Some(entry.r2), lr),
        HookKind::Memset => (1, 1, entry.r0, Some(entry.r2), lr),
        HookKind::Strcpy => (2, 1, entry.r0, stdlib::strlen(core, &mut (), entry.r1).await.ok().map(|n| n + 1), lr),
        HookKind::Strlen => (3, 1, entry.r0, Some(0), lr),
        HookKind::InlineCopy(spec) => {
            let offsets = [spec.dst_offset, spec.src_offset, spec.len_offset];
            let low = entry.r7.wrapping_add(*offsets.iter().min().unwrap() as u32);
            let size = (offsets.iter().max().unwrap() - offsets.iter().min().unwrap() + 4) as usize;
            let word = |offset: i32| wie_util::read_generic::<u32, _>(core, entry.r7.wrapping_add(offset as u32)).ok();
            // The loop leaves through the `b` just before the body (`test_decrements_first`).
            let exit = read(core, site - 2, 2).map(|b| decode_exit_b(site - 2, [b[0], b[1]]));
            slots = Some((low, size));
            match (word(spec.dst_offset), word(spec.len_offset), exit) {
                (Some(dst), Some(len), Some(exit)) => (4, 0, dst, Some(len.wrapping_add(1)), exit),
                _ => (4, 0, 0, None, 0),
            }
        }
        HookKind::RegInlineCopy(spec) => {
            let bit = |r: ArmRegister| 1 << super::LOW_REGISTERS.iter().position(|x| *x == r).unwrap();
            let len = spec.bytes(reg(spec.count));
            (5, bit(spec.src) | bit(spec.dst) | bit(spec.count), reg(spec.dst), Some(len), spec.exit_pc)
        }
    };
    let end = end & !1;
    let counts = &COUNTS[k];
    counts[FIRES].fetch_add(1, Relaxed);

    // The window to compare, as it is before either side runs. Without it there is nothing to undo with.
    let window = len.filter(|len| *len <= MAX_LEN).and_then(|len| {
        let wide = (dst.wrapping_sub(MARGIN), (len + 2 * MARGIN) as usize);
        [wide, (dst, len as usize)]
            .into_iter()
            .find_map(|(base, size)| read(core, base, size).map(|bytes| (base, bytes)))
    });
    let before_slots = slots.and_then(|(base, size)| read(core, base, size));
    let (Some((base, before)), true) = (window, slots.is_none() || before_slots.is_some()) else {
        counts[SKIPPED].fetch_add(1, Relaxed);
        return dispatch(core, hook_pc, kind, displaced, lr).await;
    };
    let state = |core: &ArmCore| -> State {
        (
            core.save_context().words(),
            read(core, base, before.len()),
            slots.and_then(|(base, size)| read(core, base, size)),
        )
    };

    // The title's own code, then back to the fire.
    core.write_bytes(site, &displaced)?;
    core.inner.lock().engine.reg_write(ArmRegister::PC, hook_pc);
    let own = run_to(core, end).then(|| state(core));
    core.write_bytes(base, &before)?;
    if let (Some((base, _)), Some(bytes)) = (slots, &before_slots) {
        core.write_bytes(base, bytes)?;
    }
    core.restore_context(&entry);
    core.write_bytes(site, &[0x80, 0xdf])?;
    if own.is_none() {
        counts[OWN_FAULT].fetch_add(1, Relaxed);
        note(format!("OWN-FAULT {} site={site:#x}", KINDS[k]));
    }

    // The hook, carried to the same point.
    let jump = dispatch(core, hook_pc, kind, displaced, lr).await?;
    if k < 4 && jump.0 & !1 != end {
        counts[OWN].fetch_add(1, Relaxed);
    }
    core.set_next_pc(jump.0)?;
    let reached = run_to(core, end);
    let (regs, mem, slots_after) = state(core);
    let resume = JumpTo(regs[15] | ((regs[16] >> 5) & 1));
    if !reached {
        counts[HOOK_FAULT].fetch_add(1, Relaxed);
        note(format!("HOOK-FAULT {} site={site:#x} stopped={:#x}", KINDS[k], regs[15]));
    }
    let (Some((own_regs, Some(own_mem), own_slots)), true, Some(mem)) = (own, reached, mem) else {
        return Ok(resume);
    };
    counts[COMPARED].fetch_add(1, Relaxed);

    let dead = |i: usize| k < 4 && regs[13].wrapping_sub(base.wrapping_add(i as u32)).wrapping_sub(1) < 128;
    let differing = (0..mem.len()).filter(|&i| mem[i] != own_mem[i] && !dead(i)).collect::<Vec<_>>();
    if let (Some(first), Some(last)) = (differing.first(), differing.last()) {
        counts[MEM].fetch_add(1, Relaxed);
        let from_dst = |i: &usize| base.wrapping_add(*i as u32).wrapping_sub(dst) as i32;
        note(format!(
            "MEM {} site={site:#x} len={:#x} bytes={} first={} last={}",
            KINDS[k],
            len.unwrap_or(0),
            differing.len(),
            from_dst(first),
            from_dst(last)
        ));
    }
    let differs = |i: usize| regs[i] != own_regs[i];
    if (0..8).any(|i| handed_back & (1 << i) != 0 && differs(i)) || differs(15) || slots_after != own_slots {
        counts[RESULT].fetch_add(1, Relaxed);
        note(format!("RESULT {} site={site:#x} len={:#x}", KINDS[k], len.unwrap_or(0)));
    }
    if (if k < 4 { 4..12 } else { 8..13 }).any(differs) || differs(13) {
        counts[CALLEE].fetch_add(1, Relaxed);
        note(format!("CALLEE {} site={site:#x}", KINDS[k]));
    }
    Ok(resume)
}
