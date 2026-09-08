//! Lock: the documented commit gates must reproduce what `rust.yml` actually runs.
//!
//! Ported from the sibling `RustJava` repo's `scripts/check-dod-ci-parity.py`, which exists
//! because hand comparison failed there five rounds in a row. `AGENTS.md` names that script and
//! says in prose "Nothing like that is ported here"; this file is that port. What it is NOT is a
//! copy — wie's two files have a different shape, and every assumption that had to change is
//! listed in `docs/worklog/2026-09-05-dod-ci-parity-checker.json`.
//!
//! What "parity" means here, stated so it cannot be assumed:
//!   axis A — the *cargo invocation set*. Every cargo command `rust.yml` runs on a PR must appear
//!            in the marked DoD region, and vice versa. Toolchain prefixes are stripped first
//!            (they are axis B), and env assignments are kept (`RUST_MIN_STACK=…` is load-bearing).
//!   axis B — the *toolchain set*. `strategy.matrix.rust` must equal the toolchains the DoD names
//!            (a bare `cargo` means the default toolchain; `cargo +beta` names `beta`).
//!
//! The two axes are compared INDEPENDENTLY, not as a cross product. That is a deliberate ceiling
//! inherited from the original: CI runs all four gates under both toolchains while the DoD only
//! doubles up clippy, and RustJava's 2026-09-04 decision round measured that gap and declined to
//! widen it. Re-argue it with numbers, not taste.
//!
//! Everything this does NOT see is listed in `ceilings()` — read it before trusting a green.

use std::collections::BTreeSet;

pub const BEGIN: &str = "COMMIT-GATES:BEGIN";
pub const END: &str = "COMMIT-GATES:END";

/// Files whose presence would make a bare `cargo` mean something other than `DEFAULT_TOOLCHAIN`.
pub const TOOLCHAIN_PIN_FILES: [&str; 2] = ["rust-toolchain.toml", "rust-toolchain"];
pub const DEFAULT_TOOLCHAIN: &str = "stable";

#[derive(Debug)]
pub struct Report {
    pub ci_cmds: BTreeSet<String>,
    pub dod_cmds: BTreeSet<String>,
    pub ci_toolchains: BTreeSet<String>,
    pub dod_toolchains: BTreeSet<String>,
    /// Non-cargo `run:` steps — setup, not gates. Excluded from axis A, never silently dropped.
    pub ci_setup: Vec<String>,
    /// Non-cargo DoD lines (`rustup toolchain install …`). Same rule, other side.
    pub dod_non_cargo: Vec<String>,
    pub pinned: Vec<String>,
    /// Hard failures that make comparison meaningless (missing markers, empty region).
    pub fatal: Vec<String>,
}

impl Report {
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        p.extend(self.fatal.iter().cloned());
        if self.ci_cmds != self.dod_cmds {
            p.push("축 A(명령)".into());
        }
        if self.ci_toolchains != self.dod_toolchains {
            p.push("축 B(toolchain)".into());
        }
        if !self.pinned.is_empty() {
            p.push(format!("toolchain 고정 파일 {:?}", self.pinned));
        }
        p
    }

    pub fn render(&self) -> String {
        let mut o = String::new();
        o.push_str("DOD-CI-PARITY  AGENTS.md COMMIT-GATES 구간  ↔  .github/workflows/rust.yml\n");
        for f in &self.fatal {
            o.push_str(&format!("  ★FATAL {f}\n"));
        }

        o.push_str(&format!(
            "\n  [축 A · 명령]  CI {}개 · DoD {}개\n",
            self.ci_cmds.len(),
            self.dod_cmds.len()
        ));
        for c in self.ci_cmds.union(&self.dod_cmds) {
            let (in_ci, in_dod) = (self.ci_cmds.contains(c), self.dod_cmds.contains(c));
            let mark = if in_ci && in_dod { "  " } else { "★!" };
            o.push_str(&format!(
                "    {mark} {c}   (CI={} DoD={})\n",
                if in_ci { "y" } else { "n" },
                if in_dod { "y" } else { "n" }
            ));
        }
        for c in self.ci_cmds.difference(&self.dod_cmds) {
            o.push_str(&format!("    ★ CI 에만 있다 — DoD 마커 구간에 이 줄을 넣어라: {c}\n"));
        }
        for c in self.dod_cmds.difference(&self.ci_cmds) {
            o.push_str(&format!("    ★ DoD 에만 있다 — CI 가 안 치는 것을 DoD 가 시킨다: {c}\n"));
        }

        o.push_str(&format!(
            "\n  [축 B · toolchain]  CI {:?} · DoD {:?}\n",
            self.ci_toolchains, self.dod_toolchains
        ));
        for t in self.ci_toolchains.difference(&self.dod_toolchains) {
            o.push_str(&format!("    ★ CI 에만 있다 — DoD 에 `cargo +{t} …` 줄이 없다\n"));
        }
        for t in self.dod_toolchains.difference(&self.ci_toolchains) {
            o.push_str(&format!("    ★ DoD 에만 있다 — CI 가 안 도는 toolchain 이다: {t}\n"));
        }

        // Never silent about what was excluded — that is how a checker quietly stops checking.
        o.push_str(&format!("\n  [제외] CI 셋업 step {}건 (cargo 를 부르지 않는다)\n", self.ci_setup.len()));
        for s in &self.ci_setup {
            o.push_str(&format!("    - {s}\n"));
        }
        o.push_str(&format!("  [제외] DoD 비-cargo 줄 {}건\n", self.dod_non_cargo.len()));
        for s in &self.dod_non_cargo {
            o.push_str(&format!("    - {s}\n"));
        }
        for p in &self.pinned {
            o.push_str(&format!("\n  ★ {p} 이 생겼다 — 맨 `cargo` 가 더는 «{DEFAULT_TOOLCHAIN}» 이 아니다.\n"));
            o.push_str("    이 검사기의 축 B 매핑이 깨졌으니 split_toolchain() 을 함께 고쳐라.\n");
        }

        let problems = self.problems();
        if problems.is_empty() {
            o.push_str(&format!(
                "\nOK 두 축 모두 대칭차 0 — 명령 {}개 · toolchain {}개로 «둘 다 일치»\n",
                self.ci_cmds.len(),
                self.ci_toolchains.len()
            ));
        } else {
            o.push_str(&format!("\nFAIL 대칭차 있음: {}\n", problems.join(" · ")));
        }
        o.push_str(&ceilings());
        o
    }
}

