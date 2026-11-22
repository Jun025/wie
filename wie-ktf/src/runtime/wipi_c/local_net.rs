//! KTF's socket calls, wired to the title's dead server — never a real host.
//!
//! The servers and their replies are `wie_backend::fake_server`'s, the one table for every carrier;
//! this file is only the shape KTF calls them in. A title with no row there sees the network fail
//! exactly as before: `MC_netConnect` and `MC_netSocket` through `wie_wipi_c::api::net`, and the
//! slots below as the fatal stubs they have always been.
//!
//! What keeps these apart from `api::net`'s `MC_netSocket*` (LGT's shape) is what f44271803135 — the
//! one KTF title listed, docs/report/0541 — was measured against:
//! - connect, write and read are extension slots 30–32, with their own argument lists;
//! - the connect's callback comes a tick later, not at once;
//! - `MC_netSetReadCB` is taken and never fires;
//! - the title has one socket, and it is always [`LOCAL_FD`];
//! - closing it answers `M_E_ERROR` and leaves it be, as the stub did.

use alloc::{boxed::Box, vec, vec::Vec};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError};
use wie_wipi_c::{MethodBody, WIPICContext, WIPICResult, api::net};

/// The descriptor `MC_netSocket` hands a listed title. Any positive value works; this one is
/// recognisable in a trace.
const LOCAL_FD: i32 = 0x4c4e;

/// WIPI-C `M_E_WOULDBLOCK`: nothing to read yet.
const WOULD_BLOCK: i32 = -19;

/// Net slot 2. A listed title gets its one socket, fresh on every call; anyone else's fails as before.
pub async fn socket(context: &mut dyn WIPICContext, domain: i32, r#type: i32) -> Result<i32> {
    if !context.system().fake_net().socket_at(LOCAL_FD) {
        return net::socket(context, domain, r#type).await;
    }

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
    let ours = context.system().fake_net().connect(fd, addr.to_le_bytes(), port);
    tracing::warn!("Net slot 30({addr:#x}, {port}) -> {}", if ours { "local server" } else { "refused" });

    let result = if ours { 0 } else { u32::MAX }; // M_E_SUCCESS / M_E_ERROR
    context.spawn(Box::new(Callback {
        cb,
        args: vec![fd as _, result, param],
    }))?;

    Ok(0)
}

/// Net slot 31 `(fd, buf, len)` — write. The whole buffer is taken; a request the server knows
/// queues its reply.
pub async fn socket_send(context: &mut dyn WIPICContext, fd: i32, buf: WIPICWord, len: i32) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("Net slot 31 on fd {fd}")));
    }

    let mut request = vec![0; len.max(0) as usize];
    context.read_bytes(buf, &mut request)?;
    let replied = context.system().fake_net().write(fd, &request) == Some(true);
    tracing::warn!("Net slot 31({len} bytes) -> {}", if replied { "reply queued" } else { "no reply" });

    Ok(len)
}

/// Net slot 32 `(fd, buf, len)` — read what the server queued, or `M_E_WOULDBLOCK`.
pub async fn socket_recv(context: &mut dyn WIPICContext, fd: i32, buf: WIPICWord, len: i32) -> Result<i32> {
    if fd != LOCAL_FD {
        return Err(WieError::Unimplemented(alloc::format!("Net slot 32 on fd {fd}")));
    }

    let bytes = context.system().fake_net().read(fd, len.max(0) as usize).unwrap_or_default();
    if bytes.is_empty() {
        return Ok(WOULD_BLOCK);
    }
    context.write_bytes(buf, &bytes)?;

    Ok(bytes.len() as _)
}

/// Net slots 6 and 33 `(fd)` — close. The answer stays the stub's `M_E_ERROR`, which is what the title
/// got while it was measured; its socket lasts until the next `MC_netSocket` replaces it.
pub async fn socket_close(context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    if fd != LOCAL_FD {
        return net::socket_close(context, fd).await;
    }
    tracing::warn!("stub MC_netSocketClose({fd})");

    Ok(-1)
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
