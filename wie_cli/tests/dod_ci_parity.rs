//! Runs the DoD ↔ `rust.yml` parity lock, and proves the lock is not vacuous.
//!
//! Wiring: this is a plain `#[test]`, so `cargo test --all` — itself one of the four documented
//! gates — runs it on every PR, in all six `rust.yml` legs, with **zero workflow change**. The
//! checker lives in a separate file that this one declares as a module, so deleting the checker
//! is a compile error rather than a silent pass. (Deleting *both* files is still green; that
//! ceiling is printed by `ceilings()` on every run rather than hidden here.)

#[path = "support/dod_ci_parity.rs"]
mod checker;

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("wie_cli has a parent")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} 를 읽지 못했다: {e}", p.display()))
}

fn pin_files_present() -> Vec<&'static str> {
    checker::TOOLCHAIN_PIN_FILES
        .iter()
        .copied()
        .filter(|f| repo_root().join(f).exists())
        .collect()
}

/// The lock itself. Green means: every cargo command CI runs is listed in the marked DoD region
/// and vice versa, and the toolchain sets match.
#[test]
fn dod_region_and_rust_yml_agree() {
    let report = checker::parity(&read("AGENTS.md"), &read(".github/workflows/rust.yml"), &pin_files_present());
    let rendered = report.render();
    println!("{rendered}");
    let problems = report.problems();
    assert!(
        problems.is_empty(),
        "DoD ↔ rust.yml 파리티가 깨졌다: {}\n{rendered}",
        problems.join(" · ")
    );
}

// ── 개악 대조 — 「어긋나게 만들면 red」를 실제로 돌려서 보인다 ──────────────────────────
// 항진명제 함정(「문자열이 들어 있는가」로 짜면 검사기 자신이 세어져 늘 green)을 피하려면,
// 통과 사례만이 아니라 «실패해야 하는 사례»가 실제로 실패하는지를 돌려 봐야 한다.

fn mutated(from: &str, to: &str) -> Vec<String> {
    let yml = read(".github/workflows/rust.yml");
    let agents = read("AGENTS.md");
    let (y, a) = (yml.replace(from, to), agents.replace(from, to));
    assert!(
        y != yml || a != agents,
        "개악 대조가 «아무것도 바꾸지 못했다» — 앵커 `{from}` 가 표류했다"
    );
    checker::parity(&a, &y, &[]).problems()
}

#[test]
fn m1_ci_gains_a_gate_the_dod_does_not_list() {
    // rust.yml 에만 게이트를 하나 더한다 → 축 A 가 잡아야 한다.
    let yml = read(".github/workflows/rust.yml").replace(
        "      - run: cargo fmt --all -- --check",
        "      - run: cargo fmt --all -- --check\n      - run: cargo deny check",
    );
    let p = checker::parity(&read("AGENTS.md"), &yml, &[]);
    assert!(
        p.problems().iter().any(|x| x.contains("축 A")),
        "CI 에만 있는 게이트를 못 잡았다\n{}",
        p.render()
    );
    assert!(p.ci_cmds.contains("cargo deny check"));
}

#[test]
fn m2_dod_loses_a_gate_ci_still_runs() {
    // DoD 마커 구간에서 wasm clippy 줄을 지운다 → 축 A 가 잡아야 한다.
    let agents = read("AGENTS.md").replace(
        "cargo clippy --target wasm32-unknown-unknown -- -D warnings   # rust.yml: wasm lint gate\n",
        "",
    );
    assert_ne!(agents, read("AGENTS.md"), "개악 대조가 아무것도 바꾸지 못했다 — 앵커 표류");
    let p = checker::parity(&agents, &read(".github/workflows/rust.yml"), &[]);
    assert!(
        p.problems().iter().any(|x| x.contains("축 A")),
        "DoD 에서 지운 게이트를 못 잡았다\n{}",
        p.render()
    );
}

#[test]
fn m3_matrix_gains_a_toolchain_the_dod_does_not_name() {
    // 축 B — 매트릭스에 nightly 를 더한다.
    let p = mutated("rust: [stable, beta]", "rust: [stable, beta, nightly]");
    assert!(p.iter().any(|x| x.contains("축 B")), "매트릭스에만 있는 toolchain 을 못 잡았다: {p:?}");
}

