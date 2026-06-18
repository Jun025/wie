use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use jvm::Jvm;

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes};

use crate::runtime::wipi_c::invoke_lcdui_main;
use crate::runtime::{SVC_CATEGORY_INIT, svc_ids::InitSvcId};

// LGT "java-interface" import module (table 0x64). The native application is an
// AOT-compiled Java program (ez-i / Xceed toolchain): its classes are emitted as
// native ARM code that registers itself with the platform through this module and
// calls platform classes (`org/kwis/...`, `java/...`) by resolved offset.
//
// Decoded import indices (see `get_java_interface_method`):
//   0x03 -> java_unk0          register main-class metadata (name, args)
//   0x06 -> java_unk12         (paired with 0x07; takes the same struct ptr)
//   0x07 -> java_unk5          register the app's OWN classes (native methods)
//   0x14 -> java_load_classes  declare IMPORTED platform classes + resolve offsets
//   0x82 -> java_unk9          (boot hook, arg always 0)
//   0x83 -> java_unk11         invoke-static org/kwis/msp/lcdui/Main.main(argv)
pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk0)?,
        0x06 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk12)?,
        0x07 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk5)?,
        0x14 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaLoadClasses)?,
        0x82 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk9)?,
        0x83 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk11)?,
        // Runtime helper resolved lazily during native method execution (called
        // first in every method with a small per-method constant — looks like a
        // method-entry / stack-check / safepoint helper). Stubbed as a no-op.
        0x54 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk84)?,
        // Other AOT-runtime helpers resolved lazily during native dispatch. Stubbed
        // as no-ops (return 0) to advance; implement properly as they prove needed.
        _ => {
            tracing::warn!("LGT java import {function_index:#x} stubbed (no-op)");
            core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceStub)?
        }
    })
}

// ---- memory-decode helpers (best-effort, never fail) ----

