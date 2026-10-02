use alloc::{boxed::Box, vec::Vec};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError};

use crate::{WIPICResult, context::WIPICContext, method::MethodBody};

pub async fn connect(context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_netConnect({cb:#x}, {param:#x})");

    struct ConnectCallback {
        cb: WIPICWord,
        param: WIPICWord,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for ConnectCallback {
        #[tracing::instrument(name = "timer", skip_all)]
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            context.system().sleep(1).await; // simulate some delay

            context.call_function(self.cb, &[u32::MAX, self.param]).await?; // callback with M_E_ERROR

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    context.spawn(Box::new(ConnectCallback { cb, param }))?;

    Ok(0)
}

// No network, so the connect fails — asynchronously, as `connect` above does: the result goes to
// `cb(fd, result, param)`. fe76e641bb3d (LGT 603, binary.mod 0x24428) ignores the return value and
// keeps a "connecting" flag until the callback flips it to connected (result 0) or failed (else).
pub async fn socket_connect(context: &mut dyn WIPICContext, fd: i32, addr: u32, port: u32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
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

pub async fn socket_close(_context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketClose({fd})");

    // -1 is outside the `WIPICError` vocabulary and stays that way: this is a stub, so every
    // specific variant would blame something ("invalid argument", "bad handle") that is not what
    // failed. Sound because this call is off the `from_raw` transmute path — census and the other
    // three sites: docs/wipi-c-abi-error-codes.md.
    Ok(-1) // M_E_ERROR
}

// No network: the socket is never created. Same -1 as `socket_close`, same reasoning.
pub async fn socket(_context: &mut dyn WIPICContext, domain: i32, r#type: i32) -> Result<i32> {
    tracing::warn!("MC_netSocket({domain}, {type}) -> -1 (no network)");

    Ok(-1) // M_E_ERROR
}

#[cfg(test)]
mod tests {
    use crate::context::test::TestContext;

    use alloc::boxed::Box;

    use super::{socket, socket_connect};

    // A KTF title's MC_netSocket was fatal; with no network the socket is refused.
    #[futures_test::test]
    async fn socket_is_refused_test() {
        assert_eq!(socket(&mut TestContext::new(), 2, 1).await.unwrap(), -1);
    }

    // An LGT title called MC_netSocketConnect (SVC 603) and died; the connect must fail through
    // `cb(fd, M_E_ERROR, param)` — the game waits on that callback, not on the return value.
    #[futures_test::test]
    async fn socket_connect_fails_through_callback_test() {
        let mut context = TestContext::new();
        assert_eq!(socket_connect(&mut context, -1, 0xaf4eedde, 0x3a9d, 0x24501, 7).await.unwrap(), 0);

        let callback = context.spawned.pop().unwrap();
        callback.call(&mut context, Box::new([])).await.unwrap();
        assert_eq!(context.call_args, [(0x24501, alloc::vec![u32::MAX, u32::MAX, 7])]);
    }
}
