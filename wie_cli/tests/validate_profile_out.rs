//! `--profile-out` on the real `wie_validate` binary: on → a folded-stack file appears, and the
//! JSON line is the same shape as without it; off → no file. Committed fixture only.

use std::process::Command;

fn run(extra: &[&std::ffi::OsStr]) -> String {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test_data/helloworld_ktf.zip");
    let out = Command::new(env!("CARGO_BIN_EXE_wie_validate"))
        .args(["--timeout", "2"])
        .args(extra)
        .arg(fixture)
        .env_remove("RUST_LOG")
        .output()
        .expect("wie_validate did not start");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn profile_out_writes_folded_stacks_only_when_asked_test() {
    let dir = std::env::temp_dir().join(format!("wie_validate_profile_out_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p.folded");

    let off = run(&[]);
    assert!(!path.exists());

    let on = run(&["--profile-out".as_ref(), path.as_os_str()]);
    let text = std::fs::read_to_string(&path).expect("--profile-out must create the file");
    std::fs::remove_dir_all(&dir).unwrap();

    // Same field count on the JSON line — the flag adds nothing to it.
    let keys = |l: &str| l.split(',').count();
    assert_eq!(off.lines().count(), 1, "{off}");
    assert_eq!(on.lines().count(), 1, "{on}");
    assert_eq!(keys(&off), keys(&on), "{off}\n{on}");

    // A KTF boot runs ARM code, so samples exist; each line is `0x..;0x.. <count>`.
    assert!(!text.is_empty(), "no samples from a KTF boot");
    for line in text.lines() {
        let (stack, count) = line.rsplit_once(' ').unwrap_or_else(|| panic!("{line:?}"));
        assert!(count.parse::<u64>().unwrap() >= 1, "{line:?}");
        assert!(
            stack.split(';').all(|f| f.starts_with("0x") && u32::from_str_radix(&f[2..], 16).is_ok()),
            "{line:?}"
        );
    }
}
