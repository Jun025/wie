use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
};

mod context;
// ★slice D (orchestrator decision ⒝, 2026-09-16): upstream's LGT-specific graphics
// implementation (1,095 lines) is KEPT IN THE TREE but NOT WIRED — every graphics SVC
// below but one routes to the shared `wie_wipi_c::api::graphics` instead. Wiring it makes
// `keydraw_lgt` FAIL (measured 3/3, slice A) because it hands the guest a different
// record ABI (`LgtFramebuffer`, 16B, no `buf`) than the guest SDK reads
// (`WIPICFramebuffer`, 20B, pixel pointer at +16). This is a DEFERRAL, not a rejection:
// re-wiring is the same 27 lines, in reverse. See docs/upstream-realign-p3-slices.md §D.
// The framebuffer record is not the only one: the graphics CONTEXT record is a second
// axis. Handing the guest only the LGT context (`LgtGraphicsContext`, 56B, via an adapter
// on `InitContext`/`SetContext`) while keeping the shared framebuffer also fails
// `keydraw_lgt` with the same `Undefined instruction` in `CletWrapperCard.paint`
// (docs/report/0262 §3) — the fixture SDK reads the shared 48B context layout directly.
// So re-wiring swaps two record ABIs, not one.
//
// The one exception is `GetFramebufferBpp` (2026-09-25): its argument is not a
// framebuffer handle — the native accessor ignores it and the local one does too — so the
// shared accessor, which reads it as a `WIPICFramebuffer`, answered garbage and 하이브리드
// drew 500 uniform frames. It touches no record the fixture SDK reads (`keydraw_lgt` PASS
// 27/27 keys either way). Held by `wipic_framebuffer_bpp_ignores_its_argument`.
//
// `InitContext`/`SetContext` (2026-09-26) are not exceptions to the record choice: they
// still hand out the shared record, and only add the three native fields a title reads
// directly (foreground/background/alpha) in slots no shared reader draws with — see
// `graphics::init_shared_context`. Held by `wipic_context_keeps_native_foreground_and_alpha`.
//
// "NOT WIRED" scopes to those 27 SVCs ONLY — the module is still entered from two other
// places, so do not read it as unreachable: `clet_register` below calls
// `graphics::{init_process_state, set_use_annunciator}`, and `init.rs` routes
// `InitSvcId::SetDisplayProperty` to `graphics::set_display_property`. Measured
// 2026-09-16: 45 of the 57 top-level items are dead (that is what the `allow` below
// suppresses; removing it yields exactly those 45 warnings) and 12 are live — 42 and 15
// since the BPP row above (`get_framebuffer_bpp`, `state`, `FRAMEBUFFER_DEPTH`). Of the
// three live fns, two RUN on every LGT boot — a `panic!` in `init_process_state`
// (:70-98) or `set_use_annunciator` (:152-157) turns `keydraw_lgt` and `helloworld_lgt`
// into FAIL/paints 0; the same probe in `set_display_property` (:121-150) leaves both
// PASS, so that SVC is reachable but no fixture triggers it. Re-measured 2026-09-17:
// still true of the fixtures — the gap is now held by a unit test instead
// (`display_property_svc_reaches_graphics_through_the_import_table`), which resolves
// `(0x1f8, 0x16)` through the import table and executes the SVC stub, so both wiring
// hops go red if either is changed. A guest fixture would still add what that cannot:
// evidence that a real LGT title calls this SVC at all.
//
// What the gates DO hold: `allow(dead_code)` silences a lint, not compilation, so all
// 1,095 lines are type-checked by all four gates — this cannot rot into a build error
// unnoticed. What they do NOT hold is BEHAVIOUR: the 42 dead items are executed by
// nothing, so a semantic drift in them is silent until the 27 lines are re-wired.
#[allow(dead_code)]
pub(super) mod graphics;

use jvm::{Jvm, Result as JvmResult, runtime::JavaLangString};
use wipi_types::lgt::CletFunctions;
use wipi_types::wipic::WIPICIndirectPtr;

use wie_backend::System;
use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{
    Result, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes, write_null_terminated_table,
};
use wie_wipi_c::{
    MethodImpl, WIPICContext, WIPICMethodBody, WIPICResult,
    api::{database, graphics as shared_graphics, kernel, media, misc, net},
};

use context::LgtWIPICContext;

use crate::runtime::{SVC_CATEGORY_WIPIC, svc_ids::WIPICSvcId};

const TIME_VALUE_PTR: u32 = 0x7fff1004;

struct WIPICMethodResult {
    result: WIPICResult,
}

impl ResultWriter<WIPICMethodResult> for WIPICMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result.results)?;
        core.set_next_pc(next_pc)?;

        Ok(())
    }
}

struct CMethodProxy {
    context: LgtWIPICContext,
    body: WIPICMethodBody,
}