#[test]
fn m4_dod_names_a_toolchain_ci_does_not_run() {
    // 축 B 의 반대 방향 — DoD 만 nightly 를 시킨다.
    let agents = read("AGENTS.md").replace("cargo +beta clippy --all", "cargo +nightly clippy --all");
    assert_ne!(agents, read("AGENTS.md"), "개악 대조가 아무것도 바꾸지 못했다 — 앵커 표류");
    let p = checker::parity(&agents, &read(".github/workflows/rust.yml"), &[]);
    assert!(
        p.problems().iter().any(|x| x.contains("축 B")),
        "DoD 에만 있는 toolchain 을 못 잡았다\n{}",
        p.render()
    );
}

#[test]
fn m5_markers_removed_is_fatal_not_green() {
    // 마커를 지우면 «정본 소실» 이다 — 조용한 통과가 되면 안 된다.
    let agents = read("AGENTS.md").replace(checker::BEGIN, "COMMIT-GATES-GONE");
    let p = checker::parity(&agents, &read(".github/workflows/rust.yml"), &[]);
    assert!(
        p.problems().iter().any(|x| x.contains("마커가 없다")),
        "마커 소실을 못 잡았다\n{}",
        p.render()
    );
}

#[test]
fn m6_toolchain_pin_file_breaks_the_default_mapping() {
    // rust-toolchain 이 생기면 맨 `cargo` 가 더는 stable 이 아니다 ⇒ 축 B 매핑이 깨진다.
    let p = checker::parity(&read("AGENTS.md"), &read(".github/workflows/rust.yml"), &["rust-toolchain.toml"]);
    assert!(
        p.problems().iter().any(|x| x.contains("toolchain 고정")),
        "핀 파일 출현을 못 잡았다\n{}",
        p.render()
    );
}

// ── 파서 자신의 가정 — wie 형식에 맞추며 «바꾼» 것들이 실제로 통하는지 ────────────────

#[test]
fn env_prefixed_block_scalar_flattens_to_one_command() {
    // 원본은 블록 스칼라를 `<셸 블록> …` 로 접는다. wie 는 그 형식으로 «진짜 게이트»를 돌리므로
    // env 대입을 평탄화한다 — 그러지 않으면 `RUST_MIN_STACK=… cargo test --all` 이 거짓 red 가 된다.
    assert_eq!(
        checker::flatten_shell("export RUST_MIN_STACK=4194304\ncargo test --all\n"),
        vec!["RUST_MIN_STACK=4194304 cargo test --all"]
    );
    assert_eq!(
        checker::flatten_shell("$env:RUST_MIN_STACK=4194304\ncargo test --all\n"),
        vec!["RUST_MIN_STACK=4194304 cargo test --all"]
    );
}

#[test]
fn os_conditional_gates_are_not_dropped() {
    // 원본은 `if:` 가 붙은 step 을 축 A 에서 제외한다. wie 의 테스트 게이트는 OS 조건부라
    // 그 규칙을 그대로 쓰면 «게이트가 사라진다». 판별자를 «cargo 를 부르는가»로 바꾼 근거다.
    let (cargo, setup, _) = checker::parse_ci(&read(".github/workflows/rust.yml"));
    assert!(
        cargo.iter().any(|c| c.contains("cargo test --all")),
        "OS 조건부 테스트 게이트가 축 A 에서 빠졌다: {cargo:?}"
    );
    assert!(
        setup.iter().any(|s| s.contains("apt-get")),
        "셋업 step 이 «제외»로 분류되지 않았다: {setup:?}"
    );
    assert!(
        !cargo.iter().any(|c| c.contains("choco")),
        "셋업 step 이 게이트로 잘못 분류됐다: {cargo:?}"
    );
}

#[test]
fn dod_region_has_both_fenced_blocks() {
    // 원본은 «첫» 코드블록만 읽는다. wie 의 마커 구간은 둘이고, 첫 것만 읽으면 beta 가 사라진다.
    let lines = checker::parse_dod(&read("AGENTS.md")).expect("마커 구간 파싱");
    assert!(lines.iter().any(|l| l.starts_with("cargo fmt")), "첫 블록을 못 읽었다: {lines:?}");
    assert!(lines.iter().any(|l| l.contains("+beta")), "둘째 블록(beta)을 못 읽었다: {lines:?}");
}
