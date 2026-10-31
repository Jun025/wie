//! Guest methods answered with a fixed value, keyed by the MD5 of the class file.
//!
//! The Java sibling of `wie-core-arm`'s `binary_patches.toml`: a per-title entry that changes
//! what the emulator *does at run time* and never touches a byte on disk. Unlike that table
//! there is no un-keyed row — an entry with no hash would reach every title that happens to
//! declare the same method name, so the hash IS the scope and a table lookup that misses
//! changes nothing.
//!
//! Every row today is the same lock: the XCE `SecureUtil` family, whose `isValid(MIDlet)`
//! compares `MIDlet-Key` against `hex(MD5(carrier prefix + MIN + SERVICE_ID + salt))`
//! (`docs/report/0386` §2). The only MIN that matches is the original buyer's real handset
//! number, so there is no value the emulator can answer with; what it can do is answer the
//! *decision*. `validate` draws 「인증 되지 않은 컨텐츠…」 and calls `System.exit(-1)` when
//! `isValid` is false, and does nothing at all when it is true — so one forced `true` is the
//! whole patch, and the number is neither read nor needed.
//!
//! Scope is deliberately per-class, not per-jar or per-method-name: two of the five titles
//! ship the family obfuscated (`b.l`, `j.if`) and one more renames it to a private `a`, so a
//! name rule would be unwritable, while a jar hash would need the jar bytes threaded down to
//! `define_class`, which only ever sees one class. Corpus-wide scan 2026-10-28: of the 84 SKT
//! packages' class entries, these five MD5s match five classes in five packages — the five the
//! operator named.
//!
//! Adding a row is an operator decision per title (the 2026-10-02 「인증 우회 0」 ruling is
//! lifted one sha at a time), not an engine judgement. Do not generalise it into a rule that
//! catches a sixth title for free.

/// A method whose body is skipped, answering `value` instead.
#[derive(Debug)]
pub(crate) struct ForcedBool {
    pub name: &'static str,
    pub descriptor: &'static str,
    pub value: bool,
}

struct ClassPatch {
    /// MD5 of the class file as the guest's jar holds it.
    md5: [u8; 16],
    methods: &'static [ForcedBool],
}

const IS_VALID: &str = "(Ljavax/microedition/midlet/MIDlet;)Z";

/// sha12 of the package each row belongs to, in a comment — the class hash is the key.
const PATCHES: [ClassPatch; 5] = [
    // 0ed66634d3dc — `b.l`, the family obfuscated; `b.m` is `validate`.
    ClassPatch {
        md5: [
            0x4b, 0xfb, 0x58, 0x1b, 0x97, 0x00, 0x02, 0xa6, 0xe1, 0x5c, 0x8f, 0x2e, 0x0a, 0xa0, 0xdb, 0x57,
        ],
        methods: &[ForcedBool {
            name: "l",
            descriptor: IS_VALID,
            value: true,
        }],
    },
    // a42f77f44955 — `j.if`; `j.a(MIDlet)` is `validate`. Reads `MIDlet-Key`..`MIDlet-Key4`.
    ClassPatch {
        md5: [
            0xc4, 0xae, 0xea, 0xc1, 0xe2, 0xde, 0x7d, 0x98, 0x10, 0xc9, 0xbc, 0x86, 0xa7, 0xe7, 0xf4, 0xaf,
        ],
        methods: &[ForcedBool {
            name: "if",
            descriptor: IS_VALID,
            value: true,
        }],
    },
    // c33090c12755 — `com/xce/security/SecureUtil`, the family unobfuscated.
    ClassPatch {
        md5: [
            0xb8, 0x1f, 0x87, 0x53, 0xf4, 0xf1, 0x51, 0x3f, 0x7c, 0x20, 0x35, 0x5e, 0x92, 0xbd, 0x6c, 0x64,
        ],
        methods: &[ForcedBool {
            name: "isValid",
            descriptor: IS_VALID,
            value: true,
        }],
    },
    // eefc947d8337 — the same class in the default package.
    ClassPatch {
        md5: [
            0x51, 0xc2, 0x18, 0xd2, 0xd9, 0x35, 0xeb, 0x91, 0x63, 0xad, 0xfa, 0x69, 0x83, 0x8d, 0x88, 0x9f,
        ],
        methods: &[ForcedBool {
            name: "isValid",
            descriptor: IS_VALID,
            value: true,
        }],
    },
    // 44b6356d13f8 — class `a`, the family with `isValid` made private. `docs/report/0418`
    // read this title's wall as its `install.dat` open; that open is a first-run marker
    // (writeLong(now) + writeInt(hashCode of `m.MIN`)) whose absence is caught and recreated
    // two instructions later, and the exit comes from `a.a(MIDlet)` right after it.
    ClassPatch {
        md5: [
            0x15, 0x56, 0xbd, 0x6f, 0xd4, 0x71, 0x12, 0xc0, 0x1c, 0xe3, 0x5f, 0x64, 0x95, 0xb7, 0x92, 0x56,
        ],
        methods: &[ForcedBool {
            name: "a",
            descriptor: IS_VALID,
            value: true,
        }],
    },
];

/// The methods to answer for a class the guest is defining, empty for every class but five.
pub(crate) fn forced_bools(class: &[u8]) -> &'static [ForcedBool] {
    let digest = md5::compute(class).0;

    match PATCHES.iter().find(|patch| patch.md5 == digest) {
        Some(patch) => {
            tracing::info!("Purchase-lock patch matched: {} method(s) forced", patch.methods.len());
            patch.methods
        }
        None => &[],
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{IS_VALID, PATCHES, forced_bools};

    /// Every row must name a distinct class, and the hash must be what selects it — a row whose
    /// hash is all zeroes (or shared with another) would reach a class nobody measured.
    #[test]
    fn every_row_is_one_distinct_class_hash() {
        let mut hashes = PATCHES.iter().map(|x| x.md5).collect::<Vec<_>>();
        hashes.sort();
        hashes.dedup();

        assert_eq!(hashes.len(), PATCHES.len());
        assert!(!hashes.contains(&[0u8; 16]));
        assert!(PATCHES.iter().all(|x| !x.methods.is_empty()));
        assert!(PATCHES.iter().flat_map(|x| x.methods).all(|x| x.descriptor == IS_VALID));
    }

    /// A class nobody listed is not patched — the table's whole scope argument.
    #[test]
    fn an_unlisted_class_is_untouched() {
        assert!(forced_bools(b"\xca\xfe\xba\xbe not a listed class").is_empty());
        assert!(forced_bools(&[]).is_empty());
    }
}
