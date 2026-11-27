//! `WIE_AOT` on the real `wie_validate` binary: unset → the ARM guest is compiled (a cache record appears),
//! `off` → nothing is compiled (no cache directory), anything else → exit 2 and no result line.
//! The cache record is the observable: the JSON line does not say which engine ran. Committed fixture only.

use std::process::Command;

fn run(aot: Option<&str>, cache: &std::path::Path) -> (Option<i32>, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wie_validate"));
    command
        .args(["--timeout", "20"]) // the fixture exits by itself; the budget only has to outlast a cold compile on a busy host
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test_data/helloworld_ktf.zip"))
        .env_remove("RUST_LOG")
        .env_remove("WIE_AOT")
        .env("WIE_AOT_CACHE_DIR", cache);
    if let Some(aot) = aot {
        command.env("WIE_AOT", aot);
    }
    let out = command.output().expect("wie_validate did not start");
    (out.status.code(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn aot_is_on_unless_wie_aot_is_off_and_a_mistyped_switch_is_refused_test() {
    let dir = std::env::temp_dir().join(format!("wie_validate_aot_switch_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let records = |cache: &std::path::Path| std::fs::read_dir(cache).map(|entries| entries.count()).unwrap_or(0);

    let (on, off, native, mistyped) = (dir.join("on"), dir.join("off"), dir.join("native"), dir.join("mistyped"));
    let default_run = run(None, &on);
    let off_run = run(Some("off"), &off);
    let native_run = run(Some("native"), &native);
    let mistyped_run = run(Some("OFF"), &mistyped);
    let counts = [records(&on), records(&off), records(&native), records(&mistyped)];
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(counts, [1, 0, 1, 0], "records written: default, off, native, mistyped");
    for (code, line) in [&default_run, &off_run, &native_run] {
        assert_eq!(*code, Some(0), "{line}");
        assert!(line.contains(r#""result":"PASS""#) && line.lines().count() == 1, "{line}");
    }
    assert_eq!(mistyped_run, (Some(2), String::new()), "a mistyped switch must not run either engine");
}
