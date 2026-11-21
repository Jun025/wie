use alloc::{collections::BTreeMap, format, string::ToString, sync::Arc, vec::Vec};

use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use super::{Entry, PatternToken, scan_pattern};
use crate::{ArmCore, engine::ArmRegister, function::JumpTo, stdlib};

const BINARY_PATCH_SVC: u32 = 0x80;

/// Development tool — not built for the browser host. See the module.
#[cfg(not(target_arch = "wasm32"))]
pub mod shadow;

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    /// LSB=1 (Thumb) is required; the engine doesn't service ARM-mode SVCs.
    pub pc: u32,
    pub kind: HookKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HookKind {
    /// ABI: dst=r0, src=r1, len=r2; returns via LR.
    ///
    /// The four library kinds answer only the calls where every correct routine writes the same
    /// bytes. The rest — overlapping buffers, a read or write the host refuses — go to the title's
    /// own routine (`resume_original`), so its answer is the answer.
    Memcpy,
    /// ABI: dst=r0, val=r1 (low byte), len=r2; returns via LR.
    Memset,
    /// ABI: dst=r0, src=r1; copies NUL-inclusive; returns via LR (R0 unchanged).
    Strcpy,
    /// ABI: str=r0; returns length in R0 via LR.
    Strlen,
    /// Replaces the body of `while (len-- > 0) *dst++ = *src++;` with all three on the stack. The
    /// test runs first, so `len` is already one below the bytes left when the body starts: the hook
    /// copies `len + 1`. Installed only where `exit_pc` is that test (`test_decrements_first`).
    ///
    /// Both loop kinds copy as the loop does (`copy_as_byte_loop`), and a fire the host cannot
    /// carry out whole gives the site back to the title (`unhook`).
    InlineCopy(InlineCopy),
    /// Replaces an inline byte-copy loop whose src/dst/count live in registers
    /// instead of on the stack. Bytes copied: `RegInlineCopy::bytes`.
    /// After the copy, `src`/`dst` are advanced by that count and `count` is
    /// rewritten so it equals what the loop would have left there (i.e.,
    /// `original - bytes`), then the dispatcher jumps to `exit_pc`.
    RegInlineCopy(RegInlineCopy),
}