async fn handle_wipic_svc(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), id: SvcId) -> Result<()> {
    let wipic_context = LgtWIPICContext::new(core.clone(), system.clone(), jvm.clone());
    let (_, lr) = core.read_pc_lr()?;
    // An unmapped id names its arguments and the guest return address, so the next round can read
    // the call site instead of re-running the title under a debugger to learn what was passed.
    let svc = WIPICSvcId::try_from(id).map_err(|_| {
        let [r0, r1, r2, r3] = [0, 1, 2, 3].map(|i| u32::get(core, i));
        wie_util::WieError::FatalError(alloc::format!(
            "Unknown LGT WIPIC SVC id {} (r0={r0:#x} r1={r1:#x} r2={r2:#x} r3={r3:#x} lr={lr:#x})",
            id.0
        ))
    })?;
    let method = match svc {
        WIPICSvcId::CletRegister => {
            return EmulatedFunction::call(&clet_register, core, &mut (system.clone(), jvm.clone()))
                .await?
                .write(core, lr);
        }
        WIPICSvcId::GetFramebufferPointer => wie_wipi_c::api::graphics::get_framebuffer_pointer.into_body(),
        WIPICSvcId::GetFramebufferWidth => wie_wipi_c::api::graphics::get_framebuffer_width.into_body(),
        WIPICSvcId::GetFramebufferHeight => wie_wipi_c::api::graphics::get_framebuffer_height.into_body(),
        WIPICSvcId::GetFramebufferBpl => wie_wipi_c::api::graphics::get_framebuffer_bpl.into_body(),
        WIPICSvcId::GetFramebufferBpp => graphics::get_framebuffer_bpp.into_body(),
        WIPICSvcId::Printk => kernel::printk.into_body(),
        WIPICSvcId::Sprintk => kernel::sprintk.into_body(),
        WIPICSvcId::TerminateProgram => terminate_program.into_body(),
        WIPICSvcId::Unk1 => unk1.into_body(),
        WIPICSvcId::Exit => kernel::exit.into_body(),
        WIPICSvcId::GetProgramName => kernel::get_program_name.into_body(),
        WIPICSvcId::Alloc => kernel::alloc.into_body(),
        WIPICSvcId::Calloc => kernel::calloc.into_body(),
        WIPICSvcId::Free => kernel::free.into_body(),
        WIPICSvcId::GetTotalMemory => get_memory.into_body(),
        WIPICSvcId::GetFreeMemory => get_memory.into_body(),
        WIPICSvcId::DefTimer => kernel::def_timer.into_body(),
        WIPICSvcId::SetTimer => kernel::set_timer.into_body(),
        WIPICSvcId::UnsetTimer => kernel::unset_timer.into_body(),
        WIPICSvcId::CurrentTime => kernel::current_time.into_body(),
        WIPICSvcId::GetSystemProperty => kernel::get_system_property.into_body(),
        WIPICSvcId::SetSystemProperty => kernel::set_system_property.into_body(),
        WIPICSvcId::GetResourceId => kernel::get_resource_id.into_body(),
        WIPICSvcId::GetResource => kernel::get_resource.into_body(),
        WIPICSvcId::Unk2 => unk2.into_body(),
        WIPICSvcId::GetImageProperty => wie_wipi_c::api::graphics::get_image_property.into_body(),
        WIPICSvcId::GetImageFramebuffer => wie_wipi_c::api::graphics::get_image_framebuffer.into_body(),
        WIPICSvcId::GetScreenFramebuffer => get_screen_framebuffer.into_body(),
        WIPICSvcId::DestroyOffscreenFramebuffer => wie_wipi_c::api::graphics::destroy_offscreen_framebuffer.into_body(),
        WIPICSvcId::CreateOffscreenFramebuffer => wie_wipi_c::api::graphics::create_offscreen_framebuffer.into_body(),
        WIPICSvcId::InitContext => graphics::init_shared_context.into_body(),
        WIPICSvcId::SetContext => graphics::set_shared_context.into_body(),
        WIPICSvcId::GetContext => wie_wipi_c::api::graphics::get_context.into_body(),
        WIPICSvcId::PutPixel => wie_wipi_c::api::graphics::put_pixel.into_body(),
        WIPICSvcId::DrawLine => wie_wipi_c::api::graphics::draw_line.into_body(),
        WIPICSvcId::DrawRect => wie_wipi_c::api::graphics::draw_rect.into_body(),
        WIPICSvcId::FillRect => wie_wipi_c::api::graphics::fill_rect.into_body(),
        WIPICSvcId::CopyFrameBuffer => wie_wipi_c::api::graphics::copy_frame_buffer.into_body(),
        WIPICSvcId::DrawImage => wie_wipi_c::api::graphics::draw_image.into_body(),
        WIPICSvcId::CopyArea => wie_wipi_c::api::graphics::copy_area.into_body(),
        WIPICSvcId::DrawArc => wie_wipi_c::api::graphics::draw_arc.into_body(),
        WIPICSvcId::FillArc => wie_wipi_c::api::graphics::fill_arc.into_body(),
        WIPICSvcId::DrawString => wie_wipi_c::api::graphics::draw_string.into_body(),
        WIPICSvcId::GetRgbPixels => wie_wipi_c::api::graphics::get_rgb_pixels.into_body(),
        WIPICSvcId::SetRgbPixels => wie_wipi_c::api::graphics::set_rgb_pixels.into_body(),
        WIPICSvcId::FlushLcd => wie_wipi_c::api::graphics::flush_lcd.into_body(),
        WIPICSvcId::GetPixelFromRgb => shared_graphics::get_pixel_from_rgb.into_body(),
        WIPICSvcId::GetRgbFromPixel => shared_graphics::get_rgb_from_pixel.into_body(),
        WIPICSvcId::GetDisplayInfo => wie_wipi_c::api::graphics::get_display_info.into_body(),
        WIPICSvcId::Repaint => shared_graphics::repaint.into_body(),
        WIPICSvcId::GetFont => shared_graphics::get_font.into_body(),
        WIPICSvcId::GetFontHeight => shared_graphics::get_font_height.into_body(),
        WIPICSvcId::GetFontAscent => shared_graphics::get_font_ascent.into_body(),
        WIPICSvcId::GetFontDescent => shared_graphics::get_font_descent.into_body(),
        WIPICSvcId::GetStringWidth => shared_graphics::get_string_width.into_body(),
        WIPICSvcId::CreateImage => wie_wipi_c::api::graphics::create_image.into_body(),
        WIPICSvcId::Unk0 => unk0.into_body(),
        WIPICSvcId::Unk11 => unk11.into_body(),
        WIPICSvcId::ImGetSupportModeCount => im_get_support_mode_count.into_body(),
        WIPICSvcId::ImGetSupportedModes => im_get_supported_modes.into_body(),
        WIPICSvcId::Unk7 => unk7.into_body(),
        WIPICSvcId::Unk6 => unk6.into_body(),
        WIPICSvcId::ImHandleInput => im_handle_input.into_body(),
        WIPICSvcId::TimeNow => time_now.into_body(),
        WIPICSvcId::TimeComponent => time_component.into_body(),
        WIPICSvcId::TimeConvert => time_convert.into_body(),
        WIPICSvcId::TimeToTm => time_to_tm.into_body(),
        WIPICSvcId::DateTimeToTm => time_to_tm.into_body(),
        WIPICSvcId::UicConfigure => uic_configure.into_body(),
        WIPICSvcId::UicSetEnable => uic_set_enable.into_body(),
        WIPICSvcId::UicSetMaxTextSize => uic_set_max_text_size.into_body(),
        WIPICSvcId::Htonl | WIPICSvcId::Ntohl => swap32.into_body(),
        WIPICSvcId::Htons | WIPICSvcId::Ntohs => swap16.into_body(),
        WIPICSvcId::InetAddr => inet_addr.into_body(),
        WIPICSvcId::OpenDatabase => database::open_database.into_body(),
        WIPICSvcId::ReadRecordSingle => database::stream_read.into_body(),
        WIPICSvcId::WriteRecordSingle => database::stream_write.into_body(),
        WIPICSvcId::CloseDatabase => database::close_database.into_body(),
        WIPICSvcId::Unk12 => database::seek_record_single.into_body(),
        WIPICSvcId::Unk9 => database::list_record_info.into_body(),
        WIPICSvcId::DeleteRecord => database::delete_database.into_body(),
        WIPICSvcId::ListRecord => database::list_record.into_body(),
        WIPICSvcId::UpdateRecord => database::update_record.into_body(),
        WIPICSvcId::SelectRecord => database::select_record.into_body(),
        WIPICSvcId::ListDatabases => database::list_databases.into_body(),
        WIPICSvcId::Unk8 => database::exists_database.into_body(),
        WIPICSvcId::Connect => net::connect.into_body(),
        WIPICSvcId::Close => net::close.into_body(),
        WIPICSvcId::Socket | WIPICSvcId::SocketAlt => net::socket.into_body(),
        WIPICSvcId::SocketWrite => net_socket_write.into_body(),
        WIPICSvcId::SocketRead => net_socket_read.into_body(),
        WIPICSvcId::SocketClose => net::socket_close.into_body(),
        WIPICSvcId::ClipCreate => media::clip_create.into_body(),
        WIPICSvcId::ClipFree => media::clip_free.into_body(),
        WIPICSvcId::ClipPutData => media::clip_put_data.into_body(),
        WIPICSvcId::Unk15 => unk15.into_body(),
        WIPICSvcId::ClipGetVolume => media::clip_get_volume.into_body(),
        WIPICSvcId::ClipSetVolume => media::clip_set_volume.into_body(),
        WIPICSvcId::Play => media::play.into_body(),
        WIPICSvcId::Pause => media::pause.into_body(),
        WIPICSvcId::Resume => media::resume.into_body(),
        WIPICSvcId::Stop => media::stop.into_body(),
        WIPICSvcId::Unk5 => unk5.into_body(),
        WIPICSvcId::Vibrator => media::vibrator.into_body(),
        WIPICSvcId::Unk14 => unk14.into_body(),
        WIPICSvcId::ClipAllocPlayer => media::clip_alloc_player.into_body(),
        WIPICSvcId::ClipFreePlayer => media::clip_free_player.into_body(),
        WIPICSvcId::Unk10 => unk10.into_body(),
        WIPICSvcId::SetMuteState => media::set_mute_state.into_body(),
        WIPICSvcId::GetMuteState => media::get_mute_state.into_body(),
        WIPICSvcId::BackLight => misc::back_light.into_body(),
        WIPICSvcId::Unk16 => unk16.into_body(),
        WIPICSvcId::ListDataStores => list_data_stores.into_body(),
    };

    EmulatedFunction::call(
        &CMethodProxy {
            context: wipic_context,
            body: method,
        },
        core,
        &mut (),
    )
    .await?
    .write(core, lr)
}

