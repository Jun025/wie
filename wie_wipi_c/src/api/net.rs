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