#[derive(Debug, Clone, Copy)]
pub struct InlineCopy {
    pub dst_offset: i32,
    pub src_offset: i32,
    pub len_offset: i32,
    pub exit_pc: u32,
    /// Set when the loop's outer code re-reads the stack slots after the body;
    /// the dispatcher then writes back `dst`/`src` advanced by the bytes copied, and `len=0`.
    pub spill_back: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RegInlineCopy {
    src: ArmRegister,
    dst: ArmRegister,
    count: ArmRegister,
    count_offset: i32,
    exit_pc: u32,
}

impl RegInlineCopy {
    /// Bytes the loop copies when entered with `count`. It copies first and then tests the count
    /// it had, so a count of zero or below still copies once.
    fn bytes(&self, count: u32) -> u32 {
        ((count as i32).max(0) as u32).wrapping_add(self.count_offset as u32)
    }
}

/// Scanned across the install-time memory range; each match becomes a `Hook`.
pub struct PatternHook {
    pub tokens: Vec<PatternToken>,
    pub kind_template: PatternHookKind,
}

pub enum PatternHookKind {
    Memcpy,
    Memset,
    Strcpy,
    Strlen,
    InlineCopy {
        /// `None` => filled from the matching `{dst}` / `{src}` / `{len}`
        /// capture (Thumb1 `SUBS Rn, #imm8` byte, negated to a stack offset).
        /// `Some(v)` => pinned by TOML when the pattern omits the capture.
        dst_offset: Option<i32>,
        src_offset: Option<i32>,
        len_offset: Option<i32>,
        /// `None` => filled from the `{exit_b}` capture's decoded branch target.
        exit_pc: Option<u32>,
        spill_back: bool,
    },
    /// Pattern-matched register-resident copy loop. The `src`/`dst`/`count`
    /// registers are read from the pattern's bit-level captures and `exit_pc`
    /// is derived at install time from `match_addr + pattern.len()` (Thumb bit
    /// set).
    RegInlineCopy {
        count_offset: i32,
    },
}

/// Expand static + pattern hooks into a single `Vec<Hook>` whose PCs are final
/// and Thumb-valid. All pattern matching happens here; downstream consumers
/// (overlap check, `apply_hooks`) only see PC + kind, never raw tokens.
///
/// Static hook PCs come from TOML, so we validate the Thumb bit up front —
/// `install_entry` runs `apply_patches` between resolve and apply, and a
/// fatal-after-write would leave guest memory partially modified.
pub fn resolve_hooks(core: &mut ArmCore, entry: &Entry, scan_ranges: &[(u32, u32)]) -> Result<Vec<Hook>> {
    for hook in &entry.hooks {
        if hook.pc & 1 == 0 {
            return Err(WieError::FatalError(format!(
                "entry {}: hook PC {:#x} targets ARM mode; only Thumb (LSB=1) is supported",
                entry.name, hook.pc
            )));
        }
    }
    let mut installed: Vec<Hook> = entry.hooks.clone();

    for pattern in &entry.hook_patterns {
        let matches = scan_pattern(core, &pattern.tokens, scan_ranges)?;
        for (match_addr, pm) in matches {
            let kind = match &pattern.kind_template {
                PatternHookKind::Memcpy => HookKind::Memcpy,
                PatternHookKind::Memset => HookKind::Memset,
                PatternHookKind::Strcpy => HookKind::Strcpy,
                PatternHookKind::Strlen => HookKind::Strlen,
                PatternHookKind::InlineCopy {
                    dst_offset,
                    src_offset,
                    len_offset,
                    exit_pc,
                    spill_back,
                } => {
                    let dst = dst_offset
                        .or_else(|| pm.dst.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing dst")))?;
                    let src = src_offset
                        .or_else(|| pm.src.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing src")))?;
                    let len = len_offset
                        .or_else(|| pm.len.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing len")))?;
                    let exit = if let Some(v) = exit_pc {
                        *v
                    } else {
                        let site = pm
                            .exit_b_site
                            .ok_or_else(|| WieError::FatalError("pattern missing exit_b site".to_string()))?;
                        let bytes = pm
                            .exit_b_bytes
                            .ok_or_else(|| WieError::FatalError("pattern missing exit_b bytes".to_string()))?;
                        decode_exit_b(site, bytes)
                    };
                    HookKind::InlineCopy(InlineCopy {
                        dst_offset: dst,
                        src_offset: src,
                        len_offset: len,
                        exit_pc: exit,
                        spill_back: *spill_back,
                    })
                }
                PatternHookKind::RegInlineCopy { count_offset } => {
                    let src = arm_register_from_index(
                        pm.src_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing src register capture")))?,
                    );
                    let dst = arm_register_from_index(
                        pm.dst_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing dst register capture")))?,
                    );
                    let count = arm_register_from_index(
                        pm.count_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing count register capture")))?,
                    );
                    let exit_pc = match_addr.wrapping_add(pattern.tokens.len() as u32) | 1;
                    HookKind::RegInlineCopy(RegInlineCopy {
                        src,
                        dst,
                        count,
                        count_offset: *count_offset,
                        exit_pc,
                    })
                }
            };
            let pc = match_addr | 1;
            if installed.iter().any(|h| h.pc == pc) {
                tracing::warn!("Hook at {pc:#x} already registered; skipping duplicate match");
                continue;
            }
            installed.push(Hook { pc, kind });
        }
    }

    installed.retain(|hook| match hook.kind {
        HookKind::InlineCopy(spec) if !test_decrements_first(core, hook.pc, &spec) => {
            tracing::warn!(
                "inline_copy at {:#x}: the loop test is not `len-- > 0`; the loop is left to run itself",
                hook.pc
            );
            false
        }
        _ => true,
    });
    Ok(installed)
}

/// Whether the code at `spec.exit_pc` is `len-- > 0` as Thumb -O0 emits it, straight before the body
/// at `hook_pc`. `InlineCopy`'s byte count holds for that test only — `--len > 0` would run one
/// iteration fewer on the same slots — and nothing in the body pattern tells the two apart.
fn test_decrements_first(core: &ArmCore, hook_pc: u32, spec: &InlineCopy) -> bool {
    let test = spec.exit_pc & !1;
    let Ok(len) = u8::try_from(spec.len_offset.wrapping_neg()) else {
        return false;
    };
    #[rustfmt::skip]
    let expected = [
        0x3b, 0x1c, len, 0x3b, // r3 = r7 - len_offset
        0x1a, 0x68, 0x11, 0x1c, // r2 = r1 = len
        0x01, 0x3a, 0x1a, 0x60, // len = r2 - 1
        0x00, 0x29, 0x00, 0xdc, // cmp r1, #0; bgt body
    ];
    let mut code = [0u8; 16];
    // + 2: the `b` that leaves the loop sits between the test and the body.
    test.wrapping_add(expected.len() as u32 + 2) == hook_pc & !1 && core.read_bytes(test, &mut code).is_ok() && code == expected
}

/// Patch the SVC instruction at every hook PC and register the dispatcher.
/// `hooks` must already be the fully expanded list from `resolve_hooks`. The
/// dispatcher is registered even when `hooks` is empty so that any later SVC
/// #0x80 (e.g., from guest code that happens to encode the same bytes) routes
/// to a single, named diagnostic instead of falling out of the engine.
pub fn apply_hooks(core: &mut ArmCore, entry_name: &str, hooks: &[Hook]) -> Result<()> {
    let mut registry = BTreeMap::new();
    for hook in hooks {
        debug_assert!(hook.pc & 1 == 1, "resolve_hooks must reject ARM-mode PCs");
        let patch_addr = hook.pc & !1;
        let mut displaced = [0u8; 2];
        core.read_bytes(patch_addr, &mut displaced)?;
        registry.insert(hook.pc, (hook.kind, displaced));
        let instruction: u16 = 0xdf00 | (BINARY_PATCH_SVC as u16 & 0xff);
        core.write_bytes(patch_addr, &instruction.to_le_bytes())?;
        tracing::info!("Hook installed at {:#x}: {:?}", hook.pc, hook.kind);
    }
    if !hooks.is_empty() {
        tracing::info!("Installed {} hooks for {}", hooks.len(), entry_name);
    }
    core.register_svc_handler(BINARY_PATCH_SVC, handle_binary_patch_svc, &Arc::new(registry))?;
    Ok(())
}

/// Negate the unsigned `SUBS Rn, #imm8` immediate: the captured byte is the
/// distance below R7, so the resulting offset is `-imm8`.
fn capture_to_offset(byte: u8) -> i32 {
    -(byte as i32)
}

const LOW_REGISTERS: [ArmRegister; 8] = [
    ArmRegister::R0,
    ArmRegister::R1,
    ArmRegister::R2,
    ArmRegister::R3,
    ArmRegister::R4,
    ArmRegister::R5,
    ArmRegister::R6,
    ArmRegister::R7,
];

/// `index` comes from a 3-bit BitMatch capture, so it is 0..=7.
fn arm_register_from_index(index: u8) -> ArmRegister {
    LOW_REGISTERS[index as usize]
}

/// Decode a Thumb `B imm11` (`11100 iiiiiiiiiii`) at `b_site`. Returns the
/// target PC with the Thumb bit set.
fn decode_exit_b(b_site: u32, bytes: [u8; 2]) -> u32 {
    let raw = u16::from_le_bytes(bytes);
    let imm11 = (raw & 0x07ff) as i32;
    let offset = if imm11 & 0x400 != 0 { imm11 - 0x800 } else { imm11 };
    let target = (b_site.wrapping_add(4) as i64).wrapping_add((offset * 2) as i64) as u32;
    target | 1
}

/// Hook PC → its kind and the two bytes the SVC replaced.
type Registry = Arc<BTreeMap<u32, (HookKind, [u8; 2])>>;

async fn handle_binary_patch_svc(core: &mut ArmCore, registry: &mut Registry) -> Result<JumpTo> {
    let (pc, lr) = core.read_pc_lr()?;
    // PC on entry is the address right after the patched 2-byte SVC. Drop any
    // Thumb bit, step back over the SVC, then re-set the Thumb bit because
    // hook PCs are stored that way.
    let hook_pc = (pc.wrapping_sub(2)) | 1;
    let (kind, displaced) = registry
        .get(&hook_pc)
        .copied()
        .ok_or_else(|| WieError::FatalError(format!("binary-patch hook fired at unregistered PC {hook_pc:#x}")))?;

    #[cfg(not(target_arch = "wasm32"))]
    if shadow::enabled() {
        return shadow::fire(core, hook_pc, kind, displaced, lr).await;
    }
    dispatch(core, hook_pc, kind, displaced, lr).await
}

async fn dispatch(core: &mut ArmCore, hook_pc: u32, kind: HookKind, displaced: [u8; 2], lr: u32) -> Result<JumpTo> {
    match kind {
        HookKind::Memcpy | HookKind::Memset | HookKind::Strcpy | HookKind::Strlen => {
            let own = u16::from_le_bytes(displaced);
            let resumable = own == BX_PC || own & PUSH_MASK == PUSH;
            match library_call(core, kind, resumable).await {
                Ok(true) => Ok(JumpTo(lr)),
                Err(e) if !resumable => Err(e),
                _ => resume_original(core, hook_pc, own),
            }
        }
        HookKind::InlineCopy(spec) => {
            let r7 = core.inner.lock().engine.reg_read(ArmRegister::R7);
            let dst_slot = r7.wrapping_add(spec.dst_offset as u32);
            let src_slot = r7.wrapping_add(spec.src_offset as u32);
            let len_slot = r7.wrapping_add(spec.len_offset as u32);
            let dst: u32 = read_generic(core, dst_slot)?;
            let src: u32 = read_generic(core, src_slot)?;
            // The loop test already took one off `len` for the iteration the hook replaced.
            let len = read_generic::<u32, _>(core, len_slot)?.wrapping_add(1);
            tracing::trace!(
                "hook inline_copy(ptr_dst={dst:#x}, ptr_src={src:#x}, len={len:#x}, exit={:#x})",
                spec.exit_pc
            );
            if !copy_as_byte_loop(core, dst, src, len)? {
                return unhook(core, hook_pc, displaced);
            }
            if spec.spill_back {
                core.write_bytes(dst_slot, &dst.wrapping_add(len).to_le_bytes())?;
                core.write_bytes(src_slot, &src.wrapping_add(len).to_le_bytes())?;
                core.write_bytes(len_slot, &0u32.to_le_bytes())?;
            }
            Ok(JumpTo(spec.exit_pc))
        }
        HookKind::RegInlineCopy(spec) => {
            let (src, dst, count_initial) = {
                let inner = core.inner.lock();
                (
                    inner.engine.reg_read(spec.src),
                    inner.engine.reg_read(spec.dst),
                    inner.engine.reg_read(spec.count),
                )
            };
            let count = spec.bytes(count_initial);
            tracing::trace!(
                "hook reg_inline_copy(src={src:#x}, dst={dst:#x}, count={count:#x}, exit={:#x})",
                spec.exit_pc
            );
            if !copy_as_byte_loop(core, dst, src, count)? {
                return unhook(core, hook_pc, displaced);
            }
            let mut inner = core.inner.lock();
            inner.engine.reg_write(spec.src, src.wrapping_add(count));
            inner.engine.reg_write(spec.dst, dst.wrapping_add(count));
            inner.engine.reg_write(spec.count, count_initial.wrapping_sub(count));
            Ok(JumpTo(spec.exit_pc))
        }
    }
}

/// Copies `len` bytes the way a forward byte loop does, overlapping buffers included — or, where
/// the host cannot reach the whole of both ranges, copies nothing and returns `false`.
///
/// With `dst` some `ahead` bytes past `src` inside the range, the loop reads back what it wrote
/// `ahead` bytes earlier. A piece no longer than `ahead` is read whole before any of it is
/// overwritten, so going piece by piece leaves the same bytes.
fn copy_as_byte_loop(core: &mut ArmCore, dst: u32, src: u32, len: u32) -> Result<bool> {
    const PIECE: u32 = 4096;
    let mut inner = core.inner.lock();
    // The margin keeps `is_mapped`'s own page rounding from wrapping.
    let whole = |at: u32| at.checked_add(len).is_some_and(|end| end < 0xfff0_0000) && inner.engine.is_mapped(at, len as usize);
    if !whole(src) || !whole(dst) {
        return Ok(false);
    }
    let piece = match dst.wrapping_sub(src) {
        0 => PIECE,
        ahead => ahead.min(PIECE),
    };
    let mut buf = [0u8; PIECE as usize];
    let mut offset = 0;
    while offset < len {
        let buf = &mut buf[..(len - offset).min(piece) as usize];
        inner.engine.mem_read(src + offset, buf.len(), buf)?;
        inner.engine.mem_write(dst + offset, buf)?;
        offset += buf.len() as u32;
    }
    Ok(true)
}

/// Gives a loop site back to the title for good: its own bytes go back and the loop runs itself,
/// from this fire on.
fn unhook(core: &mut ArmCore, hook_pc: u32, displaced: [u8; 2]) -> Result<JumpTo> {
    tracing::warn!("hook at {hook_pc:#x}: a copy the host cannot carry out; the loop is left to run itself");
    core.write_bytes(hook_pc & !1, &displaced)?;
    Ok(JumpTo(hook_pc))
}

/// Thumb `bx pc`: the thunk in front of an ARM routine.
const BX_PC: u16 = 0x4778;
/// Thumb `push {r0-r7 subset[, lr]}`: register list in bits 0-7, `lr` in bit 8.
const PUSH: u16 = 0xb400;
const PUSH_MASK: u16 = 0xfe00;

/// Runs the engine routine for a library hook. `Ok(false)` — only when `may_decline` — is a call
/// whose result depends on how the title's routine walks the buffers: they overlap.
async fn library_call(core: &mut ArmCore, kind: HookKind, may_decline: bool) -> Result<bool> {
    let (r0, r1, r2) = {
        let inner = core.inner.lock();
        (
            inner.engine.reg_read(ArmRegister::R0),
            inner.engine.reg_read(ArmRegister::R1),
            inner.engine.reg_read(ArmRegister::R2),
        )
    };
    let overlap = |len: u32| may_decline && (r0.wrapping_sub(r1) < len || r1.wrapping_sub(r0) < len);
    match kind {
        HookKind::Memcpy => {
            tracing::trace!("hook memcpy(ptr_dst={r0:#x}, ptr_src={r1:#x}, len={r2:#x})");
            if overlap(r2) {
                return Ok(false);
            }
            stdlib::memcpy(core, &mut (), r0, r1, r2).await?;
        }
        HookKind::Memset => {
            tracing::trace!("hook memset(ptr_dst={r0:#x}, val={:#x}, len={r2:#x})", r1 as u8);
            stdlib::memset(core, &mut (), r0, r1, r2).await?;
        }
        HookKind::Strcpy => {
            tracing::trace!("hook strcpy(ptr_dst={r0:#x}, ptr_src={r1:#x})");
            let len = stdlib::strlen(core, &mut (), r1).await?.wrapping_add(1);
            if overlap(len) {
                return Ok(false);
            }
            stdlib::memcpy(core, &mut (), r0, r1, len).await?;
        }
        HookKind::Strlen => {
            let len = stdlib::strlen(core, &mut (), r0).await?;
            tracing::trace!("hook strlen(ptr_str={r0:#x}) -> {len:#x}");
            core.inner.lock().engine.reg_write(ArmRegister::R0, len);
        }
        HookKind::InlineCopy(_) | HookKind::RegInlineCopy(_) => unreachable!("not a library routine"),
    }
    Ok(true)
}

/// Hands the call to the title's own routine: carries out the instruction the SVC replaced and
/// resumes right after it. The engine routine has changed no register by then, and whatever it
/// wrote before failing the routine writes again — its buffers do not overlap.
fn resume_original(core: &mut ArmCore, hook_pc: u32, own: u16) -> Result<JumpTo> {
    let site = hook_pc & !1;
    if own == BX_PC {
        return Ok(JumpTo(site + 4)); // even: ARM state
    }
    let mut inner = core.inner.lock();
    let pushed = LOW_REGISTERS
        .iter()
        .chain(&[ArmRegister::LR])
        .enumerate()
        .filter(|(bit, _)| own & (1 << bit) != 0)
        .flat_map(|(_, reg)| inner.engine.reg_read(*reg).to_le_bytes())
        .collect::<Vec<_>>();
    let sp = inner.engine.reg_read(ArmRegister::SP).wrapping_sub(pushed.len() as u32);
    inner.engine.mem_write(sp, &pushed)?;
    inner.engine.reg_write(ArmRegister::SP, sp);
    Ok(JumpTo((site + 2) | 1)) // odd: Thumb state
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use wie_util::ByteRead;

    use super::*;
    use crate::function::{RegisteredFunction, RegisteredFunctionHolder};

    fn registry_with(pc: u32, kind: HookKind) -> Registry {
        let mut map = BTreeMap::new();
        map.insert(pc, (kind, [0, 0]));
        Arc::new(map)
    }

    /// Set PC to where it would be on entry to the SVC handler for a hook at
    /// `hook_pc` (svc_addr + 2, i.e. just past the patched 2-byte SVC).
    fn set_post_svc_pc(core: &mut ArmCore, hook_pc: u32) {
        let mut inner = core.inner.lock();
        inner.engine.reg_write(ArmRegister::PC, (hook_pc & !1).wrapping_add(2));
    }

    fn entry_with_static(name: &str, hooks: Vec<Hook>) -> Entry {
        Entry {
            hash: Some([0u8; 16]),
            name: name.into(),
            hooks,
            hook_patterns: vec![],
            patches: vec![],
            patch_patterns: vec![],
        }
    }

    #[test]
    fn resolve_hooks_rejects_arm_mode_pc() -> Result<()> {
        let entry = entry_with_static(
            "arm-mode",
            vec![Hook {
                pc: 0x2000, // LSB=0 => ARM mode
                kind: HookKind::Memcpy,
            }],
        );
        let mut core = ArmCore::new(false, None)?;
        core.map(0x2000, 0x1000)?;

        let err = resolve_hooks(&mut core, &entry, &[]).unwrap_err();
        let msg = alloc::format!("{err}");
        assert!(msg.contains("ARM mode"), "unexpected error: {msg}");
        Ok(())
    }

    #[test]
    fn apply_hooks_writes_thumb_svc_instruction() -> Result<()> {
        let entry = entry_with_static(
            "patch",
            vec![Hook {
                pc: 0x2001, // Thumb
                kind: HookKind::Memcpy,
            }],
        );
        let mut core = ArmCore::new(false, None)?;
        core.map(0x2000, 0x1000)?;
        core.write_bytes(0x2000, &[0xaa, 0xbb])?;

        let hooks = resolve_hooks(&mut core, &entry, &[])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut buf = [0u8; 2];
        core.read_bytes(0x2000, &mut buf)?;
        assert_eq!(buf, [BINARY_PATCH_SVC as u8, 0xdf]);
        Ok(())
    }

    #[futures_test::test]
    async fn memcpy_dispatch_copies_bytes_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8];
        core.write_bytes(src, &data)?;

        let hook_pc = 0x10001u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::R2, data.len() as u32);
            inner.engine.reg_write(ArmRegister::LR, 0xdead_beef);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Memcpy);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 8];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, data);
        assert_eq!(core.inner.lock().engine.reg_read(ArmRegister::PC), 0xdead_beef & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn memset_dispatch_fills_bytes() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let dst = 0x10000u32;
        let len = 16u32;
        let hook_pc = 0x10401u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, 0xab);
            inner.engine.reg_write(ArmRegister::R2, len);
            inner.engine.reg_write(ArmRegister::LR, 0x3000);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Memset);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 16];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, [0xab; 16]);
        Ok(())
    }

    #[futures_test::test]
    async fn strcpy_dispatch_copies_null_terminated_string_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let s = b"hello, world!\0";
        core.write_bytes(src, s)?;
        core.write_bytes(dst, &[0xff; 32])?;

        let hook_pc = 0x10801u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::LR, 0xcafe_babe);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Strcpy);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 14];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(&out, s);
        let mut sentinel = [0u8; 1];
        core.read_bytes(dst + s.len() as u32, &mut sentinel)?;
        assert_eq!(sentinel, [0xff]);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::R0), dst);
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0xcafe_babe & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn strlen_dispatch_returns_length_in_r0() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let str_ptr = 0x10100u32;
        let s = b"abcdef\0";
        core.write_bytes(str_ptr, s)?;

        let hook_pc = 0x10c01u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, str_ptr);
            inner.engine.reg_write(ArmRegister::LR, 0x1234_5678);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Strlen);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::R0), 6);
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0x1234_5678 & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn inline_copy_dispatch_reads_frame_copies_and_jumps_to_exit() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x2000)?;

        let src = 0x10000u32;
        let dst = 0x10800u32;
        let payload = [0xaa, 0xbb, 0xcc, 0xdd];
        core.write_bytes(src, &payload)?;

        let frame = 0x11000u32;
        core.write_bytes(frame, &dst.to_le_bytes())?;
        core.write_bytes(frame + 4, &src.to_le_bytes())?;
        // As the body finds it: the loop test has already taken one off.
        core.write_bytes(frame + 8, &(payload.len() as u32 - 1).to_le_bytes())?;

        let hook_pc = 0x10201u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R7, frame);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let spec = InlineCopy {
            dst_offset: 0,
            src_offset: 4,
            len_offset: 8,
            exit_pc: 0x10401,
            spill_back: true,
        };
        let registry = registry_with(hook_pc, HookKind::InlineCopy(spec));
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 4];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, payload);

        let mut slot = [0u8; 4];
        core.read_bytes(frame, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), dst + payload.len() as u32);
        core.read_bytes(frame + 4, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), src + payload.len() as u32);
        core.read_bytes(frame + 8, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), 0);

        assert_eq!(core.inner.lock().engine.reg_read(ArmRegister::PC), 0x10400);
        Ok(())
    }

    const CODE: u32 = 0x20000;
    const DATA: u32 = 0x30000;

    /// Runs a hand-assembled routine twice from the same memory image — as it stands, and with the embedded
    /// generic entry installed over it (exactly one of its patterns must match) — and returns `r0` and the
    /// first 0x100 bytes of `DATA` from each. A hook is right when the two are equal.
    async fn own_and_hooked(code: &[u8], data: &[u8], args: &[u32]) -> Result<Vec<(u32, Vec<u8>)>> {
        let mut runs = Vec::new();
        for hooked in [false, true] {
            let mut core = ArmCore::new(false, None)?;
            core.load(code, CODE, code.len())?;
            core.map(DATA, 0x2000)?;
            core.write_bytes(DATA, data)?;
            if hooked {
                assert_eq!(crate::install_binary_patches(&mut core, &[], &[(CODE, code.len() as u32)])?, 1);
            }
            core.inner.lock().engine.reg_write(ArmRegister::SP, DATA + 0x2000);
            let r0: u32 = core.run_function(CODE | 1, args).await?;
            let mut mem = vec![0u8; 0x100];
            core.read_bytes(DATA, &mut mem)?;
            runs.push((r0, mem));
        }
        Ok(runs)
    }

    /// `while (len-- > 0) *dst++ = *src++;` as Thumb -O0 emits it, entered with `r0` = the frame whose
    /// three words below it are `len`, `dst`, `src`.
    #[rustfmt::skip]
    const STACK_LOOP: [u8; 50] = [
        0x07, 0x1c, // adds r7, r0, #0
        // test:
        0x3b, 0x1c, 0x0c, 0x3b, // r3 = &len (r7 - 12)
        0x1a, 0x68, 0x11, 0x1c, // r2 = r1 = len
        0x01, 0x3a, 0x1a, 0x60, // len = r2 - 1
        0x00, 0x29, 0x00, 0xdc, // cmp r1, #0; bgt body
        0x0d, 0xe0, // b exit
        // body:
        0x3d, 0x1c, 0x08, 0x3d, 0x29, 0x68, // r5 = &dst (r7 - 8); r1 = dst
        0x3c, 0x1f, 0x22, 0x68, // r4 = &src (r7 - 4); r2 = src
        0x08, 0x1c, 0x13, 0x1c, 0x1b, 0x78, 0x03, 0x70, // *dst = *src
        0x01, 0x32, 0x22, 0x60, 0x01, 0x31, 0x29, 0x60, // src++; dst++
        0xe8, 0xe7, // b test
        // exit:
        0x70, 0x47, // bx lr
    ];

    /// `DATA` for `STACK_LOOP`: 1..=16, filler, and the frame at `DATA + 0x80` asking for `len` bytes
    /// from `DATA` to `DATA + dst`.
    fn stack_loop_data(dst: u32, len: u32) -> [u8; 0x80] {
        let mut data = [0xee_u8; 0x80];
        data[..16].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        data[0x74..0x78].copy_from_slice(&len.to_le_bytes());
        data[0x78..0x7c].copy_from_slice(&(DATA + dst).to_le_bytes());
        data[0x7c..0x80].copy_from_slice(&DATA.to_le_bytes());
        data
    }

    /// The test runs first and has already taken one off `len` when the body — where the hook sits —
    /// starts.
    #[futures_test::test]
    async fn inline_copy_matches_the_loop_whose_test_decrements_first() -> Result<()> {
        let runs = own_and_hooked(&STACK_LOOP, &stack_loop_data(0x40, 8), &[DATA + 0x80]).await?;
        assert_eq!(runs[0].1[0x40..0x49], [1, 2, 3, 4, 5, 6, 7, 8, 0xee], "the loop itself copies 8 bytes");
        assert_eq!(
            runs[1].1, runs[0].1,
            "hooked (left) vs the loop itself (right): copied bytes and the three frame slots"
        );
        Ok(())
    }

    /// A byte loop over buffers that overlap repeats what it has just written; a copy that reads
    /// ahead of its own writes does not.
    #[futures_test::test]
    async fn inline_copy_over_overlapping_buffers_matches_the_loop() -> Result<()> {
        let runs = own_and_hooked(&STACK_LOOP, &stack_loop_data(3, 12), &[DATA + 0x80]).await?;
        assert_eq!(runs[0].1[..16], [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 16], "the loop itself");
        assert_eq!(runs[1].1, runs[0].1, "hooked (left) vs the loop itself (right)");
        Ok(())
    }

    /// A guest read through null reads 0 where a host read is refused: the loop itself copies zeroes,
    /// so the hook has to leave that fire — and the rest of them — to the loop.
    #[futures_test::test]
    async fn inline_copy_from_null_is_left_to_the_loop() -> Result<()> {
        let mut data = stack_loop_data(0x40, 8);
        data[0x7c..0x80].copy_from_slice(&0u32.to_le_bytes()); // src = null
        let runs = own_and_hooked(&STACK_LOOP, &data, &[DATA + 0x80]).await?;
        assert_eq!(runs[0].1[0x40..0x49], [0, 0, 0, 0, 0, 0, 0, 0, 0xee], "the loop itself copies 8 zeroes");
        assert_eq!(runs[1].1, runs[0].1, "hooked (left) vs the loop itself (right)");
        Ok(())
    }

    /// The register loop the `reg_inline_copy` pattern matches — `r0` = dst, `r1` = src, `r2` = count —
    /// then hands back where `src` stopped, having stored where `dst` and `count` did at `DATA + 0x60`.
    #[rustfmt::skip]
    const REGISTER_LOOP: [u8; 32] = [
        0x0b, 0x78, 0x03, 0x70, // loop: r3 = *src; *dst = r3
        0x13, 0x1c, 0x01, 0x31, 0x01, 0x30, 0x01, 0x3a, // r3 = count; src++; dst++; count--
        0x00, 0x2b, 0xf7, 0xdc, // cmp r3, #0; bgt loop
        0x02, 0x4b, 0x18, 0x60, 0x5a, 0x60, // r3 = DATA + 0x60; r3[0] = dst; r3[1] = count
        0x08, 0x1c, 0x70, 0x47, 0x00, 0x00, // r0 = src; bx lr; padding
        0x60, 0x00, 0x03, 0x00, // DATA + 0x60
    ];

    /// The loop copies before it looks at the count: `count + 1` bytes, and one byte when the count
    /// comes in at zero or below. Over overlapping buffers it repeats what it has just written.
    #[futures_test::test]
    async fn reg_inline_copy_matches_the_loop() -> Result<()> {
        let data: Vec<u8> = (1..=64).collect();
        for (dst, count, copied) in [(0x20, 7, 8), (0x20, 0, 1), (0x20, -5i32 as u32, 1), (3, 11, 12)] {
            let runs = own_and_hooked(&REGISTER_LOOP, &data, &[DATA + dst, DATA, count]).await?;
            assert_eq!(
                runs[0].0,
                DATA + copied,
                "the loop itself copies {copied} bytes for count {}",
                count as i32
            );
            assert_eq!(runs[1], runs[0], "count {}: hooked (left) vs the loop itself (right)", count as i32);
        }
        Ok(())
    }

    /// A forward byte copy behind the `bx pc` thunk the `memcpy` pattern matches.
    #[rustfmt::skip]
    const FORWARD_MEMCPY: [u8; 36] = [
        0x78, 0x47, 0x00, 0x00, // bx pc (Thumb) + padding
        0x01, 0x40, 0x2d, 0xe9, // stmfd sp!, {r0, lr}
        0x03, 0x00, 0x52, 0xe3, // cmp r2, #3
        0x01, 0x20, 0x52, 0xe2, // loop: subs r2, r2, #1
        0x01, 0x30, 0xd1, 0xa4, // ldrgeb r3, [r1], #1
        0x01, 0x30, 0xc0, 0xa4, // strgeb r3, [r0], #1
        0xfb, 0xff, 0xff, 0xaa, // bge loop
        0x01, 0x40, 0xbd, 0xe8, // ldmfd sp!, {r0, lr}
        0x1e, 0xff, 0x2f, 0xe1, // bx lr
    ];

    /// With the destination 3 bytes after the source a forward copy repeats the first 3 bytes — the
    /// routine's own answer, which a read-then-write copy does not give.
    #[futures_test::test]
    async fn memcpy_forward_overlap_gives_the_routines_own_result() -> Result<()> {
        let data: Vec<u8> = (1..=64).collect();

        let runs = own_and_hooked(&FORWARD_MEMCPY, &data, &[DATA + 3, DATA, 32]).await?;
        assert_eq!(
            runs[0].1[..9],
            [1, 2, 3, 1, 2, 3, 1, 2, 3],
            "the routine itself repeats the first 3 bytes"
        );
        assert_eq!(runs[1], runs[0], "hooked (left) vs the routine itself (right)");
        Ok(())
    }

    /// The shadow comparison has to be able to go red: one call, judged against the hook kind that
    /// belongs on the routine and against one that does not. (Nothing else calls `shadow::fire` in
    /// this binary, so the counters are this test's.)
    #[cfg(not(target_arch = "wasm32"))]
    #[futures_test::test]
    async fn shadow_passes_the_right_hook_and_catches_a_wrong_one() -> Result<()> {
        let return_to = 0x50001;
        for kind in [HookKind::Memcpy, HookKind::Memset] {
            let mut core = ArmCore::new(false, None)?;
            core.load(&FORWARD_MEMCPY, CODE, FORWARD_MEMCPY.len())?;
            core.map(DATA, 0x2000)?;
            core.write_bytes(DATA, &(1..=64).collect::<Vec<u8>>())?;
            core.write_bytes(CODE, &[BINARY_PATCH_SVC as u8, 0xdf])?;
            {
                let mut inner = core.inner.lock();
                inner.engine.reg_write(ArmRegister::R0, DATA + 0x80);
                inner.engine.reg_write(ArmRegister::R1, DATA);
                inner.engine.reg_write(ArmRegister::R2, 32);
                inner.engine.reg_write(ArmRegister::SP, DATA + 0x2000);
                inner.engine.reg_write(ArmRegister::LR, return_to);
                let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
                inner.engine.reg_write(ArmRegister::Cpsr, (cpsr & !0x3f) | 0x1f | 0x20);
            }
            let displaced = [FORWARD_MEMCPY[0], FORWARD_MEMCPY[1]];
            assert_eq!(shadow::fire(&mut core, CODE | 1, kind, displaced, return_to).await?.0, return_to);
        }
        let report = shadow::report();
        let right =
            r#""memcpy":{"fires":1,"compared":1,"mem_mismatch":0,"result_mismatch":0,"callee_saved_mismatch":0,"own":0,"skipped":0,"own_fault":0}"#;
        let wrong = r#""memset":{"fires":1,"compared":1,"mem_mismatch":1,"result_mismatch":0,"#;
        assert!(report.contains(right) && report.contains(wrong), "{report}");
        Ok(())
    }

    /// A byte-wise `strlen` behind the prologue the `strlen` pattern matches. A guest read through null
    /// reads 0 (`Arm32CpuMemory::read_page`), so the routine itself answers 0 where a host read faults.
    #[futures_test::test]
    async fn strlen_of_null_gives_the_routines_own_result() -> Result<()> {
        #[rustfmt::skip]
        let code = [
            0x30, 0xb5, // push {r4, r5, lr}
            0x03, 0x23, 0x03, 0x40, 0x04, 0x1c, 0x00, 0x2b, // r3 = r0 & 3; r4 = r0; cmp r3, #0
            0x00, 0x20, // movs r0, #0
            0x23, 0x5c, 0x00, 0x2b, 0x01, 0xd0, // loop: r3 = r4[r0]; if r3 == 0 goto done
            0x01, 0x30, 0xfa, 0xe7, // r0++; b loop
            0x30, 0xbd, // done: pop {r4, r5, pc}
        ];
        let runs = own_and_hooked(&code, b"abc\0", &[0]).await?;
        assert_eq!(runs[0].0, 0, "the routine itself reads 0 at null");
        assert_eq!(runs[1], runs[0], "hooked (left) vs the routine itself (right)");

        let runs = own_and_hooked(&code, b"abc\0", &[DATA]).await?;
        assert_eq!((runs[0].0, runs[1].0), (3, 3));
        Ok(())
    }

    #[futures_test::test]
    async fn install_then_execute_hits_dispatcher_end_to_end() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x20000, 0x2000)?;
        core.map(0x30000, 0x1000)?;

        let src = 0x30000u32;
        let dst = 0x30200u32;
        let payload = [9u8, 8, 7, 6];
        core.write_bytes(src, &payload)?;

        let hook_pc = 0x20001u32;
        let entry = entry_with_static(
            "e2e",
            vec![Hook {
                pc: hook_pc,
                kind: HookKind::Memcpy,
            }],
        );
        let hooks = resolve_hooks(&mut core, &entry, &[])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut opcode = [0u8; 2];
        core.read_bytes(0x20000, &mut opcode)?;
        assert_eq!(opcode[1], 0xdf);
        assert_eq!(opcode[0] as u32, BINARY_PATCH_SVC);

        let return_addr = 0x40000u32 | 1;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::R2, payload.len() as u32);
            inner.engine.reg_write(ArmRegister::LR, return_addr);
            inner.engine.reg_write(ArmRegister::PC, hook_pc);

            let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
            inner.engine.reg_write(ArmRegister::Cpsr, (cpsr & !0x3f) | 0x1f | 0x20);
            inner.engine.reg_write(ArmRegister::SP, 0x20f00);
        }

        let result = {
            let mut inner = core.inner.lock();
            inner.engine.run(0, 10)?
        };
        let category = match result.stop_reason {
            crate::engine::EngineStopReason::Svc { category, lr, spsr } => {
                let mut inner = core.inner.lock();
                inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                inner.engine.reg_write(ArmRegister::PC, lr);
                category
            }
            _ => panic!("expected Svc"),
        };
        assert_eq!(category, BINARY_PATCH_SVC);

        let registry = registry_with(hook_pc, HookKind::Memcpy);
        let mut core_clone = core.clone();
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry)
            .call(&mut core_clone)
            .await?;

        let mut out = [0u8; 4];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, payload);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), return_addr & !1);
        Ok(())
    }

    #[test]
    fn pattern_scan_matches_single_hit() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x50000, 0x200)?;

        let pat_bytes = [0xaa, 0xbb, 0xcc, 0xdd];
        let match_addr = 0x50020u32;
        core.write_bytes(match_addr, &pat_bytes)?;

        let entry = Entry {
            hash: None,
            name: "scan-single".into(),
            hooks: vec![],
            hook_patterns: vec![PatternHook {
                tokens: vec![
                    PatternToken::Literal(0xaa),
                    PatternToken::Literal(0xbb),
                    PatternToken::Literal(0xcc),
                    PatternToken::Literal(0xdd),
                ],
                kind_template: PatternHookKind::Memcpy,
            }],
            patches: vec![],
            patch_patterns: vec![],
        };

        let hooks = resolve_hooks(&mut core, &entry, &[(0x50000, 0x200)])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut out = [0u8; 2];
        core.read_bytes(match_addr, &mut out)?;
        assert_eq!(out[1], 0xdf);
        assert_eq!(out[0] as u32, BINARY_PATCH_SVC);
        Ok(())
    }

    #[test]
    fn pattern_scan_capture_to_offset_negates_imm8() {
        assert_eq!(capture_to_offset(0x08), -8);
        assert_eq!(capture_to_offset(0x0c), -12);
        // imm8 in `SUBS Rn, #imm8` is unsigned 0..=255, so high values must
        // negate to large negative offsets, not wrap through `i8`.
        assert_eq!(capture_to_offset(0x80), -128);
        assert_eq!(capture_to_offset(0xfc), -252);
    }

    #[test]
    fn pattern_scan_exit_b_computes_forward_branch_target() {
        let bytes = [0x02, 0xe0];
        let site = 0x100u32;
        let exit = decode_exit_b(site, bytes);
        assert_eq!(exit, (site + 4 + 4) | 1);

        let neg = decode_exit_b(site, [0xfe, 0xe7]);
        assert_eq!(neg, site | 1);
    }

    #[test]
    fn pattern_duplicate_pc_warns_once_and_skips() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x60000, 0x100)?;
        core.write_bytes(0x60010, &[0x11, 0x22, 0x33, 0x44])?;

        let entry = Entry {
            hash: None,
            name: "dup".into(),
            hooks: vec![],
            hook_patterns: vec![
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
            ],
            patches: vec![],
            patch_patterns: vec![],
        };
        let hooks = resolve_hooks(&mut core, &entry, &[(0x60000, 0x100)])?;
        assert_eq!(hooks.len(), 1, "duplicate PC should be skipped");
        Ok(())
    }
}
