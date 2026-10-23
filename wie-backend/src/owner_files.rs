//! The original owner's files shipped inside an archive (a dump of their handset): some start a
//! title in a state that is theirs, not a new player's. Each carrier's `from_archive` hands its
//! file map and jar here before mounting; the lists are keyed by the MD5 of the jar, which names one
//! build of one title whatever the carrier, so one list serves KTF and SKT alike.

use alloc::{borrow::ToOwned, collections::BTreeMap, format, string::String, vec::Vec};

/// The original owner's settings, shipped in the archive, that start the title with its sound off.
/// Measured per title, and dropped so the title starts as a fresh install does: removing only these
/// files brings the sound back in the 30 s probes (0 plays → 1–10), and on the titles whose settings
/// screen was opened, turning the sound up there does the same — the game honours them, nothing in
/// the engine is silent (docs/report/0417, 0537). Deliberately a list: other shipped settings and
/// saves load as they always have.
pub const OWNER_SOUND_OFF_SETTINGS: &[([u8; 16], &[&str])] = &[
    // Not a title: `WIRING_PROBE_JAR`, so each carrier's tests can see its `from_archive` route here.
    (
        [
            0x26, 0x7b, 0x1e, 0x2a, 0x93, 0xff, 0x10, 0x59, 0x16, 0x5a, 0xc6, 0xe0, 0xd5, 0xc2, 0x3e, 0x4a,
        ],
        &["probe.sav"],
    ),
    // KTF
    // 2fc792485d91: one 11,255-byte record: settings and progress together.
    (
        [
            0x26, 0xc9, 0x60, 0x35, 0x44, 0x6a, 0x98, 0x31, 0xcd, 0x3f, 0x53, 0xf7, 0x98, 0x48, 0x74, 0x30,
        ],
        &["sp.dat.db", "sp.dat.idx"],
    ),
    // 3185174d2121: 7 bytes.
    (
        [
            0xe9, 0x18, 0x85, 0xfb, 0x2c, 0x0f, 0xec, 0x9e, 0xa9, 0x7a, 0x0d, 0x29, 0xc1, 0x5e, 0xd7, 0x2f,
        ],
        &["setup.dat"],
    ),
    // 5e53e490c6f1: 12 bytes; its settings screen shows the sound level at 0.
    (
        [
            0x1a, 0x21, 0x23, 0xeb, 0x97, 0xe2, 0x51, 0xdb, 0xcb, 0x4b, 0x49, 0x0c, 0xee, 0x86, 0x30, 0xea,
        ],
        &["opt.txt"],
    ),
    // 6a885f89343c: a 6-byte DB; the title's other three DBs are not it.
    (
        [
            0x52, 0xfd, 0x30, 0xf7, 0x1d, 0x56, 0x67, 0x96, 0x2a, 0xa0, 0xab, 0x99, 0xf4, 0x70, 0x68, 0xb0,
        ],
        &["sky4.db", "sky4.idx"],
    ),
    // bfa8ec352451: one record: the owner's name, scores and option flags.
    (
        [
            0xcf, 0xa9, 0xe1, 0x86, 0x07, 0xf6, 0xdc, 0x07, 0xc6, 0x54, 0x82, 0x06, 0xbd, 0x25, 0xc2, 0xdf,
        ],
        &["kill.db", "kill.idx"],
    ),
    // de00506611a5: four records.
    (
        [
            0x05, 0x55, 0x29, 0x80, 0x3d, 0x08, 0xea, 0x81, 0x6b, 0xa2, 0x12, 0x2a, 0x7f, 0x8d, 0xa6, 0x87,
        ],
        &["haga2.db", "haga2.idx"],
    ),
    // 7218e8720f8c: 8 bytes among the data files; the title writes 8 bytes back when it is absent.
    (
        [
            0x93, 0x82, 0xc5, 0x4f, 0x44, 0xd6, 0xdc, 0xea, 0x90, 0xa9, 0x09, 0x51, 0x6c, 0x94, 0x2a, 0xc6,
        ],
        &["12_0.ida"],
    ),
    // 8bfd08fe4370: 18 bytes; the title writes 18 bytes back when it is absent.
    (
        [
            0x01, 0xe6, 0x86, 0xc6, 0xfb, 0x5a, 0x46, 0xde, 0x39, 0xb3, 0x7e, 0xa4, 0x18, 0x51, 0x9e, 0x3b,
        ],
        &["kjik.mcf"],
    ),
    // 34ab350dc98a: 3 bytes; the second is the sound level and the owner's is 0 (set to 3, it plays).
    (
        [
            0x1c, 0x40, 0xaa, 0x0b, 0x3c, 0x04, 0xa7, 0x60, 0x95, 0xaa, 0x68, 0x6c, 0x28, 0xcf, 0xd0, 0x46,
        ],
        &["BG0"],
    ),
    // c7f543c73b91: four ints; the first is the sound level and the owner's is 0 (set to 3, it plays).
    (
        [
            0x75, 0x09, 0x2c, 0xa0, 0x24, 0xa3, 0x54, 0x66, 0x08, 0xca, 0x10, 0x47, 0xbd, 0x9d, 0x30, 0x06,
        ],
        &["option.dat"],
    ),
    // db8ef04a6504: a 6-byte DB; the owner's first byte is 0, the title writes 3 there when it is absent.
    (
        [
            0x64, 0x87, 0x7e, 0x4b, 0x6d, 0x24, 0x50, 0x2a, 0xb8, 0xdb, 0x3e, 0xb0, 0x5b, 0x6f, 0x48, 0xb2,
        ],
        &["config"],
    ),
    // e085e193211d: 8 bytes; the first is the sound level and the owner's is 0 (set to 5, it plays).
    (
        [
            0x08, 0xa2, 0xe4, 0x6b, 0xe3, 0x74, 0x81, 0x68, 0xcc, 0x0b, 0x00, 0x21, 0x5e, 0x57, 0x87, 0x90,
        ],
        &["ga/cf.ga"],
    ),
    // 36b82cb67723: the profile and its backup copy — the owner's name, and a first int that is the
    // sound level, 0 (set to 3, it plays). The other eight `peng*` files are not it.
    (
        [
            0x47, 0xe3, 0x24, 0xc4, 0x31, 0x55, 0x49, 0xd8, 0x04, 0x7a, 0x5b, 0xfc, 0xbb, 0xf1, 0xac, 0x19,
        ],
        &["peng0.txt", "pengB0.txt"],
    ),
    // b907b0faf483: 6 bytes; the second is «sound on» and the owner's is 0 (set to 1, it plays).
    (
        [
            0x0f, 0x42, 0xb6, 0xec, 0x06, 0x92, 0xfd, 0x29, 0x31, 0x20, 0x7c, 0xb4, 0x75, 0x19, 0x81, 0xc3,
        ],
        &["config.sav"],
    ),
    // 5a59f62d1f1a: 4 bytes, shipped twice (bare and under `P/`); the first is the sound level and the
    // owner's is 0 (set to 3, it plays). `save.dat` beside it is not it.
    (
        [
            0x84, 0x05, 0x24, 0x81, 0xea, 0x39, 0x45, 0x7b, 0xa8, 0xf8, 0xd7, 0xfb, 0x06, 0x25, 0x2e, 0x57,
        ],
        &["config.dat"],
    ),
    // SKT
    // c361632541a7: 12 bytes: the tutorial, speed and sound options (`CGame` writes its defaults, sound on, when it is absent).
    (
        [
            0x2f, 0x75, 0x01, 0x25, 0x56, 0xc0, 0x15, 0x3f, 0x69, 0x8c, 0xc1, 0x6d, 0x17, 0x0d, 0x9e, 0xb5,
        ],
        &["pnjrestop"],
    ),
    // 7089dec0e8df: 4 bytes of options only; the first is «sound on» and the owner's is 0 (the title takes 1 when it is absent).
    (
        [
            0x0f, 0x32, 0x7d, 0x60, 0x84, 0x97, 0x76, 0xae, 0x89, 0x15, 0x02, 0x2a, 0x0f, 0xcb, 0xb5, 0x66,
        ],
        &["slot3.sav"],
    ),
];

