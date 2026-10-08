//! KTF client.bin images that begin with their relocation table — 3 of 296 (`adf::is_relocation_prefixed`;
//! docs/report/0340 · 0455 · 0466).
//!
//! The data structures are the standard KTF AOT ones (class · method · field records, `JavaFullName`,
//! the vtable and instance layout), reached differently:
//! - no `WIPI_exe` export. The image header points at a class registry; the class records sit just
//!   below the GOT, and a class is found by its descriptor's name.
//! - a descriptor's parent and interfaces are name references `(index << 1) | 1` into the header's
//!   name table, linked here when the class is first loaded. Exception tables keep theirs
//!   (`JavaMethod::exception_class_matches`).
//! - one host interface, `MNInterface` (`MN_*` below), instead of `InitParam4` + `WIPI_JBInterface`.
//! - the thread context is passed in `fp` and read through the image's own cell at `+0x20`, and its
//!   C helpers run on a native stack at `[fp, #0x34]` (`enter`).
use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

use jvm::{ClassInstance, Jvm};
use jvm_types::FieldAccessFlags;
use wipi_types::ktf::{
    ExeInterfaceFunctions,
    java::{JavaClass as RawJavaClass, JavaClassDescriptor as RawJavaClassDescriptor, JavaFieldDefinition as RawJavaField},
};

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext, EmulatedFunction, JumpTo, ResultWriter, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, read_null_terminated_table, write_generic};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{
        SVC_CATEGORY_INIT, SVC_CATEGORY_RELOCATED,
        init::register_init_svc_handler,
        java::{
            interface::{
                get_field, get_java_method, java_array_new, java_check_type, java_class_load, java_new, java_throw, java_throw_instance,
                monitor_enter, monitor_exit, register_class,
            },
            jvm_support::{JavaClassDefinition, JavaVtable, KtfJvmSupport},
            resume_here,
        },
        svc_ids::InitSvcId,
    },
};

// Image header words, after relocation.
const HEADER_CLASS_REGISTRY: u32 = 0x00;
/// Where the image's constant data starts (its first table is the `MNInterface` name strings).
const HEADER_STRINGS: u32 = 0x0c;
const HEADER_NAMES: u32 = 0x10;
const HEADER_GOT_START: u32 = 0x18;
const HEADER_GOT_END: u32 = 0x1c;
/// The image reads the current thread context through this cell (`GOT[0x180]` points here).
const HEADER_THREAD_CELL: u32 = 0x20;
const HEADER_INIT: u32 = 0x24;

const CLASS_RECORD_SIZE: u32 = size_of::<RawJavaClass>() as u32;

// Thread context words the image reads off `fp`.
/// Scratch: a helper that calls into the runtime parks its Java `sp` here across the call.
const CONTEXT_SAVED_SP: u32 = 0x24;
const CONTEXT_FUNCTIONS: u32 = 0x30;
const CONTEXT_NATIVE_STACK: u32 = 0x34;
const CONTEXT_JVM: u32 = 0x38;
/// Below the entry `sp`, where the guest's Java frames start; its C helpers use the gap above.
// ponytail: a fixed gap — the helpers' frames are a few words deep, and a nested entry takes a new gap.
const NATIVE_STACK_GAP: u32 = 0x4000;

// SVC ids in `SVC_CATEGORY_RELOCATED` that are not `MNInterface` slots (those are their byte offset).
const SVC_GET_CLASS: u32 = 0x1000;
const SVC_RESTORE: u32 = 0x1001;
const SVC_MONITOR_ENTER: u32 = 0x1002;
const SVC_MONITOR_EXIT: u32 = 0x1003;

/// Six zero words at image `+0x480` that the image's call helpers jump through (`ldr rN, [pc, …];
/// mov pc, rN`) and the host fills: invoke · invoke native · monitorenter (synchronized method) ·
/// monitorexit · loop safepoint · monitorenter (synchronized block). docs/report/0466 §2.
const RUNTIME_WORDS: u32 = 0x480;
/// Thumb code put after the bss for the two invoke words and the safepoint. An invoke helper arrives
/// with `r0` = method record, `r1` = this and the caller's `r2, r3` pushed, and jumps to the body:
/// `pop {r2, r3}; push {r3}; ldr r3, [r0]; mov ip, r3; pop {r3}; bx ip` — then `bx lr`.
const TRAMPOLINES: [u16; 7] = [0xbc0c, 0xb408, 0x6803, 0x469c, 0xbc08, 0x4760, 0x4770];
const TRAMPOLINE_INVOKE: u32 = 0;
const TRAMPOLINE_RETURN: u32 = 12;
const MN_INTERFACE_SLOTS: u32 = 64;

