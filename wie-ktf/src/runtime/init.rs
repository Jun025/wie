use alloc::{format, string::String};
use core::mem::size_of;
use jvm::Jvm;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use wipi_types::ktf::{ExeInterface, ExeInterfaceFunctions, InitParam0, InitParam3, InitParam4, WipiExe};

use crate::{
    adf::{is_relocation_prefixed, parse_bss_size},
    emulator::IMAGE_BASE,
    runtime::{
        SVC_CATEGORY_INIT,
        java::interface::{get_wipi_jb_interface, java_array_new, java_check_type, java_class_load, java_new, java_throw, java_throw_instance},
        relocated,
        svc_ids::InitSvcId,
        wipi_c::{interface::get_wipic_knl_interface, register_wipic_svc_handler},
    },
};

pub fn register_init_svc_handler(core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_INIT, handle_init_svc, jvm)
}

async fn handle_init_svc(core: &mut ArmCore, jvm: &mut Jvm, id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;
    let id = InitSvcId::try_from(id)?;

    // The guest's `new`s and `throw`s each get a frame of their own: `instantiate_class` roots what it
    // makes in the top frame, and the guest code calling here runs under a host call that, for a game
    // loop, never returns. The guest root scan keeps what the guest goes on holding.
    let frame = matches!(
        id,
        InitSvcId::JavaThrow | InitSvcId::JavaThrowInstance | InitSvcId::JavaNew | InitSvcId::JavaArrayNew
    );
    if frame {
        wie_jvm_support::guest_roots::stress_collect(jvm, core.id());
        jvm.push_native_frame();
    }
    let result = async {
        match id {
            InitSvcId::GetInterface => get_interface(core, core.read_param(0)?).await?.write(core, lr),
            InitSvcId::JavaThrow => EmulatedFunction::call(&java_throw, core, jvm).await?.write(core, lr),
            InitSvcId::JavaThrowInstance => EmulatedFunction::call(&java_throw_instance, core, jvm).await?.write(core, lr),
            InitSvcId::JavaCheckType => EmulatedFunction::call(&java_check_type, core, jvm).await?.write(core, lr),
            InitSvcId::JavaNew => EmulatedFunction::call(&java_new, core, jvm).await?.write(core, lr),
            InitSvcId::JavaArrayNew => EmulatedFunction::call(&java_array_new, core, jvm).await?.write(core, lr),
            InitSvcId::JavaClassLoad => EmulatedFunction::call(&java_class_load, core, jvm).await?.write(core, lr),
            InitSvcId::Alloc => EmulatedFunction::call(&alloc, core, &mut ()).await?.write(core, lr),
        }
    }
    .await;
    if frame {
        jvm.pop_frame();
    }
    result
}

