use alloc::{boxed::Box, vec, vec::Vec};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError};

use crate::{WIPICResult, context::WIPICContext, method::MethodBody};

// `MC_netSocketRead`/`Write`'s «try again when the callback fires» — caf9d76ffd13's socket class
// (binary.mod 0x197fa, 0x19906) compares against exactly this before it calls `MC_netSetReadCB`.
const M_E_WOULDBLOCK: i32 = -19;

/// `cb(args…)` on the next turn of the executor, as the handset delivers network callbacks;
/// `MC_netConnect`'s after a 1ms sleep, as it always has.
struct Callback {
    cb: WIPICWord,
    args: Vec<WIPICWord>,
    delay: bool,
}

#[async_trait::async_trait]
impl MethodBody<WieError> for Callback {
    async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
        if self.delay {
            context.system().sleep(1).await; // simulate some delay
        }

        context.call_function(self.cb, &self.args).await?;

        Ok(WIPICResult { results: Vec::new() })
    }
}

// Fails through `cb(M_E_ERROR, param)` — unless the title has a server listed in
// `wie_backend::fake_server`, which stands in for one that no longer exists.
pub async fn connect(context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    let online = context.system().fake_net().online();
    tracing::warn!(
        "MC_netConnect({cb:#x}, {param:#x}) -> callback {}",
        if online { "M_E_SUCCESS (fake server)" } else { "M_E_ERROR" }
    );

    let result = if online { 0 } else { u32::MAX };
    context.spawn(Box::new(Callback {
        cb,
        args: vec![result, param],
        delay: true,
    }))?;

    Ok(0)
}