#[async_trait::async_trait]
impl EmulatedFunction<(), WIPICMethodResult, ()> for CMethodProxy {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<WIPICMethodResult> {
        let a0 = u32::get(core, 0);
        let a1 = u32::get(core, 1);
        let a2 = u32::get(core, 2);
        let a3 = u32::get(core, 3);
        let a4 = u32::get(core, 4);
        let a5 = u32::get(core, 5);
        let a6 = u32::get(core, 6);
        let a7 = u32::get(core, 7);
        let a8 = u32::get(core, 8);

        let result = self
            .body
            .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
            .await?;

        Ok(WIPICMethodResult { result })
    }
}

pub fn register_wipic_svc_handler(core: &mut ArmCore, system: &System, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_WIPIC, handle_wipic_svc, &(system.clone(), jvm.clone()))
}

async fn clet_register(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), function_table: u32, a1: u32) -> Result<()> {
    tracing::debug!("clet_register({function_table:#x}, {a1:#x})");

    let (screen_width, screen_height) = {
        let screen = system.platform().screen();
        (screen.width(), screen.height())
    };
    graphics::init_process_state(core, screen_width, screen_height)?;
    graphics::set_use_annunciator(core, a1)?;
    let functions: CletFunctions = read_generic(core, function_table)?;

    jvm.put_static_field("net/wie/CletWrapper", "startClet", "I", functions.start_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "pauseClet", "I", functions.pause_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "resumeClet", "I", functions.resume_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "destroyClet", "I", functions.destroy_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "paintClet", "I", functions.paint_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "handleCletEvent", "I", functions.handle_clet_event as i32)
        .await
        .unwrap();

    let main_class_name = JavaLangString::from_rust_string(jvm, "net/wie/CletWrapper").await.unwrap();
    let mut args_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
    jvm.store_array(&mut args_array, 0, vec![main_class_name]).await.unwrap();

    let result: JvmResult<()> = jvm
        .invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args_array,))
        .await;

    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    Ok(())
}

