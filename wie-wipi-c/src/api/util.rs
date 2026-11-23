use alloc::string::String;

use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn htonl(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtonl({val:#x})");

    Ok(val.swap_bytes())
}

pub async fn htons(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilHtons({val})");

    Ok((val as u16).to_be() as _) // XXX we're always on little endian
}

pub async fn ntohs(_context: &mut dyn WIPICContext, val: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_utilNtohs({val})");

    Ok(u16::from_be(val as u16) as _) // XXX we're always on little endian
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

#[cfg(test)]
mod tests {
    use wie_util::{ByteWrite, Result};

    use crate::context::test::TestContext;

    use super::inet_addr;

    #[futures_test::test]
    async fn ipv4_conversion_returns_network_order() -> Result<()> {
        let mut context = TestContext::new();
        for (text, expected) in [
            (b"127.0.0.1\0".as_slice(), [127, 0, 0, 1]),
            (b"192.168.1.25\0".as_slice(), [192, 168, 1, 25]),
            (b"0.0.0.0\0".as_slice(), [0, 0, 0, 0]),
            // ours: leading zeros are decimal digits (`Ipv4Addr::from_str`, upstream's parser, refuses them)
            (b"010.001.002.003\0".as_slice(), [10, 1, 2, 3]),
        ] {
            context.write_bytes(0x1000, text)?;
            assert_eq!(inet_addr(&mut context, 0x1000).await?.to_le_bytes(), expected);
        }
        for text in [b"256.0.0.1\0".as_slice(), b"invalid\0".as_slice(), b"\xff\0".as_slice()] {
            context.write_bytes(0x1000, text)?;
            assert_eq!(inet_addr(&mut context, 0x1000).await?, u32::MAX);
        }
        Ok(())
    }
}
