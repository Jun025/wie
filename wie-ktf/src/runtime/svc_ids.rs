use wie_core_arm::SvcId;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InitSvcId {
    GetInterface = 0,
    JavaThrow = 1,
    JavaCheckType = 2,
    JavaNew = 3,
    JavaArrayNew = 4,
    JavaClassLoad = 5,
    Alloc = 6,
}

impl TryFrom<SvcId> for InitSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::GetInterface,
            1 => Self::JavaThrow,
            2 => Self::JavaCheckType,
            3 => Self::JavaNew,
            4 => Self::JavaArrayNew,
            5 => Self::JavaClassLoad,
            6 => Self::Alloc,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF init SVC id {}", value.0))),
        })
    }
}

impl From<InitSvcId> for u32 {
    fn from(value: InitSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum JavaSvcId {
    JavaJump1 = 7,
    JavaJump2 = 8,
    JavaJump3 = 9,
    GetJavaMethod = 10,
    GetField = 11,
    JbUnk4 = 12,
    JbUnk5 = 13,
    MonitorEnter = 14,
    JbUnk8 = 15,
    RegisterClass = 16,
    RegisterJavaString = 17,
    CallNative = 18,
}

impl TryFrom<SvcId> for JavaSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            7 => Self::JavaJump1,
            8 => Self::JavaJump2,
            9 => Self::JavaJump3,
            10 => Self::GetJavaMethod,
            11 => Self::GetField,
            12 => Self::JbUnk4,
            13 => Self::JbUnk5,
            14 => Self::MonitorEnter,
            15 => Self::JbUnk8,
            16 => Self::RegisterClass,
            17 => Self::RegisterJavaString,
            18 => Self::CallNative,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF Java SVC id {}", value.0))),
        })
    }
}

