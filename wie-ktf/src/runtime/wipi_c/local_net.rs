//! A title's dead server, answered inside the emulator — never a real host.
//!
//! Only titles listed in [`SERVERS`] see any of this; for every other title `MC_netConnect` and
//! `MC_netSocket` fail exactly as before (`wie_wipi_c::api::net`). A listed title gets a working
//! connection whose only peer is its row's `reply`.
//!
//! The one row today is f44271803135 (AID `0103A2B4`), measured in docs/report/ for
//! wie-ktf-dragoneyes2-connecting-wall-offline-path. Its boot reads a saved `option` record and, when
//! the «download complete» byte there is 0, asks its download server which of its eight `P/` files
//! to fetch. The archive ships all eight at the sizes the image itself hard-codes, so the server's
//! answer could only ever be «nothing»: a file list naming no file. The client then finds every file
//! complete, sets the byte, saves `option` and starts the game. Without a reply it re-dials forever
//! on «CONNECTING» — the client has no timeout in that state.
//!
//! ponytail: one pending-reply buffer for the whole process — one listed title, one socket. Per-System
//! state when two listed sockets can be open at once (the LGT server-wall round is the merge point).

use alloc::{boxed::Box, vec, vec::Vec};

use spin::Mutex;

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError, read_generic, write_generic};
use wie_wipi_c::{MethodBody, WIPICContext, WIPICResult, api::net};

/// The descriptor `MC_netSocket` hands a listed title. Any positive value works; this one is
/// recognisable in a trace.
const LOCAL_FD: i32 = 0x4c4e;

/// WIPI-C `M_E_WOULDBLOCK`: nothing to read yet.
const WOULD_BLOCK: i32 = -19;

struct LocalServer {
    aid: &'static str,
    addr: [u8; 4],
    port: u16,
    /// One whole request packet as the client wrote it → the bytes the server sends back.
    reply: fn(&[u8]) -> Option<Vec<u8>>,
}

const SERVERS: &[LocalServer] = &[LocalServer {
    aid: "0103A2B4",
    addr: [211, 110, 18, 253],
    port: 8503,
    reply: download_list_nothing,
}];

/// f44271803135's download protocol: `[len u16 LE][id u16 LE][body]`, `len` counting from `id`.
/// Request `0x2af8` asks for the file list; reply `0x2af9` is `[count u8]` then per file
/// `[name len u8][name][size u32 LE]` (client parser at image 0x1025b6). Count 0 leaves the
/// sizes the image hard-codes, which the shipped files already match.
fn download_list_nothing(request: &[u8]) -> Option<Vec<u8>> {
    const LIST_REQUEST: u16 = 0x2af8;
    const LIST_REPLY: u16 = 0x2af9;

    let id = u16::from_le_bytes(request.get(2..4)?.try_into().ok()?);
    (id == LIST_REQUEST).then(|| {
        let [lo, hi] = LIST_REPLY.to_le_bytes();
        vec![3, 0, lo, hi, 0]
    })
}

static PENDING: Mutex<Vec<u8>> = Mutex::new(Vec::new());

fn server_for(context: &mut dyn WIPICContext) -> Option<&'static LocalServer> {
    let aid = context.system().aid();
    SERVERS.iter().find(|server| server.aid == aid)
}

/// Net slot 0. A listed title's network comes up; anyone else's fails as before.
pub async fn connect(context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    if server_for(context).is_none() {
        return net::connect(context, cb, param).await;
    }

    tracing::warn!("MC_netConnect({cb:#x}, {param:#x}) -> local server");
    context.spawn(Box::new(Callback { cb, args: vec![0, param] }))?; // M_E_SUCCESS

    Ok(0)
}