// No network, so the connect fails — asynchronously, as `connect` above does: the result goes to
// `cb(fd, result, param)`. fe76e641bb3d (LGT 603, binary.mod 0x24428) ignores the return value and
// keeps a "connecting" flag until the callback flips it to connected (result 0) or failed (else).
// A listed fake server answers with `cb(fd, M_E_SUCCESS, param)`; `addr` is `inet_addr`'s (the
// dotted quad's bytes in memory order) and `port` came through `htons`.
pub async fn socket_connect(context: &mut dyn WIPICContext, fd: i32, addr: u32, port: u32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    if context.system().fake_net().connect(fd, addr.to_le_bytes(), (port as u16).swap_bytes()) {
        tracing::warn!("MC_netSocketConnect({fd}, {addr:#x}, {port:#x}, {cb:#x}, {param:#x}) -> callback M_E_SUCCESS (fake server)");
        context.spawn(Box::new(Callback {
            cb,
            args: vec![fd as WIPICWord, 0, param],
            delay: false,
        }))?;
        return Ok(0);
    }
    tracing::warn!("MC_netSocketConnect({fd}, {addr:#x}, {port:#x}, {cb:#x}, {param:#x}) -> callback M_E_ERROR (no network)");

    struct SocketConnectCallback {
        fd: i32,
        cb: WIPICWord,
        param: WIPICWord,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for SocketConnectCallback {
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            context.call_function(self.cb, &[self.fd as WIPICWord, u32::MAX, self.param]).await?; // M_E_ERROR

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    context.spawn(Box::new(SocketConnectCallback { fd, cb, param }))?;

    Ok(0)
}

pub async fn close(_context: &mut dyn WIPICContext) -> Result<()> {
    tracing::warn!("stub MC_netClose()");

    Ok(())
}

pub async fn socket_close(context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketClose({fd})");
    if context.system().fake_net().close(fd) {
        return Ok(0);
    }

    // -1 is outside the `WIPICError` vocabulary and stays that way: this is a stub, so every
    // specific variant would blame something ("invalid argument", "bad handle") that is not what
    // failed. Sound because this call is off the `from_raw` transmute path — census and the other
    // three sites: docs/wipi-c-abi-error-codes.md.
    Ok(-1) // M_E_ERROR
}

// No network: the socket is never created. Same -1 as `socket_close`, same reasoning — unless the
// title has a fake server listed.
pub async fn socket(context: &mut dyn WIPICContext, domain: i32, r#type: i32) -> Result<i32> {
    if let Some(fd) = context.system().fake_net().socket() {
        tracing::warn!("MC_netSocket({domain}, {type}) -> {fd} (fake server)");
        return Ok(fd);
    }
    tracing::warn!("MC_netSocket({domain}, {type}) -> -1 (no network)");

    Ok(-1) // M_E_ERROR
}

// The whole buffer is always taken; a reply the fake server queues wakes a pending read callback.
pub async fn socket_write(context: &mut dyn WIPICContext, fd: i32, buf: u32, len: i32) -> Result<i32> {
    let mut data = vec![0; len.max(0) as usize];
    context.read_bytes(buf, &mut data)?;
    let Some(replied) = context.system().fake_net().write(fd, &data) else {
        tracing::warn!("MC_netSocketWrite({fd}, {buf:#x}, {len}) -> -1 (no network)");
        return Ok(-1);
    };
    let waiting = if replied { context.system().fake_net().take_read_cb(fd) } else { None };
    if let Some((cb, param)) = waiting {
        context.spawn(Box::new(Callback {
            cb,
            args: vec![fd as WIPICWord, 0, param],
            delay: false,
        }))?;
    }

    Ok(len)
}

// Nothing queued yet is `M_E_WOULDBLOCK`, which is what makes a title register `MC_netSetReadCB`.
pub async fn socket_read(context: &mut dyn WIPICContext, fd: i32, buf: u32, len: i32) -> Result<i32> {
    let Some(data) = context.system().fake_net().read(fd, len.max(0) as usize) else {
        tracing::warn!("MC_netSocketRead({fd}, {buf:#x}, {len}) -> -1 (no network)");
        return Ok(-1);
    };
    if data.is_empty() {
        return Ok(M_E_WOULDBLOCK);
    }
    context.write_bytes(buf, &data)?;

    Ok(data.len() as i32)
}

// `MC_netSetReadCB(fd, cb, param)`: one shot — `cb(fd, M_E_SUCCESS, param)` once a reply is queued.
pub async fn set_read_cb(context: &mut dyn WIPICContext, fd: i32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    let readable = context.system().fake_net().readable(fd);
    if readable {
        context.spawn(Box::new(Callback {
            cb,
            args: vec![fd as WIPICWord, 0, param],
            delay: false,
        }))?;
    } else {
        context.system().fake_net().set_read_cb(fd, cb, param);
    }

    Ok(0)
}

// `MC_netSetWriteCB(fd, cb, param)`: a write never blocks here, so the socket is writable now.
pub async fn set_write_cb(context: &mut dyn WIPICContext, fd: i32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    context.spawn(Box::new(Callback {
        cb,
        args: vec![fd as WIPICWord, 0, param],
        delay: false,
    }))?;

    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::context::test::TestContext;

    use alloc::{boxed::Box, vec};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite};

    use super::{WIPICContext, connect, set_read_cb, socket, socket_connect, socket_read, socket_write};

    fn context(aid: &str) -> TestContext {
        TestContext::with_system(System::new(Box::new(TestPlatform::new()), "", aid, DefaultTaskRunner))
    }

    // A KTF title's MC_netSocket was fatal; with no network the socket is refused.
    #[futures_test::test]
    async fn socket_is_refused_test() {
        assert_eq!(socket(&mut context(""), 2, 1).await.unwrap(), -1);
    }

    // An LGT title called MC_netSocketConnect (SVC 603) and died; the connect must fail through
    // `cb(fd, M_E_ERROR, param)` — the game waits on that callback, not on the return value.
    #[futures_test::test]
    async fn socket_connect_fails_through_callback_test() {
        let mut context = context("");
        assert_eq!(socket_connect(&mut context, -1, 0xaf4eedde, 0x3a9d, 0x24501, 7).await.unwrap(), 0);

        let callback = context.spawned.pop().unwrap();
        callback.call(&mut context, Box::new([])).await.unwrap();
        assert_eq!(context.call_args, [(0x24501, vec![u32::MAX, u32::MAX, 7])]);
    }

    // caf9d76ffd13's whole exchange through the WIPI-C calls, as its socket class makes them: the
    // connects succeed, the first read would block and leaves a read callback, the login's reply
    // wakes it, and the reply reads back whole.
    #[futures_test::test]
    async fn listed_title_talks_to_its_fake_server_test() {
        let mut context = context("00026E9C");
        connect(&mut context, 0x196f9, 9).await.unwrap();
        assert_eq!(context.spawned.len(), 1); // `cb(M_E_SUCCESS, param)`, a tick later
        context.spawned.clear();
        let fd = socket(&mut context, 2, 1).await.unwrap();
        assert!(fd >= 0);
        // `inet_addr("218.145.70.36")` and `htons(31106)`, as the title passes them
        socket_connect(&mut context, fd, u32::from_le_bytes([218, 145, 70, 36]), 0x8279, 0x1979d, 9)
            .await
            .unwrap();
        for callback in core::mem::take(&mut context.spawned) {
            callback.call(&mut context, Box::new([])).await.unwrap();
        }
        assert_eq!(context.call_args, [(0x1979d, vec![fd as u32, 0, 9])]);

        let buf = context.alloc_raw(0x40).unwrap();
        assert_eq!(socket_read(&mut context, fd, buf, 4).await.unwrap(), -19);
        set_read_cb(&mut context, fd, 0x19941, 9).await.unwrap();
        let mut login = vec![0x3f, 0, 0x02, 0x01];
        login.resize(0x3f, 0);
        context.write_bytes(buf, &login).unwrap();
        assert_eq!(socket_write(&mut context, fd, buf, 0x3f).await.unwrap(), 0x3f);
        let woken = context.spawned.pop().unwrap();
        woken.call(&mut context, Box::new([])).await.unwrap();
        assert_eq!(context.call_args.last(), Some(&(0x19941, vec![fd as u32, 0, 9])));

        assert_eq!(socket_read(&mut context, fd, buf, 0x40).await.unwrap(), 46);
        let mut reply = [0; 5];
        context.read_bytes(buf, &mut reply).unwrap();
        assert_eq!(reply, [46, 0, 0x03, 0x01, 1]);
    }
}