pub async fn load(core: &mut ArmCore, jvm: &Jvm, data: &[u8], bss_size: u32) -> Result<u32> {
    let word = |index: usize| u32::from_le_bytes([data[index * 4], data[index * 4 + 1], data[index * 4 + 2], data[index * 4 + 3]]);
    let count = word(1) as usize;
    let image = &data[(2 + count) * 4..];

    let trampolines = (IMAGE_BASE + image.len() as u32 + bss_size).next_multiple_of(4);
    core.load(image, IMAGE_BASE, (trampolines - IMAGE_BASE) as usize + size_of_val(&TRAMPOLINES))?;
    write_generic(core, trampolines, TRAMPOLINES)?;
    // Two passes, as 0340 measured: the table, then the GOT (which the table does not cover).
    for index in 2..2 + count {
        relocate(core, IMAGE_BASE + word(index))?;
    }
    let got_start: u32 = read_generic(core, IMAGE_BASE + HEADER_GOT_START)?;
    let got_end: u32 = read_generic(core, IMAGE_BASE + HEADER_GOT_END)?;
    for address in (got_start..got_end).step_by(4) {
        relocate(core, address)?;
    }
    wie_jvm_support::guest_roots::add_region(core.id(), IMAGE_BASE, image.len() as u32 + bss_size);

    register_init_svc_handler(core, jvm)?;
    core.register_svc_handler(SVC_CATEGORY_RELOCATED, handle_svc, jvm)?;

    let words: [u32; 6] = read_generic(core, IMAGE_BASE + RUNTIME_WORDS)?;
    if words != [0; 6] {
        return Err(WieError::FatalError(format!(
            "Relocated image: runtime words at +0x480 are not empty: {words:#x?}"
        )));
    }
    let invoke = (trampolines + TRAMPOLINE_INVOKE) | 1;
    let monitor_enter = core.make_svc_stub(SVC_CATEGORY_RELOCATED, SVC_MONITOR_ENTER)?;
    let monitor_exit = core.make_svc_stub(SVC_CATEGORY_RELOCATED, SVC_MONITOR_EXIT)?;
    // ponytail: the safepoint returns at once — the core already preempts on an instruction budget.
    let safepoint = (trampolines + TRAMPOLINE_RETURN) | 1;
    write_generic(
        core,
        IMAGE_BASE + RUNTIME_WORDS,
        [invoke, invoke, monitor_enter, monitor_exit, safepoint, monitor_enter],
    )?;

    // `[fp, #0x30]` is copied into each try record, and a throw resumes at its `+4` — as for the
    // standard runtime's records, whose table the image supplies itself.
    let functions = Allocator::alloc(core, 8)?;
    let restore = core.make_svc_stub(SVC_CATEGORY_RELOCATED, SVC_RESTORE)?;
    write_generic(core, functions, [0, restore])?;
    let names: u32 = read_generic(core, IMAGE_BASE + HEADER_NAMES)?;
    KtfJvmSupport::set_relocated_abi(core, IMAGE_BASE + HEADER_THREAD_CELL, functions, names)?;

    adopt_string_literals(core, jvm).await?;

    // `init(param)` calls `param[0]("MNInterface", -1, -1)`, keeps the table and returns 0.
    let param = Allocator::alloc(core, 4)?;
    let get_interface = core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetInterface)?;
    write_generic(core, param, get_interface)?;
    let init: u32 = read_generic(core, IMAGE_BASE + HEADER_INIT)?;
    let result = core.run_function::<u32>(init, &[param]).await?;
    if result != 0 {
        return Err(WieError::FatalError(format!("Relocated image init failed with code {result:#x}")));
    }

    let functions = ExeInterfaceFunctions {
        fn_get_class: core.make_svc_stub(SVC_CATEGORY_RELOCATED, SVC_GET_CLASS)?,
        ..bytemuck::Zeroable::zeroed()
    };
    let ptr_functions = Allocator::alloc(core, size_of::<ExeInterfaceFunctions>() as u32)?;
    write_generic(core, ptr_functions, functions)?;

    Ok(ptr_functions)
}

/// The image's string literals are prebuilt objects: a `String` `{fields = self + 4 → [0x280, value, 0,
/// length]}` over a `char[]` `{self + 4 → [0, length, chars]}`. Two things differ from this JVM's
/// objects: their header word places the class at a fixed record (`char[]` the first, `String` the
/// second, `0x280 >> 5 = 0x14` = one `JavaClass` record) where the host's classes are wherever they
/// were made, and an object's class sits at `+4` — exactly where these compact literals start their fields.
/// The image only ever reads an object through its fields pointer, so each literal keeps its address
/// and fields and gets a header of the host's shape. 215 · 289 · 208 literals in the three images,
/// every one of that shape.
async fn adopt_string_literals(core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
    const STRING_SLOT: u32 = (5 * 4) << 5;
    const CHARS_SLOT: u32 = 0;

    let string_class = resolve(jvm, "java/lang/String").await?;
    let chars_class = resolve(jvm, "[C").await?;
    let string_slot = header_word(core, string_class)?;
    let chars_slot = header_word(core, chars_class)?;
    let start: u32 = read_generic(core, IMAGE_BASE + HEADER_STRINGS)?;
    let end: u32 = read_generic(core, IMAGE_BASE + HEADER_GOT_START)?;

    let mut count = 0;
    for address in (start..end - 16).step_by(4) {
        let [fields, slot, value, offset, length]: [u32; 5] = read_generic(core, address)?;
        if fields != address + 4 || slot != STRING_SLOT || offset != 0 {
            continue;
        }
        let Ok([chars_fields, chars_slot_word, chars_length]) = read_generic::<[u32; 3], _>(core, value) else {
            continue;
        };
        if chars_fields != value + 4 || chars_slot_word != CHARS_SLOT || chars_length != length {
            continue;
        }

        // `char[]`: its fields stay in place; the header moves out, since only the `String` points at it.
        write_generic(core, value + 4, chars_slot)?;
        let chars = Allocator::alloc(core, 8)?;
        write_generic(core, chars, [value + 4, chars_class])?;
        // `String`: code and data hold its address, so the header stays and the fields move out.
        let string_fields = Allocator::alloc(core, 16)?;
        write_generic(core, string_fields, [string_slot, chars, 0, length])?;
        write_generic(core, address, [string_fields, string_class])?;
        count += 1;
    }
    tracing::debug!("Adopted {count} string literals");

    Ok(())
}