/// The same owner's «sound off», for titles that cannot lose the file holding it: without it one
/// title exits at start and another freezes (docs/report/0440, 0517), and some hold the owner's
/// progress in the same file (docs/report/0537). Only the measured bytes are rewritten to the value
/// that plays — (jar MD5, file, offset, bytes); the rest of the file loads as shipped.
pub const OWNER_SOUND_OFF_BYTES: &[([u8; 16], &str, usize, &[u8])] = &[
    // Not a title: `WIRING_PROBE_JAR`, as above.
    (
        [
            0x26, 0x7b, 0x1e, 0x2a, 0x93, 0xff, 0x10, 0x59, 0x16, 0x5a, 0xc6, 0xe0, 0xd5, 0xc2, 0x3e, 0x4a,
        ],
        "probe.opt",
        0,
        &[1],
    ),
    // KTF
    // 5aa438fbdc87: `op_save.db` (5 bytes), the first is the sound level and the owner's is 0 (3 plays).
    (
        [
            0x09, 0xbf, 0x6d, 0x6d, 0x5b, 0xf4, 0x8f, 0xe6, 0x59, 0xee, 0xb1, 0x56, 0x3c, 0xed, 0x5a, 0xb9,
        ],
        "op_save.db",
        0,
        &[3],
    ),
    // 6c9f969f089f: `personal_data` (868 bytes), the big-endian int at +4 is the sound-off flag, the owner's is 1.
    (
        [
            0x8f, 0xc4, 0xf1, 0x4b, 0x20, 0xe4, 0xcd, 0xd0, 0x26, 0x39, 0xfd, 0x32, 0x26, 0xd2, 0xaa, 0xac,
        ],
        "personal_data",
        4,
        &[0, 0, 0, 0],
    ),
    // SKT
    // abc0c5789259: `Rms` (315 bytes) is the whole game context; byte 30 is «sound off» and the owner's is 1.
    (
        [
            0xbd, 0xb4, 0x09, 0x74, 0x14, 0x18, 0x6a, 0x1c, 0x19, 0xd5, 0x9c, 0x85, 0x3d, 0xee, 0xf6, 0x42,
        ],
        "Rms",
        30,
        &[0],
    ),
    // 6e93f26fa2f5: `data.sav` (581 bytes) holds the owner's progress too; +568 is «scenario cleared», +569 is «sound on» and the owner's is 0.
    (
        [
            0x02, 0x08, 0x5b, 0x4b, 0x1d, 0xda, 0x8d, 0x39, 0x41, 0xa5, 0x4d, 0x23, 0x1d, 0x77, 0xd4, 0xe3,
        ],
        "data.sav",
        569,
        &[1],
    ),
    // ccb45e6b8d80: `sc.txt` (71 bytes) is `/`-separated digits; the 33rd value, at +64, is «sound on» and the owner's is `0`.
    (
        [
            0x16, 0x0e, 0x14, 0x7e, 0x50, 0x9a, 0x4f, 0x90, 0x3a, 0x15, 0xbe, 0x71, 0x08, 0x69, 0x93, 0xe1,
        ],
        "sc.txt",
        64,
        b"1",
    ),
    // f483ba078c14: `option` (2 bytes), the first is the sound mode and the owner's is 3 = off (the title takes 1 when it is absent).
    (
        [
            0x12, 0x29, 0xf4, 0x11, 0xee, 0xfd, 0x06, 0x43, 0x82, 0xd9, 0x0a, 0x6d, 0x90, 0x4f, 0x7c, 0x09,
        ],
        "option",
        0,
        &[1],
    ),
    // fb80e97cbc57: `ds2.opt` (295 bytes), the first is the sound mode and the owner's is 0 = off (the title takes 1 when it is absent).
    (
        [
            0xa8, 0xa6, 0x13, 0xb3, 0xaf, 0x6c, 0xad, 0xa5, 0x7e, 0xab, 0x78, 0x52, 0xbf, 0x24, 0xf4, 0x73,
        ],
        "ds2.opt",
        0,
        &[1],
    ),
];