/// Read a null-terminated C string at `address` as a Rust String (lossy).
fn read_cstring(core: &ArmCore, address: u32) -> Option<String> {
    if address == 0 {
        return None;
    }
    let bytes = read_null_terminated_string_bytes(core, address).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Read `count` u32 words starting at `address` (stops early on a read error).
fn peek_words(core: &ArmCore, address: u32, count: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(count);
    let mut cursor = address;
    for _ in 0..count {
        match read_generic::<u32, _>(core, cursor) {
            Ok(v) => out.push(v),
            Err(_) => break,
        }
        cursor += 4;
    }
    out
}

pub async fn java_unk0(core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    // (main_class_name_ptr, params_ptr, flag_str_ptr) — e.g. ("Game", _, "true")
    tracing::debug!(
        "java_unk0(main_class={:?}, {a1:#x}, flag={:?})",
        read_cstring(core, a0),
        read_cstring(core, a2)
    );

    Ok(())
}

pub async fn java_unk5(core: &mut ArmCore, _: &mut (System, Jvm), a0: u32, a1: u32) -> Result<()> {
    // a0: the application's OWN native class registry.
    //   [0]    = handle count
    //   [1]    = 0
    //   [2..]  = `count` class HANDLES (each = class_header + 0x4c); the handle's
    //            +0x08 word points back to the class header record in `.data`.
    //   [2+n..]= trailing per-class byte array (small counts; role unconfirmed).
    // Each class record carries native method/field tables whose method bodies are
    // ARM code pointers (`.text`). See docs/lgt_native_classes.md and native_class.rs.
    //
    // This is read-only decoding only — registering these as native-backed JVM
    // classes is the remaining work (see BRIDGE_REPORT.md).
    let count = read_generic::<u32, _>(core, a0).unwrap_or(0);
    tracing::debug!("java_unk5: app registry @ {a0:#x} ({count} class handles, aux @ {a1:#x}) — not yet bridged");

    if tracing::enabled!(tracing::Level::DEBUG) {
        for i in 0..count.min(64) {
            let handle = match read_generic::<u32, _>(core, a0 + 8 + i * 4) {
                Ok(h) => h,
                Err(_) => continue,
            };
            match crate::runtime::java::native_class::parse_native_class_from_handle(core, handle) {
                Ok(class) => {
                    tracing::debug!(
                        "  class[{i}] {:?} (tag={:#x} access={:#x}) parent={:?} methods={} fields={}",
                        class.name,
                        class.tag,
                        class.access_flags,
                        class.parent_name,
                        class.methods.len(),
                        class.fields.len()
                    );
                    for m in class.methods.iter().take(3) {
                        tracing::debug!("      {}{} code={:#x} locals={}", m.name, m.signature, m.code_ptr, m.num_locals);
                    }
                }
                Err(e) => tracing::debug!("  class[{i}] handle={handle:#x} parse failed: {e}"),
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn java_load_classes(
    core: &mut ArmCore,
    _: &mut (System, Jvm),
    classes: u32,
    fields: u32,
    static_fields: u32,
    virtual_methods: u32,
    a4: u32,
    static_methods: u32,
    field_offsets: u32,
    static_field_offsets: u32,
    virtual_method_offsets: u32,
    a9: u32,
    static_method_offsets: u32,
) -> Result<()> {
    // Declares the platform classes the app imports and resolves the layout the
    // native code uses to dispatch into them. Inputs:
    //   classes        = LgtJavaImportedClass[count] (count-prefixed); each entry is
    //                    { ptr_name, _, static_field_off/cnt, virtual_method_off/cnt,
    //                      _, static_method_off/cnt } (24 bytes).
    //   fields/static_fields/virtual_methods/a4/static_methods = arrays of
    //                    { ptr_name, ptr_type } pairs the imported classes reference.
    // Outputs (writable app RAM, e.g. 0x15006f4): the platform is expected to fill
    //   *_offsets with the resolved indices/vtable offsets so the native code can
    //   call platform methods. Not yet implemented (see BRIDGE_REPORT.md).
    let _ = (fields, static_fields, virtual_methods, a4, static_methods);
    let _ = (field_offsets, static_field_offsets, virtual_method_offsets, a9, static_method_offsets);

    let count = read_generic::<u32, _>(core, classes).unwrap_or(0);
    if tracing::enabled!(tracing::Level::DEBUG) {
        tracing::debug!("java_load_classes: {count} imported platform classes @ {classes:#x}");
        for i in 0..count.min(64) {
            let base = classes + 4 + i * 24;
            let ptr_name = read_generic::<u32, _>(core, base).unwrap_or(0);
            tracing::debug!("  import[{i}] {:?}", read_cstring(core, ptr_name));
        }
    }

    Ok(())
}

pub async fn java_unk9(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::debug!("java_unk9({a0:#x})");

    Ok(())
}

pub async fn java_unk11(core: &mut ArmCore, (_system, jvm): &mut (System, Jvm), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    // Decoded calling convention (LGT java-interface import 0x83 — invoke-static):
    //   a0 = ptr to class name cstring  (observed: "org/kwis/msp/lcdui/Main")
    //   a1 = 0 (unused / implicit method "main")
    //   a2 = argc
    //   a3 = ptr to argv (array of `argc` cstring pointers)
    // argv[0] is the application's main Jlet class name (e.g. "Game"). This mirrors
    // the WIPI-C clet boot, which invokes the same Main.main with "net/wie/CletWrapper".
    let _ = a1;
    let class_name = read_cstring(core, a0).unwrap_or_default();
    let argc = a2 as usize;
    let argv_ptrs = peek_words(core, a3, argc.min(16));
    let argv: Vec<String> = argv_ptrs.iter().map(|&p| read_cstring(core, p).unwrap_or_default()).collect();

    tracing::debug!("java_unk11: invoke-static {class_name}.main argv={argv:?}");

    if class_name != "org/kwis/msp/lcdui/Main" {
        return Err(WieError::Unimplemented(format!(
            "LGT java_unk11: unexpected invoke target {class_name} (argv={argv:?})"
        )));
    }
    let main_class = argv.first().cloned().unwrap_or_default();
    if main_class.is_empty() {
        return Err(WieError::FatalError("LGT java_unk11: empty main class name in argv[0]".into()));
    }

    // Boot the application's main Jlet through the shared lcdui Main path.
    invoke_lcdui_main(jvm, &main_class).await
}

pub async fn java_unk12(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::debug!("java_unk12({a0:#x})");

    Ok(())
}

/// java-interface import `0x54`: runtime helper resolved lazily at the start of
/// native method execution. Treated as a no-op (returns 0) until its semantics are
/// confirmed. Appears benign (a stack/safepoint check).
pub async fn java_interface_unk84(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::trace!("java_interface_unk84({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

/// Generic no-op stub for not-yet-implemented java-interface imports (returns 0).
pub async fn java_interface_stub(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::debug!("java_interface_stub({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}) -> 0");

    Ok(0)
}