fn header_word(core: &mut ArmCore, ptr_class: u32) -> Result<u32> {
    let class = JavaClassDefinition::from_raw(ptr_class, core);

    KtfJvmSupport::object_header(core, &class)
}

fn relocate(core: &mut ArmCore, address: u32) -> Result<()> {
    let value: u32 = read_generic(core, address)?;
    write_generic(core, address, value.wrapping_add(IMAGE_BASE))
}

pub fn get_mn_interface(core: &mut ArmCore) -> Result<u32> {
    let table = Allocator::alloc(core, MN_INTERFACE_SLOTS * 4)?;
    for slot in 0..MN_INTERFACE_SLOTS {
        let stub = core.make_svc_stub(SVC_CATEGORY_RELOCATED, slot * 4)?;
        write_generic(core, table + slot * 4, stub)?;
    }

    Ok(table)
}

async fn handle_svc(core: &mut ArmCore, jvm: &mut Jvm, id: SvcId) -> Result<JumpTo> {
    dispatch(core, jvm, id).await?;

    Ok(resume_here(core))
}

async fn dispatch(core: &mut ArmCore, jvm: &mut Jvm, id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    match id.0 {
        SVC_GET_CLASS => EmulatedFunction::call(&get_class, core, jvm).await?.write(core, lr),
        SVC_RESTORE => restore(core),
        SVC_MONITOR_ENTER => EmulatedFunction::call(&monitor_enter, core, jvm).await?.write(core, lr),
        SVC_MONITOR_EXIT => EmulatedFunction::call(&monitor_exit, core, jvm).await?.write(core, lr),
        slot => {
            // `new` and `throw` get a frame of their own, as in `init::handle_init_svc`.
            let frame = matches!(
                slot,
                MN_THROW | MN_THROW_INSTANCE | MN_NEW | MN_ARRAY_NEW | MN_PRIMITIVE_ARRAY_NEW | MN_MULTI_ARRAY_NEW
            );
            if frame {
                wie_jvm_support::guest_roots::stress_collect(jvm, core.id());
                jvm.push_native_frame();
            }
            tracing::trace!(
                "MNInterface +{slot:#x}({:#x}, {:#x}, {:#x}) from {lr:#x}",
                core.read_param(0)?,
                core.read_param(1)?,
                core.read_param(2)?
            );
            let result = call_slot(core, jvm, slot, lr).await;
            if frame {
                jvm.pop_frame();
            }
            result
        }
    }
}

// `MNInterface` slots, by byte offset. Each meaning is read off the image's own wrapper for it
// (docs/report/0466 §2 — the wrapper addresses there).
/// `(class name, 0)` — throw a new instance (the wrappers' failure path: `("java/lang/Error", 0)`).
const MN_THROW: u32 = 0x20;
/// `(exception, 0)` — `athrow`.
const MN_THROW_INSTANCE: u32 = 0x24;
/// `(class)` → instance.
const MN_NEW: u32 = 0x38;
/// `(array class, length)` → array; the class comes from `MN_ARRAY_CLASS`.
const MN_ARRAY_NEW: u32 = 0x3c;
/// `(out, class name)` → 0 and the class in `*out`.
const MN_CLASS_LOAD: u32 = 0x40;
/// `(instance)` → its class.
const MN_CLASS_OF: u32 = 0x44;
/// `(class, instance)` → is-instance byte.
const MN_INSTANCE_OF: u32 = 0x48;
/// `(class, field name)` → field record.
const MN_GET_FIELD: u32 = 0x54;
/// `(class)` — run `<clinit>` once; the image then sets nothing, it tests the class's `+0x12 & 8`.
const MN_CLASS_INIT: u32 = 0x60;
/// `(class, method name)` → method record.
const MN_GET_METHOD: u32 = 0x64;
/// `(element class)` → its array class.
const MN_ARRAY_CLASS: u32 = 0x6c;
/// `(newarray atype × 4, length)` → array. The image passes 0x10 · 0x20 · 0x24 · 0x28 (191 sites in
/// `1d5831e42a8a`) — `T_BOOLEAN`, `T_BYTE`, `T_SHORT`, `T_INT` times four.
const MN_PRIMITIVE_ARRAY_NEW: u32 = 0x70;
/// `(array class, dimensions, lengths)` — `multianewarray`; `lengths` is the guest stack where the
/// helper pushed them, outermost first.
const MN_MULTI_ARRAY_NEW: u32 = 0x74;