/// What this lock does NOT see. Printed on every run, pass or fail — a ceiling that only appears
/// in a doc is a ceiling nobody reads.
///
/// ★**THIS LIST IS THE SINGLE SOURCE OF TRUTH.** Any ceiling list elsewhere (worklogs, reports) is a
/// dated snapshot, not an authority — if they disagree, this one wins. It lives here because this is
/// the list the check actually prints, so a reader of a green run and a reader of the list see the
/// same thing; a copy that nothing prints is the copy that goes stale (measured 2026-09-08: the
/// worklog copy carried 9 items while this carried 7).
pub fn ceilings() -> String {
    "\n  [천장 — 이 검사가 «못 보는» 것 · ★이 목록이 «정본»이다(다른 곳의 목록은 그 시점 «사본»)]\n\
     \x20   1. OS 축(macos/ubuntu/windows) — 로컬 재현 불가. CI 가 유일한 그물이다.\n\
     \x20   2. 교차곱(4게이트 × 2toolchain) — 두 축을 «독립»으로만 본다(원본 판정 승계).\n\
     \x20   3. `rust.yml` 밖의 워크플로 — ★«못 보는 것»이 아니라 ★**«보지 않기로 «정한» 것»**이다.\n\
     \x20      ★2026-09-06 «확장하지 않기로» 결정했다(제안 2026-09-05-dod-ci-parity-checker#p1). 이유는\n\
     \x20      취향이 아니라 «술어가 없다»는 것이다: 네 게이트는 rust.yml 과 «양방향 포함 = 상등»이\n\
     \x20      성립해 집합 비교가 서지만, 게이트 밖은 어느 방향도 서지 않는다 — 실측(건강한 트리):\n\
     \x20      ★as-of 2026-09-06 트리: 문서 8 · CI run: 23 · 교집합 «2» · 차집합 «27»(문서−CI 6 · CI−문서 21)\n\
     \x20      · 그중 참 결함 «0». ★이 다섯 수는 «그때의 값»이다 — 워크플로가 바뀌면 달라지고, 다시 재려면\n\
     \x20      기각된 갈래 A 시제품을 다시 만들어야 한다(일부러 커밋하지 않았다 — 상주하면 켜고 싶어진다).\n\
     \x20      ⇒ ★**수를 «현재»로 인용하지 마라. 결정은 수가 아니라 «술어가 없다»에 서 있다.**\n\
     \x20      ★배제 대상 전수(as-of 2026-09-08 · 워크플로 8개 중 rust.yml 을 뺀 «7개»):\n\
     \x20        · 판정 대상이었고 «넣지 않기로» 한 것 3 — engine-contract.yml · web.yml · coverage.yml\n\
     \x20          (PR/push 에 도는 검사라 후보였다. 위 차집합 27 이 이 셋을 두고 나온 수다.)\n\
     \x20        · 애초에 «후보가 아닌» 것 4 — publish-artifact.yml(릴리스·하류 전파) ·\n\
     \x20          rust-audit.yaml(schedule 전용 — «커밋 전 게이트»가 아니다) ·\n\
     \x20          dependabot.yaml(PR 자동 승인/자동 머지) · opencode.yml(issue_comment 봇).\n\
     \x20          ⇒ ★이 넷은 «커밋 전에 사람이 돌리는 명령»을 갖지 않으므로 상등 술어의 좌변이 비어 있다.\n\
     \x20      문서−CI 6 은 전부 정상이다(CI 가 안 도는 wie_validate·audit·verify, 이름만 다른\n\
     \x20      npm run build:wasm ↔ bash scripts/build-wasm.sh, 1:N 인 npm run frontend ↔ npm ci+npm run build,\n\
     \x20      메모리로 만들어 쓰는 make-draw-fixture). CI−문서 21 은 apt/choco·gh release·tarpaulin 등\n\
     \x20      «커밋 전 명령이 아닌 것»이다. ⇒ 켜면 첫날 red 27 · 참 0 이고, 그 예외 목록이 곧 검사를\n\
     \x20      무력화한다. 상세·갈래별 비용 = docs/worklog/2026-09-06-parity-outside-gate-decision.json.\n\
     \x20   4. 비-cargo 게이트 — `node …` 같은 게이트가 rust.yml 에 생기면 «셋업»으로 분류돼 안 보인다.\n\
     \x20   5. 동의어·플래그 순서 — 정규화가 «공백 접기 + 꼬리 주석 제거»뿐이라 `--all`↔`--workspace` 는 «다르다»로 본다.\n\
     \x20   6. YAML 손파서 — 앵커·다중문서·인용 스칼라 미지원(원본과 같은 선택: 의존성 0).\n\
     \x20   7. ★이 검사 «자신»의 삭제 — `tests/dod_ci_parity.rs` 와 이 파일을 «둘 다» 지우면 green 이다.\n\
     \x20      (한쪽만 지우면 컴파일이 깨져 red 다 — 그것이 이 2파일 구성의 이유다.)\n\
     \x20   8. 순서·중복 — 집합 비교라 스텝 «순서»와 «중복 실행»은 안 본다.\n\
     \x20   9. 줄끝(CRLF) — 정규화는 `tests/dod_ci_parity.rs` 의 `read()` «한 곳»에서 한다. 그것을 지우면\n\
     \x20      «로컬은 green 이고 windows 다리만 red» 다(실측) — 로컬 그물이 없다.\n"
        .into()
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip a leading shell comment (`cargo fmt … # rust.yml: formatting gate`).
fn strip_comment(s: &str) -> &str {
    match s.find(" #") {
        Some(i) => &s[..i],
        None => s,
    }
}

/// `cargo +beta clippy …` -> (`beta`, `cargo clippy …`); bare cargo -> (default, cmd).
/// A non-cargo line yields `None` and is not part of either axis.
pub fn split_toolchain(cmd: &str) -> (Option<String>, String) {
    let c = norm(cmd);
    if let Some(rest) = c.strip_prefix("cargo +") {
        if let Some((tc, tail)) = rest.split_once(' ') {
            return (Some(tc.to_string()), norm(&format!("cargo {tail}")));
        }
    }
    // Env-prefixed forms (`RUST_MIN_STACK=4194304 cargo test --all`) count as the default
    // toolchain: the env assignment is part of the command, not a toolchain selector.
    if c.starts_with("cargo ") || c.split_whitespace().any(|w| w == "cargo") {
        return (Some(DEFAULT_TOOLCHAIN.to_string()), c);
    }
    (None, c)
}

fn is_cargo(cmd: &str) -> bool {
    cmd.split_whitespace().any(|w| w == "cargo")
}

/// Flatten a shell snippet into one command line: `export A=B` / `$env:A=B` become an env prefix
/// on the command that follows. wie runs a *real gate* through that form (the windows /
/// non-windows `RUST_MIN_STACK` test steps), so folding it into an opaque "<shell block>" — what
/// the original does — would drop the gate the DoD cares most about.
pub fn flatten_shell(body: &str) -> Vec<String> {
    let mut env: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let assign = line
            .strip_prefix("export ")
            .map(str::to_string)
            .or_else(|| line.strip_prefix("$env:").map(str::to_string));
        match assign {
            Some(a) if a.contains('=') && !a.contains(' ') => env.push(norm(&a)),
            _ => {
                let mut cmd = env.join(" ");
                if !cmd.is_empty() {
                    cmd.push(' ');
                }
                cmd.push_str(line);
                out.push(norm(&cmd));
                env.clear();
            }
        }
    }
    out
}

