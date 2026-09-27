use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use encoding_rs::EUC_KR;

use wie_backend::extract_zip;
use wie_util::{Result, WieError};

pub struct KtfAdf {
    pub name: String,
    pub aid: String,
    pub pid: String,
    pub mclass: String,
    pub display_size: Option<(u32, u32)>,
}

impl KtfAdf {
    pub fn parse(data: &[u8]) -> Self {
        let mut name = String::new();
        let mut aid = String::new();
        let mut pid = String::new();
        let mut mclass = String::new();
        let mut display_size = None;

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"Name:") {
                name = EUC_KR.decode(&line[5..]).0.trim().to_string();
            } else if line.starts_with(b"AID:") {
                aid = String::from_utf8_lossy(&line[4..]).trim().into();
            } else if line.starts_with(b"PID:") {
                pid = String::from_utf8_lossy(&line[4..]).trim().into();
            } else if line.starts_with(b"MClass:") {
                mclass = String::from_utf8_lossy(&line[7..]).trim().into();
            } else if line.starts_with(b"DisplaySize:") {
                display_size = parse_display_size(&line[12..]);
            }
        }

        Self {
            name,
            aid,
            pid,
            mclass,
            display_size,
        }
    }
}

fn parse_display_size(data: &[u8]) -> Option<(u32, u32)> {
    let value = core::str::from_utf8(data).ok()?.trim();
    let separator = value.find(['*', 'x', 'X'])?;
    let width: u32 = value[..separator].trim().parse().ok()?;
    let height: u32 = value[separator + 1..].trim().parse().ok()?;

    (width > 0 && height > 0).then_some((width, height))
}

pub fn find_client_bin(jar: &[u8]) -> Result<(String, Vec<u8>)> {
    let files: BTreeMap<String, Vec<u8>> = extract_zip(jar)?;

    files
        .into_iter()
        .find(|(name, _)| name.starts_with("client.bin"))
        .ok_or_else(|| WieError::FatalError("client.bin* not found in jar".to_string()))
}

pub fn parse_bss_size(filename: &str) -> Result<u32> {
    filename
        .strip_prefix("client.bin")
        .ok_or_else(|| WieError::FatalError(format!("Filename does not start with 'client.bin': {filename}")))?
        .parse::<u32>()
        .map_err(|e| WieError::FatalError(format!("Invalid bss_size in filename {filename}: {e}")))
}

/// A `client.bin` that begins with its relocation table instead of the self-relocating entry stub:
/// `[u32 bss_size][u32 count][count × u32 image offsets, ascending][image]`, where the first word
/// repeats the filename's bss size. The image behind it has no `WIPI_exe` export and addresses its
/// globals through `sl` (a different runtime ABI), so running it the stub way jumps into the table.
/// Measured on 3 KTF titles; the standard stub is `04 e0 c0 46` in all 190 working KTF titles.
pub fn is_relocation_prefixed(data: &[u8], bss_size: u32) -> bool {
    let word = |index: usize| data.get(index * 4..index * 4 + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let (Some(first), Some(count)) = (word(0), word(1)) else {
        return false;
    };
    let count = count as usize;
    if first != bss_size || count == 0 || (2 + count) * 4 > data.len() {
        return false;
    }
    let image_len = (data.len() - (2 + count) * 4) as u32;
    let offsets = (2..2 + count).map(|index| word(index).unwrap_or(u32::MAX));

    offsets.clone().zip(offsets.skip(1)).all(|(a, b)| a < b) && word(1 + count).is_some_and(|last| last < image_len)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{KtfAdf, is_relocation_prefixed, parse_bss_size};

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|value| value.to_le_bytes()).collect()
    }

    #[test]
    fn relocation_prefixed_client_bin_is_detected() {
        // bss 64 · 3 offsets · 16-byte image
        let data = words(&[64, 3, 0, 8, 12, 0, 0, 0, 0]);
        assert!(is_relocation_prefixed(&data, 64));
        // filename bss disagrees with the first word
        assert!(!is_relocation_prefixed(&data, 1096));
        // offsets not ascending
        assert!(!is_relocation_prefixed(&words(&[64, 3, 0, 12, 8, 0, 0, 0, 0]), 64));
        // last offset past the image
        assert!(!is_relocation_prefixed(&words(&[64, 3, 0, 8, 16, 0, 0, 0, 0]), 64));
        // the standard entry stub (`b.n` + `nop`)
        assert!(!is_relocation_prefixed(&words(&[0x46c0_e004, 0x2004_0224, 0x0002_0001]), 1096));
    }

    #[test]
    fn parse_adf_full() {
        let data = b"Name:\xc2\xa5\xbf\xe4\xc2\xa5\xbf\xe4\xc5\xb8\xc0\xcc\xc4\xef2\nAID:foo\nPID:bar\nMClass:baz\nDisplaySize:176*220\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.name, "짜요짜요타이쿤2");
        assert_eq!(adf.aid, "foo");
        assert_eq!(adf.pid, "bar");
        assert_eq!(adf.mclass, "baz");
        assert_eq!(adf.display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_crlf() {
        let data = b"AID:foo\r\nPID:bar\r\nMClass:baz\r\nDisplaySize:176*220\r\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "foo");
        assert_eq!(adf.pid, "bar");
        assert_eq!(adf.mclass, "baz");
        assert_eq!(adf.display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_empty() {
        let adf = KtfAdf::parse(b"");
        assert!(adf.name.is_empty());
        assert!(adf.aid.is_empty());
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
        assert_eq!(adf.display_size, None);
    }

    #[test]
    fn parse_adf_partial() {
        let data = b"AID:only\n";
        let adf = KtfAdf::parse(data);
        assert!(adf.name.is_empty());
        assert_eq!(adf.aid, "only");
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
        assert_eq!(adf.display_size, None);
    }

    #[test]
    fn parse_adf_display_size_variants() {
        assert_eq!(KtfAdf::parse(b"DisplaySize: 176 x 220\r\n").display_size, Some((176, 220)));
        assert_eq!(KtfAdf::parse(b"DisplaySize:176X220\n").display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_invalid_display_size() {
        assert_eq!(KtfAdf::parse(b"DisplaySize:invalid\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:0*220\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:176*0\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:4294967296*220\n").display_size, None);
    }

    #[test]
    fn parse_bss_size_ok() {
        assert_eq!(parse_bss_size("client.bin12345").unwrap(), 12345);
        assert_eq!(parse_bss_size("client.bin0").unwrap(), 0);
    }

    #[test]
    fn parse_bss_size_missing_marker() {
        assert!(parse_bss_size("not_a_client_bin_name").is_err());
    }

    #[test]
    fn parse_bss_size_no_digits() {
        assert!(parse_bss_size("client.bin").is_err());
    }

    #[test]
    fn parse_bss_size_non_numeric() {
        assert!(parse_bss_size("client.binABC").is_err());
    }
}