async fn call_slot(core: &mut ArmCore, jvm: &mut Jvm, slot: u32, lr: u32) -> Result<()> {
    // These take a class, and the image may hand over its own class record straight from a code cell —
    // one no name lookup has linked yet.
    if matches!(slot, MN_NEW | MN_GET_FIELD | MN_GET_METHOD | MN_CLASS_INIT | MN_ARRAY_CLASS) {
        ensure_linked(core, jvm, core.read_param(0)?).await?;
    }

    match slot {
        MN_THROW => EmulatedFunction::call(&java_throw, core, jvm).await?.write(core, lr),
        MN_THROW_INSTANCE => EmulatedFunction::call(&java_throw_instance, core, jvm).await?.write(core, lr),
        MN_NEW => EmulatedFunction::call(&java_new, core, jvm).await?.write(core, lr),
        MN_ARRAY_NEW => EmulatedFunction::call(&java_array_new, core, jvm).await?.write(core, lr),
        MN_PRIMITIVE_ARRAY_NEW => EmulatedFunction::call(&primitive_array_new, core, jvm).await?.write(core, lr),
        MN_CLASS_LOAD => EmulatedFunction::call(&java_class_load, core, jvm).await?.write(core, lr),
        MN_CLASS_OF => EmulatedFunction::call(&class_of, core, &mut ()).await?.write(core, lr),
        MN_INSTANCE_OF => EmulatedFunction::call(&instance_of, core, jvm).await?.write(core, lr),
        MN_GET_FIELD => EmulatedFunction::call(&get_field, core, &mut ()).await?.write(core, lr),
        MN_CLASS_INIT => EmulatedFunction::call(&class_init, core, jvm).await?.write(core, lr),
        MN_GET_METHOD => EmulatedFunction::call(&get_java_method, core, &mut ()).await?.write(core, lr),
        MN_ARRAY_CLASS => EmulatedFunction::call(&array_class, core, jvm).await?.write(core, lr),
        MN_MULTI_ARRAY_NEW => EmulatedFunction::call(&multi_array_new, core, jvm).await?.write(core, lr),
        _ => {
            let args = (0..4).map(|x| core.read_param(x)).collect::<Result<Vec<_>>>()?;
            Err(WieError::Unimplemented(format!("MNInterface +{slot:#x} from {lr:#x}, args {:#x?}", args)))
        }
    }
}

async fn primitive_array_new(core: &mut ArmCore, jvm: &mut Jvm, atype: u32, length: u32) -> Result<u32> {
    let element = match atype / 4 {
        4 => b'Z',
        5 => b'C',
        6 => b'F',
        7 => b'D',
        8 => b'B',
        9 => b'S',
        10 => b'I',
        11 => b'J',
        _ => return Err(WieError::FatalError(format!("Relocated image: unknown newarray type {atype:#x}"))),
    };

    java_array_new(core, jvm, element as u32, length).await
}

async fn class_of(core: &mut ArmCore, _: &mut (), ptr_instance: u32) -> Result<u32> {
    read_generic(core, ptr_instance + 4)
}

async fn instance_of(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32, ptr_instance: u32) -> Result<u32> {
    java_check_type(core, jvm, ptr_class, ptr_instance, 0).await
}

async fn class_init(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<()> {
    register_class(core, jvm, ptr_class).await?;

    let flags: u16 = read_generic(core, ptr_class + 0x12)?;
    write_generic(core, ptr_class + 0x12, flags | 8)
}

async fn array_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_element: u32) -> Result<u32> {
    let element = JavaClassDefinition::from_raw(ptr_element, core).name()?;
    let name = if element.starts_with('[') {
        format!("[{element}")
    } else {
        format!("[L{element};")
    };

    resolve(jvm, &name).await
}

async fn multi_array_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32, dimensions: u32, ptr_lengths: u32) -> Result<u32> {
    let name = JavaClassDefinition::from_raw(ptr_class, core).name()?;
    let lengths = (0..dimensions)
        .map(|index| read_generic(core, ptr_lengths + index * 4))
        .collect::<Result<Vec<u32>>>()?;
    tracing::trace!("multi_array_new({name}, {lengths:?})");

    match new_array(jvm, &name, &lengths).await {
        Ok(array) => Ok(KtfJvmSupport::class_instance_raw(&array)),
        Err(x) => Err(wie_jvm_support::JvmSupport::to_wie_err(jvm, x).await),
    }
}

#[allow(clippy::double_must_use)] // temporary until https://github.com/rust-lang/rust-clippy/issues/17529 fix lands
#[async_recursion::async_recursion]
async fn new_array(jvm: &Jvm, name: &str, lengths: &[u32]) -> jvm::Result<Box<dyn ClassInstance>> {
    let mut array = jvm.instantiate_array(&name[1..], lengths[0] as _).await?;
    if lengths.len() > 1 {
        for index in 0..lengths[0] as usize {
            let element = new_array(jvm, &name[1..], &lengths[1..]).await?;
            jvm.store_array(&mut array, index, vec![element]).await?;
        }
    }

    Ok(array)
}

async fn resolve(jvm: &Jvm, name: &str) -> Result<u32> {
    match jvm.resolve_class(name).await {
        Ok(class) => KtfJvmSupport::class_definition_raw(&*class.definition),
        Err(x) => Err(wie_jvm_support::JvmSupport::to_wie_err(jvm, x).await),
    }
}

/// The class loader's `get_class`: the class record named `ptr_name`, linked, or 0.
async fn get_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_name: u32) -> Result<u32> {
    let name = read_null_terminated_string_bytes(core, ptr_name)?;
    // The image's array class records are not handed out: the JVM makes its own array classes.
    if name.first() == Some(&b'[') {
        return Ok(0);
    }

    let registry: u32 = read_generic(core, IMAGE_BASE + HEADER_CLASS_REGISTRY)?;
    let count: u32 = read_generic(core, registry + 4)?;
    let got_start: u32 = read_generic(core, IMAGE_BASE + HEADER_GOT_START)?;
    for ptr_class in (got_start - count * CLASS_RECORD_SIZE..got_start).step_by(CLASS_RECORD_SIZE as usize) {
        let raw: RawJavaClass = read_generic(core, ptr_class)?;
        if raw.ptr_next != ptr_class + 4 {
            return Err(WieError::FatalError(format!("Relocated image: no class record at {ptr_class:#x}")));
        }
        let descriptor: RawJavaClassDescriptor = read_generic(core, raw.ptr_descriptor)?;
        if read_null_terminated_string_bytes(core, descriptor.ptr_name)? == name {
            link(core, jvm, ptr_class).await?;
            return Ok(ptr_class);
        }
    }

    Ok(0)
}

