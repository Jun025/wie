use alloc::{borrow::ToOwned, boxed::Box, str, string::String, vec, vec::Vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::WIPICWord;

use wie_backend::Database;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, read_quotable_token, write_generic};

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
// The handset's free storage, not a quota anyone measured: callers only compare it from below. A KTF
// install-time check refuses 2,153,472 bytes and passes 2,201,600 (974e0df9ab1e · «2103KB 필요») — 1 MB
// ended that title at its first screen. 16 MB clears every known caller with room (docs/report/0449).
const KTF_DATABASE_STORAGE_LIMIT: u64 = 16 * 1024 * 1024;
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
    let name = resolve_db_name(context, name).await;

    // Validate before any repository side effects. Mode 4 deletes record 1
    // up front, so a too-long name reaching that path would wipe data we
    // can't open a handle for anyway.
    if name.len() > MAX_NAME_LEN {
        tracing::warn!("MC_dbOpenDataBase: name {name:?} too long ({} > {MAX_NAME_LEN})", name.len());
        return Ok(-22); // M_E_BADRECID — closest WIPI parameter-error idiom in this file
    }

    // A packaged resource backs the DB for every mode (mode 4 keeps it). A packaged
    // KTF `P/` file only seeds a DB that has nothing saved yet — mode 4 is a create
    // and starts empty, as it does with no package at all. Writes always go to the
    // repository; the shipped `P/` bytes never change, and once saved, the saved
    // record wins.
    let resource = read_packaged_resource(context, &name).await?;
    let resource_backed = resource.is_some();
    let packaged = match resource {
        Some(data) => Some(data),
        None if mode != 4 => context.system().filesystem().virtual_file(&name),
        None => None,
    };

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
        if mode == 4 && !resource_backed {
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

/// KTF WIPI-C **Interface4 slot 0** — the header's `MC_dbOpenDataBase(name, rsize, create, mode)`.
///
/// KTF's `Database` table slot 0 is a different, stream-style open (`open_database`). This table is
/// unnamed in this repo; slot 0's shape is measured on three titles — `("SaveData", 0xeec, 0, 1)`,
/// `("FG_102", 0x80, 0, 1)`, `(<name>, 0x80, 1, 1)` — a name, a record size, a create flag and a
/// mode, which is exactly the header's record-database open. It returns the same handle the other
/// database entry points already read, so any of them the title reaches next works on it.
pub async fn open_record_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, record_size: i32, create: i32, mode: i32) -> Result<i32> {
    tracing::debug!("KTF Interface4[0] MC_dbOpenDataBase({ptr_name:#x}, {record_size}, {create}, {mode})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        return Ok(-22);
    };
    let name = resolve_db_name(context, name).await;
    if name.len() > MAX_NAME_LEN {
        return Ok(-22);
    }

    let system = context.system();
    let pid = system.pid().to_owned();
    if create == 0 && !system.platform().database_repository().exists(&name, &pid).await {
        return Ok(-12); // M_E_NOENT
    }
    system.platform().database_repository().open(&name, &pid).await;

    let mut handle = DatabaseHandle::zeroed();
    handle.magic = DATABASE_HANDLE_MAGIC;
    handle.name[..name.len()].copy_from_slice(name.as_bytes());

    let ptr_handle = context.alloc_raw(size_of::<DatabaseHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;

    Ok(ptr_handle as _)
}

/// `MC_dbInsertRecord(fd, buf, len)` — a new record; returns its id.
pub async fn insert_record(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbInsertRecord({db_id:#x}, {buf_ptr:#x}, {buf_len})");

    let Some(mut db) = get_database_from_db_id(context, db_id).await? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    let mut buf = vec![0; buf_len as usize];
    context.read_bytes(buf_ptr, &mut buf)?;

    Ok(db.add(&buf).await as _)
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

/// KTF WIPI-C **Database slot 7** (header name `MC_dbListRecords`).
///
/// With a real handle in `r0` it is the header's call and goes to `list_record`. KTF titles also
/// call it as `f(src_name, dst_name, 1)` — read off the caller, not guessed: an installer writes the
/// jar's split chunks into `lo.z_`, closes it, then calls slot 7 with `r0 = "lo.z_"`, `r1 = "lo.dsk"`
/// and branches on `r0 >= 0` to the next install stage, `< 0` to «Data 인스톨에 실패»
/// (`docs/report/0401`). The next boot stats `lo.dsk`, so this is a rename. Until 2026-10-01 the name
/// fell into `list_record`, which answered M_E_INVALIDHANDLE, and the install never finished.
pub async fn list_record_or_rename_ktf(context: &mut dyn WIPICContext, a0: i32, a1: WIPICWord, a2: WIPICWord) -> Result<i32> {
    if load_handle(context, a0)?.is_some() {
        return list_record(context, a0, a1, a2).await;
    }

    let (Ok(src), Ok(dst)) = (
        String::from_utf8(read_null_terminated_string_bytes(context, a0 as _)?),
        String::from_utf8(read_null_terminated_string_bytes(context, a1)?),
    ) else {
        return Ok(-22);
    };
    let (src, dst) = (resolve_db_name(context, src).await, resolve_db_name(context, dst).await);
    if src == dst {
        // POSIX rename(x, x): nothing to do. The copy below would delete `dst` — the source — first.
        tracing::debug!("KTF db rename({src:?} -> {dst:?}, {a2}) -> 0 (same name)");
        return Ok(0);
    }
    let system = context.system();
    let pid = system.pid().to_owned();
    if !system.platform().database_repository().exists(&src, &pid).await {
        tracing::debug!("KTF db rename({src:?} -> {dst:?}, {a2}) -> -12 (no source)");
        return Ok(-12); // M_E_NOENT
    }

    // ponytail: copy + delete through the existing `Database` trait — the backends have no rename.
    // Holds the DB in host memory once; add `DatabaseRepository::rename` if that ever matters.
    let source = system.platform().database_repository().open(&src, &pid).await;
    system.platform().database_repository().delete(&dst, &pid).await;
    let mut target = system.platform().database_repository().open(&dst, &pid).await;
    for id in source.get_record_ids().await {
        if let Some(data) = source.get(id).await {
            target.set(id, &data).await;
        }
    }
    drop(source);
    system.platform().database_repository().delete(&src, &pid).await;

    tracing::debug!("KTF db rename({src:?} -> {dst:?}, {a2}) -> 0");
    Ok(0)
}

/// KTF WIPI-C **Database slot 8** (header name `MC_dbSortRecords`) — ensure a directory exists.
///
/// Disassembled arity is 2 — `f(r0 = path, r1 = 1)`. Report 0175 measured tokens `"res"`/`"ga"`,
/// each the directory part of a path the same call site then opens, and left four meanings open;
/// its H4 was «select/ensure a directory». `docs/report/0401` measured the deciding caller: a title's
/// libc shim routes one POSIX op to this slot as `(path, 1)` (the next op goes to slot 9 with the
/// same shape), and the title calls it with `"/shared"`, then `stat("/shared")` through slot 5,
/// and exits when that stat fails — `mkdir` then check. So this records the directory as an empty
/// database under that name, which is what slot 5 and slot 16 look up; a name that already exists
/// is left alone. It still answers 0: the three callers measured in 0175 discard the result.
/// Until 2026-10-01 it did nothing, and that title quit right after its data install.
pub async fn sort_records(context: &mut dyn WIPICContext, arg0: WIPICWord, arg1: WIPICWord) -> Result<i32> {
    let token = read_quotable_token(context, arg0);
    tracing::debug!("KTF database slot 8 (header name: MC_dbSortRecords)({arg0:#x}, {arg1:#x}) token={token:?} — ensure directory");

    let Some(name) = read_null_terminated_string_bytes(context, arg0)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
    else {
        return Ok(0);
    };
    let name = resolve_db_name(context, name).await;
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Ok(0);
    }
    let system = context.system();
    let pid = system.pid().to_owned();
    if !system.platform().database_repository().exists(&name, &pid).await {
        system.platform().database_repository().open(&name, &pid).await;
    }

    Ok(0)
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
/// KTF slot 11 (header name `MC_dbGetRecordSize`) is served by this too: its callers pass no
/// argument and refuse to save when the answer is below the save's length (or is not above 0x176f at boot).
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

/// LGT `MC_dbOpenDataBase`: mode 8 opens for writing and creates the database when it is missing.
///
/// 22 LGT titles open with mode 8, and 8 of them open with nothing but modes 1 and 8 — no other way to
/// make a save. Mode 8 is followed by a write at 60 of its 100 call sites. So on the handset an opened mode-8
/// database exists from that moment, empty. The generic path only creates it on the first write, and
/// 63332c51d514 asks `list_record_info` about it before writing: it opens its save with mode 8, takes
/// any of -1·-3·-9·-11·-12·-13·-24 as «no save», and otherwise reads the size from the entry without
/// looking at the return value — an empty file has to answer size 0 there (docs/report/0456).
pub async fn open_database_lgt(context: &mut dyn WIPICContext, ptr_name: WIPICWord, mode: i32, r#type: i32) -> Result<i32> {
    if mode == 8
        && let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?)
    {
        let name = resolve_db_name(context, name).await;
        let system = context.system();
        let pid = system.pid().to_owned();
        if name.len() <= MAX_NAME_LEN && !system.platform().database_repository().exists(&name, &pid).await {
            system.platform().database_repository().open(&name, &pid).await;
        }
    }

    open_database(context, ptr_name, mode, r#type).await
}

pub async fn list_record_info(context: &mut dyn WIPICContext, ptr_name: WIPICWord, buf_ptr: WIPICWord, capacity: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecordInfo({ptr_name:#x}, {buf_ptr:#x}, {capacity})");

    let Ok(name) = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?) else {
        return Ok(-22);
    };
    let name = resolve_db_name(context, name).await;
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

    // An existing database with no record is an empty file: one entry of size 0. Callers read the
    // entry whenever this returns 0 (85 of 120 LGT call sites branch on it before touching the entry), so
    // returning 0 with nothing written handed them stack garbage as the size.
    if ids.is_empty() {
        if capacity > 0 {
            write_generic(context, buf_ptr, 1u32)?;
            write_generic(context, buf_ptr + 4, 0u32)?;
            write_generic(context, buf_ptr + 8, 0u32)?;
        }
        return Ok(0);
    }

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
    let name = resolve_db_name(context, name).await;
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
    let name = resolve_db_name(context, name).await;
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

/// KTF custom slot 4 — `lseek(handle, offset, whence)`, not the header's `MC_dbSelectRecord`.
/// `whence` is 0 = from the start, 1 = from the current position, 2 = from the end, and the return
/// value is the new position. Read off a title's own libc shim, not guessed (`docs/report/0401`):
/// its POSIX `lseek` passes SEEK_CUR/SEEK_END through as 1/2 (anything else as 0) and returns this
/// slot's value unchanged, its `ftell` is `lseek(fd, 0, 1)`, and its `fseek(SEEK_CUR)` adds
/// `ftell()` to the offset and seeks with 0. The corpus uses the same shapes: `(h, 0, 2)` then
/// `(h, 0, 0)` is a size probe before reading (11 KTF titles), `(h, n, 1)` skips `n` bytes.
///
/// Until 2026-10-01 every whence was treated as «from the start» and the answer was always 0:
/// a size probe read every file as empty, and a title that seeks from `ftell()` landed at a
/// negative offset — the installer of `docs/report/0401` failed with «잘못된 리소스 파일».
/// Seeking past the end is allowed (multi-slot saves write at a fixed offset; `stream_write`
/// zero-fills the gap); a negative result is refused with -22 and moves nothing.
pub async fn select_record_ktf(context: &mut dyn WIPICContext, db_id: i32, offset: i32, whence: WIPICWord, _arg3: WIPICWord) -> Result<i32> {
    tracing::debug!("db.lseek({db_id:#x}, {offset}, whence={whence:#x})");

    let Some(mut handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };

    // The two cursors move together on a seek and each only advances on its own op, so the one
    // that moved since is the position.
    let base = match whence {
        0 => 0,
        1 => handle.read_cursor.max(handle.write_cursor) as i64,
        2 => handle.buffer_len as i64,
        _ => return Ok(-22),
    };
    let Ok(position) = u32::try_from(base + offset as i64) else {
        return Ok(-22); // M_E_BADRECID
    };
    if position > i32::MAX as u32 {
        return Ok(-22);
    }
    handle.read_cursor = position;
    handle.write_cursor = position;
    write_generic(context, db_id as _, handle)?;

    Ok(position as i32)
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
    let name = resolve_db_name(context, name).await;

    // Pull record 1's size as the "valid save" indicator the game checks
    // against 0xC7 in v2[2]. A saved DB wins; otherwise a packaged `P/` file
    // answers with its shipped size (see `read_packaged_database`).
    let system = context.system();
    let pid = system.pid().to_owned();
    let record_size = if system.platform().database_repository().exists(&name, &pid).await {
        let db = system.platform().database_repository().open(&name, &pid).await;
        db.get(1).await.map(|x| x.len() as u32).unwrap_or(0)
    } else if let Some(data) = read_packaged_database(context, &name).await? {
        data.len() as u32
    } else {
        tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> -22 (not found)");
        return Ok(-22);
    };

    if out_buf != 0 {
        write_generic(context, out_buf, 0u32)?;
        write_generic(context, out_buf + 4, 0u32)?;
        write_generic(context, out_buf + 8, record_size)?;
    }

    tracing::debug!("db.stat_by_name({name:?}, mode={mode}) -> 0 (size={record_size})");
    Ok(0)
}

/// KTF custom slot 15 — the open DB's size in bytes. Read off the one caller in the corpus
/// (`docs/report/0401`): `size = lseek(h, 0, 2); if (size > 0) size = slot15(h); lseek(h, 0, 0)`,
/// which became reachable once slot 4 stopped answering every size probe with 0.
pub async fn file_size_ktf(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    let Some(handle) = load_handle(context, db_id)? else {
        return Ok(-25); // M_E_INVALIDHANDLE
    };
    tracing::debug!("db.size({db_id:#x}) -> {}", handle.buffer_len);
    Ok(handle.buffer_len as i32)
}

/// KTF custom slot 16 — `MC_dbExists(name, 1, …)`. **0 = the DB exists, -12
/// (`M_E_NOENT`) = it does not** — the same 0-is-success convention as slot 5
/// (`stat_by_name_ktf`) and LGT's `exists_database`.
///
/// The polarity is read off the guests, not guessed. Over the 266 KTF archives in
/// the local corpus, 19 titles reach this slot at boot; 18 test the result with
/// `cmp r0, #0` right after the call (the 19th branches first and was not
/// decoded), and 15 of those fold it straight into `ret == 0`. 155972cac664's own wrapper
/// (`0x148960`) is `return MC_dbExists(name, 1) == 0`, and a `1` from it sends
/// the title down the load path (`stat` then open `Config.dat` in read mode) —
/// so 0 is «exists». This slot used to answer 1/0, i.e. inverted: «exists» for
/// every missing DB and «missing» for every saved one.
///
/// A packaged `P/` file counts as existing (see `read_packaged_database`) — that
/// title ships `FirstRun.dat`/`Certification.dat`/`Config.dat` there and reads
/// them on first launch.
pub async fn exists_database_ktf(context: &mut dyn WIPICContext, name_ptr: WIPICWord, _arg1: i32, _arg2: i32) -> Result<i32> {
    let name = match read_null_terminated_string_bytes(context, name_ptr) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("MC_dbExists invalid utf8 name @ {name_ptr:#x}, answering not-found");
                return Ok(-12);
            }
        },
        Err(_) => {
            tracing::warn!("MC_dbExists unreadable name @ {name_ptr:#x}, answering not-found");
            return Ok(-12);
        }
    };
    let name = resolve_db_name(context, name).await;

    let system = context.system();
    let pid = system.pid().to_owned();
    let exists = system.platform().database_repository().exists(&name, &pid).await || read_packaged_database(context, &name).await?.is_some();

    let result = if exists { 0 } else { -12 }; // M_E_NOENT
    tracing::debug!("MC_dbExists({name:?}) -> {result}");
    Ok(result)
}

/// The repository key for a guest DB name: KTF titles name the same DB with and without a leading
/// `/` — one installer stats and renames to `lo.dsk`, then opens `/lo.dsk` (`docs/report/0401`).
/// The name as given wins when it exists, so a save already stored under a `/` key keeps working;
/// otherwise a saved DB under the bare name is used.
async fn resolve_db_name(context: &mut dyn WIPICContext, name: String) -> String {
    let bare = name.trim_start_matches('/');
    if bare.len() == name.len() || bare.is_empty() {
        return name;
    }
    let system = context.system();
    let pid = system.pid().to_owned();
    if !system.platform().database_repository().exists(&name, &pid).await && system.platform().database_repository().exists(bare, &pid).await {
        return bare.to_owned();
    }
    name
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

/// The bytes a database named `name` starts from when nothing has been saved yet.
///
/// Two sources, in order: a context resource (LGT's jar), then an archive file the
/// emulator loaded into the filesystem's virtual layer — KTF zips ship their initial
/// save data under `P/` (e.g. `P/Config.dat`) and `wie_ktf::emulator::load` puts it
/// there with the `P/` stripped, where KTF's resource lookup (jar only) cannot see it.
/// Only the *packaged* bytes count: a file the guest wrote through the file API is
/// not a database.
async fn read_packaged_database(context: &mut dyn WIPICContext, name: &str) -> Result<Option<Vec<u8>>> {
    if let Some(data) = read_packaged_resource(context, name).await? {
        return Ok(Some(data));
    }

    Ok(context.system().filesystem().virtual_file(name))
}

async fn read_packaged_resource(context: &mut dyn WIPICContext, name: &str) -> Result<Option<Vec<u8>>> {
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

    use crate::context::{WIPICContext, test::TestContext};

    use super::{
        KTF_DATABASE_STORAGE_LIMIT, close_database, delete_database, exists_database, exists_database_ktf, file_size_ktf, get_number_of_records,
        insert_record, list_databases, list_record, list_record_info, list_record_or_rename_ktf, open_database, open_database_lgt,
        open_record_database, select_record, select_record_ktf, sort_records, stat_by_name_ktf, stream_read, stream_write, update_record,
    };

    /// KTF Interface4 is the header's record database: the call sequence three titles make —
    /// open without create (absent: M_E_NOENT), open with create, insert, count, list, select.
    #[futures_test::test]
    async fn record_database_round_trip_test() {
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"SaveData\0").unwrap();
        context.write_bytes(0x1100, b"abcd").unwrap();

        assert_eq!(open_record_database(&mut context, 0x1000, 0xeec, 0, 1).await.unwrap(), -12);
        let fd = open_record_database(&mut context, 0x1000, 0xeec, 1, 1).await.unwrap();
        assert!(fd > 0);

        let id = insert_record(&mut context, fd, 0x1100, 4).await.unwrap();
        assert!(id > 0);
        assert_eq!(get_number_of_records(&mut context, fd).await.unwrap(), 1);
        assert_eq!(list_record(&mut context, fd, 0x1200, 12).await.unwrap(), 1);
        let mut word = [0; 4];
        context.read_bytes(0x1200, &mut word).unwrap();
        assert_eq!(word, (id as u32).to_le_bytes());
        assert_eq!(select_record(&mut context, fd, id, 0x1300, 128).await.unwrap(), 0);
        context.read_bytes(0x1300, &mut word).unwrap();
        assert_eq!(&word, b"abcd");

        // A later open without create finds it.
        assert!(open_record_database(&mut context, 0x1000, 0xeec, 0, 1).await.unwrap() > 0);
    }

    /// KTF database slot 8 is a no-op that returns 0 and **never touches guest memory**.
    ///
    /// The first version of `sort_records` wrote record ids into what it took to be
    /// a caller-supplied buffer; measured, that "buffer" argument is `0x1`, so the
    /// loop would have started at guest address 1. The sentinels sit at `0x0` and at
    /// `0x1000` (passed as `arg0` below), so a regression that writes *through an
    /// argument* fails here rather than in a guest.
    #[futures_test::test]
    async fn slot8_ensures_a_directory_and_never_writes_guest_memory_test() {
        let mut context = database_test_context();
        const SENTINEL: [u8; 16] = [0xAB; 16];
        for base in [0x0u32, 0x1000] {
            context.write_bytes(base, &SENTINEL).unwrap();
        }
        context.write_bytes(0x2000, b"res\0").unwrap();

        // The arguments real guests pass (a token and 1), the header's null-callback
        // shape, and an unreadable pointer.
        for args in [(0x2000u32, 0x1u32), (0x1000, 0x0), (0xFFFF_0000, 0x1)] {
            assert_eq!(sort_records(&mut context, args.0, args.1).await.unwrap(), 0);
        }

        for base in [0x0u32, 0x1000] {
            let mut seen = [0u8; 16];
            context.read_bytes(base, &mut seen).unwrap();
            assert_eq!(seen, SENTINEL, "slot 8 wrote to guest memory at {base:#x}");
        }

        // `mkdir` then `stat`: the path now answers as present (docs/report/0401).
        assert_eq!(stat_by_name_ktf(&mut context, 0x2000, 0, 1, 0).await.unwrap(), 0);
    }

    // A title's install check wants more than 2103 KB free; below that it quits at its first screen.
    #[futures_test::test]
    async fn ktf_available_database_storage_covers_a_2103kb_install() {
        let mut context = database_test_context();
        assert!(list_databases(&mut context).await.unwrap() >= 2150 * 1024);
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
    async fn lgt_write_mode_8_creates_the_database_and_its_info_is_one_empty_entry() {
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"SAV0\0").unwrap();
        // what the caller's stack slot held before the call — the size it read when nothing was written
        context.write_bytes(0x2100, &[0xef; 12]).unwrap();

        assert_eq!(list_record_info(&mut context, 0x1000, 0x2100, 1).await.unwrap(), -12);
        assert!(open_database_lgt(&mut context, 0x1000, 8, 1).await.unwrap() > 0);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), 0);
        assert_eq!(list_record_info(&mut context, 0x1000, 0x2100, 1).await.unwrap(), 0);
        let mut entry = [0; 12];
        context.read_bytes(0x2100, &mut entry).unwrap();
        assert_eq!(entry, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[futures_test::test]
    async fn generic_mode_8_still_creates_lazily() {
        // KTF shares `open_database`; the mode-8 evidence is LGT's, so KTF keeps create-on-write
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"SAV0\0").unwrap();

        assert!(open_database(&mut context, 0x1000, 8, 1).await.unwrap() > 0);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), -12);
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

    /// KTF slot 16 answers 0 for «exists» and -12 for «missing» — the polarity the
    /// guests test (`cmp r0, #0` → `ret == 0`). A saved DB and a packaged `P/` file
    /// both count; a name found nowhere does not.
    #[futures_test::test]
    async fn ktf_exists_is_zero_for_saved_or_packaged_and_noent_otherwise() {
        let mut context = database_test_context();
        context.system().filesystem().add_virtual("Config.dat", b"cfg-00".to_vec());
        context.write_bytes(0x1000, b"Config.dat\0").unwrap();
        context.write_bytes(0x1100, b"Save0.dat\0").unwrap();

        assert_eq!(exists_database_ktf(&mut context, 0x1000, 1, 0).await.unwrap(), 0);
        assert_eq!(exists_database_ktf(&mut context, 0x1100, 1, 0).await.unwrap(), -12);

        let db_id = open_database(&mut context, 0x1100, 4, 1).await.unwrap();
        assert!(db_id > 0);
        assert_eq!(exists_database_ktf(&mut context, 0x1100, 1, 0).await.unwrap(), 0);
    }

    /// A packaged `P/` file is a DB the title can stat and read before it ever saves:
    /// stat reports the shipped size, a read-mode open (mode 1) is seeded with the
    /// shipped bytes instead of failing with M_E_NOENT.
    #[futures_test::test]
    async fn ktf_packaged_p_file_is_stat_and_read_as_a_seeded_database() {
        let mut context = database_test_context();
        context.system().filesystem().add_virtual("Config.dat", b"cfg-00".to_vec());
        context.write_bytes(0x1000, b"Config.dat\0").unwrap();

        assert_eq!(stat_by_name_ktf(&mut context, 0x1000, 0x1800, 1, 0).await.unwrap(), 0);
        let mut size = [0; 4];
        context.read_bytes(0x1808, &mut size).unwrap();
        assert_eq!(u32::from_le_bytes(size), 6);

        let db_id = open_database(&mut context, 0x1000, 1, 1).await.unwrap();
        assert!(db_id > 0);
        assert_eq!(stream_read(&mut context, db_id, 0x2000, 6).await.unwrap(), 6);
        let mut data = [0; 6];
        context.read_bytes(0x2000, &mut data).unwrap();
        assert_eq!(&data, b"cfg-00");
    }

    /// Mode 4 is a create: a name backed only by a `P/` file starts empty, as it
    /// would with no package, and the shipped bytes are left untouched — so a title
    /// that re-saves over a packaged name still truncates.
    #[futures_test::test]
    async fn ktf_create_mode_over_packaged_p_file_starts_empty() {
        let mut context = database_test_context();
        context.system().filesystem().add_virtual("FirstRun.dat", b"\x01\0\0\0".to_vec());
        context.write_bytes(0x1000, b"FirstRun.dat\0").unwrap();

        let db_id = open_database(&mut context, 0x1000, 4, 1).await.unwrap();
        assert!(db_id > 0);
        assert_eq!(stream_read(&mut context, db_id, 0x2000, 4).await.unwrap(), -23); // M_E_EOF — nothing seeded
        assert_eq!(
            context.system().filesystem().virtual_file("FirstRun.dat").as_deref(),
            Some(&b"\x01\0\0\0"[..])
        );
    }

    /// KTF slot 7 with a handle is the header's list; with two names it renames — the
    /// installer's `lo.z_` → `lo.dsk` step (`docs/report/0401`).
    #[futures_test::test]
    async fn ktf_slot7_lists_for_a_handle_and_renames_for_two_names() {
        let mut context = database_test_context();
        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, b"chunk").unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 5).await.unwrap(), 5);
        assert_eq!(list_record_or_rename_ktf(&mut context, db_id, 0x2100, 1).await.unwrap(), 1);
        assert_eq!(close_database(&mut context, db_id).await.unwrap(), 0);

        context.write_bytes(0x3000, b"lo.dsk\0").unwrap();
        assert_eq!(list_record_or_rename_ktf(&mut context, 0x1000, 0x3000, 1).await.unwrap(), 0);
        assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), -12);
        assert_eq!(stat_by_name_ktf(&mut context, 0x3000, 0x1800, 1, 0).await.unwrap(), 0);
        let mut size = [0; 4];
        context.read_bytes(0x1808, &mut size).unwrap();
        assert_eq!(u32::from_le_bytes(size), 5);

        // The source is gone now, so a second rename has nothing to move.
        assert_eq!(list_record_or_rename_ktf(&mut context, 0x1000, 0x3000, 1).await.unwrap(), -12);
    }

    /// Renaming a DB onto itself keeps it — also when `/x` resolves to an existing bare `x`.
    #[futures_test::test]
    async fn ktf_slot7_rename_onto_itself_keeps_the_data() {
        let mut context = database_test_context();
        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, b"save").unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 4).await.unwrap(), 4);
        assert_eq!(close_database(&mut context, db_id).await.unwrap(), 0);
        context.write_bytes(0x3000, b"/records\0").unwrap();

        for src in [0x3000, 0x1000] {
            assert_eq!(list_record_or_rename_ktf(&mut context, src, 0x1000, 1).await.unwrap(), 0);
            assert_eq!(exists_database(&mut context, 0x1000, 1).await.unwrap(), 0);
            let db_id = open_database(&mut context, 0x1000, 0, 0).await.unwrap();
            assert_eq!(stream_read(&mut context, db_id, 0x2100, 4).await.unwrap(), 4);
            let mut data = [0; 4];
            context.read_bytes(0x2100, &mut data).unwrap();
            assert_eq!(&data, b"save");
            assert_eq!(close_database(&mut context, db_id).await.unwrap(), 0);
        }
    }

    /// KTF slot 4 is `lseek`: whence 0/1/2 = start/current/end, the new position comes back.
    #[futures_test::test]
    async fn ktf_slot4_is_lseek_and_returns_the_position() {
        let mut context = database_test_context();
        let db_id = open_test_database(&mut context).await;
        context.write_bytes(0x2000, b"0123456789").unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 10).await.unwrap(), 10);

        assert_eq!(select_record_ktf(&mut context, db_id, 0, 2, 0).await.unwrap(), 10); // size probe
        assert_eq!(file_size_ktf(&mut context, db_id).await.unwrap(), 10); // slot 15
        assert_eq!(select_record_ktf(&mut context, db_id, 0, 0, 0).await.unwrap(), 0);
        assert_eq!(stream_read(&mut context, db_id, 0x2100, 4).await.unwrap(), 4);
        assert_eq!(select_record_ktf(&mut context, db_id, 0, 1, 0).await.unwrap(), 4); // ftell
        assert_eq!(select_record_ktf(&mut context, db_id, 2, 1, 0).await.unwrap(), 6);
        assert_eq!(select_record_ktf(&mut context, db_id, -3, 2, 0).await.unwrap(), 7);
        assert_eq!(stream_read(&mut context, db_id, 0x2100, 3).await.unwrap(), 3);
        let mut tail = [0; 3];
        context.read_bytes(0x2100, &mut tail).unwrap();
        assert_eq!(&tail, b"789");

        // A negative result is refused and leaves the position where it was.
        assert_eq!(select_record_ktf(&mut context, db_id, -982, 0, 0).await.unwrap(), -22);
        assert_eq!(select_record_ktf(&mut context, db_id, 0, 1, 0).await.unwrap(), 10);
        // Past the end is a position; writing there zero-fills the gap.
        assert_eq!(select_record_ktf(&mut context, db_id, 12, 0, 0).await.unwrap(), 12);
    }

    /// `/name` reaches a DB saved as `name`, but a DB already saved under `/name` keeps winning.
    #[futures_test::test]
    async fn leading_slash_name_reaches_the_bare_db_without_shadowing_a_slashed_one() {
        let mut context = database_test_context();
        context.write_bytes(0x1000, b"lo.dsk\0").unwrap();
        context.write_bytes(0x1100, b"/lo.dsk\0").unwrap();
        assert_eq!(open_database(&mut context, 0x1100, 1, 1).await.unwrap(), -12);

        let db_id = open_database(&mut context, 0x1000, 2, 1).await.unwrap();
        context.write_bytes(0x2000, b"bare").unwrap();
        assert_eq!(stream_write(&mut context, db_id, 0x2000, 4).await.unwrap(), 4);
        let db_id = open_database(&mut context, 0x1100, 1, 1).await.unwrap();
        assert_eq!(select_record_ktf(&mut context, db_id, 0, 2, 0).await.unwrap(), 4);

        let pid = alloc::string::String::from(context.system().pid());
        let mut slashed = context.system().platform().database_repository().open("/lo.dsk", &pid).await;
        slashed.set(1, b"slashed!").await;
        let db_id = open_database(&mut context, 0x1100, 1, 1).await.unwrap();
        assert_eq!(select_record_ktf(&mut context, db_id, 0, 2, 0).await.unwrap(), 8);
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