/// Pull `- run:` steps and `rust: [...]` out of the workflow. Hand-rolled on purpose: the original
/// is stdlib-only because CI installs nothing for it, and wie's `rust.yml` is 115 lines.
pub fn parse_ci(text: &str) -> (Vec<String>, Vec<String>, BTreeSet<String>) {
    let mut toolchains = BTreeSet::new();
    let mut cargo_cmds = Vec::new();
    let mut setup = Vec::new();

    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();

        if let Some(rest) = t.strip_prefix("rust:") {
            let rest = rest.trim();
            if let Some(inner) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
                toolchains.extend(inner.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
            }
            i += 1;
            continue;
        }

        // A `run:` key, with or without the `- ` list marker.
        let run_body = t.strip_prefix("- run:").or_else(|| t.strip_prefix("run:"));
        if let Some(v) = run_body {
            let v = v.trim();
            let cmds = if v == "|" || v == ">" || v == "|-" || v == ">-" {
                // Block scalar: take the more-indented lines that follow.
                let indent = line.len() - line.trim_start().len();
                let mut body = String::new();
                i += 1;
                while i < lines.len() {
                    let l = lines[i];
                    if l.trim().is_empty() {
                        body.push('\n');
                        i += 1;
                        continue;
                    }
                    if l.len() - l.trim_start().len() <= indent {
                        break;
                    }
                    body.push_str(l.trim());
                    body.push('\n');
                    i += 1;
                }
                flatten_shell(&body)
            } else {
                i += 1;
                vec![norm(v)]
            };
            for c in cmds {
                if is_cargo(&c) {
                    cargo_cmds.push(c);
                } else {
                    setup.push(c);
                }
            }
            continue;
        }
        i += 1;
    }
    (cargo_cmds, setup, toolchains)
}