/// Load an image class record the guest holds directly, the way a name lookup would (`get_class`).
async fn ensure_linked(core: &mut ArmCore, jvm: &Jvm, ptr_class: u32) -> Result<()> {
    let got_start: u32 = read_generic(core, IMAGE_BASE + HEADER_GOT_START)?;
    if !(IMAGE_BASE..got_start).contains(&ptr_class) {
        return Ok(());
    }
    let raw: RawJavaClass = read_generic(core, ptr_class)?;
    if raw.ptr_vtable != 0 {
        return Ok(());
    }
    let name = JavaClassDefinition::from_raw(ptr_class, core).name()?;

    resolve(jvm, &name).await.map(|_| ())
}

async fn link(core: &mut ArmCore, jvm: &Jvm, ptr_class: u32) -> Result<()> {
    let raw: RawJavaClass = read_generic(core, ptr_class)?;
    if raw.ptr_vtable != 0 {
        return Ok(());
    }

    let mut descriptor: RawJavaClassDescriptor = read_generic(core, raw.ptr_descriptor)?;
    descriptor.ptr_parent_class = resolve_reference(core, jvm, descriptor.ptr_parent_class).await?;

    // A class's instance field offsets start at 0 and its `fields_size` counts its own fields only
    // (a `Card` subclass in `1d5831e42a8a` puts its first field at 0); the parent's fields come first.
    let base = if descriptor.ptr_parent_class == 0 {
        0
    } else {
        JavaClassDefinition::from_raw(descriptor.ptr_parent_class, core).field_size()? as u32
    };
    if base != 0 && descriptor.ptr_fields_or_element_type != 0 {
        for ptr_field in read_null_terminated_table(core, descriptor.ptr_fields_or_element_type)? {
            let field: RawJavaField = read_generic(core, ptr_field)?;
            if !FieldAccessFlags::from_bits_truncate(field.access_flags as u16).contains(FieldAccessFlags::STATIC) {
                write_generic(core, ptr_field + 12, field.offset_or_value + base)?;
            }
        }
    }
    descriptor.fields_size += base as u16;
    write_generic(core, raw.ptr_descriptor, descriptor)?;
    for index in 0..descriptor.interface_count as u32 {
        let address = descriptor.ptr_interfaces + index * 4;
        let reference = read_generic(core, address)?;
        let ptr_interface = resolve_reference(core, jvm, reference).await?;
        write_generic(core, address, ptr_interface)?;
    }

    let vtable = JavaVtable::new(core, &JavaClassDefinition::from_raw(ptr_class, core))?;
    write_generic(core, ptr_class + 12, vtable.ptr_raw)?;
    write_generic(core, ptr_class + 16, vtable.len()? as u16)
}

/// A class reference: a record pointer, or `(index << 1) | 1` into the header's name table.
async fn resolve_reference(core: &mut ArmCore, jvm: &Jvm, reference: u32) -> Result<u32> {
    if reference & 1 == 0 {
        return Ok(reference);
    }
    let names: u32 = read_generic(core, IMAGE_BASE + HEADER_NAMES)?;
    let ptr_name: u32 = read_generic(core, names + (reference >> 1) * 4)?;
    let name = String::from_utf8_lossy(&read_null_terminated_string_bytes(core, ptr_name)?).into_owned();

    resolve(jvm, &name).await
}

/// `longjmp` to a try record (`r0` = record `+0x18`, `r1` = the handler's index): the try helper
/// saved `sp, lr, r4–r7, r8, sb, -, sl` there and returns the index to its caller, which dispatches on it.
fn restore(core: &mut ArmCore) -> Result<()> {
    let mut context = core.save_context();
    let base = context.r0;
    let words: [u32; 10] = read_generic(core, base)?;
    context.r0 = context.r1;
    context.sp = words[0];
    context.lr = words[1];
    [context.r4, context.r5, context.r6, context.r7, context.r8, context.sb] = words[2..8].try_into().unwrap();
    context.sl = words[9];
    context.pc = words[1] & !1;
    context.cpsr = if words[1] & 1 == 1 { context.cpsr | 0x20 } else { context.cpsr & !0x20 };
    core.restore_context(&context);

    Ok(())
}

pub(crate) struct Entry {
    context: ArmCoreContext,
    ptr_thread_context: u32,
    saved_sp: u32,
    native_stack: u32,
}

/// Before the host runs a method of a relocated image: `fp` = the thread context, filled with what
/// the image reads off it, and the native stack set at the entry `sp` with the Java frames below it.
pub(crate) fn enter(core: &mut ArmCore) -> Result<Option<Entry>> {
    let Some(abi) = KtfJvmSupport::relocated_abi(core)? else {
        return Ok(None);
    };
    let ptr_thread_context = KtfJvmSupport::current_thread_context(core)?;
    let context = core.save_context();
    // Both are per-entry: a helper parked in the runtime (`new` → `<clinit>`) may be the caller of this
    // entry, and its parked `sp` must survive the nested helpers.
    let saved_sp: u32 = read_generic(core, ptr_thread_context + CONTEXT_SAVED_SP)?;
    let native_stack = read_generic(core, ptr_thread_context + CONTEXT_NATIVE_STACK)?;

    write_generic(core, ptr_thread_context + CONTEXT_FUNCTIONS, abi.functions)?;
    write_generic(core, ptr_thread_context + CONTEXT_NATIVE_STACK, context.sp)?;
    write_generic(core, ptr_thread_context + CONTEXT_JVM, abi.ptr_jvm_context)?;
    // Called from the image's C helpers, `sp` is on the native stack, above the caller's Java frames,
    // and the new frames go below the caller's Java `sp`. The helpers keep it in one of two places:
    // the four that may run Java (`new` → `<clinit>`) park it at `+0x24`; the 24 that resolve a class,
    // field or method (`MN_CLASS_LOAD` → the class loader, which runs here) push it under their return
    // address at the native stack top. Without the second, `sp - NATIVE_STACK_GAP` lands on the
    // outermost Java frame (docs/report/0476).
    let java_sp = match saved_sp {
        0 => context.sp.min(pushed_java_sp(core, native_stack, context.sp)?.unwrap_or(context.sp)),
        parked => context.sp.min(parked),
    };
    core.restore_context(&ArmCoreContext {
        fp: ptr_thread_context,
        sp: java_sp - NATIVE_STACK_GAP,
        ..context.clone()
    });

    Ok(Some(Entry {
        context,
        ptr_thread_context,
        saved_sp,
        native_stack,
    }))
}

