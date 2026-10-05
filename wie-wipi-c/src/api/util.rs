use alloc::string::String;

use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn htons(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtons({val})");

    Ok((val as u16).to_be() as _) // XXX we're always on little endian
}

/// LGT WIPIC 904 (`inet_addr`) and KTF util slot 4 (`MC_utilInetAddrInt`): a dotted quad to an address in network order, `0xffffffff`
/// (`INADDR_NONE`) when the string is not one. Only the four-decimal-part form is accepted —
/// the one form measured at a call site (LGT; KTF 30c7bd6fb01b passes `"218.145.70.36"`); the BSD shorthand forms are not guessed at.
pub async fn inet_addr(context: &mut dyn WIPICContext, ptr_cp: u32) -> Result<u32> {
    let text = read_null_terminated_string_bytes(context, ptr_cp)?;
    tracing::debug!("inet_addr({:?})", String::from_utf8_lossy(&text));

    Ok(parse_dotted_quad(&text).map_or(u32::MAX, u32::from_le_bytes))
}

fn parse_dotted_quad(text: &[u8]) -> Option<[u8; 4]> {
    let mut out = [0u8; 4];
    let mut parts = text.split(|&b| b == b'.');
    for slot in &mut out {
        let part = parts.next()?;
        if part.is_empty() || part.len() > 3 || !part.iter().all(u8::is_ascii_digit) {
            return None;
        }
        *slot = core::str::from_utf8(part).ok()?.parse().ok()?;
    }

    parts.next().is_none().then_some(out)
}
