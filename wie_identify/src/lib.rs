//! Which game a file is, without booting it.
//!
//! The engine's per-title tables recognise a build by the MD5 of the jar it runs
//! (`docs/contracts/runtime-adjustments.tsv`); the shell recognised a file by the sha256 of the whole
//! archive, so the same game zipped again was a stranger to it. [`identify`] gives both one unit.
//! It is the only place the key is computed: `compat.json`'s `contentKey` comes from the
//! `wie_identify` binary and the browser's `identify()` export is this function.
#![no_std]
extern crate alloc;

use alloc::{format, string::String};

use wie_backend::extract_zip;
use wie_ktf::KtfEmulator;
use wie_lgt::{LgtEmulator, detect_compile_model};
use wie_skt::SktEmulator;

pub struct Identity {
    /// The loader that takes the file: "KTF" / "LGT" / "SKT" / "J2ME".
    pub platform: &'static str,
    /// LGT only: "clet" / "aot-java".
    pub model: Option<&'static str>,
    /// `zip:<md5>` — the jar the loader runs out of an archive — or `jar:<md5>`, a jar uploaded on
    /// its own. `None` when the archive holds no such jar (it cannot boot either).
    ///
    /// Everything else in an archive is left out on purpose: the wrapping (entry order, compression,
    /// timestamps, wrapper directories — `extract_zip` already sees through those), the descriptor,
    /// and the previous owner's saves and settings. Measured over the 429 corpus rows
    /// (docs/report/0567): 21 groups share a jar, none of them splits on support status, and only 8
    /// would survive hashing every file. The two containers never share a key, because a bare jar
    /// boots without its descriptor and outside every `from_archive` table.
    pub content_key: Option<String>,
}

/// `None` = no loader takes this file. Same dispatch order as the hosts' loaders.
pub fn identify(filename: &str, data: &[u8]) -> Option<Identity> {
    if filename.ends_with("zip") {
        let files = extract_zip(data).ok()?;
        let (platform, jar) = if KtfEmulator::loadable_archive(&files) {
            ("KTF", KtfEmulator::archive_jar(&files))
        } else if LgtEmulator::loadable_archive(&files) {
            ("LGT", LgtEmulator::archive_jar(&files))
        } else if SktEmulator::loadable_archive(&files) {
            ("SKT", SktEmulator::archive_jar(&files))
        } else {
            return None;
        };
        let model = if platform == "LGT" { detect_compile_model(&files) } else { None };

        Some(Identity {
            platform,
            model: model.map(|x| x.as_str()),
            content_key: jar.map(|jar| content_key("zip", jar)),
        })
    } else if filename.ends_with("jar") {
        let (platform, model) = if KtfEmulator::loadable_jar(data) {
            ("KTF", None)
        } else if LgtEmulator::loadable_jar(data) {
            ("LGT", extract_zip(data).ok().and_then(|inner| detect_compile_model(&inner)))
        } else if SktEmulator::loadable_jar(data) {
            ("SKT", None)
        } else {
            ("J2ME", None)
        };

        Some(Identity {
            platform,
            model: model.map(|x| x.as_str()),
            content_key: Some(content_key("jar", data)),
        })
    } else {
        None
    }
}