/// The Java `sp` a resolve helper pushed at `[native stack top - 8]` (`push {r2, lr}` · `push {r1, r2,
/// lr}` · `push {r3, lr}`, the old `sp` in `r2`/`r3`) — if `sp` is on that native stack and the word is
/// where this entry's Java frames are. The four throw helpers save nothing; they get `None` — unless
/// an earlier resolve call left its word there, which passes if the new frame still fits the stack.
fn pushed_java_sp(core: &ArmCore, native_stack: u32, sp: u32) -> Result<Option<u32>> {
    let java_top = native_stack.wrapping_sub(NATIVE_STACK_GAP);
    if !(java_top < sp && sp <= native_stack) {
        return Ok(None);
    }
    let pushed: u32 = read_generic(core, native_stack - 8)?;
    // The new frame goes `NATIVE_STACK_GAP` below `pushed`; it must not leave the thread's stack.
    let bottom = core.current_stack_base().unwrap_or(0).saturating_add(NATIVE_STACK_GAP);

    // ponytail: 1 MB = one guest thread stack (wie-core-arm `thread.rs`); a stale word outside it is ignored.
    Ok((pushed <= java_top && java_top - pushed < 0x10_0000 && pushed >= bottom).then_some(pushed))
}

/// Whether a try record belongs to a frame of the innermost host entry: its frames lie below that
/// entry's native stack top (`enter`), an outer entry's above it.
pub(crate) fn entry_owns_record(core: &ArmCore, ptr_record: u32) -> Result<bool> {
    let ptr_thread_context = KtfJvmSupport::current_thread_context(core)?;
    let native_stack: u32 = read_generic(core, ptr_thread_context + CONTEXT_NATIVE_STACK)?;
    let record_sp: u32 = read_generic(core, ptr_record + 0x18)?;

    Ok(record_sp < native_stack)
}

