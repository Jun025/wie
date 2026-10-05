use wie_core_arm::SvcId;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum InitSvcId {
    ImportTable = 0,
    ImportFunction = 1,
    SetDisplayProperty = 2,
    ApplicationJarPath = 3,
}

impl TryFrom<SvcId> for InitSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::ImportTable,
            1 => Self::ImportFunction,
            2 => Self::SetDisplayProperty,
            3 => Self::ApplicationJarPath,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown LGT init SVC id {}", value.0))),
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum JavaSystemSvcId {
    InterfaceUnk0 = 0,
    DestroyRuntimeContext = 1,
    CreateRuntimeContext = 2,
    LinkImportedClasses = 3,
    SetJarPath = 4,
    StartApplication = 5,
    RegisterClass = 6,
    ResolveClass = 7,
    InitializeClass = 8,
    GetArrayType = 9,
    Instantiate = 10,
    InstantiateArray = 11,
    Unk54 = 12,
    Unk55 = 13,
    StringLiteral = 14,
    PushExceptionFrame = 15,
    PopExceptionFrame = 16,
    StoreReferenceArray = 17,
    GetStringClass = 18,
    GetStringArrayClass = 19,
    PendingException = 20,
    StoreReferenceArrayUnchecked = 21,
    InstantiateMultiArray = 22,
    LinkPublicClass = 23,
    IsClassAssignable = 24,
    ThrowException = 25,
    RaiseNullPointerException = 26,
    RaiseArrayIndexException = 27,
    RaiseArithmeticException = 28,
    Unk1 = 29,
    Unk2 = 30,
    Unk3 = 31,
    GetInterfaceDispatchTable = 32,
    MonitorEnter = 33,
    MonitorExit = 34,
    StoreLongArray = 35,
    LoadLongArray = 36,
    GetInterfaceMethodTable = 37,
    RaiseClassCastException = 38,
}

impl TryFrom<SvcId> for JavaSystemSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0 => Self::InterfaceUnk0,
            1 => Self::DestroyRuntimeContext,
            2 => Self::CreateRuntimeContext,
            3 => Self::LinkImportedClasses,
            4 => Self::SetJarPath,
            5 => Self::StartApplication,
            6 => Self::RegisterClass,
            7 => Self::ResolveClass,
            8 => Self::InitializeClass,
            9 => Self::GetArrayType,
            10 => Self::Instantiate,
            11 => Self::InstantiateArray,
            12 => Self::Unk54,
            13 => Self::Unk55,
            14 => Self::StringLiteral,
            15 => Self::PushExceptionFrame,
            16 => Self::PopExceptionFrame,
            17 => Self::StoreReferenceArray,
            18 => Self::GetStringClass,
            19 => Self::GetStringArrayClass,
            20 => Self::PendingException,
            21 => Self::StoreReferenceArrayUnchecked,
            22 => Self::InstantiateMultiArray,
            23 => Self::LinkPublicClass,
            24 => Self::IsClassAssignable,
            25 => Self::ThrowException,
            26 => Self::RaiseNullPointerException,
            27 => Self::RaiseArrayIndexException,
            28 => Self::RaiseArithmeticException,
            29 => Self::Unk1,
            30 => Self::Unk2,
            31 => Self::Unk3,
            32 => Self::GetInterfaceDispatchTable,
            33 => Self::MonitorEnter,
            34 => Self::MonitorExit,
            35 => Self::StoreLongArray,
            36 => Self::LoadLongArray,
            37 => Self::GetInterfaceMethodTable,
            38 => Self::RaiseClassCastException,
            _ => {
                return Err(wie_util::WieError::FatalError(alloc::format!(
                    "Unknown LGT Java system SVC id {}",
                    value.0
                )));
            }
        })
    }
}

impl From<JavaSystemSvcId> for u32 {
    fn from(value: JavaSystemSvcId) -> Self {
        value as u32
    }
}