impl From<JavaSvcId> for u32 {
    fn from(value: JavaSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICKernelMethodId {
    Printk = 0,
    Sprintk = 1,
    GetExecNames = 2,
    Execute = 3,
    Mexecute = 4,
    Load = 5,
    Mload = 6,
    Exit = 7,
    ProgramStop = 8,
    GetCurProgramId = 9,
    GetParentProgramId = 10,
    GetAppManagerId = 11,
    GetProgramInfo = 12,
    GetAccessLevel = 13,
    GetProgramName = 14,
    CreateSharedBuf = 15,
    DestroySharedBuf = 16,
    GetSharedBuf = 17,
    GetSharedBufSize = 18,
    ResizeSharedBuf = 19,
    Alloc = 20,
    Calloc = 21,
    Free = 22,
    GetTotalMemory = 23,
    GetFreeMemory = 24,
    DefTimer = 25,
    SetTimer = 26,
    UnsetTimer = 27,
    CurrentTime = 28,
    GetSystemProperty = 29,
    SetSystemProperty = 30,
    GetResourceId = 31,
    GetResource = 32,
    Reserved1 = 33,
    Reserved2 = 34,
    Reserved3 = 35,
    Reserved4 = 36,
    Reserved5 = 37,
    Reserved6 = 38,
    Reserved7 = 39,
    Reserved8 = 40,
    Reserved9 = 41,
    Reserved10 = 42,
    Reserved11 = 43,
    SendMessage = 44,
    SetTimerEx = 45,
    GetSystemState = 46,
    CreateSystemProgressBar = 47,
    SetSystemProgressBar = 48,
    DestroySystemProgressBar = 49,
    ExecuteEx = 50,
    GetProcAddress = 51,
    Unload = 52,
    CreateSysMessageBox = 53,
    DestroySysMessageBox = 54,
    GetProgramIdList = 55,
    GetProgramInfo2 = 56,
    Reserved12 = 57,
    Reserved13 = 58,
    CreateAppPrivateArea = 59,
    GetAppPrivateArea = 60,
    CreateLibPrivateArea = 61,
    GetLibPrivateArea = 62,
    GetPlatformVersion = 63,
    GetToken = 64,
}

impl From<WIPICKernelMethodId> for u16 {
    fn from(value: WIPICKernelMethodId) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for WIPICKernelMethodId {
    type Error = wie_util::WieError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Printk,
            1 => Self::Sprintk,
            2 => Self::GetExecNames,
            3 => Self::Execute,
            4 => Self::Mexecute,
            5 => Self::Load,
            6 => Self::Mload,
            7 => Self::Exit,
            8 => Self::ProgramStop,
            9 => Self::GetCurProgramId,
            10 => Self::GetParentProgramId,
            11 => Self::GetAppManagerId,
            12 => Self::GetProgramInfo,
            13 => Self::GetAccessLevel,
            14 => Self::GetProgramName,
            15 => Self::CreateSharedBuf,
            16 => Self::DestroySharedBuf,
            17 => Self::GetSharedBuf,
            18 => Self::GetSharedBufSize,
            19 => Self::ResizeSharedBuf,
            20 => Self::Alloc,
            21 => Self::Calloc,
            22 => Self::Free,
            23 => Self::GetTotalMemory,
            24 => Self::GetFreeMemory,
            25 => Self::DefTimer,
            26 => Self::SetTimer,
            27 => Self::UnsetTimer,
            28 => Self::CurrentTime,
            29 => Self::GetSystemProperty,
            30 => Self::SetSystemProperty,
            31 => Self::GetResourceId,
            32 => Self::GetResource,
            33 => Self::Reserved1,
            34 => Self::Reserved2,
            35 => Self::Reserved3,
            36 => Self::Reserved4,
            37 => Self::Reserved5,
            38 => Self::Reserved6,
            39 => Self::Reserved7,
            40 => Self::Reserved8,
            41 => Self::Reserved9,
            42 => Self::Reserved10,
            43 => Self::Reserved11,
            44 => Self::SendMessage,
            45 => Self::SetTimerEx,
            46 => Self::GetSystemState,
            47 => Self::CreateSystemProgressBar,
            48 => Self::SetSystemProgressBar,
            49 => Self::DestroySystemProgressBar,
            50 => Self::ExecuteEx,
            51 => Self::GetProcAddress,
            52 => Self::Unload,
            53 => Self::CreateSysMessageBox,
            54 => Self::DestroySysMessageBox,
            55 => Self::GetProgramIdList,
            56 => Self::GetProgramInfo2,
            57 => Self::Reserved12,
            58 => Self::Reserved13,
            59 => Self::CreateAppPrivateArea,
            60 => Self::GetAppPrivateArea,
            61 => Self::CreateLibPrivateArea,
            62 => Self::GetLibPrivateArea,
            63 => Self::GetPlatformVersion,
            64 => Self::GetToken,
            _ => {
                return Err(wie_util::WieError::FatalError(alloc::format!(
                    "Unknown KTF WIPIC kernel method id {value}"
                )));
            }
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICGraphicsMethodId {
    GetImageProperty = 0,
    GetImageFramebuffer = 1,
    GetScreenFramebuffer = 2,
    DestroyOffscreenFramebuffer = 3,
    CreateOffscreenFramebuffer = 4,
    InitContext = 5,
    SetContext = 6,
    GetContext = 7,
    PutPixel = 8,
    DrawLine = 9,
    DrawRect = 10,
    FillRect = 11,
    CopyFrameBuffer = 12,
    DrawImage = 13,
    CopyArea = 14,
    DrawArc = 15,
    FillArc = 16,
    DrawString = 17,
    DrawUnicodeString = 18,
    GetRgbPixels = 19,
    SetRgbPixels = 20,
    FlushLcd = 21,
    GetPixelFromRgb = 22,
    GetRgbFromPixel = 23,
    GetDisplayInfo = 24,
    Repaint = 25,
    GetFont = 26,
    GetFontHeight = 27,
    GetFontAscent = 28,
    GetFontDescent = 29,
    GetStringWidth = 30,
    GetUnicodeStringWidth = 31,
    CreateImage = 32,
    DestroyImage = 33,
    DecodeNextImage = 34,
    EncodeImage = 35,
    PostEvent = 36,
    HandleInput = 37,
    SetCurrentMode = 38,
    GetCurrentMode = 39,
    GetSupportModeCount = 40,
    GetSupportedModes = 41,
    FillPolygon = 42,
    DrawPolygon = 43,
    ShowAnnunciator = 44,
    GetAnnunciatorInfo = 45,
    SetAnnunciatorIcon = 46,
    GetIdleHelpLineInfo = 47,
    ShowHelpLine = 48,
    GetCharGlyph = 49,
    CreateImageEx = 50,
    HideHelpLine = 51,
    SetCloneScreenFramebuffer = 52,
    GetFontEx = 53,
    GetFontLists = 54,
    GetFontInfo = 55,
    SetFontHelpLine = 56,
    GetFontHelpLine = 57,
    EncodeImageEx = 58,
    GetImageInfo = 59,
}

impl From<WIPICGraphicsMethodId> for u16 {
    fn from(value: WIPICGraphicsMethodId) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for WIPICGraphicsMethodId {
    type Error = wie_util::WieError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::GetImageProperty,
            1 => Self::GetImageFramebuffer,
            2 => Self::GetScreenFramebuffer,
            3 => Self::DestroyOffscreenFramebuffer,
            4 => Self::CreateOffscreenFramebuffer,
            5 => Self::InitContext,
            6 => Self::SetContext,
            7 => Self::GetContext,
            8 => Self::PutPixel,
            9 => Self::DrawLine,
            10 => Self::DrawRect,
            11 => Self::FillRect,
            12 => Self::CopyFrameBuffer,
            13 => Self::DrawImage,
            14 => Self::CopyArea,
            15 => Self::DrawArc,
            16 => Self::FillArc,
            17 => Self::DrawString,
            18 => Self::DrawUnicodeString,
            19 => Self::GetRgbPixels,
            20 => Self::SetRgbPixels,
            21 => Self::FlushLcd,
            22 => Self::GetPixelFromRgb,
            23 => Self::GetRgbFromPixel,
            24 => Self::GetDisplayInfo,
            25 => Self::Repaint,
            26 => Self::GetFont,
            27 => Self::GetFontHeight,
            28 => Self::GetFontAscent,
            29 => Self::GetFontDescent,
            30 => Self::GetStringWidth,
            31 => Self::GetUnicodeStringWidth,
            32 => Self::CreateImage,
            33 => Self::DestroyImage,
            34 => Self::DecodeNextImage,
            35 => Self::EncodeImage,
            36 => Self::PostEvent,
            37 => Self::HandleInput,
            38 => Self::SetCurrentMode,
            39 => Self::GetCurrentMode,
            40 => Self::GetSupportModeCount,
            41 => Self::GetSupportedModes,
            42 => Self::FillPolygon,
            43 => Self::DrawPolygon,
            44 => Self::ShowAnnunciator,
            45 => Self::GetAnnunciatorInfo,
            46 => Self::SetAnnunciatorIcon,
            47 => Self::GetIdleHelpLineInfo,
            48 => Self::ShowHelpLine,
            49 => Self::GetCharGlyph,
            50 => Self::CreateImageEx,
            51 => Self::HideHelpLine,
            52 => Self::SetCloneScreenFramebuffer,
            53 => Self::GetFontEx,
            54 => Self::GetFontLists,
            55 => Self::GetFontInfo,
            56 => Self::SetFontHelpLine,
            57 => Self::GetFontHelpLine,
            58 => Self::EncodeImageEx,
            59 => Self::GetImageInfo,
            _ => {
                return Err(wie_util::WieError::FatalError(alloc::format!(
                    "Unknown KTF WIPIC graphics method id {value}"
                )));
            }
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum WIPICDatabaseMethodId {
    OpenDatabase = 0,
    /// KTF extension. Reads from the open record using a per-handle byte
    /// cursor; signature `(fd, buf, len)`.
    StreamRead = 1,
    /// KTF extension. Writes to the open record at the per-handle cursor
    /// (no recid argument); signature `(fd, buf, len)`.
    StreamWrite = 2,
    CloseDatabase = 3,
    SelectRecord = 4,
    UpdateRecord = 5,
    DeleteRecord = 6,
    ListRecord = 7,
    SortRecords = 8,
    GetAccessMode = 9,
    GetNumberOfRecords = 10,
    GetRecordSize = 11,
    ListDatabases = 12,
    Unk13 = 13,
    Unk14 = 14,
    Unk15 = 15,
    Exists = 16,
}

impl From<WIPICDatabaseMethodId> for u16 {
    fn from(value: WIPICDatabaseMethodId) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for WIPICDatabaseMethodId {
    type Error = wie_util::WieError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::OpenDatabase,
            1 => Self::StreamRead,
            2 => Self::StreamWrite,
            3 => Self::CloseDatabase,
            4 => Self::SelectRecord,
            5 => Self::UpdateRecord,
            6 => Self::DeleteRecord,
            7 => Self::ListRecord,
            8 => Self::SortRecords,
            9 => Self::GetAccessMode,
            10 => Self::GetNumberOfRecords,
            11 => Self::GetRecordSize,
            12 => Self::ListDatabases,
            13 => Self::Unk13,
            14 => Self::Unk14,
            15 => Self::Unk15,
            16 => Self::Exists,
            _ => {
                return Err(wie_util::WieError::FatalError(alloc::format!(
                    "Unknown KTF WIPIC database method id {value}"
                )));
            }
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum WIPICTableId {
    Kernel = 0,
    Util = 1,
    Misc = 2,
    Graphics = 3,
    Interface3 = 4,
    Interface4 = 5,
    Interface5 = 6,
    Database = 7,
    Interface7 = 8,
    Uic = 9,
    Media = 10,
    Net = 11,
    Interface11 = 12,
    Interface12 = 13,
    Interface13 = 14,
    Interface14 = 15,
    Interface15 = 16,
    Interface16 = 17,
}

impl WIPICTableId {
    pub fn function_id(self, method_id: impl Into<u16>) -> u32 {
        ((self as u32) << 16) | method_id.into() as u32
    }
}

impl TryFrom<u32> for WIPICTableId {
    type Error = wie_util::WieError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Kernel,
            1 => Self::Util,
            2 => Self::Misc,
            3 => Self::Graphics,
            4 => Self::Interface3,
            5 => Self::Interface4,
            6 => Self::Interface5,
            7 => Self::Database,
            8 => Self::Interface7,
            9 => Self::Uic,
            10 => Self::Media,
            11 => Self::Net,
            12 => Self::Interface11,
            13 => Self::Interface12,
            14 => Self::Interface13,
            15 => Self::Interface14,
            16 => Self::Interface15,
            17 => Self::Interface16,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown KTF WIPIC table id {value}"))),
        })
    }
}

/// The message KTF's *unidentified kernel extension* slots report.
///
/// ── Why the name `MC_knlReservedN` had to stop being the whole message ───────
/// Those thirteen variants above (`Reserved1`-`Reserved13`, ids 33-43 and 57-58)
/// are **this repo's placeholders**, not slots the WIPI specification set aside.
/// Measured 2026-09-19 against `docs/reference/WIPIHeader.h`: the string
/// `Reserved` appears **0 times** (the single case-insensitive hit is
/// `int m_reserved;`, an unrelated struct field), and the spec's kernel section
/// simply **ends** at `E_MC_knlGetResource` — the next field is
/// `E_MC_grpGetImageProperty`. So the spec has no kernel slot at index 33 or
/// beyond; everything up there is KTF's own extension space. This file already
/// knows that for ids 44+, which it labels `OEMC_knl…` ("OEM C"). The
/// `Reserved*` block is the same territory wearing the wrong label.
///
/// That mislabel had a cost: a triage round classified the resulting failure as
/// "a slot the spec left blank, so it cannot be touched" and prescribed
/// *documentation*. It is not untouchable — it is un-reverse-engineered, which
/// is a different verdict with a different price.
///
/// ── What id 36 actually does, since this round went and looked ───────────────
/// One title reaches it. Disassembled at its call site (and cross-checked twice:
/// the fault's `IP` is `0x24` = 36, and the guest loads the entry from
/// `knl_interface + 0x90` = field index 36):
///
/// ```text
/// f(r0 = "MXUserMemInterf", r1 = -1, r2 = -1, r3 = 0, [sp+0] = 0)   // arity 5
/// -> the caller stores the result in a global, then immediately calls
///    result->slot0(buffer, 0x14400)                                  // 82,944 bytes
/// ```
///
/// So it takes an **interface name** and hands back an **interface pointer**
/// whose first entry is then initialised with a memory pool. That is a named
/// extension-interface lookup, and it is very likely the door through which the
/// `Interface3`-`Interface16` tables below are reached. ★**That last sentence is
/// a hypothesis, not a measurement** — nothing here traced a returned pointer to
/// one of those tables. The arity and the two argument shapes are measured; the
/// *name* of the function is not, and this round deliberately did not invent one.
pub fn ktf_kernel_extension_message(id: u16, placeholder: &str) -> alloc::string::String {
    alloc::format!(
        "unidentified KTF kernel extension — `{placeholder}` is this repo's placeholder name for slot {id}, \
         NOT a slot reserved by the WIPI specification (docs/reference/WIPIHeader.h has no kernel entry at \
         this index; its kernel section ends at MC_knlGetResource). It is un-reverse-engineered, not untouchable"
    )
}

#[cfg(test)]
mod tests {
    use super::{WIPICKernelMethodId, ktf_kernel_extension_message};

    /// The message must carry the correction, not just a name.
    ///
    /// The point of this round is that `MC_knlReserved4` reads as "the spec left
    /// this blank", which sent a previous triage round to *document* a failure
    /// that is in fact reverse-engineerable. So the string has to keep three
    /// things: the slot number, the placeholder it used to be called, and the
    /// statement that the specification has no such slot. Dropping any one of
    /// them puts the old reading back.
    #[test]
    fn ktf_kernel_extension_message_says_it_is_ours_not_the_spec_s() {
        let m = ktf_kernel_extension_message(36, "MC_knlReserved4");

        for needle in [
            "slot 36",
            "MC_knlReserved4",
            "placeholder",
            "NOT a slot reserved by the WIPI specification",
        ] {
            assert!(m.contains(needle), "message lost {needle:?}: {m}");
        }
        assert!(
            !m.contains("reserved slot") && !m.contains("Reserved slot"),
            "the message must not re-assert the classification it exists to correct: {m}"
        );
    }

    /// The ids this round reasoned about are the ids the enum actually has.
    ///
    /// `Reserved4 = 36` is what the faulting guest's `IP` (`0x24`) decodes to and
    /// what `knl_interface + 0x90` (field index 36) points at — two independent
    /// confirmations that the slot under discussion is this variant. A renumber
    /// would silently detach the measurement in the doc comment above from the
    /// code it describes.
    #[test]
    fn reserved_block_ids_match_what_was_measured() {
        assert_eq!(WIPICKernelMethodId::Reserved4 as u16, 36);
        assert_eq!(WIPICKernelMethodId::Reserved1 as u16, 33);
        assert_eq!(WIPICKernelMethodId::Reserved11 as u16, 43);
        assert_eq!(WIPICKernelMethodId::Reserved12 as u16, 57);
        assert_eq!(WIPICKernelMethodId::Reserved13 as u16, 58);
    }
}
