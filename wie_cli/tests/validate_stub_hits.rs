//! Runs the `wie_validate` binary and reads `stub_hits` / `svc_stub_slots` off its JSON line —
//! the place those fields are consumed. The unit tests in `wie_validate.rs` prove the layer
//! counts; only a run of the real binary proves `main` installs it, the runtimes' `warn!("stub …")`
//! and `make_svc_stub` trace lines reach it, and the numbers land on the line callers parse.
//!
//! Committed fixtures only (no game bytes). `helloworld_ktf.zip` hits the MIDP `Font` stubs on
//! boot and binds ~1,400 SVC stubs; `draw_j2me.jar` — extracted at test time from
//! `draw_j2me.zip`, since only the zip is committed — boots and renders a J2ME guest with no ARM
//! core; `draw_j2me.zip` itself fails to load, so nothing runs at all — the zero control.

use std::process::Command;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test_data").join(name)
}

fn run(fixture_name: &str) -> String {
    run_path(&fixture(fixture_name))
}

fn run_path(path: &std::path::Path) -> String {
    let fixture = path.display();
    let out = Command::new(env!("CARGO_BIN_EXE_wie_validate"))
        .args(["--timeout", "2"])
        .arg(path)
        .env_remove("RUST_LOG")
        .output()
        .expect("wie_validate did not start");
    let stdout = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "{fixture}: expected exactly one JSON line, got {stdout:?}");
    lines[0].to_string()
}

/// The `{…}` / `[…]`-balanced value after `"key":`.
fn object<'a>(line: &'a str, key: &str) -> &'a str {
    let start = line.find(&format!("\"{key}\":")).unwrap_or_else(|| panic!("no {key} in {line}")) + key.len() + 3;
    let mut depth = 0;
    for (i, c) in line[start..].char_indices() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return &line[start..=start + i];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced {key} in {line}");
}

fn number(object: &str, key: &str) -> u64 {
    let start = object.find(&format!("\"{key}\":")).unwrap_or_else(|| panic!("no {key} in {object}")) + key.len() + 3;
    object[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

#[test]
fn stub_hits_and_svc_stub_slots_are_on_the_cli_line_test() {
    let line = run("helloworld_ktf.zip");

    let stubs = object(&line, "stub_hits");
    assert!(number(stubs, "count") >= 1, "a KTF boot hits the Font stubs: {stubs}");
    assert!(number(stubs, "distinct") >= 1, "{stubs}");
    assert!(
        stubs.contains(r#"{"name":"javax.microedition.lcdui.Font::<init>","count":"#),
        "stub name missing from `first`: {stubs}"
    );

    let slots = object(&line, "svc_stub_slots");
    assert!(number(slots, "used") >= 1, "KTF binds SVC stubs: {slots}");
    assert_eq!(number(slots, "capacity"), 4096, "{slots}");
}

#[test]
fn svc_stub_slots_capacity_is_null_without_an_arm_core_test() {
    // `test_data/draw_j2me.jar` is not in the tree; running that path is a read error, i.e. the
    // same "nothing ran" axis as the zip test below. It LOOKS fine on a dev tree only because
    // `scripts/make-draw-fixture.mjs` leaves a git-ignored copy there — CI and a clean checkout
    // have none. Extract the committed jar instead so a J2ME guest runs everywhere.
    let archive = wie_backend::extract_zip(&std::fs::read(fixture("draw_j2me.zip")).unwrap()).unwrap();
    let dir = std::env::temp_dir().join(format!("wie_validate_stub_hits_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let jar = dir.join("draw_j2me.jar");
    std::fs::write(&jar, &archive["draw_j2me.jar"]).unwrap();
    let line = run_path(&jar);
    std::fs::remove_dir_all(&dir).unwrap();

    // "It ran", not "it PASSed": whether the first paint fits `--timeout 2` depends on host load
    // (measured FAIL/no-frame at loadavg ~100, PASS on an idle host). A JVM that reached the
    // guest hits the MIDP `Font` stubs, which a read error cannot.
    assert!(line.contains(r#""platform":"j2me""#), "{line}");
    assert!(!line.contains("Read error"), "the jar must load: {line}");
    assert!(
        number(object(&line, "stub_hits"), "count") >= 1,
        "the J2ME guest must actually run: {line}"
    );
    assert_eq!(object(&line, "svc_stub_slots"), r#"{"used":0,"capacity":null}"#);
}

#[test]
fn stub_hits_is_zero_when_nothing_ran_test() {
    let line = run("draw_j2me.zip");
    assert!(line.contains(r#""result":"FAIL""#), "{line}");
    assert_eq!(object(&line, "stub_hits"), r#"{"count":0,"distinct":0,"first":[]}"#);
    assert_eq!(object(&line, "svc_stub_slots"), r#"{"used":0,"capacity":null}"#);
}