/// `MC_netSocketWrite` (net table index 4, SVC 0x25c) and `MC_netSocketRead`
/// (index 5, 0x25d). The emulator has no real network, so instead of crashing on
/// an unknown SVC we return -1 (error). Games (테라-영원의혼돈) that attempt an
/// online connection then see the write/read fail and fall back to offline play
/// rather than dying. Shared `wie_wipi_c::api::net` has no socket write/read, so
/// these stay LGT-local (restored from `02ad8b5c`; the base swap #161 dropped them).
/// The in-game network path is not reachable from a headless boot.
// LGT titles draw a soft-key bar BELOW the screen height they are told: 알바타이쿤2 sizes its
// canvas `height + 24` by this exact width table (binary.mod 0x289c4), and 메이플스토리 도적편 blits
// down to row 343 of a 320-row screen. Without the rows that write lands on the next heap block's
// header and the allocator later fails with ~127MB free (docs/report/0392 §4, 0400).
fn softkey_rows(width: u32) -> u32 {
    match width {
        120 | 128 => 14,
        176 => 20,
        240 | 320 => 24,
        _ => 0,
    }
}

async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: u32) -> Result<WIPICIndirectPtr> {
    let width = context.system().platform().screen().width();
    wie_wipi_c::api::graphics::screen_framebuffer(context, a0, softkey_rows(width))
}

async fn net_socket_write(_context: &mut dyn WIPICContext, fd: u32, buf: u32, len: u32, _a3: u32) -> Result<i32> {
    tracing::warn!("MC_netSocketWrite(fd={fd:#x}, buf={buf:#x}, len={len:#x}) -> -1 (no network)");

    Ok(-1)
}

async fn net_socket_read(_context: &mut dyn WIPICContext, fd: u32, buf: u32, len: u32, _a3: u32) -> Result<i32> {
    tracing::warn!("MC_netSocketRead(fd={fd:#x}, buf={buf:#x}, len={len:#x}) -> -1 (no network)");

    Ok(-1)
}

