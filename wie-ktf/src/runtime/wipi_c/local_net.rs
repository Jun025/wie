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
/// got while it was measured; its socket lasts until the next `MC_netSocket` replaces it. The title
/// never reads the answer — its one close, image 0x103dee, goes straight on to `MC_netClose` — so 0
/// would play the same and is no reason to move (docs/report/0577).
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

// What the header's list keeps apart from LGT's shape, through the table the guest calls — so the slot
// a call sits in is held too. The values are f44271803135's measured run (docs/report/0541, 0569 §1).
#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use test_utils::{TestClock, TestPlatform};
    use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic};
    use wie_wipi_c::WIPICContext;

    use crate::runtime::{
        svc_ids::WIPICTableId,
        wipi_c::{
            context::{KtfWIPICContext, test::run_on},
            method_table::get_method_body,
        },
    };

    const FD: u32 = 0x4c4e; // written out: a test that read `LOCAL_FD` would follow it anywhere
    const HOST: u32 = 0xfd12_6ed3; // inet_addr("211.110.18.253"), as the title passes it
    const PORT: u32 = 0x3721; // htons(8503)
    const UNSET: [u32; 2] = [0x5a5a_5a5a; 2];

    async fn call(context: &mut KtfWIPICContext, slot: u16, args: &[u32]) -> Result<i32> {
        let body = get_method_body(WIPICTableId::Net, slot).unwrap();

        Ok(body.call(context, args.into()).await?.results[0] as i32)
    }

    /// A guest `cb(fd, result, param)` that stores `fd` and `result` at `param`, and a block for it.
    fn callback(context: &mut KtfWIPICContext) -> Result<(u32, u32)> {
        let code = context.alloc_raw(6)?;
        write_generic(context, code, [0x6010u16, 0x6051, 0x4770])?; // str r0,[r2]; str r1,[r2,#4]; bx lr
        let seen = context.alloc_raw(8)?;
        write_generic(context, seen, UNSET)?;

        Ok((code | 1, seen))
    }

    /// What the callback stored at `seen`, once it has — or `UNSET` after `millis` of the clock.
    async fn wait(context: &mut KtfWIPICContext, clock: &TestClock, seen: u32, millis: u64) -> Result<[u32; 2]> {
        let until = clock.peek() + millis;
        loop {
            let stored: [u32; 2] = read_generic(context, seen)?;
            if stored != UNSET || clock.peek() >= until {
                return Ok(stored);
            }
            context.system().yield_now().await;
        }
    }

    #[test]
    fn listed_title_is_answered_in_the_shape_it_was_measured_against() -> Result<()> {
        // 1µs a read: time passes with the executor's turns, never with the host's load
        let clock = TestClock::stepping_micros(1);
        run_on(TestPlatform::with_clock(clock.clone()), "0103A2B4", move |mut context| {
            Box::pin(async move {
                let (cb, seen) = callback(&mut context)?;

                // one socket, always the same descriptor — not LGT's 0, 1, 2…
                assert_eq!(call(&mut context, 2, &[2, 1]).await?, FD as i32);
                assert_eq!(call(&mut context, 2, &[2, 1]).await?, FD as i32);

                // slot 30 answers through the callback, a millisecond later and not before
                let asked = clock.peek();
                assert_eq!(call(&mut context, 30, &[FD, HOST, PORT, 0, cb, seen]).await?, 0);
                assert_eq!(wait(&mut context, &clock, seen, 50).await?, [FD, 0]);
                assert_eq!(clock.peek() - asked, 1, "the connect callback's delay");

                // slot 31 takes the list request whole; slot 13 is accepted and never fires, even
                // with the reply already waiting (LGT's would fire at once)
                let buf = context.alloc_raw(158)?;
                let mut request = vec![156, 0, 0xf8, 0x2a];
                request.resize(158, 0);
                context.write_bytes(buf, &request)?;
                assert_eq!(call(&mut context, 31, &[FD, buf, 158]).await?, 158);
                write_generic(&mut context, seen, UNSET)?;
                assert_eq!(call(&mut context, 13, &[FD, cb, seen]).await?, 0);
                assert_eq!(wait(&mut context, &clock, seen, 5).await?, UNSET);

                // slot 32 reads it back, then would block
                assert_eq!(call(&mut context, 32, &[FD, buf, 16, 3000]).await?, 5);
                let mut reply = [0; 5];
                context.read_bytes(buf, &mut reply)?;
                assert_eq!(reply, [3, 0, 0xf9, 0x2a, 0]);
                assert_eq!(call(&mut context, 32, &[FD, buf, 16, 3000]).await?, -19);

                // both closes answer M_E_ERROR and leave the socket (LGT's: 0, socket gone)
                for close in [6, 33] {
                    assert_eq!(call(&mut context, close, &[FD]).await?, -1, "slot {close}");
                    assert!(context.system().fake_net().contains(FD as i32), "slot {close}");
                }

                // any other host or port is refused through the same callback
                assert_eq!(call(&mut context, 30, &[FD, HOST, PORT + 0x100, 0, cb, seen]).await?, 0);
                assert_eq!(wait(&mut context, &clock, seen, 50).await?, [FD, u32::MAX]);

                Ok(())
            })
        })
    }

    #[test]
    fn a_descriptor_this_file_did_not_hand_out_is_fatal() -> Result<()> {
        fn fatal(result: Result<i32>) -> bool {
            matches!(result, Err(WieError::Unimplemented(_)))
        }

        // an unlisted title: no socket, and the slots are the fatal stubs they have always been
        run_on(TestPlatform::new(), "00000000", |mut context| {
            Box::pin(async move {
                assert_eq!(call(&mut context, 2, &[2, 1]).await?, -1);
                for (slot, args) in [
                    (30, &[u32::MAX, HOST, PORT, 0, 0, 0][..]),
                    (31, &[u32::MAX, 0, 0]),
                    (32, &[u32::MAX, 0, 0, 3000]),
                    (13, &[u32::MAX, 0, 0]),
                ] {
                    assert!(fatal(call(&mut context, slot, args).await), "slot {slot}");
                }
                assert_eq!(call(&mut context, 6, &[u32::MAX]).await?, -1);
                assert_eq!(call(&mut context, 33, &[u32::MAX]).await?, -1);

                Ok(())
            })
        })?;

        // the listed title, on a descriptor that is not its one (0 is what LGT's shape would hand it) —
        // and at LGT's connect, write and read, which KTF's table still does not have
        run_on(TestPlatform::new(), "0103A2B4", |mut context| {
            Box::pin(async move {
                assert_eq!(call(&mut context, 2, &[2, 1]).await?, FD as i32);
                for (slot, args) in [
                    (30, &[0, HOST, PORT, 0, 0, 0][..]),
                    (31, &[0, 0, 0]),
                    (32, &[0, 0, 0, 3000]),
                    (13, &[0, 0, 0]),
                ] {
                    assert!(fatal(call(&mut context, slot, args).await), "slot {slot}");
                }
                for slot in 3..=5 {
                    assert!(fatal(call(&mut context, slot, &[FD, HOST, PORT, 0, 0]).await), "slot {slot}");
                }

                Ok(())
            })
        })
    }
}