/// A «jar» no title ships, listed in both lists above: a carrier test mounting it with `probe.sav`
/// and `probe.opt` sees the first dropped and the second rewritten only if `from_archive` calls
/// `turn_owner_sound_on` — the rows alone cannot show that, since the real jars are game bytes.
pub const WIRING_PROBE_JAR: &[u8] = b"wie owner-files wiring probe";

/// Start the title with the sound on where the owner left it off: both lists above, for this jar.
pub fn turn_owner_sound_on(files: &mut BTreeMap<String, Vec<u8>>, jar: &[u8]) {
    drop_owner_files(files, jar, OWNER_SOUND_OFF_SETTINGS);
    rewrite_owner_bytes(files, jar, OWNER_SOUND_OFF_BYTES);
}

/// Remove the files `table` names for this jar, bare and under either private directory spelling
/// (KTF mounts all three at the same path; SKT ships them bare).
pub fn drop_owner_files(files: &mut BTreeMap<String, Vec<u8>>, jar: &[u8], table: &[([u8; 16], &[&str])]) {
    let hash = md5::compute(jar).0;
    for (_, names) in table.iter().filter(|(md5, _)| *md5 == hash) {
        for name in *names {
            for key in [(*name).to_owned(), format!("P/{name}"), format!("p/{name}")] {
                if files.remove(&key).is_some() {
                    tracing::info!("Not mounting the original owner's {key}");
                }
            }
        }
    }
}