async fn unk0(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk0({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // graphics

    Ok(0)
}

async fn unk1(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk1({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // kernel

    Ok(0)
}

async fn unk2(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::warn!("stub unk2");

    // OEMC_knlGetProgramInfo? get app id
    let app_id = context.system().aid().to_string();
    let result = context.alloc_raw((app_id.len() + 1) as u32)?;
    write_null_terminated_string_bytes(context, result, app_id.as_bytes())?;

    Ok(result)
}

// 0x12c..0x130 is the input-method group: `MC_imGetSurpportModeCount`, `MC_imGetSupportedModes`,
// `MC_imSetCurrentMode`, `MC_imGetCurrentMode`, `MC_imHandleInput` (docs/reference/WIPIHeader.h:1442-1446,
// same order). Established from guest call sites, not from the header alone: 아니마 loops
// `i < 0x12c()` over `0x12d()[i]`, matches each name and passes `i` to `0x12e`; 제노니아1 reads
// `0x12d()[0]` straight into `strstr` — returning NULL there is the `address: 0` wall.
// Names are the vocabulary LGT guests search for (KO, EN/L, EN/S, N123 — 33 binaries in the corpus).
// ponytail: the ORDER is 아니마's own enum (0 EN/S, 1 EN/L, 2 KO, 3 N123), not a measured device order.
const IM_MODES: [&str; 4] = ["EN/S", "EN/L", "KO", "N123"];

async fn im_get_support_mode_count(_context: &mut dyn WIPICContext) -> Result<u32> {
    Ok(IM_MODES.len() as u32)
}

async fn im_get_supported_modes(context: &mut dyn WIPICContext) -> Result<u32> {
    let mut names = vec![];
    for mode in IM_MODES {
        let name = context.alloc_raw(mode.len() as u32 + 1)?;
        write_null_terminated_string_bytes(context, name, mode.as_bytes())?;
        names.push(name);
    }
    let table = context.alloc_raw((names.len() as u32 + 1) * 4)?;
    write_null_terminated_table(context, table, &names)?;

    Ok(table)
}

// `(key, type, buf1, *size1, buf2, *size2)` — the header's shape, and what 그랜드체이스/놈ZERO pass.
// At the sites read (그랜드체이스 2/2, 놈ZERO 3/7) callers zero-fill both buffers, preset the sizes and
// ignore the return. Registering it matters once 0x12c reports modes: 그랜드체이스 goes on to
// call it and dies on `Unknown LGT WIPIC SVC id 304` without this row.
// ponytail: no IME — input is not composed; implement when a title needs typed text.
async fn im_handle_input(_context: &mut dyn WIPICContext, key: u32, r#type: u32) -> Result<u32> {
    tracing::warn!("stub MC_imHandleInput({key:#x}, {:#x})", r#type);

    Ok(0)
}

async fn unk5(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk5({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}

async fn unk6(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk6({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk7(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk7({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn time_now(context: &mut dyn WIPICContext, component_class: u32) -> Result<u32> {
    let epoch_seconds = context.system().platform().now().raw() / 1000;
    tracing::debug!("LGT_timeNow({component_class:#x}) -> {epoch_seconds}");

    write_time_value(context, epoch_seconds as u32)
}

/// WIPIC 900/902 (`htonl`/`ntohl`): the guest is little-endian, so network order is a swap.
async fn swap32(_context: &mut dyn WIPICContext, value: u32) -> Result<u32> {
    Ok(value.swap_bytes())
}

/// WIPIC 901/903 (`htons`/`ntohs`). Callers pass a sign-extended short and re-narrow the
/// result themselves, so only the low 16 bits are read and the answer is zero-extended.
async fn swap16(_context: &mut dyn WIPICContext, value: u32) -> Result<u32> {
    Ok(u32::from((value as u16).swap_bytes()))
}

/// WIPIC 904 (`inet_addr`): a dotted quad to an address in network order, `0xffffffff`
/// (`INADDR_NONE`) when the string is not one. Only the four-decimal-part form is accepted —
/// the one form measured at a call site; the BSD shorthand forms are not guessed at.
async fn inet_addr(context: &mut dyn WIPICContext, ptr_cp: u32) -> Result<u32> {
    let text = read_null_terminated_string_bytes(context, ptr_cp)?;
    tracing::debug!("LGT inet_addr({:?})", String::from_utf8_lossy(&text));

    Ok(parse_dotted_quad(&text).map_or(u32::MAX, u32::from_le_bytes))
}

fn parse_dotted_quad(text: &[u8]) -> Option<[u8; 4]> {
    let mut out = [0u8; 4];
    let mut parts = text.split(|&b| b == b'.');
    for slot in &mut out {
        let part = parts.next()?;
        if part.is_empty() || part.len() > 3 || !part.iter().all(u8::is_ascii_digit) {
            return None;
        }
        *slot = core::str::from_utf8(part).ok()?.parse().ok()?;
    }

    parts.next().is_none().then_some(out)
}

async fn time_component(_context: &mut dyn WIPICContext, name: u32) -> Result<u32> {
    tracing::debug!("LGT_timeComponent({name:#x})");

    Ok(name)
}

async fn time_convert(context: &mut dyn WIPICContext, date_time: u32, component: u32) -> Result<u32> {
    tracing::debug!("LGT_timeConvert({date_time:#x}, {component:#x})");

    let timestamp = read_time_value(context, date_time)?;
    write_time_value(context, timestamp)
}

async fn time_to_tm(context: &mut dyn WIPICContext, time_value: u32, out_ptr: u32) -> Result<i32> {
    tracing::debug!("LGT_timeToTm({time_value:#x}, {out_ptr:#x})");

    let timestamp = read_time_value(context, time_value)?;
    let (year, month, day, hour, minute, second) = unix_seconds_to_utc(timestamp as i64);
    write_generic(context, out_ptr, second)?;
    write_generic(context, out_ptr + 4, minute)?;
    write_generic(context, out_ptr + 8, hour)?;
    write_generic(context, out_ptr + 12, day)?;
    write_generic(context, out_ptr + 16, month - 1)?;
    write_generic(context, out_ptr + 20, year - 1900)?;

    Ok(0)
}

fn write_time_value(context: &mut dyn WIPICContext, timestamp: u32) -> Result<u32> {
    let time_value_ptr: u32 = read_generic(context, TIME_VALUE_PTR)?;
    let memory = if time_value_ptr != 0 {
        WIPICIndirectPtr(time_value_ptr)
    } else {
        let memory = context.alloc(4)?;
        write_generic(context, TIME_VALUE_PTR, memory.0)?;
        memory
    };
    write_generic(context, context.data_ptr(memory)?, timestamp)?;
    Ok(memory.0)
}

fn read_time_value(context: &mut dyn WIPICContext, handle: u32) -> Result<u32> {
    read_generic(context, context.data_ptr(WIPICIndirectPtr(handle))?)
}

fn unix_seconds_to_utc(timestamp: i64) -> (i32, i32, i32, i32, i32, i32) {
    let days = timestamp.div_euclid(86_400);
    let seconds_of_day = timestamp.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (seconds_of_day / 3600) as i32;
    let minute = ((seconds_of_day % 3600) / 60) as i32;
    let second = (seconds_of_day % 60) as i32;

    (year, month, day, hour, minute, second)
}

fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_param = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_param + 2) / 5 + 1;
    let month = month_param + if month_param < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year as i32, month as i32, day as i32)
}

async fn unk10(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk10({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk11(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk11({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

/// WIPIC kernel index 4 (`0x68`): ends the program. The guest does not expect it to return.
///
/// Identified from the call sites, not from a symbol — none of the LGT images names it. Six
/// titles that stopped on a first-run notice («게임을 다시 실행하여 주시기 바랍니다 · 아무키나
/// 누르세요», «메모리 부족 … 리셋하여 주시기 바랍니다», «다운로드 되었습니다 … EZ-i 메뉴로
/// 이동합니다») all do the same three things on the key: write a marker database (open with
/// create + close, or delete + recreate), then call this with ONE integer in r0 (0, 2, 0x1b or
/// -1 depending on the title — never a pointer), then discard the return value. With the old
/// stub returning 0 the title fell back into its notice loop and redrew it on every key (27 calls
/// in 27 keys on one title); what the notice text asks for is the phone taking the user back to
/// the menu. So the engine's half is the same as `MC_knlExit` (`0x6b`): tell the host the guest
/// asked to stop. Relaunching — with the database kept, which is where the marker lives — is the
/// host's half, and a relaunched title passes the notice (see `docs/report` for the round).
///
/// KTF's table puts `MC_knlMExecute` at this index; the argument here is not a program name, so
/// that is not what this is, and the name is deliberately behavioural.
async fn terminate_program(context: &mut dyn WIPICContext, code: i32) -> Result<()> {
    tracing::debug!("LGT WIPIC terminate_program({code})");

    context.system().platform().exit();

    Ok(())
}

/// `MC_knlGetTotalMemory` / `MC_knlGetFreeMemory` on LGT: a fixed 4MiB for both.
///
/// The shared kernel answers 1MiB, and one LGT title shows «메모리 부족 — 단말기 리셋» and quits
/// when free memory is ≤ 1,500,000 (`0x16e360`). It is LGT-only because KTF must stay at 1MiB: a
/// KTF title sizes its pool as `free - 100KB` and dies at boot once that pool passes 1MiB
/// (measured at 1,200,000). LGT callers were swept at 2/4/8MiB with no other change; the
/// emulator's real heap is not answered because it is 256MiB and allocation-dependent — see
/// `docs/report` for the round.
async fn get_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::debug!("LGT MC_knlGetTotalMemory/GetFreeMemory()");

    Ok(LGT_MEMORY)
}

const LGT_MEMORY: i32 = 0x400000;

/// UIC 809/811/833 (`MC_uicConfigure`/`SetEnable`/`SetMaxTextSize`): the components 800–802 hand out
/// are not real, so there is nothing to place, enable or size — accept and report success.
async fn uic_configure(_context: &mut dyn WIPICContext, comp: u32, x: i32, y: i32, w: i32, h: i32, flags: u32) -> Result<u32> {
    tracing::debug!("stub LGT MC_uicConfigure({comp:#x}, {x}, {y}, {w}, {h}, {flags:#x})");

    Ok(0)
}

async fn uic_set_enable(_context: &mut dyn WIPICContext, comp: u32, enable: u32) -> Result<u32> {
    tracing::debug!("stub LGT MC_uicSetEnable({comp:#x}, {enable})");

    Ok(0)
}

async fn uic_set_max_text_size(_context: &mut dyn WIPICContext, comp: u32, size: i32) -> Result<u32> {
    tracing::debug!("stub LGT MC_uicSetMaxTextSize({comp:#x}, {size})");

    Ok(0)
}

/// WIPIC 1100: the phone's data-store names, NUL-separated, into `buf`. An empty list — see the id.
async fn list_data_stores(context: &mut dyn WIPICContext, buf: u32, len: i32) -> Result<i32> {
    tracing::debug!("LGT list_data_stores({buf:#x}, {len}) -> empty");

    if buf != 0 && len > 0 {
        write_generic(context, buf, 0u8)?;
    }

    Ok(0)
}

async fn unk14(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk14({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}

async fn unk15(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk15({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}

async fn unk16(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk16({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // misc

    Ok(0)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicBool, Ordering};

    use test_utils::{TestPlatform, TestPlatformEvent};
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::Allocator;
    use wie_util::{ByteWrite, Result, read_generic};
    use wipi_types::wipic::WIPICFramebuffer;

    use super::{graphics, register_wipic_svc_handler};
    use crate::runtime::{SVC_CATEGORY_WIPIC, java::init_jvm, svc_ids::WIPICSvcId};

    /// WIPIC `0x68` ends the program: it reaches `Platform::exit`, the way `MC_knlExit` does.
    ///
    /// Six LGT titles call it from their first-run notice with one integer (0x1b here is what
    /// one of them passes). As a stub returning 0 it never told the host anything, and each title
    /// redrew the notice on every key forever; the host could not tell «waiting» from «done».
    #[test]
    fn wipic_0x68_terminates_the_program() -> Result<()> {
        let exited = Arc::new(AtomicBool::new(false));
        let exited_clone = exited.clone();
        let platform = TestPlatform::with_event_handler(move |event| {
            if let TestPlatformEvent::Exit = event {
                exited_clone.store(true, Ordering::Relaxed);
            }
        });
        let mut system = System::new(Box::new(platform), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            assert_eq!(WIPICSvcId::TerminateProgram as u32, 0x68);
            let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::TerminateProgram)?;
            let _: u32 = core.run_function(stub, &[0x1b]).await?;

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        assert!(exited.load(Ordering::Relaxed), "0x68 must reach Platform::exit");

        Ok(())
    }

    /// LGT memory queries answer above the 1,500,000 threshold one title checks, and free ≤ total.
    #[test]
    fn wipic_memory_queries_clear_the_low_memory_notice() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            let free_stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::GetFreeMemory)?;
            let total_stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::GetTotalMemory)?;
            let free: u32 = core.run_function(free_stub, &[]).await?;
            let total: u32 = core.run_function(total_stub, &[]).await?;
            assert!(free > 1_500_000, "free {free} would show the low-memory notice");
            assert!(free <= total);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// WIPIC 900–904 answer through the SVC table: byte swaps and `inet_addr`.
    ///
    /// Before, the table had no row for any of them and the first call was a fatal
    /// «Unknown LGT WIPIC SVC id» — two titles died at boot on 900 (the value feeds their
    /// backlight colour), others on 904 when opening a connection. The inputs are the ones
    /// measured at call sites: port `0xffffa482` (a sign-extended short), a dotted-quad string (the test uses a TEST-NET address).
    #[test]
    fn wipic_byte_order_section_answers() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            let call = async |core: &mut wie_core_arm::ArmCore, id: WIPICSvcId, arg: u32| -> Result<u32> {
                let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, id)?;
                core.run_function(stub, &[arg]).await
            };

            for (id, number) in [(WIPICSvcId::Htonl, 900), (WIPICSvcId::Ntohl, 902)] {
                assert_eq!(id as u32, number);
                assert_eq!(call(&mut core, id, 0x0000_ffff).await?, 0xffff_0000);
            }
            for (id, number) in [(WIPICSvcId::Htons, 901), (WIPICSvcId::Ntohs, 903)] {
                assert_eq!(id as u32, number);
                assert_eq!(call(&mut core, id, 0xffff_a482).await?, 0x82a4);
            }

            assert_eq!(WIPICSvcId::InetAddr as u32, 904);
            let text = Allocator::alloc(&mut core, 32)?;
            core.write_bytes(text, b"192.0.2.45\0")?;
            assert_eq!(call(&mut core, WIPICSvcId::InetAddr, text).await?.to_le_bytes(), [192, 0, 2, 45]);
            core.write_bytes(text, b"192.0.2\0")?;
            assert_eq!(call(&mut core, WIPICSvcId::InetAddr, text).await?, u32::MAX);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// `MC_GRP_GET_FRAME_BUFFER_BPP` answers the display depth whatever its argument is.
    ///
    /// LGT titles pass a value that is not a framebuffer handle here (하이브리드 passes
    /// `0x4904dbe5`, an odd address, every frame). The shared accessor read that as a
    /// `WIPICFramebuffer` record and returned whatever sat at +12, and the title then drew
    /// nothing visible: 500 uniform frames. Routed to the LGT accessor, which ignores its
    /// argument the way the native one does, the same title draws its first screen.
    /// The argument below points at zeroed memory, so the shared accessor answers 0.
    #[test]
    fn wipic_framebuffer_bpp_ignores_its_argument() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            graphics::init_process_state(&mut core, 240, 320)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::GetFramebufferBpp)?;
            assert_eq!(WIPICSvcId::GetFramebufferBpp as u32, 0x36);

            let not_a_handle = Allocator::alloc(&mut core, 0x20)?;
            core.write_bytes(not_a_handle, &[0; 0x20])?;
            let bpp: u32 = core.run_function(stub, &[not_a_handle]).await?;
            assert_eq!(bpp, 16);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// The LGT screen framebuffer has 24 soft-key rows below the height it reports (240 or 320 wide).
    ///
    /// Two LGT titles draw those rows (docs/report/0400); without them the 25th row lands on the
    /// next heap block's header. The reported height must not grow — every LGT title lays out by it.
    #[test]
    fn wipic_screen_framebuffer_has_softkey_rows_below_its_height() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::GetScreenFramebuffer)?;
            let handle: u32 = core.run_function(stub, &[0]).await?;

            let framebuffer: WIPICFramebuffer = read_generic(&core, handle)?;
            assert_eq!((framebuffer.width, framebuffer.height), (320, 240)); // TestPlatform's screen
            let allocated: u32 = read_generic(&core, framebuffer.buf.0 - 4)?;
            assert_eq!(allocated, framebuffer.bpl * (240 + 24));

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// `MC_grpInitContext`/`MC_grpSetContext` leave the text colour where LGT titles read it.
    ///
    /// The record stays the shared 52B layout (`keydraw_lgt` embeds that struct), but the
    /// native offsets a title reads directly — foreground +16, alpha +24 — must hold the
    /// native values. With the shared SVCs +16 stays 0 and alpha 0, and 0236 §2-2's title
    /// drew its 2,125 glyph pixels as `#fffbff` on white.
    #[test]
    fn wipic_context_keeps_native_foreground_and_alpha() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            let init = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::InitContext)?;
            let set = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::SetContext)?;

            let record = Allocator::alloc(&mut core, 52)?;
            let _: u32 = core.run_function(init, &[record]).await?;
            let _: u32 = core.run_function(set, &[record, 1, 0xf800]).await?; // fg
            let _: u32 = core.run_function(set, &[record, 2, 0x1234]).await?; // bg
            let word = |offset: u32| read_generic::<u32, _>(&core, record + offset);
            assert_eq!(word(12)?, 0xf800, "shared fgpxl, read by the host");
            assert_eq!(word(16)?, 0xf800, "native foreground, read by the title");
            assert_eq!(word(20)?, 0x1234, "native background");
            assert_eq!(word(24)?, 255, "native alpha");

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    /// The ids named 2026-09-30 answer through the SVC table, and an id still unmapped names its arguments.
    ///
    /// Eight LGT titles stopped on `Unknown LGT WIPIC SVC id` 809 · 1100 · 1212 · 2000, each at a call site
    /// read from the image (see each id in `svc_ids`). The answers are the no-device ones: no socket (-1),
    /// an empty data-store list, and accepted UIC/media calls. The unmapped-id message carries r0–r3 and lr
    /// because without them each id cost a debugger rerun of the title before its call site could be read.
    #[test]
    fn wipic_svc_ids_named_from_call_sites_answer() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let (jvm, mut core, _) = init_jvm(&system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;

            for (id, number) in [(WIPICSvcId::Socket, 602), (WIPICSvcId::SocketAlt, 2000)] {
                assert_eq!(id as u32, number);
                let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, id)?;
                assert_eq!(
                    core.run_function::<u32>(stub, &[2, 1]).await?,
                    u32::MAX,
                    "SVC {number} must answer «no socket»"
                );
            }
            for (id, number) in [
                (WIPICSvcId::UicConfigure, 809),
                (WIPICSvcId::UicSetEnable, 811),
                (WIPICSvcId::UicSetMaxTextSize, 833),
                (WIPICSvcId::Pause, 1211),
                (WIPICSvcId::Resume, 1212),
            ] {
                assert_eq!(id as u32, number);
                let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, id)?;
                assert_eq!(core.run_function::<u32>(stub, &[0, 0x5a, 0xf8, 0x3c]).await?, 0);
            }

            assert_eq!(WIPICSvcId::ListDataStores as u32, 1100);
            let buf = Allocator::alloc(&mut core, 16)?;
            core.write_bytes(buf, b"PHONEBOOK\0")?;
            let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICSvcId::ListDataStores)?;
            let _: u32 = core.run_function(stub, &[buf, 0xffe]).await?;
            assert_eq!(read_generic::<u8, _>(&core, buf)?, 0, "1100 must hand back an empty list");

            let stub = core.make_svc_stub(SVC_CATEGORY_WIPIC, 0x7d1u32)?;
            let err = core.run_function::<u32>(stub, &[0x11, 0x22, 0x33, 0x44]).await.unwrap_err();
            let text = alloc::format!("{err}");
            assert!(
                text.contains("Unknown LGT WIPIC SVC id 2001 (r0=0x11 r1=0x22 r2=0x33 r3=0x44 lr=0x"),
                "{text}"
            );

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