impl From<InitSvcId> for u32 {
    fn from(value: InitSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum WIPICSvcId {
    CletRegister = 0x03,
    GetFramebufferPointer = 0x32,
    GetFramebufferWidth = 0x33,
    GetFramebufferHeight = 0x34,
    GetFramebufferBpl = 0x35,
    GetFramebufferBpp = 0x36,
    Printk = 0x64,
    Sprintk = 0x65,
    // Identified 2026-09-27 by behaviour, not by name: the call that ends the program after an
    // LGT title's first-run notice (재실행 요청 · 메모리 확보 · 다운로드 완료 → EZ-i 메뉴). See
    // `wipi_c::terminate_program`. Kernel section index 4.
    TerminateProgram = 0x68,
    Unk1 = 0x6a,
    Exit = 0x6b,
    GetProgramName = 0x6f,
    Alloc = 0x75,
    Calloc = 0x76,
    Free = 0x77,
    GetTotalMemory = 0x78,
    GetFreeMemory = 0x79,
    DefTimer = 0x7a,
    SetTimer = 0x7b,
    UnsetTimer = 0x7c,
    CurrentTime = 0x7d,
    GetSystemProperty = 0x7e,
    SetSystemProperty = 0x7f,
    GetResourceId = 0x80,
    GetResource = 0x81,
    Unk2 = 0x97,
    GetImageProperty = 0xc8,
    GetImageFramebuffer = 0xc9,
    GetScreenFramebuffer = 0xca,
    DestroyOffscreenFramebuffer = 0xcb,
    CreateOffscreenFramebuffer = 0xcc,
    InitContext = 0xcd,
    SetContext = 0xce,
    GetContext = 0xcf,
    PutPixel = 0xd0,
    DrawLine = 0xd1,
    DrawRect = 0xd2,
    FillRect = 0xd3,
    CopyFrameBuffer = 0xd4,
    DrawImage = 0xd5,
    CopyArea = 0xd7,
    DrawArc = 0xd8,
    FillArc = 0xd9,
    DrawString = 0xda,
    GetRgbPixels = 0xdc,
    SetRgbPixels = 0xdd,
    FlushLcd = 0xde,
    GetPixelFromRgb = 0xdf,
    GetRgbFromPixel = 0xe0,
    GetDisplayInfo = 0xe1,
    Repaint = 0xe2,
    GetFont = 0xe3,
    GetFontHeight = 0xe4,
    GetFontAscent = 0xe5,
    GetFontDescent = 0xe6,
    GetStringWidth = 0xe7,
    CreateImage = 0xe9,
    Unk0 = 0xeb,
    // `MC_grpPostEvent` — named by argument shape, not index; see `wipi_c::handle_wipic_svc`.
    PostEvent = 0xee,
    // Drawn as `MC_grpDrawPolygon` — named by argument shape only. By KTF order this slot may be
    // `MC_grpFillPolygon` instead; unresolved, see `wipi_c::handle_wipic_svc`.
    DrawPolygon = 0xf0,
    ImGetSupportModeCount = 0x12c,
    ImGetSupportedModes = 0x12d,
    Unk7 = 0x12e,
    Unk6 = 0x12f,
    ImHandleInput = 0x130,
    TimeNow = 0x320,
    TimeComponent = 0x321,
    TimeConvert = 0x322,
    TimeToTm = 0x323,
    // The 0x320 section is UIC, in KTF's `MC_uic*` order — measured 2026-09-30 at one call site shared
    // by two titles (1cd151222bde · 8f7758fa43b6): 800() → 801("TextComponent") → 802(ctx, class) →
    // 809(comp, x, y, w, h, 3) → 833(comp, n) → 811(comp, 1), and 803(comp) on the old one first.
    // KTF index 9/11/33 = Configure/SetEnable/SetMaxTextSize; the argument counts match. 800–803
    // keep their older names and bodies (they only hand back a handle nothing reads).
    UicConfigure = 0x329,
    UicSetEnable = 0x32b,
    // KTF index 30 = InsertText. 1cd151222bde 0x1b38 (2026-10-01): memset(this+8, 0, 256) →
    // strcpy(this+8, s) → n = strlen(this+8) → 830(this[4], 0, this+8, n) — the same component
    // handle 809/811/833 take, a position, the text and its length.
    UicInsertText = 0x33e,
    UicSetMaxTextSize = 0x341,
    DateTimeToTm = 0x338,
    // 0x384.. is a byte-order section, identified 2026-09-27 from call sites (no symbols):
    // 900/902 take and return 32-bit values stored big-endian into packets, 901/903 16-bit
    // ones (901 turns a constant port 26100 into the connect argument), 904 takes a dotted-quad
    // string literal and its result is the connect address. The 32/16 split is
    // measured; which of each pair is hton vs ntoh is the BSD order and cannot matter on a
    // little-endian guest — both are the same swap.
    Htonl = 0x384,
    Htons = 0x385,
    Ntohl = 0x386,
    Ntohs = 0x387,
    InetAddr = 0x388,
    OpenDatabase = 0x190,
    ReadRecordSingle = 0x191,
    WriteRecordSingle = 0x192,
    CloseDatabase = 0x193,
    Unk12 = 0x194,
    Unk9 = 0x195,
    DeleteRecord = 0x196,
    ListRecord = 0x197,
    UpdateRecord = 0x198,
    SelectRecord = 0x199,
    // `f(dir, buf, 0x3ff, 1)` into a zeroed 0x400 buffer, `== 0` checked; the caller then walks `buf`
    // as NUL-separated names ending in an empty one (87b04639cdfe 0x15a88 "tbl/B" — keeps 3-digit
    // names above 11 — and 0x15da4 "tbl/O" — counts `NNN.dat`). The jar ships `tbl/B/001`–`011` and
    // no `tbl/O`: the packaged songs are 1–11 and anything listed is extra content. That is the
    // WIPI `MC_fsList(name, buf, len, mode)` shape.
    ListDirectory = 0x19a,
    ListDatabases = 0x19c,
    Unk8 = 0x1a0,
    Connect = 0x258,
    Close = 0x259,
    // `f(2, 1)` — AF_INET/SOCK_STREAM, KTF net index 2 `MC_netSocket`. One call site
    // (863b8ab6a21d) picks between this and 0x7d0 on a flag with the same two arguments.
    Socket = 0x25a,
    // KTF net index 3 `MC_netSocketConnect(fd, addr, port, cb, param)`. fe76e641bb3d 0x24428:
    // fd from 602, addr from 904 (a dotted-quad literal), port through 901, cb 0x24501 reads r1 as
    // the result, param 0 on the stack.
    SocketConnect = 0x25b,
    // net base 0x258 + index: idx4 SocketWrite, idx5 SocketRead, idx6 SocketClose.
    SocketWrite = 0x25c,
    SocketRead = 0x25d,
    SocketClose = 0x25e,
    ClipCreate = 0x4b0,
    ClipFree = 0x4b1,
    ClipPutData = 0x4b3,
    Unk15 = 0x4b6,
    ClipGetVolume = 0x4b8,
    ClipSetVolume = 0x4b9,
    Play = 0x4ba,
    // Between Play and Stop in KTF's order (Play, Pause, Resume, Stop). b7699c10dfd1 calls 1211 or 1212
    // on the same clip (`this+4`) from one function, chosen by a boolean argument, and tests `== 1`.
    Pause = 0x4bb,
    Resume = 0x4bc,
    Stop = 0x4bd,
    Unk5 = 0x4c0,
    Vibrator = 0x4c1,
    Unk14 = 0x4c2,
    ClipAllocPlayer = 0x4c5,
    ClipFreePlayer = 0x4c6,
    Unk10 = 0x4ce,
    SetMuteState = 0x4d1,
    GetMuteState = 0x4d2,
    // `f(buf, 0xffe)` into a zeroed 0x1000 buffer, return value unused; the caller then splits `buf`
    // on NUL and skips SMSDATA · MMSDATA · CALLHISTORY · SCHEDULE · NOTICE · PHONEBOOK · PHOTO ·
    // ALARM · MORNINGCALL (87b04639cdfe) — a list of the phone's named data stores. Which stores is
    // not known, so it answers an empty list, which is also the only honest answer here.
    ListDataStores = 0x44c,
    BackLight = 0x578,
    Unk16 = 0x581,
    // Same `(2, 1)` as `Socket` at the same call site, on the other side of a flag; 3ff5948e235e
    // calls it right after `inet_addr`/`htons` and branches on `< 0`. a23f3c9fc2cb imports it WITHOUT
    // 0x25a and calls `f(2, 1)` at 0x3ebc4, then tests `< 0` and -14/-7/-99; -1 takes its own
    // «socket failed» branch. Which socket variant it is does not matter while there is no network.
    SocketAlt = 0x7d0,
}

impl TryFrom<SvcId> for WIPICSvcId {
    type Error = wie_util::WieError;

    fn try_from(value: SvcId) -> Result<Self, Self::Error> {
        Ok(match value.0 {
            0x03 => Self::CletRegister,
            0x32 => Self::GetFramebufferPointer,
            0x33 => Self::GetFramebufferWidth,
            0x34 => Self::GetFramebufferHeight,
            0x35 => Self::GetFramebufferBpl,
            0x36 => Self::GetFramebufferBpp,
            0x64 => Self::Printk,
            0x65 => Self::Sprintk,
            0x68 => Self::TerminateProgram,
            0x6a => Self::Unk1,
            0x6b => Self::Exit,
            0x6f => Self::GetProgramName,
            0x75 => Self::Alloc,
            0x76 => Self::Calloc,
            0x77 => Self::Free,
            0x78 => Self::GetTotalMemory,
            0x79 => Self::GetFreeMemory,
            0x7a => Self::DefTimer,
            0x7b => Self::SetTimer,
            0x7c => Self::UnsetTimer,
            0x7d => Self::CurrentTime,
            0x7e => Self::GetSystemProperty,
            0x7f => Self::SetSystemProperty,
            0x80 => Self::GetResourceId,
            0x81 => Self::GetResource,
            0x97 => Self::Unk2,
            0xc8 => Self::GetImageProperty,
            0xc9 => Self::GetImageFramebuffer,
            0xca => Self::GetScreenFramebuffer,
            0xcb => Self::DestroyOffscreenFramebuffer,
            0xcc => Self::CreateOffscreenFramebuffer,
            0xcd => Self::InitContext,
            0xce => Self::SetContext,
            0xcf => Self::GetContext,
            0xd0 => Self::PutPixel,
            0xd1 => Self::DrawLine,
            0xd2 => Self::DrawRect,
            0xd3 => Self::FillRect,
            0xd4 => Self::CopyFrameBuffer,
            0xd5 => Self::DrawImage,
            0xd7 => Self::CopyArea,
            0xd8 => Self::DrawArc,
            0xd9 => Self::FillArc,
            0xda => Self::DrawString,
            0xdc => Self::GetRgbPixels,
            0xdd => Self::SetRgbPixels,
            0xde => Self::FlushLcd,
            0xdf => Self::GetPixelFromRgb,
            0xe0 => Self::GetRgbFromPixel,
            0xe1 => Self::GetDisplayInfo,
            0xe2 => Self::Repaint,
            0xe3 => Self::GetFont,
            0xe4 => Self::GetFontHeight,
            0xe5 => Self::GetFontAscent,
            0xe6 => Self::GetFontDescent,
            0xe7 => Self::GetStringWidth,
            0xe9 => Self::CreateImage,
            0xeb => Self::Unk0,
            0xee => Self::PostEvent,
            0xf0 => Self::DrawPolygon,
            0x12c => Self::ImGetSupportModeCount,
            0x12d => Self::ImGetSupportedModes,
            0x12e => Self::Unk7,
            0x12f => Self::Unk6,
            0x130 => Self::ImHandleInput,
            0x320 => Self::TimeNow,
            0x321 => Self::TimeComponent,
            0x322 => Self::TimeConvert,
            0x323 => Self::TimeToTm,
            0x329 => Self::UicConfigure,
            0x32b => Self::UicSetEnable,
            0x33e => Self::UicInsertText,
            0x341 => Self::UicSetMaxTextSize,
            0x338 => Self::DateTimeToTm,
            0x384 => Self::Htonl,
            0x385 => Self::Htons,
            0x386 => Self::Ntohl,
            0x387 => Self::Ntohs,
            0x388 => Self::InetAddr,
            0x190 => Self::OpenDatabase,
            0x191 => Self::ReadRecordSingle,
            0x192 => Self::WriteRecordSingle,
            0x193 => Self::CloseDatabase,
            0x194 => Self::Unk12,
            0x195 => Self::Unk9,
            0x196 => Self::DeleteRecord,
            0x197 => Self::ListRecord,
            0x198 => Self::UpdateRecord,
            0x199 => Self::SelectRecord,
            0x19a => Self::ListDirectory,
            0x19c => Self::ListDatabases,
            0x1a0 => Self::Unk8,
            0x258 => Self::Connect,
            0x259 => Self::Close,
            0x25a => Self::Socket,
            0x25b => Self::SocketConnect,
            0x25c => Self::SocketWrite,
            0x25d => Self::SocketRead,
            0x25e => Self::SocketClose,
            0x4b0 => Self::ClipCreate,
            0x4b1 => Self::ClipFree,
            0x4b3 => Self::ClipPutData,
            0x4b6 => Self::Unk15,
            0x4b8 => Self::ClipGetVolume,
            0x4b9 => Self::ClipSetVolume,
            0x4ba => Self::Play,
            0x4bb => Self::Pause,
            0x4bc => Self::Resume,
            0x4bd => Self::Stop,
            0x4c0 => Self::Unk5,
            0x4c1 => Self::Vibrator,
            0x4c2 => Self::Unk14,
            0x4c5 => Self::ClipAllocPlayer,
            0x4c6 => Self::ClipFreePlayer,
            0x4ce => Self::Unk10,
            0x4d1 => Self::SetMuteState,
            0x4d2 => Self::GetMuteState,
            0x578 => Self::BackLight,
            0x581 => Self::Unk16,
            0x44c => Self::ListDataStores,
            0x7d0 => Self::SocketAlt,
            _ => return Err(wie_util::WieError::FatalError(alloc::format!("Unknown LGT WIPIC SVC id {}", value.0))),
        })
    }
}

impl From<WIPICSvcId> for u32 {
    fn from(value: WIPICSvcId) -> Self {
        value as u32
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum StdlibSvcId {
    Unk2 = 0x3f6,
    Sprintf = 0x3f7,
    /// `f(buf, fmt, ap)` — identified at its one measured call site (2dbde9acca99, 2026-09-27): a
    /// variadic function (`push {r0-r3}` on entry) formats its own `fmt, ...` into a stack buffer
    /// and passes `ap` as the address of its first variadic slot (`add r2, sp, #0x120`, the word
    /// after the saved `fmt`), then opens the resulting path.
    Vsprintf = 0x3f9,
    Atoi = 0x3fb,
    /// `f(str, &end, base)` — acc4215b7ec0's only call site (`0x36192`, 2026-10-02): it copies a
    /// `|`-delimited field such as `0xFFAE08` out of a text resource, calls `f(field, &end, 16)`, and
    /// formats the result with `"%d"`. That is strtol's shape (strtoul's too — the two agree on every
    /// value this title passes, all `<= 0xFFFFFF`; revisit if a title parses past `0x7fffffff`).
    Strtol = 0x3ff,
    Rand = 0x403,
    Srand = 0x404,
    Strcpy = 0x405,
    Strncpy = 0x406,
    Strcat = 0x407,
    /// `f(dst, src, n)` — 4fdbd64c9fbd, in a key handler (2026-09-27): `memset(dst, 0, 0x42)`,
    /// a two-argument string call on `dst`, then `if (n) f(dst, local_buf, n)`. Its import slot sits
    /// between that two-argument call's and memmove's (0x415) in the image's ascending slot table,
    /// and in this table each string call is followed by its counted form (0x405 strcpy / 0x406
    /// strncpy). strncat is the one `(dst, src, n)` string function not yet placed. Shape plus
    /// position, not a symbol name — revisit if a title passes something other than a string.
    Strncat = 0x408,
    Strcmp = 0x409,
    Unk4 = 0x40a,
    Strstr = 0x410,
    Strlen = 0x411,
    Memcpy = 0x414,
    Memmove = 0x415,
    Memset = 0x418,
    Time = 0x41a,
    Localtime = 0x420,
    Unk3 = 0x424,
    /// Identified from the caller, not guessed: the only call site measured (b7699c10dfd1, 2026-09-27)
    /// is C++ `operator new` — `if (size == 0) size = 1; p = f(size); if (p) return p;` else the
    /// exception path.
    Malloc = 0x426,
    /// Its pair, from the same image: `operator delete` is `if (p) f(p);`, reaching this slot.
    Free = 0x428,
}

impl From<StdlibSvcId> for u32 {
    fn from(value: StdlibSvcId) -> Self {
        value as u32
    }
}

#[cfg(test)]
mod tests {
    use wie_core_arm::SvcId;

    use super::{JavaSystemSvcId, WIPICSvcId};

    /// The Java-system ids are dense, and the `try_from` table is written by hand next to the enum,
    /// so adding a variant without a table row compiles and only fails when a guest reaches it:
    /// `StoreLongArray` (import 0xfd) did exactly that, and its handler's own unit test stayed green
    /// because it calls the handler directly. Every declared id must come back as itself.
    #[test]
    fn java_system_svc_ids_round_trip_and_table_stays_fail_closed() {
        let last = JavaSystemSvcId::RaiseClassCastException as u32;
        for id in 0..=last {
            let variant = JavaSystemSvcId::try_from(SvcId(id)).unwrap_or_else(|_| panic!("Java system SVC id {id} has no try_from row"));
            assert_eq!(u32::from(variant), id);
        }
        assert!(JavaSystemSvcId::try_from(SvcId(last + 1)).is_err());
    }

    /// `MC_netSocketWrite` (`0x25c`) and `MC_netSocketRead` (`0x25d`) are in the table.
    ///
    /// `02ad8b5c` added both rows so 테라-영원의혼돈 gets `-1` and falls back to offline play instead of
    /// dying on `Unknown LGT WIPIC SVC id 604`; the base swap (#161) dropped them and nothing noticed,
    /// because no fixture reaches the network path. Shared `wie_wipi_c::api::net` has no socket
    /// write/read, so these cannot be re-wired to it — they are LGT-local `-1` stubs again.
    #[test]
    fn wipic_svc_604_605_socket_write_read_are_in_the_table() {
        let write = WIPICSvcId::try_from(SvcId(604)).expect("SVC 604 (MC_netSocketWrite) must be in the table");
        assert!(matches!(write, WIPICSvcId::SocketWrite));
        assert_eq!(u32::from(write), 0x25c);

        let read = WIPICSvcId::try_from(SvcId(605)).expect("SVC 605 (MC_netSocketRead) must be in the table");
        assert!(matches!(read, WIPICSvcId::SocketRead));
        assert_eq!(u32::from(read), 0x25d);
    }

    /// SVC `0x581` resolves, and an id that is genuinely unmapped still errors.
    ///
    /// `handle_wipic_svc` branches on exactly this conversion, so an id missing
    /// from the table is what produced "Unknown LGT WIPIC SVC id 1409" for one
    /// title (upstream `dlunch/wie#1260`). The second half is the half that keeps
    /// mattering: the table has to stay **fail-closed**, because a catch-all that
    /// resolved unknown ids would turn "we have never seen this call" into a
    /// silent pass — the shape this repo keeps having to name.
    ///
    /// ── Why this test is being re-added rather than written ──────────────────
    /// It existed (`4e37e9e4`), and the crate rename `wie_lgt` → `wie-lgt`
    /// dropped it along with the file it lived in; a machine audit found it by
    /// diffing *function names* across the swap. Before restoring it, both halves
    /// were re-measured against HEAD rather than assumed, because the audit's own
    /// note warned that a sibling case (`misc_unk9`) was unrestorable — its target
    /// function no longer exists. Here the target does: `Unk16 = 0x581` is still
    /// declared, `0x581 => Self::Unk16` is still in `try_from`, and `0x582` still
    /// appears nowhere in this file, so the old body applies unchanged.
    #[test]
    fn wipic_svc_0x581_maps_and_table_stays_fail_closed() {
        let id = WIPICSvcId::try_from(SvcId(0x581)).expect("SVC 0x581 (misc index 9) must be in the table");
        assert_eq!(u32::from(id), 0x581);

        // ...and ids that really are unmapped still error rather than silently resolving.
        assert!(WIPICSvcId::try_from(SvcId(0x582)).is_err());
    }

    /// `MC_dbListDataBase` (412 = `0x19c`) is in the table.
    ///
    /// The row was in the pre-swap tree (`d70b93f8`) and #161 dropped it; 리듬페스티발 ×2 and 하이브리드 then
    /// died on `Unknown LGT WIPIC SVC id 412` during boot. It came back as upstream `3e203809`'s mapping.
    #[test]
    fn wipic_svc_412_list_databases_is_in_the_table() {
        let id = WIPICSvcId::try_from(SvcId(0x19c)).expect("SVC 412 (MC_dbListDataBase) must be in the table");
        assert!(matches!(id, WIPICSvcId::ListDatabases));
        assert_eq!(u32::from(id), 412);
    }

    /// `MC_grpGetContext` (207 = `0xcf`) is in the table, between its two neighbours.
    ///
    /// Same shape as 412 above: present in `d70b93f8`, dropped by #161, and 바이오크로니클 died on
    /// `Unknown LGT WIPIC SVC id 207`. `InitContext`/`SetContext` never left, so only the getter was missing.
    #[test]
    fn wipic_svc_207_get_context_is_in_the_table() {
        let id = WIPICSvcId::try_from(SvcId(0xcf)).expect("SVC 207 (MC_grpGetContext) must be in the table");
        assert!(matches!(id, WIPICSvcId::GetContext));
        assert_eq!(u32::from(id), 207);
    }

    /// `MC_netSocketConnect` (603 = `0x25b`) is in the table. fe76e641bb3d died on
    /// `Unknown LGT WIPIC SVC id 603` at its connect call site (docs/report/0408).
    #[test]
    fn wipic_svc_603_socket_connect_is_in_the_table() {
        let id = WIPICSvcId::try_from(SvcId(603)).expect("SVC 603 (MC_netSocketConnect) must be in the table");
        assert!(matches!(id, WIPICSvcId::SocketConnect));
        assert_eq!(u32::from(id), 0x25b);
    }
}