pub async fn load_native(
    core: &mut ArmCore,
    system: &mut System,
    jvm: &Jvm,
    filename: &str,
    data: &[u8],
    ptr_jvm_context: u32,
    ptr_current_jvm_thread_context: u32,
) -> Result<u32> {
    let bss_size = parse_bss_size(filename)?;
    if is_relocation_prefixed(data, bss_size) {
        return relocated::load(core, jvm, data, bss_size).await;
    }

    core.load(data, IMAGE_BASE, data.len() + bss_size as usize)?;
    // Guest code may keep an object pointer in a global; the GC root scan reads the image. There is
    // no section table to tell code from data, so it reads all of it — a code word that looks like a
    // pointer into an instance only keeps that instance one collection longer.
    wie_jvm_support::guest_roots::add_region(core.id(), IMAGE_BASE, data.len() as u32 + bss_size);

    // Patterns target instruction encodings, which the guest self-rebase at
    // IMAGE_BASE+1 doesn't rewrite — so installing here is sound and skips a
    // re-scan after relocation. Hash-matched entries take priority over
    // hash-less generic ones; only one entry is installed because each install
    // claims fresh SVC categories from a fixed base and they would collide.
    //
    // The scan range covers the whole loaded image because KTF binaries don't
    // expose a code/metadata boundary at this point. Safety relies on the
    // patterns being long enough (and `{exit_b}` strict enough) that a
    // metadata-region collision is implausible; tighten patterns rather than
    // narrow the range if that ever becomes false.
    wie_core_arm::install_binary_patches(core, data, &[(IMAGE_BASE, data.len() as u32)])?;

    // The KTF SDK runtime asks for this interface by name at init; an image without it was not built
    // on that runtime (the repo's own keydraw fixture — `wipi` crate, packed graphics context).
    let sdk_runtime = data.windows(22).any(|w| w == b"WIPICX_incMemInterface");
    register_wipic_svc_handler(core, system, jvm, sdk_runtime)?;
    register_init_svc_handler(core, jvm)?;

    tracing::debug!("Loaded at {IMAGE_BASE:#x}, size {:#x}, bss {bss_size:#x}", data.len());

    let wipi_exe = core.run_function(IMAGE_BASE + 1, &[bss_size]).await?;
    tracing::debug!("Got wipi_exe {wipi_exe:#x}");

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { unk: 0 })?;

    let param_3 = InitParam3 {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        boolean: b'Z' as u32,
        char: b'C' as u32,
        float: b'F' as u32,
        double: b'D' as u32,
        byte: b'B' as u32,
        short: b'S' as u32,
        int: b'I' as u32,
        long: b'J' as u32,
    };

    let ptr_param_3 = Allocator::alloc(core, size_of::<InitParam3>() as u32)?;
    write_generic(core, ptr_param_3, param_3)?;

    let param_4 = init_param_4(core)?;

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let wipi_exe: WipiExe = read_generic(core, wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    tracing::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_current_jvm_thread_context, ptr_jvm_context, ptr_param_3, ptr_param_4],
        )
        .await?;

    if result != 0 {
        return Err(WieError::FatalError(format!("Init failed with code {result:#x}")));
    }

    // call init
    let result = core.run_function::<u32>(wipi_exe.fn_init, &[]).await?;
    if result != 0 {
        return Err(WieError::FatalError(format!("wipi init failed with code {result:#x}")));
    }

    Ok(exe_interface.ptr_functions)
}

async fn get_interface(core: &mut ArmCore, ptr_name: u32) -> Result<u32> {
    tracing::trace!("get_interface({ptr_name:#x})");

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?)
        .map_err(|e| WieError::FatalError(format!("get_interface: non-UTF8 interface name at {ptr_name:#x}: {e}")))?;

    match name.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core),
        "WIPI_JBInterface" => get_wipi_jb_interface(core),
        "MNInterface" => relocated::get_mn_interface(core),
        _ => {
            tracing::warn!("Unknown {name}");

            Ok(0)
        }
    }
}

fn init_param_4(core: &mut ArmCore) -> Result<InitParam4> {
    Ok(InitParam4 {
        fn_get_interface: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetInterface)?,
        fn_java_throw: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaThrow)?,
        // `throw e` for an exception the guest already constructed (`fn_java_throw` builds one from a
        // class name). Read off two titles' AOT throw helper: `jump_2(e, 0, param4.unk1)`, called
        // right after `new Exception` — left 0, that throw was a null native jump (docs/report/0438 §3).
        unk1: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaThrowInstance)?,
        unk2: 0,
        fn_java_check_type: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaCheckType)?,
        fn_java_new: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaNew)?,
        fn_java_array_new: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaArrayNew)?,
        fn_visit_gc_root: 0,
        fn_java_class_load: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaClassLoad)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::Alloc)?,
    })
}

async fn alloc(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<u32> {
    tracing::trace!("alloc({a0})");

    Allocator::alloc(core, a0)
}

#[cfg(test)]
mod tests {
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use super::{SVC_CATEGORY_INIT, alloc, init_param_4};

    // Two titles' AOT throw helper calls this slot with an exception they already built; 0 there is a
    // null native jump at the first `throw` (docs/report/0438 §3).
    #[test]
    fn init_param_4_wires_throw_instance() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        // Any handler: stubs need their category registered, and this test never calls one.
        core.register_svc_handler(SVC_CATEGORY_INIT, alloc, &())?;
        let param_4 = init_param_4(&mut core)?;

        assert_ne!(param_4.unk1, 0);
        assert_ne!(param_4.unk1, param_4.fn_java_throw);

        Ok(())
    }
}