/// Net slot 2.
pub async fn socket(context: &mut dyn WIPICContext, domain: i32, r#type: i32) -> Result<i32> {
    if server_for(context).is_none() {
        return net::socket(context, domain, r#type).await;
    }

    PENDING.lock().clear();
    Ok(LOCAL_FD)
}

/// Net slot 30 `(fd, addr, port, _, cb, param)` — connect, answered through `cb(fd, result, param)`
/// (f44271803135 image 0x104e9c; callback 0x104c70 reads the result from its second argument).
/// Succeeds only for the listed title's own host and port.
pub async fn socket_open(
    context: &mut dyn WIPICContext,
    fd: i32,
    addr: u32,
    port: u32,
    _unknown: u32,
    cb: WIPICWord,
    param: WIPICWord,
) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("Net slot 30 on fd {fd}")));
    }

    let port = (port as u16).swap_bytes(); // MC_utilHtons'd
    let ours = server_for(context).is_some_and(|server| server.addr == addr.to_le_bytes() && server.port == port);
    tracing::warn!("Net slot 30({addr:#x}, {port}) -> {}", if ours { "local server" } else { "refused" });

    let result = if ours { 0 } else { u32::MAX }; // M_E_SUCCESS / M_E_ERROR
    context.spawn(Box::new(Callback { cb, args: vec![fd as _, result, param] }))?;

    Ok(0)
}

/// Net slot 31 `(fd, buf, len)` — write. The whole buffer is taken; a request the server knows
/// queues its reply.
pub async fn socket_send(context: &mut dyn WIPICContext, fd: i32, buf: WIPICWord, len: i32) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("Net slot 31 on fd {fd}")));
    }

    let request = (0..len.max(0) as u32).map(|i| read_generic(context, buf + i)).collect::<Result<Vec<u8>>>()?;
    let reply = server_for(context).and_then(|server| (server.reply)(&request));
    tracing::warn!("Net slot 31({len} bytes) -> reply {:?}", reply.as_ref().map(Vec::len));
    PENDING.lock().extend(reply.unwrap_or_default());

    Ok(len)
}

/// Net slot 32 `(fd, buf, len)` — read what the server queued, or `M_E_WOULDBLOCK`.
pub async fn socket_recv(context: &mut dyn WIPICContext, fd: i32, buf: WIPICWord, len: i32) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("Net slot 32 on fd {fd}")));
    }

    let bytes = {
        let mut pending = PENDING.lock();
        let n = pending.len().min(len.max(0) as usize);
        pending.drain(..n).collect::<Vec<u8>>()
    };
    if bytes.is_empty() {
        return Ok(WOULD_BLOCK);
    }
    for (i, byte) in bytes.iter().enumerate() {
        write_generic(context, buf + i as u32, *byte)?;
    }

    Ok(bytes.len() as _)
}

/// Net slot 13 `MC_netSetReadCB(fd, cb, param)` — the client registers it after a `WOULDBLOCK`.
/// Every reply is queued by the write that asked for it, so there is never late data to announce.
pub async fn set_read_cb(_context: &mut dyn WIPICContext, fd: i32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("MC_netSetReadCB on fd {fd}")));
    }
    tracing::warn!("MC_netSetReadCB({fd}, {cb:#x}, {param:#x}) -> no late data");

    Ok(0)
}

struct Callback {
    cb: WIPICWord,
    args: Vec<WIPICWord>,
}

#[async_trait::async_trait]
impl MethodBody<WieError> for Callback {
    async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
        context.system().sleep(1).await;
        context.call_function(self.cb, &self.args).await?;

        Ok(WIPICResult { results: Vec::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::download_list_nothing;

    #[test]
    fn the_list_request_is_answered_with_an_empty_list() {
        assert_eq!(download_list_nothing(&[2, 0, 0xf8, 0x2a]), Some(alloc::vec![3, 0, 0xf9, 0x2a, 0]));
        assert_eq!(download_list_nothing(&[2, 0, 0xfb, 0x2a]), None);
        assert_eq!(download_list_nothing(&[2]), None);
    }
}