/// Every fenced block inside the COMMIT-GATES marker region — NOT just the first one.
/// The original reads "the first fenced block in §Definition of Done"; wie's region holds two
/// (the four gates, then the beta lint gate), and reading only the first drops `beta` from axis B.
pub fn parse_dod(text: &str) -> Result<Vec<String>, String> {
    let b = text
        .find(BEGIN)
        .ok_or_else(|| format!("AGENTS.md 에 `{BEGIN}` 마커가 없다 — 정본이 사라졌다"))?;
    let e = text
        .find(END)
        .ok_or_else(|| format!("AGENTS.md 에 `{END}` 마커가 없다 — 정본이 사라졌다"))?;
    if e <= b {
        return Err("COMMIT-GATES 마커 순서가 뒤집혔다".into());
    }
    let region = &text[b..e];

    let mut out = Vec::new();
    let mut inside = false;
    for line in region.lines() {
        if line.trim_start().starts_with("```") {
            inside = !inside;
            continue;
        }
        if inside && !line.trim().is_empty() {
            out.push(norm(strip_comment(line.trim())));
        }
    }
    if out.is_empty() {
        return Err("COMMIT-GATES 구간에 코드블록이 없다 — 정본이 비었다".into());
    }
    Ok(out)
}

pub fn parity(agents_md: &str, rust_yml: &str, pin_files_present: &[&str]) -> Report {
    let (ci_cargo, ci_setup, ci_toolchains) = parse_ci(rust_yml);

    let mut fatal = Vec::new();
    let dod_lines = match parse_dod(agents_md) {
        Ok(v) => v,
        Err(e) => {
            fatal.push(e);
            Vec::new()
        }
    };
    if ci_toolchains.is_empty() {
        fatal.push("rust.yml 에서 `rust: [...]` 매트릭스를 못 찾았다 — 축 B 를 잴 수 없다".into());
    }
    if ci_cargo.is_empty() && fatal.is_empty() {
        fatal.push("rust.yml 에서 cargo 게이트를 하나도 못 찾았다 — 파서가 형식을 놓쳤다".into());
    }

    let mut ci_cmds = BTreeSet::new();
    for c in &ci_cargo {
        let (_, cmd) = split_toolchain(c);
        ci_cmds.insert(cmd);
    }

    let (mut dod_cmds, mut dod_toolchains, mut dod_non_cargo) = (BTreeSet::new(), BTreeSet::new(), Vec::new());
    for line in &dod_lines {
        if !is_cargo(line) {
            dod_non_cargo.push(line.clone());
            continue;
        }
        let (tc, cmd) = split_toolchain(line);
        dod_cmds.insert(cmd);
        if let Some(t) = tc {
            dod_toolchains.insert(t);
        }
    }

    Report {
        ci_cmds,
        dod_cmds,
        ci_toolchains,
        dod_toolchains,
        ci_setup,
        dod_non_cargo,
        pinned: pin_files_present.iter().map(|s| s.to_string()).collect(),
        fatal,
    }
}