pub(crate) fn leave(core: &mut ArmCore, entry: Entry) -> Result<()> {
    write_generic(core, entry.ptr_thread_context + CONTEXT_SAVED_SP, entry.saved_sp)?;
    write_generic(core, entry.ptr_thread_context + CONTEXT_NATIVE_STACK, entry.native_stack)?;
    core.restore_context(&entry.context);

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use bytemuck::Zeroable;
    use jvm::{ClassInstance, Jvm, runtime::JavaLangString};
    use wipi_types::ktf::java::{JavaClass as RawJavaClass, JavaClassDescriptor as RawJavaClassDescriptor};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore, RUN_FUNCTION_LR};
    use wie_util::{ByteWrite, Result, read_generic, write_generic};

    use crate::{
        emulator::IMAGE_BASE,
        runtime::{
            SVC_CATEGORY_RELOCATED,
            java::jvm_support::{JavaClassDefinition, JavaClassInstance, KtfJvmSupport, KtfJvmThreadContext},
        },
    };

    use super::{CONTEXT_NATIVE_STACK, CONTEXT_SAVED_SP, NATIVE_STACK_GAP, SVC_RESTORE, adopt_string_literals, enter, get_class, handle_svc, leave};

    const IMAGE_SIZE: u32 = 0x1000;
    const GOT: u32 = IMAGE_BASE + 0xf00;

    // A JVM over an empty image whose header has the words the loader reads, and a thread context.
    async fn init(system: &mut System) -> Result<(Jvm, ArmCore, u32)> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x10000)?;
        context.sp = stack + 0x10000;
        core.restore_context(&context);

        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;
        KtfJvmSupport::set_current_thread_context(&mut core, ptr_thread_context)?;
        let (jvm, _) = KtfJvmSupport::init(&mut core, system, None).await?;

        core.load(&[0; IMAGE_SIZE as usize], IMAGE_BASE, IMAGE_SIZE as usize)?;
        write_generic(&mut core, IMAGE_BASE + 0x0c, IMAGE_BASE + 0x100)?; // constant data
        write_generic(&mut core, IMAGE_BASE + 0x10, IMAGE_BASE + 0x80)?; // names
        write_generic(&mut core, IMAGE_BASE + 0x18, GOT)?;
        core.register_svc_handler(SVC_CATEGORY_RELOCATED, handle_svc, &jvm)?;
        let functions = Allocator::alloc(&mut core, 8)?;
        let restore = core.make_svc_stub(SVC_CATEGORY_RELOCATED, SVC_RESTORE)?;
        write_generic(&mut core, functions, [0, restore])?;
        KtfJvmSupport::set_relocated_abi(&mut core, IMAGE_BASE + 0x20, functions, IMAGE_BASE + 0x80)?;

        Ok((jvm, core, ptr_thread_context))
    }

    // `run!(|jvm, core, thread_context| { … })` — the test body on a guest thread over `init`.
    macro_rules! run {
        (|$jvm:ident, $core:ident, $thread_context:ident| $body:block) => {{
            let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
            let done = Arc::new(AtomicBool::new(false));
            let done_clone = done.clone();
            let mut system_clone = system.clone();
            system.spawn(async move || {
                #[allow(unused_mut, unused_variables)]
                let (mut $jvm, mut $core, $thread_context) = init(&mut system_clone).await?;
                $body
                done_clone.store(true, Ordering::Relaxed);
                Ok(())
            });
            while !done.load(Ordering::Relaxed) {
                system.tick()?;
            }
            Ok(())
        }};
    }

    // A `Card` subclass puts its first field at 0 and names its parent `(index << 1) | 1`.
    #[test]
    fn a_class_is_linked_under_its_parent_by_name() -> Result<()> {
        run!(|jvm, core, thread_context| {
            let names = IMAGE_BASE + 0x80;
            let strings = IMAGE_BASE + 0x400;
            core.write_bytes(strings, b"java/lang/Thread\0Foo\0\x01I+x\0")?;
            write_generic(&mut core, names + 3 * 4, strings)?;

            let ptr_class = GOT - size_of::<RawJavaClass>() as u32;
            let descriptor = IMAGE_BASE + 0x200;
            let fields = IMAGE_BASE + 0x280;
            let field = IMAGE_BASE + 0x290;
            write_generic(&mut core, IMAGE_BASE, IMAGE_BASE + 0x300)?; // the registry: [end, count]
            write_generic(&mut core, IMAGE_BASE + 0x300, [0u32, 1])?;
            write_generic(
                &mut core,
                ptr_class,
                RawJavaClass {
                    ptr_next: ptr_class + 4,
                    unk1: 0x500,
                    ptr_descriptor: descriptor,
                    ptr_vtable: 0,
                    vtable_count: 0,
                    unk_flag: 0x8000,
                },
            )?;
            write_generic(
                &mut core,
                descriptor,
                RawJavaClassDescriptor {
                    ptr_name: strings + 17,
                    unk1: 0,
                    ptr_parent_class: (3 << 1) | 1,
                    ptr_methods: IMAGE_BASE + 0x2f0,
                    ptr_interfaces: 0,
                    ptr_fields_or_element_type: fields,
                    method_count: 0,
                    fields_size: 4,
                    access_flag: 0x21,
                    interface_count: 0,
                    unk7: 1,
                    unk8: 0,
                },
            )?;
            write_generic(&mut core, fields, [field, 0])?;
            write_generic(&mut core, field, [0u32, ptr_class, strings + 21, 0])?;

            let ptr_name = Allocator::alloc(&mut core, 4)?;
            core.write_bytes(ptr_name, b"Foo\0")?;
            assert_eq!(get_class(&mut core, &mut jvm, ptr_name).await?, ptr_class);

            let parent = jvm.resolve_class("java/lang/Thread").await.unwrap();
            let parent = JavaClassDefinition::from_raw(KtfJvmSupport::class_definition_raw(&*parent.definition)?, &core);
            let descriptor: RawJavaClassDescriptor = read_generic(&core, descriptor)?;
            assert_eq!(descriptor.ptr_parent_class, parent.ptr_raw);
            let base = parent.field_size()? as u32;
            assert_ne!(base, 0);
            assert_eq!(read_generic::<u32, _>(&core, field + 12)?, base, "own fields follow the parent's");
            assert_eq!(descriptor.fields_size as u32, base + 4);
            assert_ne!(JavaClassDefinition::from_raw(ptr_class, &core).ptr_vtable()?, 0);
        })
    }

    #[test]
    fn a_string_literal_reads_as_a_string() -> Result<()> {
        run!(|jvm, core, thread_context| {
            // `String {self + 4 → [slot 5, value, 0, 2]}` over `char[] {self + 4 → [slot 0, 2, 'A', 'B']}`.
            let string = IMAGE_BASE + 0x100;
            let chars = IMAGE_BASE + 0x140;
            write_generic(&mut core, string, [string + 4, (5 * 4) << 5, chars, 0, 2])?;
            write_generic(&mut core, chars, [chars + 4, 0, 2, 0x0042_0041])?;

            adopt_string_literals(&mut core, &jvm).await?;

            let instance: Box<dyn ClassInstance> = Box::new(JavaClassInstance::from_raw(string, &core));
            assert_eq!(JavaLangString::to_rust_string(&jvm, &instance).await.unwrap(), "AB");
        })
    }

    // A `new` helper parks the Java `sp` and calls the runtime on the native stack; a `<clinit>` that
    // call runs must go below the parked frames, not into them.
    #[test]
    fn a_nested_entry_goes_below_the_parked_java_frames() -> Result<()> {
        run!(|jvm, core, ptr_thread_context| {
            let native_sp = core.save_context().sp;
            let parked = native_sp - 0x1000;
            write_generic(&mut core, ptr_thread_context + CONTEXT_SAVED_SP, parked)?;
            write_generic(&mut core, ptr_thread_context + CONTEXT_NATIVE_STACK, native_sp + 0x40)?;

            let entry = enter(&mut core)?.unwrap();
            let context = core.save_context();
            assert_eq!(context.fp, ptr_thread_context);
            assert_eq!(context.sp, parked - NATIVE_STACK_GAP);
            assert_eq!(read_generic::<u32, _>(&core, ptr_thread_context + CONTEXT_NATIVE_STACK)?, native_sp);

            leave(&mut core, entry)?;
            assert_eq!(core.save_context().sp, native_sp);
            assert_eq!(
                read_generic::<u32, _>(&core, ptr_thread_context + CONTEXT_NATIVE_STACK)?,
                native_sp + 0x40
            );
            assert_eq!(read_generic::<u32, _>(&core, ptr_thread_context + CONTEXT_SAVED_SP)?, parked);
        })
    }

    // A resolve helper parks nothing: it pushes the Java `sp` under its return address at the native
    // stack top and calls the runtime there. `MN_CLASS_LOAD` runs the class loader, whose frames must
    // go below the caller's Java frames — `sp - NATIVE_STACK_GAP` is inside the outermost one.
    #[test]
    fn a_nested_entry_goes_below_the_java_sp_a_resolve_helper_pushed() -> Result<()> {
        run!(|jvm, core, ptr_thread_context| {
            let native_top = core.save_context().sp;
            let java_sp = native_top - NATIVE_STACK_GAP - 0xd8;
            write_generic(&mut core, native_top - 8, [java_sp, RUN_FUNCTION_LR])?; // push {r2, lr}
            write_generic(&mut core, ptr_thread_context + CONTEXT_SAVED_SP, 0u32)?;
            write_generic(&mut core, ptr_thread_context + CONTEXT_NATIVE_STACK, native_top)?;
            let mut context = core.save_context();
            context.sp = native_top - 0x1c;
            core.restore_context(&context);

            let entry = enter(&mut core)?.unwrap();
            assert_eq!(core.save_context().sp, java_sp - NATIVE_STACK_GAP);

            leave(&mut core, entry)?;
            assert_eq!(core.save_context().sp, native_top - 0x1c);
        })
    }

    // A word at the native stack top that is not a Java `sp` of this entry (a throw helper pushes
    // nothing there) is ignored: the frames go where they went before resolve helpers were read.
    #[test]
    fn a_nested_entry_ignores_a_word_that_is_not_a_java_sp() -> Result<()> {
        run!(|jvm, core, ptr_thread_context| {
            let native_top = core.save_context().sp;
            let java_top = native_top - NATIVE_STACK_GAP;
            write_generic(&mut core, ptr_thread_context + CONTEXT_SAVED_SP, 0u32)?;
            write_generic(&mut core, ptr_thread_context + CONTEXT_NATIVE_STACK, native_top)?;
            let mut context = core.save_context();
            context.sp = native_top - 0x1c;
            core.restore_context(&context);

            for word in [RUN_FUNCTION_LR, java_top + 4, java_top - 0x10_0000] {
                write_generic(&mut core, native_top - 8, word)?;
                let entry = enter(&mut core)?.unwrap();
                assert_eq!(core.save_context().sp, native_top - 0x1c - NATIVE_STACK_GAP, "{word:#x}");
                leave(&mut core, entry)?;
            }
        })
    }

    // A stale Java `sp` near the bottom of the thread's stack would put the new frames below it.
    #[test]
    fn a_nested_entry_stays_inside_the_thread_stack() -> Result<()> {
        run!(|jvm, core, ptr_thread_context| {
            let _thread = core.run_in_thread(|| async { Ok(()) })?;
            let id = *core.get_thread_ids().iter().max().unwrap();
            let _guard = core.enter_thread_context(id);
            let native_top = core.save_context().sp;
            let base = core.current_stack_base().unwrap();
            write_generic(&mut core, ptr_thread_context + CONTEXT_SAVED_SP, 0u32)?;
            write_generic(&mut core, ptr_thread_context + CONTEXT_NATIVE_STACK, native_top)?;
            let mut context = core.save_context();
            context.sp = native_top - 0x1c;
            core.restore_context(&context);

            for (pushed, sp) in [(base + 0x100, native_top - 0x1c - NATIVE_STACK_GAP), (base + NATIVE_STACK_GAP, base)] {
                write_generic(&mut core, native_top - 8, pushed)?;
                let entry = enter(&mut core)?.unwrap();
                assert_eq!(core.save_context().sp, sp, "{pushed:#x}");
                leave(&mut core, entry)?;
            }
        })
    }

    // A caught throw resumes after the try helper's call with the handler index in r0 — and keeps
    // running there: the SVC handler's own return must not go back to `lr`.
    #[test]
    fn a_catch_resumes_where_the_try_helper_returned() -> Result<()> {
        run!(|jvm, core, thread_context| {
            let code = IMAGE_BASE + 0x500;
            core.write_bytes(code, &[0x2a, 0x30, 0x00, 0xbd])?; // adds r0, #42; pop {pc}
            let stack = Allocator::alloc(&mut core, 0x40)?;
            write_generic(&mut core, stack + 0x20, RUN_FUNCTION_LR)?;
            // The record from +0x18: sp, lr, r4–r7, r8, sb, (exception), sl.
            let record = Allocator::alloc(&mut core, 0x40)?;
            write_generic(&mut core, record + 0x18, [stack + 0x20, code | 1, 4, 5, 6, 7, 8, 9, 0, 10])?;

            let [_, restore]: [u32; 2] = read_generic(&core, KtfJvmSupport::relocated_abi(&core)?.unwrap().functions)?;
            let result = core.run_function::<u32>(restore, &[record + 0x18, 7]).await?;
            assert_eq!(result, 7 + 42);
        })
    }
}
