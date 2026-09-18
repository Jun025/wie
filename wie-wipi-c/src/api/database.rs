use alloc::{borrow::ToOwned, boxed::Box, format, str, string::String, vec, vec::Vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::WIPICWord;

use wie_backend::Database;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::context::WIPICContext;

/// Per-handle state for KTF's stream-style database API.
///
/// KTF's `stream_read` / `stream_write` slots behave like a record-scoped
/// `fread` / `fwrite` pair rather than the standard WIPI record-by-id API
/// — the same record id 1 is walked sequentially with implicit cursors.
/// The original interface field names (`read_record_single`,
/// `write_record_single`) were a pre-disassembly guess; the impl-side names
/// `stream_read` / `stream_write` reflect the verified semantics.
///
/// The handle, including its read/write cursors and the in-memory mirror
/// of record 1, lives entirely in emulated memory: the `DatabaseHandle`
/// struct sits at the pointer returned from `open_database`, and the
/// mirror itself is a separate guest-heap allocation referenced by
/// `buffer_ptr`. Every op reads the struct, mutates it, writes it back —
/// no host-side global state.
///
/// `select_record` with a non-zero recid is treated as a seek: KTF apps
/// use slot 4 to position the cursor at known byte offsets within the
/// single backing record, e.g. for multi-slot save files.
#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct DatabaseHandle {
    magic: u32,
    name: [u8; 32], // TODO hardcoded max size
    read_cursor: u32,
    write_cursor: u32,
    buffer_ptr: u32,
    buffer_len: u32,
    buffer_capacity: u32,
}

const MIN_BUFFER_CAPACITY: u32 = 64;
const KTF_DATABASE_STORAGE_LIMIT: u64 = 1024 * 1024;
// "MCDB" — sentinel at the start of the handle struct so we can distinguish
// a real DB handle pointer from an unrelated guest pointer (e.g. a C-string
// name pointer that KTF's slot 6 passes through the same SVC argument slot).
const DATABASE_HANDLE_MAGIC: u32 = 0x4D434442;
const MAX_NAME_LEN: usize = 31; // leave a byte for null terminator inside the 32-byte field

pub async fn open_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, mode: i32, r#type: i32) -> Result<i32> {
    tracing::debug!("MC_dbOpenDataBase({ptr_name:#x}, {mode}, {type})");

    // Guest-provided C string — invalid UTF-8 must not bring down the
    // emulator. Treat it as a bad parameter and return -22, matching the
    // fail-soft behaviour of the other name-keyed entry points in this
    // file (`stat_by_name_ktf`, `exists_database_ktf`).
    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        tracing::warn!("MC_dbOpenDataBase: invalid utf8 name @ {ptr_name:#x}");
        return Ok(-22);
    };

    // Validate before any repository side effects. Mode 4 deletes record 1
    // up front, so a too-long name reaching that path would wipe data we
    // can't open a handle for anyway.
    if name.len() > MAX_NAME_LEN {
        tracing::warn!("MC_dbOpenDataBase: name {name:?} too long ({} > {MAX_NAME_LEN})", name.len());
        return Ok(-22); // M_E_BADRECID — closest WIPI parameter-error idiom in this file
    }

    let packaged = read_packaged_database(context, &name).await?;

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(&name, &pid).await;

    if !exists && packaged.is_none() && mode == 1 {
        return Ok(-12); // M_E_NOENT
    }

    // Mode 4 (`MC_DB_CREATE`) wipes any prior contents up front unless the
    // DB is backed by a packaged resource. Other modes seed the per-handle
    // buffer with the existing record or packaged data so seek+overlay writes
    // preserve unrelated bytes (multi-slot saves at fixed byte offsets).
    let initial: Vec<u8> = if exists {
        let mut db = system.platform().database_repository().open(&name, &pid).await;
        if mode == 4 && packaged.is_none() {
            db.delete(1).await;
            Vec::new()
        } else if let Some(data) = db.get(1).await {
            data
        } else if let Some(data) = packaged {
            db.set(1, &data).await;
            data
        } else {
            Vec::new()
        }
    } else if let Some(data) = packaged {
        let mut db = system.platform().database_repository().open(&name, &pid).await;
        db.set(1, &data).await;
        data
    } else if mode == 4 {
        system.platform().database_repository().open(&name, &pid).await;
        Vec::new()
    } else {
        Vec::new()
    };

    let name_bytes = name.as_bytes();

    let mut handle = DatabaseHandle {
        magic: DATABASE_HANDLE_MAGIC,
        name: [0; 32],
        read_cursor: 0,
        write_cursor: 0,
        buffer_ptr: 0,
        buffer_len: 0,
        buffer_capacity: 0,
    };
    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    if !initial.is_empty() {
        let cap = (initial.len() as u32).max(MIN_BUFFER_CAPACITY);
        let buf_ptr = context.alloc_raw(cap)?;
        context.write_bytes(buf_ptr, &initial)?;
        handle.buffer_ptr = buf_ptr;
        handle.buffer_len = initial.len() as u32;
        handle.buffer_capacity = cap;
    }

    let ptr_handle = context.alloc_raw(size_of::<DatabaseHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;

    tracing::debug!("Created database handle {ptr_handle:#x} for {name}");

    Ok(ptr_handle as _)
}