/// Rewrite the bytes `table` names for this jar, wherever `drop_owner_files` would look.
pub fn rewrite_owner_bytes(files: &mut BTreeMap<String, Vec<u8>>, jar: &[u8], table: &[([u8; 16], &str, usize, &[u8])]) {
    let hash = md5::compute(jar).0;
    for (_, name, offset, value) in table.iter().filter(|(md5, ..)| *md5 == hash) {
        for key in [(*name).to_owned(), format!("P/{name}"), format!("p/{name}")] {
            if let Some(bytes) = files.get_mut(&key).and_then(|data| data.get_mut(*offset..offset + value.len())) {
                bytes.copy_from_slice(value);
                tracing::info!("Turning the original owner's sound on in {key}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, string::ToString, vec, vec::Vec};

    use super::{OWNER_SOUND_OFF_BYTES, OWNER_SOUND_OFF_SETTINGS, WIRING_PROBE_JAR, drop_owner_files, rewrite_owner_bytes, turn_owner_sound_on};

    fn md5(hex: &str) -> [u8; 16] {
        core::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
    }

    fn names(hex: &str) -> Option<&'static [&'static str]> {
        OWNER_SOUND_OFF_SETTINGS.iter().find(|(m, _)| *m == md5(hex)).map(|(_, n)| *n)
    }

    fn row(hex: &str) -> Option<(&'static str, usize, &'static [u8])> {
        OWNER_SOUND_OFF_BYTES
            .iter()
            .find(|(m, ..)| *m == md5(hex))
            .map(|(_, n, o, v)| (*n, *o, *v))
    }

    #[test]
    fn owner_files_dropped_for_listed_jar_only() {
        let archive = |names: &[&str]| names.iter().map(|name| (name.to_string(), vec![1])).collect::<BTreeMap<_, _>>();
        let table: &[([u8; 16], &[&str])] = &[(md5::compute(b"listed").0, &["res/save.sav", "res/savem.sav"])];

        let mut files = archive(&["A.jar", "__adf__", "P/res/save.sav", "res/savem.sav", "P/res/other.sav"]);
        drop_owner_files(&mut files, b"listed", table);
        assert_eq!(files.keys().collect::<Vec<_>>(), ["A.jar", "P/res/other.sav", "__adf__"]);

        // A lowercase private directory is mounted at the same path, so it is dropped the same way.
        let mut files = archive(&["A.jar", "p/res/save.sav", "p/res/other.sav"]);
        drop_owner_files(&mut files, b"listed", table);
        assert_eq!(files.keys().collect::<Vec<_>>(), ["A.jar", "p/res/other.sav"]);

        // Any other jar — every other title — mounts exactly what it shipped.
        let mut files = archive(&["A.jar", "P/res/save.sav", "P/res/savem.sav"]);
        drop_owner_files(&mut files, b"other", table);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn owner_bytes_rewritten_for_listed_jar_only() {
        let table: &[([u8; 16], &str, usize, &[u8])] = &[(md5::compute(b"listed").0, "s.db", 1, &[9, 9])];
        let archive = || {
            [("P/s.db".to_string(), vec![1, 2, 3, 4]), ("t.db".to_string(), vec![1, 2, 3, 4])]
                .into_iter()
                .collect::<BTreeMap<_, _>>()
        };

        let mut files = archive();
        rewrite_owner_bytes(&mut files, b"listed", table);
        assert_eq!(files["P/s.db"], [1, 9, 9, 4]);
        assert_eq!(files["t.db"], [1, 2, 3, 4]);

        let mut files = archive();
        rewrite_owner_bytes(&mut files, b"other", table);
        assert_eq!(files, archive());

        // The real lists touch nothing for a jar they do not name.
        let mut files = archive();
        turn_owner_sound_on(&mut files, b"other");
        assert_eq!(files, archive());
    }

    // Measured, not derivable: with these rows the titles play from the start (docs/report/0519, 0526,
    // 0537). The jars are game bytes and cannot be here, so this pins the rows rather than the effect.
    // The MD5s are `md5 -q <jar>` output pasted as-is, so a slip copying them into the byte rows fails here.
    #[test]
    fn owner_sound_off_rows_cover_the_measured_titles() {
        // KTF
        assert_eq!(names("0f42b6ec0692fd2931207cb4751981c3"), Some(&["config.sav"][..])); // b907b0faf483
        assert_eq!(names("84052481ea39457ba8f8d7fb06252e57"), Some(&["config.dat"][..])); // 5a59f62d1f1a
        assert_eq!(row("09bf6d6d5bf48fe659eeb1563ced5ab9"), Some(("op_save.db", 0, &[3][..]))); // 5aa438fbdc87
        assert_eq!(row("8fc4f14b20e4cdd02639fd3226d2aaac"), Some(("personal_data", 4, &[0, 0, 0, 0][..]))); // 6c9f969f089f

        // SKT
        assert_eq!(names("2f75012556c0153f698cc16d170d9eb5"), Some(&["pnjrestop"][..])); // c361632541a7
        assert_eq!(names("0f327d60849776ae8915022a0fcbb566"), Some(&["slot3.sav"][..])); // 7089dec0e8df
        assert_eq!(row("bdb4097414186a1c19d59c853deef642"), Some(("Rms", 30, &[0][..]))); // abc0c5789259
        assert_eq!(row("02085b4b1dda8d3941a54d231d77d4e3"), Some(("data.sav", 569, &[1][..]))); // 6e93f26fa2f5
        assert_eq!(row("160e147e509a4f903a15be71086993e1"), Some(("sc.txt", 64, &b"1"[..]))); // ccb45e6b8d80
        assert_eq!(row("1229f411eefd064382d90a6d904f7c09"), Some(("option", 0, &[1][..]))); // f483ba078c14
        assert_eq!(row("a8a613b3af6cada57eab7852bf24f473"), Some(("ds2.opt", 0, &[1][..]))); // fb80e97cbc57
    }

    // These titles keep their owner's progress in the file that holds «sound on», so it must never
    // be dropped — only that byte moves (docs/report/0537).
    #[test]
    fn a_file_holding_progress_is_rewritten_never_dropped() {
        for (hex, file) in [
            ("bdb4097414186a1c19d59c853deef642", "Rms"),      // abc0c5789259: the whole game context
            ("02085b4b1dda8d3941a54d231d77d4e3", "data.sav"), // 6e93f26fa2f5: save slots and autosave
        ] {
            assert_eq!(names(hex).filter(|names| names.contains(&file)), None, "{hex} {file}");
            assert_eq!(row(hex).map(|(name, ..)| name), Some(file), "{hex} {file}");
        }
    }

    #[test]
    fn wiring_probe_rows_are_keyed_to_the_probe_jar() {
        assert_eq!(md5::compute(WIRING_PROBE_JAR).0, md5("267b1e2a93ff1059165ac6e0d5c23e4a"));
        assert_eq!(names("267b1e2a93ff1059165ac6e0d5c23e4a"), Some(&["probe.sav"][..]));
        assert_eq!(row("267b1e2a93ff1059165ac6e0d5c23e4a"), Some(("probe.opt", 0, &[1][..])));
    }
}