// ponytail: MD5 because it is the engine tables' own key, so the ledger and this agree by
// construction. Matching a listed game needs a second preimage, which MD5 still withstands; move
// to a longer hash together with the tables if they ever move.
fn content_key(container: &str, jar: &[u8]) -> String {
    format!("{container}:{:x}", md5::compute(jar))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
    use std::io::{Cursor, Write};

    use wie_backend::extract_zip;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    use super::identify;

    const KTF: &[u8] = include_bytes!("../../test_data/helloworld_ktf.zip");
    const LGT: &[u8] = include_bytes!("../../test_data/helloworld_lgt.zip");
    const J2ME: &[u8] = include_bytes!("../../test_data/draw_j2me.jar");

    fn md5_of(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    fn the_jar(files: &BTreeMap<String, Vec<u8>>) -> &Vec<u8> {
        let mut jars = files.iter().filter(|(name, _)| name.ends_with(".jar"));
        let jar = jars.next().unwrap().1;
        assert!(jars.next().is_none(), "the fixture holds one jar");
        jar
    }

    fn zip_of<'a>(entries: impl Iterator<Item = (String, &'a [u8])>, method: CompressionMethod) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in entries {
            zip.start_file(name, SimpleFileOptions::default().compression_method(method)).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn key(filename: &str, data: &[u8]) -> Option<String> {
        identify(filename, data).unwrap().content_key
    }

    #[test]
    fn the_key_is_the_md5_of_the_jar_the_loader_runs() {
        let ktf = identify("a.zip", KTF).unwrap();
        assert_eq!((ktf.platform, ktf.model), ("KTF", None));
        assert_eq!(ktf.content_key, Some(format!("zip:{}", md5_of(the_jar(&extract_zip(KTF).unwrap())))));

        let lgt = identify("a.zip", LGT).unwrap();
        assert_eq!((lgt.platform, lgt.model), ("LGT", Some("clet")));
        assert_eq!(lgt.content_key, Some(format!("zip:{}", md5_of(the_jar(&extract_zip(LGT).unwrap())))));

        let j2me = identify("a.jar", J2ME).unwrap();
        assert_eq!((j2me.platform, j2me.model), ("J2ME", None));
        assert_eq!(j2me.content_key, Some(format!("jar:{}", md5_of(J2ME))));
    }

    #[test]
    fn the_wrapping_and_the_owner_files_do_not_move_the_key_and_one_jar_byte_does() {
        let files = extract_zip(KTF).unwrap();
        let original = key("a.zip", KTF);

        // Another order, another compression, inside two wrapper directories.
        let rewrapped = zip_of(
            files.iter().rev().map(|(name, data)| (format!("dump/game/{name}"), data.as_slice())),
            CompressionMethod::Deflated,
        );
        assert_ne!(rewrapped, KTF);
        assert_eq!(key("b.zip", &rewrapped), original);

        // Another owner's phone: a save the fixture does not have.
        let other_dump = zip_of(
            files
                .iter()
                .map(|(name, data)| (name.clone(), data.as_slice()))
                .chain([("P/save.dat".into(), &b"owner"[..])]),
            CompressionMethod::Stored,
        );
        assert_eq!(key("c.zip", &other_dump), original);

        let mut patched = files.clone();
        *patched.iter_mut().find(|(name, _)| name.ends_with(".jar")).unwrap().1.last_mut().unwrap() ^= 1;
        let patched = zip_of(
            patched.iter().map(|(name, data)| (name.clone(), data.as_slice())),
            CompressionMethod::Stored,
        );
        assert_ne!(key("d.zip", &patched), original);
    }

    #[test]
    fn a_bare_jar_never_shares_a_key_with_its_archive() {
        let jar = the_jar(&extract_zip(KTF).unwrap()).clone();
        assert_eq!(key("a.zip", KTF), Some(format!("zip:{}", md5_of(&jar))));
        assert_eq!(key("a.jar", &jar), Some(format!("jar:{}", md5_of(&jar))));
    }

    #[test]
    fn skt_archives_and_files_no_loader_takes() {
        let skt = zip_of(
            [("g.msd".into(), &b"MIDlet-Name: g\n"[..]), ("g.jar".into(), &b"jar"[..])].into_iter(),
            CompressionMethod::Stored,
        );
        let skt = identify("a.zip", &skt).unwrap();
        assert_eq!(
            (skt.platform, skt.model, skt.content_key),
            ("SKT", None, Some(format!("zip:{}", md5_of(b"jar"))))
        );

        // A descriptor whose jar is not in the archive: recognised, and no key.
        let no_jar = zip_of([("__adf__".into(), &b"AID:A1\n"[..])].into_iter(), CompressionMethod::Stored);
        let no_jar = identify("a.zip", &no_jar).unwrap();
        assert_eq!((no_jar.platform, no_jar.content_key), ("KTF", None));

        let stray = zip_of([("readme.txt".into(), &b"x"[..])].into_iter(), CompressionMethod::Stored);
        assert!(identify("a.zip", &stray).is_none());
        assert!(identify("a.zip", b"not a zip").is_none());
        assert!(identify("a.jad", b"MIDlet-Name: g\n").is_none());
    }
}
