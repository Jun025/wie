//! Static classification of an LGT container as a WIPI-C **clet** or an
//! AOT-compiled **aot-java** app — a read-only, pre-execution discriminator.
//!
//! Both kinds ship the same container shape (a `binary.mod` ELF inside the app
//! jar, plus `app_info`), so the container form alone cannot tell them apart.
//! The distinguishing fact is which platform import table the native code binds:
//! every import site is a 16-byte thunk `str lr; bl <dispatcher>; .word table;
//! .word index` (see `docs/lgt_abi.md` §1). An **aot-java** app imports through
//! table `0x64` (the "java-interface" module); a WIPI-C **clet** imports through
//! table `0x1fb`. The two are mutually exclusive in practice — verified across
//! the full local corpus (working/lgt 54 all clet; broken/lgt 24 aot-java + 22
//! clet, matching the known AOT-Java count, zero ambiguous).
//!
//! This is the same static signal the §7 investigation used to identify the 24
//! AOT-Java titles offline; it executes nothing and renders nothing (§7 render
//! stays frozen). It only reads the app's own import thunks.

use alloc::{collections::BTreeMap, string::String, vec::Vec};

use elf::{ElfBytes, endian::AnyEndian};

use wie_backend::extract_zip;

/// Which compile model an LGT app uses. `as_str` gives the stable wire label the
/// web surface exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LgtCompileModel {
    /// WIPI-C clet (imports through table `0x1fb`). wie has a working render path.
    Clet,
    /// AOT-compiled Java app (imports through table `0x64`). Boots but does not
    /// yet render (the §7 wall).
    AotJava,
}

impl LgtCompileModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clet => "clet",
            Self::AotJava => "aot-java",
        }
    }
}

// Import table ids that appear as the `.word table` of a thunk.
const TABLE_JAVA_INTERFACE: u32 = 0x64; // aot-java
const ARM_BL_MASK: u32 = 0xff00_0000;
const ARM_BL_OP: u32 = 0xeb00_0000;

/// Classify the LGT container in `files` without executing it. Returns `None`
/// only if no `binary.mod` can be located (not a loadable LGT container).
///
/// Locates `binary.mod` either directly in `files` or inside the app jar, then
/// scans its executable code for an import thunk that binds table `0x64`. A
/// `0x64` thunk ⇒ [`LgtCompileModel::AotJava`]; its absence ⇒
/// [`LgtCompileModel::Clet`] (clets bind `0x1fb`).
pub fn detect_compile_model(files: &BTreeMap<String, Vec<u8>>) -> Option<LgtCompileModel> {
    let binary_mod = find_binary_mod(files)?;
    Some(scan_binary_mod(&binary_mod))
}

fn find_binary_mod(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
    // Some containers hold binary.mod at the top level; most nest it in the jar.
    for (name, data) in files {
        let name = name.trim_start_matches("P/");
        if name == "binary.mod" || name.ends_with("/binary.mod") {
            return Some(data.clone());
        }
    }
    for (name, data) in files {
        if name.to_ascii_lowercase().ends_with(".jar")
            && let Ok(inner) = extract_zip(data)
            && let Some(bm) = inner.get("binary.mod")
        {
            return Some(bm.clone());
        }
    }
    None
}

fn scan_binary_mod(data: &[u8]) -> LgtCompileModel {
    // Prefer scanning only executable sections (where thunks live). If the ELF
    // won't parse, fall back to a raw scan — the BL context keeps it precise
    // enough that a whole-file scan produced zero false hits across the corpus.
    if let Ok(elf) = ElfBytes::<AnyEndian>::minimal_parse(data)
        && let Some(shdrs) = elf.section_headers()
    {
        for shdr in shdrs {
            if shdr.sh_flags & (elf::abi::SHF_EXECINSTR as u64) == 0 {
                continue;
            }
            if let Ok((bytes, _)) = elf.section_data(&shdr)
                && has_java_interface_thunk(bytes)
            {
                return LgtCompileModel::AotJava;
            }
        }
        return LgtCompileModel::Clet;
    }

    if has_java_interface_thunk(data) {
        LgtCompileModel::AotJava
    } else {
        LgtCompileModel::Clet
    }
}

/// True if `code` contains an import thunk that binds table `0x64`: a `BL`
/// instruction immediately followed by the word `0x64` (the thunk's `.word
/// table`). Scanning for the `BL` context (not a bare `0x64` constant) is what
/// keeps this free of false positives.
fn has_java_interface_thunk(code: &[u8]) -> bool {
    let mut prev_is_bl = false;
    for w in code.chunks_exact(4) {
        let word = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
        if prev_is_bl && word == TABLE_JAVA_INTERFACE {
            return true;
        }
        prev_is_bl = (word & ARM_BL_MASK) == ARM_BL_OP;
    }
    false
}