pub async fn close_database(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbCloseDataBase({db_id:#x})");

    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    // The buffer was kept in sync with disk via write-through on every
    // `stream_write`, so close just frees the guest-heap allocations.
    if handle.buffer_ptr != 0 && handle.buffer_capacity > 0 {
        context.free_raw(handle.buffer_ptr, handle.buffer_capacity)?;
    }
    context.free_raw(db_id as _, size_of::<DatabaseHandle>() as _)?;

    Ok(0) // success
}

pub async fn list_record(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecords({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    let Some(db) = get_database_from_db_id(context, db_id).await? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    let ids = db.get_record_ids().await;

    let mut cursor = 0;
    for &id in &ids {
        write_generic(context, buf_ptr + cursor, id)?;
        cursor += size_of::<WIPICWord>() as u32;
    }

    Ok(ids.len() as _)
}

/// KTF WIPI-C **Database slot 8**. The standard WIPI header calls this
/// `MC_dbSortRecords`; **this code does not claim to know what KTF puts there.**
///
/// ── The arity is 2, and it is disassembled rather than inferred ──────────────
/// The first version of this function implemented the header's signature —
/// `(fd, M_Int32 *buf, M_Int32 len, compare, filter)` — and wrote record ids into
/// what it took to be `buf`. The second withdrew the 4th and 5th and kept three.
/// Both were reading registers the caller never loaded with an argument.
///
/// The client images load verbatim at `IMAGE_BASE = 0x100000`
/// (`wie_ktf::emulator`), so a guest address minus `0x100000` is a file offset in
/// `client.bin<bss>` and the call site can be read statically. The slot-8 call
/// sites — **three**, see the scan note below — all have this shape
/// (`0103451A.jar` at `0x10571a`; `01031C0A.jar` at `0x1243c8` and `0x128f3a`):
///
/// ```text
///   ldr  r3, [pc, #k1]      ; k1 and k2 are DIFFERENT literal-pool entries:
///   add  r3, sl             ; r3 is reloaded between the two uses below, so
///   ldr  r3, [r3]           ; r2 and r0 come from different globals
///   ldr  r2, [r3]           ; r2 = &WIPICDatabaseInterface  (global slot: see below)
///   ldr  r3, [pc, #k2]      ; <-- the reload
///   add  r3, sl
///   movs r1, #1             ; arg1 — an immediate at all three slot-8 sites
///   ldr  r0, [r3]           ; arg0 — a pointer read out of a different guest global
///   ldr  r3, [r2, #0x20]    ; r3 = table[8] = this function's own SVC stub
///   bl   __call_via_r3      ; ARM ADS veneer whose whole body is `bx r3`
/// ```
///
/// * **`r3` is call machinery.** `sort_records` is the 9th `TargetPtr` of
///   `WIPICDatabaseInterface`. `TargetPtr` is `u32` in the build this engine
///   ships, so that is `8 × 4 = +0x20` — under the `simulation` feature it is
///   `usize` and the offset would be `+0x40`, which is worth knowing before
///   redoing the arithmetic but does not change the conclusion, because the
///   conclusion does not rest on it (next bullet). `bx r3` therefore enters
///   *this* entry point and the register still holds its address when the SVC
///   fires. A 4th parameter invents one.
/// * **`r2` is call machinery too, and the register dump proves it directly.**
///   At the fault `R3 = 0x71001781` **is** `[R2 + 0x20]`, and that value is the
///   SortRecords SVC stub: `PC = 0x7100178a` is inside it and `IP = 0x70008` is
///   the id `SvcId::get` reads, `(WIPICTableId::Database << 16) | 8`. So `R2`
///   *is* the database interface — no arithmetic, no heuristic. (The weaker
///   argument this comment used to make — "`R2` is identical in both dumps
///   while `R0` differs" — is not wrong but proves little: a constant argument
///   would also be identical in both dumps.)
/// * **The 5th "argument" is the caller's saved `r10`.** `read_param(4)` resolves
///   to `[SP+0]`, and `[SP+0] == SL` in both dumps (`0x134ac8` / `0x13e878`).
///
/// So the call is `f(r0, r1)` with `r1 == 1`, and the header's five-parameter
/// shape fits none of it.
///
/// **How "three sites" was counted, and what identifies them.** A decoder-driven
/// sweep of both images finds every `ldr rT,[rN,#0x20]` whose `rT` is then called
/// (`blx rT`, or `bl` into a `bx rT` veneer) — **15 hits in `0103451A`, 18 in
/// `01031C0A`**. Most of those are *other* tables: the discriminator is not which
/// register holds the base (allocation is arbitrary; `01031C0A` reaches three
/// different interfaces through `r2` alone) but **which global the table pointer
/// was loaded from**. Back-tracking each hit to that global leaves
/// `0103451A: 0x134c38` and `01031C0A: 0x13eb30` as the interface the dump above
/// identifies, and exactly three hits resolve to those. The remaining ones land
/// on other globals (`0x068b80`/`0x068b40`, `0x13eb2c`, `0x13eb54`, `0x073e40`,
/// `0x073e00`) and pass different shapes — `0x13eb54`'s two sites push a stack
/// argument, so that table's slot 8 takes at least five. **What the sweep cannot
/// see** is written down with it, next to the scanner itself, in
/// `~/orchestrator/reports/evidence/wie-game-lab-repair-campaign-pilot-unimpl-stub-fix3/`.
///
/// **What `r0` points at**, since "another name pointer" was as far as the
/// previous revision got: in both images the address lands on a tail-merged
/// string literal inside the title's own resource-path pool — `0x13184c` is
/// `"res"` (the tail of `"res/anidata.res"`) and `0x135ae4` is `"ga"` (the tail
/// of `"/ga/per.ga"`). Short NUL-terminated ASCII tokens, and in particular not
/// handles: `open_database` returns an `alloc_raw` pointer, which lives in the
/// emulator heap, not in the guest image. What the guest *means* by the token is
/// not settled here.
///
/// None of that is surprising for this table: `select_record_ktf` and
/// `stat_by_name_ktf` below already carry the note that KTF's slots diverge from
/// the header, and the struct comment at the top of this file records that the
/// original field names were "a pre-disassembly guess". Slot 8 is the same shape,
/// and the first two versions of this function repeated that guess.
///
/// ── So it refuses, and it refuses LOUDLY ─────────────────────────────────────
/// It takes the two arguments the disassembly accounts for, and names them
/// `arg0`/`arg1` because naming them `db_id`/`buf_ptr` would assert the header
/// layout the measurement refuted. Reading fewer registers costs nothing *here* —
/// the function always fails, so a dropped argument cannot change behaviour,
/// while a register that was never an argument turns into a published coordinate
/// that sends the next round somewhere that does not exist. That is exactly what
/// happened to `r3`. **It never writes to guest memory** — the withdrawn write
/// loop would have written record ids starting at `0x1`, which is precisely the
/// "corrupts the guest's heap silently" this doc comment used to warn about while
/// doing it. `sort_records_never_writes_to_guest_memory_test` pins that.
///
/// **What `r0` actually is (2026-09-19), and how much that narrows it.** The
/// previous revision left it at "a short ASCII token". It is more specific than
/// that, and the extra facts came from reading the *other* calls through the same
/// interface rather than from staring harder at this one:
///
/// * **The token is the title's own file EXTENSION.** `0x13184c` is `"res"`, the
///   tail of `"res/anidata.res"`; `0x135ae4` is `"ga"`, the tail of
///   `"/ga/per.ga"`. Both are tail-merged literals, and both images carry a
///   string-pointer table whose entries point at exactly such tails (path, and
///   path + k for the basename and the extension) — so the "token" is a member
///   of that table, not an ad-hoc string.
/// * **The names these titles actually open are NOT that token.** Sweeping every
///   indirect call through the same database global finds slot 0 called with
///   `"res/save.sav"` (`0103451A`) and `"/ga/aysis.dat"` (`01031C0A`). So slot 8
///   is being handed something categorically different from a database name.
/// * **The return value is discarded at all three call sites.** Nothing tests
///   `r0` afterwards — `0x10571c` falls into an unrelated global load, `0x1243ca`
///   into a run of `bl`s, `0x128f3c` into the `Open` below. Whatever slot 8 is,
///   these titles do not read its answer.
/// * **Its neighbours in the call sequence pin the shape.** At `01031C0A`
///   `0x128f08` the order is `slot16("/ga/aysis.dat", 1)` → *if non-zero* →
///   `slot8("ga", 1)` → `slot0("/ga/aysis.dat", 8, 1)` → store the fd. Slot 16 is
///   `Exists` in this table and takes `(name, type)`; slot 8 takes the **same
///   two-argument shape** with the same constant `1`.
///
/// **What that does NOT settle, and this lineage has been wrong twice already:**
/// which operation it is. "Register the app's file extension", "delete by
/// pattern", "list databases of this type" all fit `f(ext, 1)` with an ignored
/// result, and nothing above separates them. A name is not written here until
/// something does.
///
/// **Part of the slot numbering is now guest-confirmed, which none of it was
/// before.** At `0103451A:0x117f70` the sweep sees slot 0 return a value that is
/// then threaded as the first argument into slot 2 and slot 3 — `Open` →
/// `StreamWrite` → `Close`, the order this file already assumed from the KTF
/// header. That is evidence for **slots 0, 2 and 3**. Slot 1 is observed through
/// the same object (six sites) but its argument threading was not checked, and
/// slots 4-7 and 9-15 were not observed at all in these two images: this
/// confirms part of the table, not the table.
///
/// **Two method notes worth keeping, because both cost time to rediscover.** The
/// interface is reached as `table = *(*global)` — the global holds an object
/// whose *first word* is the table, so a sweep that stops after one dereference
/// misses these call sites. And the sl-relative pointer table is **relocated at
/// load by a delta that is not `IMAGE_BASE`** (measured `+0xE40` for `0103451A`,
/// `+0x1330` for `01031C0A`): read a static word, add that delta, and the string
/// lands exactly. The delta is checkable rather than fitted — for `0103451A` it
/// predicts `*(sl+0x150) == 0x14a940`, which is what the register dump's `R5`
/// holds.
///
/// So the diagnostic below now quotes the token itself. That is deliberately the
/// only behaviour change: the next title to reach this slot names its own
/// extension in the failure instead of costing somebody a disassembly.
pub async fn sort_records(context: &mut dyn WIPICContext, arg0: WIPICWord, arg1: WIPICWord) -> Result<i32> {
    let token = slot8_token(context, arg0);
    tracing::debug!("KTF database slot 8 (header name: MC_dbSortRecords)({arg0:#x}, {arg1:#x}) token={token:?}");

    let quoted = match &token {
        Some(t) => format!(" ({t:?})"),
        None => String::new(),
    };

    Err(WieError::Unimplemented(format!(
        "8: KTF database slot 8 (header name MC_dbSortRecords) — argument layout unknown, \
         measured r0={arg0:#x}{quoted} r1={arg1:#x} (arity 2: r2 holds the interface table, r3 the callee address); \
         the header's (fd, buf, len, compare, filter) does not fit"
    )))
}

/// Longest token `sort_records` will quote back from guest memory.
///
/// Bounded and printable-only on purpose. `arg0` is a guest pointer and nothing
/// here can prove it is not a path — `wie_validate`'s `--guest-stdout` flag is
/// opt-in for exactly that reason, and AGENTS.md's smoke-gate note draws the same
/// line ("identifiers and expected status only, never paths or bytes"). The two
/// tokens measured in the field are 3 and 2 bytes; 16 leaves room without turning
/// this into a general string dump. Anything longer, anything non-printable, and
/// anything unreadable is simply not quoted — the raw pointer is still reported,
/// so the diagnostic never gets *worse* than it was.
const SLOT8_TOKEN_MAX: usize = 16;

fn slot8_token(context: &mut dyn WIPICContext, ptr: WIPICWord) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    // Failure-tolerant by construction: a bad pointer must yield "no token", not
    // an error that replaces the Unimplemented this function exists to raise.
    let bytes = read_null_terminated_string_bytes(context, ptr).ok()?;
    if bytes.is_empty() || bytes.len() > SLOT8_TOKEN_MAX || !bytes.iter().all(|b| (0x20..0x7f).contains(b)) {
        return None;
    }
    str::from_utf8(&bytes).ok().map(ToOwned::to_owned)
}

/// `MC_dbGetNumberOfRecords(dbID)` — number of records in the database, or the
/// M_E_INVALIDHANDLE error for a bad handle. Routes to the existing DB layer.
pub async fn get_number_of_records(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbGetNumberOfRecords({db_id:#x})");

    let Some(db) = get_database_from_db_id(context, db_id).await? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    Ok(db.get_record_ids().await.len() as _)
}

/// Returns the database storage available to a KTF application.
///
/// Although the standard interface names function ID 12 `MC_dbListDataBase`,
/// KTF titles use its no-argument return value as an available-storage byte count.
/// Known callers reject values below 0x100 and 0x1200 respectively.
pub async fn list_databases(context: &mut dyn WIPICContext) -> Result<i32> {
    let system = context.system();
    let pid = system.pid().to_owned();
    let usage = system.platform().database_repository().usage(&pid).await;
    let available = KTF_DATABASE_STORAGE_LIMIT.saturating_sub(usage).min(i32::MAX as u64) as i32;

    tracing::debug!("MC_dbListDataBase() = {available} (used={usage}, limit={KTF_DATABASE_STORAGE_LIMIT})");
    Ok(available)
}

pub async fn seek_record_single(context: &mut dyn WIPICContext, db_id: i32, offset: i32, origin: i32) -> Result<i32> {
    tracing::debug!("MC_dbSeekRecordSingle({db_id:#x}, {offset}, {origin})");

    let Some(mut handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    let base = match origin {
        0 => 0,
        1 => handle.read_cursor as i64,
        2 => handle.buffer_len as i64,
        // -1, though `Invalid = -9` would fit this arm exactly — the one outlier where the
        // vocabulary does. Left alone deliberately: changing it is a behaviour change whose
        // compatibility cannot be checked in this repo, and unlike `get_resource` (2026-09-07)
        // there is no UB here to buy with that price. docs/wipi-c-abi-error-codes.md §2.
        _ => return Ok(-1),
    };
    let position = (base + offset as i64).clamp(0, handle.buffer_len as i64) as u32;
    handle.read_cursor = position;
    handle.write_cursor = position;
    write_generic(context, db_id as _, handle)?;

    Ok(position as i32)
}

pub async fn list_record_info(context: &mut dyn WIPICContext, ptr_name: WIPICWord, buf_ptr: WIPICWord, capacity: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecordInfo({ptr_name:#x}, {buf_ptr:#x}, {capacity})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        return Ok(-22);
    };
    let system = context.system();
    let pid = system.pid().to_owned();

    if !system.platform().database_repository().exists(&name, &pid).await {
        if let Some(data) = read_packaged_database(context, &name).await? {
            if capacity > 0 {
                write_generic(context, buf_ptr, 1u32)?;
                write_generic(context, buf_ptr + 4, 0u32)?;
                write_generic(context, buf_ptr + 8, data.len() as u32)?;
            }
            return Ok(0);
        }
        return Ok(-12); // M_E_NOENT
    }

    let db = system.platform().database_repository().open(&name, &pid).await;
    let ids = db.get_record_ids().await;

    let mut written = 0;
    for id in ids {
        if written >= capacity {
            break;
        }

        let Some(data) = db.get(id).await else {
            continue;
        };

        let entry_ptr = buf_ptr + written * 12;
        write_generic(context, entry_ptr, id)?;
        write_generic(context, entry_ptr + 4, 0u32)?;
        write_generic(context, entry_ptr + 8, data.len() as u32)?;
        written += 1;
    }

    Ok(0)
}

pub async fn exists_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, r#type: i32) -> Result<i32> {
    tracing::debug!("MC_dbExistsDataBase({ptr_name:#x}, {type})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        return Ok(-22);
    };
    if read_packaged_database(context, &name).await?.is_some() {
        return Ok(0);
    }

    let system = context.system();
    let pid = system.pid().to_owned();
    if system.platform().database_repository().exists(&name, &pid).await {
        Ok(0)
    } else {
        Ok(-12) // M_E_NOENT
    }
}

pub async fn stream_write(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("db.stream_write({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    let Some(mut handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    // Cursor + len is guest-controlled, so guard the arithmetic. An
    // overflowed `new_end` would silently bypass the capacity check below
    // and let a write spill into unrelated guest memory.
    let Some(new_end) = handle.write_cursor.checked_add(buf_len) else {
        return Ok(-22); // M_E_BADRECID — closest "bad parameter" code
    };

    let old_len = handle.buffer_len;

    // Grow the guest-heap buffer if the next write would land past its
    // end. Doubling-on-demand starting from MIN_BUFFER_CAPACITY keeps the
    // realloc count amortized; alloc/free is a guest-side `WIPICContext`
    // primitive so we copy old bytes via host-side scratch.
    if new_end > handle.buffer_capacity {
        let Some(rounded) = new_end.checked_next_power_of_two() else {
            return Ok(-22);
        };
        let new_cap = rounded.max(MIN_BUFFER_CAPACITY);
        let new_ptr = context.alloc_raw(new_cap)?;
        if handle.buffer_len > 0 && handle.buffer_ptr != 0 {
            let mut old_data = vec![0u8; handle.buffer_len as usize];
            context.read_bytes(handle.buffer_ptr, &mut old_data)?;
            context.write_bytes(new_ptr, &old_data)?;
        }
        if handle.buffer_ptr != 0 && handle.buffer_capacity > 0 {
            context.free_raw(handle.buffer_ptr, handle.buffer_capacity)?;
        }
        handle.buffer_ptr = new_ptr;
        handle.buffer_capacity = new_cap;
    }

    // If the write_cursor was seeked past the prior end (e.g. via a slot 4
    // multi-slot save), the bytes between the old end and the cursor were
    // never initialised. `alloc_raw` doesn't guarantee zeroed memory and
    // the snapshot below is flushed straight to disk, so explicitly zero
    // the gap to avoid leaking heap residue into the save file. This must
    // run for `buf_len == 0` too: `new_end == write_cursor` still extends
    // `buffer_len`, so the gap would otherwise be snapshotted uninitialised.
    if handle.write_cursor > old_len {
        let gap_size = (handle.write_cursor - old_len) as usize;
        let zeros = vec![0u8; gap_size];
        context.write_bytes(handle.buffer_ptr + old_len, &zeros)?;
    }

    if buf_len > 0 {
        let mut buf = vec![0u8; buf_len as usize];
        context.read_bytes(buf_ptr, &mut buf)?;
        context.write_bytes(handle.buffer_ptr + handle.write_cursor, &buf)?;
    }

    handle.write_cursor = new_end;
    if new_end > handle.buffer_len {
        handle.buffer_len = new_end;
    }
    write_generic(context, db_id as _, handle)?;

    // Write-through to disk on every stream_write. Some titles tear down
    // the game without making a final `close_database` call after their
    // save sequence — relying on close as the only flush point loses all
    // the writes that landed since the session opened. Flushing eagerly
    // costs an extra small file write per call but keeps the on-disk state
    // consistent if the process exits or the title forgets to close.
    let mut snapshot = vec![0u8; handle.buffer_len as usize];
    if handle.buffer_ptr != 0 && handle.buffer_len > 0 {
        context.read_bytes(handle.buffer_ptr, &mut snapshot)?;
    }
    if let Some(mut db) = open_db_for_handle(context, &handle).await {
        db.set(1, &snapshot).await;
    }

    Ok(buf_len as _)
}

/// Standard WIPI `MC_dbDeleteRecord(handle, rec_id)` — delete a single
/// record by id from an open DB handle.
pub async fn delete_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({db_id:#x}, {rec_id})");

    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    let Some(mut db) = open_db_for_handle(context, &handle).await else {
        return Ok(-25);
    };
    let ok = db.delete(rec_id as u32).await;
    Ok(if ok { 0 } else { -22 })
}

/// KTF reuses slot 6 with two call shapes that share the same SVC signature:
///
///  - standard WIPI: `delete_record(handle, rec_id)`
///  - KTF custom:    `(name_ptr, type)` — used as a name-keyed cleanup
///
/// Both pass two ints, so we disambiguate by reading the magic field at
/// `a0`. A real handle starts with `DATABASE_HANDLE_MAGIC`; a name pointer
/// (or anything else) does not, and we fall back to a no-op.
pub async fn delete_record_ktf(context: &mut dyn WIPICContext, a0: i32, a1: i32) -> Result<i32> {
    if load_handle(context, a0)?.is_some() {
        return delete_record(context, a0, a1).await;
    }

    // Not a real handle — KTF name-keyed form. No-op preserves saves; the
    // bytes of a name string would otherwise round-trip into the standard
    // path and silently delete record 1 of the just-saved DB.
    tracing::debug!("MC_dbDeleteRecord(name-keyed @ {a0:#x}, {a1}) -> 0 (no-op)");
    Ok(0)
}

pub async fn delete_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, flags: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteDataBase({ptr_name:#x}, {flags})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        return Ok(-22);
    };
    let system = context.system();
    let pid = system.pid().to_owned();

    let deleted = system.platform().database_repository().delete(&name, &pid).await;
    if deleted || !system.platform().database_repository().exists(&name, &pid).await {
        Ok(0)
    } else {
        Ok(-12) // M_E_NOENT
    }
}

pub async fn update_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbUpdateRecord({db_id:#x}, {rec_id}, {buf_ptr:#x}, {buf_len})");

    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    let Some(mut db) = open_db_for_handle(context, &handle).await else {
        return Ok(-25);
    };
    if rec_id < 0 {
        return Ok(-22);
    }
    let rec_id = rec_id as u32;
    if db.get(rec_id).await.is_none() {
        return Ok(-22);
    }

    let mut buf = vec![0; buf_len as usize];
    context.read_bytes(buf_ptr, &mut buf)?;

    if db.set(rec_id, &buf).await { Ok(0) } else { Ok(-22) }
}

pub async fn select_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({db_id:#x}, {rec_id}, {buf_ptr:#x}, {buf_len})");

    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    let Some(db) = open_db_for_handle(context, &handle).await else {
        return Ok(-25);
    };
    if rec_id < 0 {
        return Ok(-22);
    }

    if let Some(data) = db.get(rec_id as u32).await {
        if buf_len < data.len() as u32 {
            return Ok(-18); // M_E_SHORTBUF
        }
        context.write_bytes(buf_ptr, &data)?;
        Ok(0)
    } else {
        Ok(-22)
    }
}

pub async fn stream_read(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("db.stream_read({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    let Some(mut handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    if handle.read_cursor >= handle.buffer_len {
        // Don't touch buf — caller may have passed a sentinel (NULL) that
        // we shouldn't write to. Some titles do this past EOF.
        //
        // -23 is outside the `WIPICError` vocabulary and stays: there is no EOF variant, and
        // `NoSuchEntry`/`Invalid` would both be wrong (the handle is valid, the record exists, the
        // cursor is simply past the end). Widening the enum needs real-device evidence this repo
        // does not have. The line above is why changing the number is riskier here than at the
        // other outliers — this path is observed in real titles. docs/wipi-c-abi-error-codes.md §3.
        return Ok(-23); // M_E_EOF
    }

    let take = core::cmp::min(buf_len, handle.buffer_len - handle.read_cursor);
    if take == 0 {
        return Ok(0);
    }

    // Copy from the guest-heap buffer into the caller's destination via
    // host-side scratch; `WIPICContext` doesn't expose an in-guest memmove.
    let mut data = vec![0u8; take as usize];
    context.read_bytes(handle.buffer_ptr + handle.read_cursor, &mut data)?;
    context.write_bytes(buf_ptr, &data)?;

    handle.read_cursor += take;
    write_generic(context, db_id as _, handle)?;

    Ok(take as _)
}

/// KTF custom slot 4 — repurposed from standard `MC_dbSelectRecord` into a
/// stream-control op `(handle, offset, mode)` that seeks both read/write
/// cursors. The standard WIPI signature `(db_id, rec_id, buf_ptr, buf_len)`
/// is not implemented; LGT routes do not use this slot.
pub async fn select_record_ktf(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, mode: WIPICWord, _buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({db_id:#x}, {rec_id}, mode={mode:#x}, {_buf_len})");

    let Some(mut handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    // KTF reuses slot 4 as a stream-control op `(handle, offset, mode)`. The
    // shapes observed across games:
    //
    //   - `(handle, slot_offset, 0)` — multi-slot save files store each
    //     slot at a known byte offset within record 1; this seeks both
    //     cursors so the next read/write hits the right slot while
    //     preserving the bytes belonging to the other slots.
    //   - `(handle, 0, 0)` and `(handle, 0, 2)` — rewinds both cursors.
    //     mode=0 vs 2 isn't a length and isn't truncate (truncating on
    //     mode=2 on the read path destroys a prefetched buffer during a
    //     subsequent re-open and wipes the saved record). Both are treated
    //     as plain seek-and-rewind.
    if rec_id >= 0 {
        let offset = rec_id as u32;
        handle.read_cursor = offset;
        handle.write_cursor = offset;
        write_generic(context, db_id as _, handle)?;
        return Ok(0);
    }

    Ok(-22) // M_E_BADRECID
}

/// Slot 5 — KTF custom `db_stat_by_name`. From observed call shape:
///
/// ```text
/// int32 v2[3];
/// ret = slot5(name_ptr, &v2, mode, fn_self_ptr);
/// if (ret == 0 && v2[2] > 0xC7) "valid save";
/// ```
///
/// Takes a name plus a 12-byte (3-int) output struct, and returns 0 when
/// the DB exists with a non-trivial payload. The third int is treated as a
/// size threshold (must exceed 199 bytes). We fill the struct with
/// `{0, 0, record_size}` and return 0 on hit, -22 on miss.
pub async fn stat_by_name_ktf(context: &mut dyn WIPICContext, name_ptr: WIPICWord, out_buf: WIPICWord, mode: i32, _arg3: i32) -> Result<i32> {
    let name = match read_null_terminated_string_bytes(context, name_ptr) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return Ok(-22),
        },
        Err(_) => return Ok(-22),
    };

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(&name, &pid).await;
    if !exists {
        tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> -22 (not found)");
        return Ok(-22);
    }

    // Pull record 1's size as the "valid save" indicator the game checks
    // against 0xC7 in v2[2].
    let db = system.platform().database_repository().open(&name, &pid).await;
    let record_size = db.get(1).await.map(|x| x.len() as u32).unwrap_or(0);

    if out_buf != 0 {
        write_generic(context, out_buf, 0u32)?;
        write_generic(context, out_buf + 4, 0u32)?;
        write_generic(context, out_buf + 8, record_size)?;
    }

    tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> 0 (size={record_size})");
    Ok(0)
}

/// KTF custom slot 16 — `MC_dbExists(name)`. Observed call shape across
/// multiple titles is `(name_ptr, 1, size_hint_or_zero, callback_garbage)`.
/// Titles call it before deciding whether to take the load or fresh-init
/// path. Returning 1 unconditionally makes them try to load nonexistent
/// state on first run and trip later, so we read the C string at `a0` and
/// answer based on the real persisted state.
pub async fn exists_database_ktf(context: &mut dyn WIPICContext, name_ptr: WIPICWord, _arg1: i32, _arg2: i32) -> Result<i32> {
    let name = match read_null_terminated_string_bytes(context, name_ptr) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("MC_dbExists invalid utf8 name @ {name_ptr:#x}, defaulting to 0");
                return Ok(0);
            }
        },
        Err(_) => {
            tracing::warn!("MC_dbExists unreadable name @ {name_ptr:#x}, defaulting to 0");
            return Ok(0);
        }
    };

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(&name, &pid).await;

    let result = if exists { 1 } else { 0 };
    tracing::debug!("MC_dbExists({name:?}) -> {result}");
    Ok(result)
}

/// Read a `DatabaseHandle` from guest memory if `db_id` looks like one.
///
/// Returns `Ok(None)` for any pointer that's obviously not a handle —
/// out-of-range, missing the magic sentinel — so callers can return
/// `M_E_INVALIDHANDLE` instead of panicking on garbage input.
fn load_handle(context: &mut dyn WIPICContext, db_id: i32) -> Result<Option<DatabaseHandle>> {
    if db_id < 0x10000 {
        return Ok(None);
    }
    let handle: DatabaseHandle = read_generic(context, db_id as _)?;
    if handle.magic != DATABASE_HANDLE_MAGIC {
        return Ok(None);
    }
    Ok(Some(handle))
}

async fn open_db_for_handle(context: &mut dyn WIPICContext, handle: &DatabaseHandle) -> Option<Box<dyn Database>> {
    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let db_name = str::from_utf8(&handle.name[..name_length]).ok()?;

    let system = context.system();
    let pid = system.pid().to_owned();

    Some(system.platform().database_repository().open(db_name, &pid).await)
}

async fn get_database_from_db_id(context: &mut dyn WIPICContext, db_id: i32) -> Result<Option<Box<dyn Database>>> {
    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(None);
    };
    Ok(open_db_for_handle(context, &handle).await)
}

async fn read_packaged_database(context: &mut dyn WIPICContext, name: &str) -> Result<Option<Vec<u8>>> {
    if context.get_resource_size(name).await?.is_none() {
        return Ok(None);
    }

    Ok(Some(context.read_resource(name).await?))
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite};

    use crate::context::test::TestContext;

    use super::{
        KTF_DATABASE_STORAGE_LIMIT, SLOT8_TOKEN_MAX, delete_database, exists_database, list_databases, list_record_info, open_database,
        select_record, sort_records, stream_read, stream_write, update_record,
    };

    /// KTF database slot 8 refuses, and refuses **without touching guest memory**.
    ///
    /// The first version of `sort_records` wrote record ids into what it took to be
    /// a caller-supplied buffer. Measured against the two guests that reach the
    /// slot, that "buffer" argument is `0x1` — so the write loop would have started
    /// at guest address 1. This pins the property the rewrite bought: whatever the
    /// two arguments turn out to mean, nothing is written until somebody knows.
    ///
    /// The sentinel sits at `0x0` and at `0x1000`: the first covers the address
    /// the old loop would have started writing at, and the second is passed in as
    /// `arg0` by the second case below, so a regression that writes *through an
    /// argument* fails here rather than in a guest. (It no longer spans a
    /// heap-range address — dropping `r2` took the only heap-range value out of
    /// the inputs, and that value was never an argument in the first place.)
    ///
    /// It also pins the **error message**, because that string is this round's
    /// actual product: the coordinate the next round starts from. Dropping
    /// `r1={arg1:#x}` from it is otherwise a silent green.
    #[futures_test::test]
    async fn sort_records_never_writes_to_guest_memory_test() {
        let mut context = database_test_context();
        const SENTINEL: [u8; 16] = [0xAB; 16];
        for base in [0x0u32, 0x1000] {
            context.write_bytes(base, &SENTINEL).unwrap();
        }

        // The arguments a real guest passed (2026-09-18), plus a run with the
        // header's null-callback shape, which the withdrawn code treated as
        // "write everything".
        for args in [(0x13184cu32, 0x1u32), (0x1000, 0x0)] {
            let err = sort_records(&mut context, args.0, args.1).await.unwrap_err();
            assert!(
                matches!(err, wie_util::WieError::Unimplemented(ref m)
                    if m.contains("argument layout unknown")
                        && m.contains(&alloc::format!("measured r0={:#x}", args.0))
                        && m.contains(&alloc::format!("r1={:#x}", args.1))),
                "slot 8 must refuse rather than act on a guessed layout, and must report both \
                 measured arguments (that string is the next round's starting coordinate), got {err:?}"
            );
        }

        for base in [0x0u32, 0x1000] {
            let mut seen = [0u8; 16];
            context.read_bytes(base, &mut seen).unwrap();
            assert_eq!(seen, SENTINEL, "slot 8 wrote to guest memory at {base:#x}");
        }
    }

    /// Slot 8 quotes the token `r0` points at — and quotes **only** a short,
    /// printable, readable one.
    ///
    /// The point of the quote is that the next title to reach this slot names its
    /// own extension in the failure instead of costing somebody a disassembly:
    /// the two measured tokens, `"res"` and `"ga"`, took a synchronised Thumb
    /// sweep of two images to recover. The point of the *bounds* is that `r0` is
    /// a guest pointer and nothing here can prove it is not a path — so the
    /// three negative cases below are as load-bearing as the positive one, and a
    /// change that widens the filter fails here rather than in someone's log.
    ///
    /// Every case still asserts the raw `r0=` is present, because the quote is an
    /// addition: a regression that loses the token must not also lose the
    /// coordinate that was already there.
    #[futures_test::test]
    async fn sort_records_quotes_only_a_short_printable_token_test() {
        let mut context = database_test_context();

        // The two tokens real guests passed, measured 2026-09-19 by resolving the
        // sl-relative pointer table in `0103451A` / `01031C0A`.
        for (addr, token) in [(0x2000u32, &b"res\0"[..]), (0x2100, &b"ga\0"[..])] {
            context.write_bytes(addr, token).unwrap();
            let err = sort_records(&mut context, addr, 1).await.unwrap_err();
            let wie_util::WieError::Unimplemented(m) = err else {
                panic!("slot 8 must stay Unimplemented")
            };
            let want = alloc::format!("{:?}", str::from_utf8(&token[..token.len() - 1]).unwrap());
            assert!(
                m.contains(&want) && m.contains(&alloc::format!("measured r0={addr:#x}")),
                "slot 8 must quote the token AND keep the raw pointer, got {m}"
            );
        }

        // Not quoted: too long, non-printable, unreadable, null. The message keeps
        // the raw pointer in every one of them.
        let long = [b'a'; SLOT8_TOKEN_MAX + 1];
        context.write_bytes(0x2200, &long).unwrap();
        context.write_bytes(0x2300, b"ab\x01cd\0").unwrap();
        for (addr, why) in [
            (0x2200u32, "longer than SLOT8_TOKEN_MAX"),
            (0x2300, "contains a control byte"),
            (0xFFFF_0000, "unreadable"),
            (0x0, "null"),
        ] {
            let err = sort_records(&mut context, addr, 1).await.unwrap_err();
            let wie_util::WieError::Unimplemented(m) = err else {
                panic!("slot 8 must stay Unimplemented")
            };
            assert!(
                !m.contains(" (\"") && m.contains(&alloc::format!("measured r0={addr:#x}")),
                "slot 8 must not quote a token that is {why}, and must still report r0, got {m}"
            );
        }
    }

    #[futures_test::test]
    async fn ktf_available_database_storage_tracks_app_usage() {
        let mut context = database_test_context();
        assert_eq!(list_databases(&mut context).await.unwrap(), KTF_DATABASE_STORAGE_LIMIT as i32);

        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, &[1, 2, 3, 4]).unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 4).await.unwrap(), 4);

        assert_eq!(list_databases(&mut context).await.unwrap(), KTF_DATABASE_STORAGE_LIMIT as i32 - 4);
    }

    #[futures_test::test]
    async fn lgt_exists_database_reports_missing_and_existing_database() {
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"records\0").unwrap();

        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), -12);
        let db_id = open_database(&mut context, 0x1000, 0, 0).await.unwrap();
        context.write_bytes(0x2000, &[1]).unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 1).await.unwrap(), 1);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), 0);
    }

    #[futures_test::test]
    async fn lgt_create_mode_materializes_empty_database() {
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"records\0").unwrap();

        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), -12);
        let db_id = open_database(&mut context, 0x1000, 4, 0).await.unwrap();
        assert!(db_id > 0);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), 0);
    }

    #[futures_test::test]
    async fn lgt_update_and_select_record_use_standard_record_ids() {
        let mut context = database_test_context();
        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, &[1, 2, 3]).unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 3).await.unwrap(), 3);
        context.write_bytes(0x2010, &[4, 5]).unwrap();

        assert_eq!(update_record(&mut context, db_id, 1, 0x2010, 2).await.unwrap(), 0);
        assert_eq!(select_record(&mut context, db_id, 1, 0x2100, 2).await.unwrap(), 0);

        let mut data = [0; 2];
        context.read_bytes(0x2100, &mut data).unwrap();
        assert_eq!(data, [4, 5]);
    }

    #[futures_test::test]
    async fn lgt_list_record_info_and_delete_database_use_database_name() {
        let mut context = database_test_context();
        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, &[1, 2, 3, 4]).unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 4).await.unwrap(), 4);

        assert_eq!(list_record_info(&mut context, 0x1000, 0x2100, 1).await.unwrap(), 0);
        let mut entry = [0; 12];
        context.read_bytes(0x2100, &mut entry).unwrap();
        assert_eq!(u32::from_le_bytes(entry[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(entry[8..12].try_into().unwrap()), 4);

        assert_eq!(delete_database(&mut context, 0x1000, 1).await.unwrap(), 0);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), -12);
    }

    #[futures_test::test]
    async fn lgt_open_database_materializes_packaged_database() {
        let mut context = database_test_context().with_resource("kickass", b"seed-data");
        context.write_bytes(0x1000, b"kickass\0").unwrap();

        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), 0);
        let db_id = open_database(&mut context, 0x1000, 1, 0).await.unwrap();
        assert!(db_id > 0);
        assert_eq!(stream_read(&mut context, db_id, 0x2000, 9).await.unwrap(), 9);

        let mut data = [0; 9];
        context.read_bytes(0x2000, &mut data).unwrap();
        assert_eq!(&data, b"seed-data");
    }

    fn database_test_context() -> TestContext {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        TestContext::with_system(system)
    }

    async fn open_test_database(context: &mut TestContext) -> i32 {
        context.write_bytes(0x1000, b"records\0").unwrap();
        open_database(context, 0x1000, 0, 0).await.unwrap()
    }
}
