## [2026-10-23] Security audit 6일 연속 red — 자문 13건 처분: 10건 상향 · zip 3건 이름 억제 (wie-rust-audit-main-red-since-1018-advisory-batch)

**무엇을**: `rust-audit.yaml` 이 10-18 ~ 10-23 매일 failure 였다. 무-플래그 `cargo audit`(cargo-audit 0.22.2 · advisory-db 1,350건 · `Cargo.lock` 557 패키지)이 낸 **취약점 13건**을 전수 처분했다.

| ID | 크레이트 | 현재 → 수정 판 | 우리 도달 경로 | 처분 |
|---|---|---|---|---|
| RUSTSEC-2026-0338 · 0340 · 0343 | quinn-proto | 0.11.17 → **0.11.19** | 빌드 그래프 0(lockfile 에만 — `cargo tree -i … -e all --target all` 빈 답) | `cargo update -p` |
| RUSTSEC-2026-0346 · 0347 · 0348 · 0349 | rustls | 0.23.45 → **0.23.46** | 네이티브만(`sentry → ureq`) · WASM 0 · TLS 경로는 안 쟀다 | `cargo update -p` |
| RUSTSEC-2026-0382 | rustls-webpki | 0.103.15 → **0.103.16** | 네이티브만 · WASM 0 | `cargo update -p` |
| RUSTSEC-2026-0381 | webpki-roots | 1.0.9 → **1.0.11** | 네이티브만 · WASM 0 | `cargo update -p` |
| RUSTSEC-2026-0383 | webpki-root-certs | 1.0.9 → **1.0.11** | 빌드 그래프 0 | `cargo update -p` |
| RUSTSEC-2026-0367 · 0368 · 0369 | zip | 8.6.0 → (9.0.0 에만 패치) | ★**없음** — `extract`/`ZipStreamReader` 호출 0(우리·rustjava-runtime 둘 다 메모리 읽기만) | ★**이름 억제**(`--ignore` ×3) |

경고 2건(ttf-parser RUSTSEC-2026-0192 unmaintained · chacha20 0.10.1 yanked)은 이미 기대 목록·대장 §A 에 있어 손대지 않았다.

**왜 zip 만 억제인가**: 패치가 zip 9 에만 있는데 `rustjava-runtime` 이 0.1.1 ~ 0.3.2 내내 `^8.6`, `^9.0.5` 는 0.3.3 부터다(crates.io 인덱스 실측). wie-backend 만 9 로 올리면 rustjava 쪽 8.6.0 이 남아 자문이 그대로다 ⇒ RustJava 핀 이동(Constraint 8 · 설계 사안) 없이는 못 닫는다. 도달성 논증과 해제 조건은 `docs/project-kb/02_status.md` **§D**(신설)에 있다.

**억제가 조용히 낡지 않게**: `scripts/check-audit-warnings.mjs` 에 IGNORES 절을 더했다 — ⑴두 워크플로(`rust-audit.yaml`·`pr-audit.yml`)의 `--ignore` 목록이 같은가 ⑵억제한 ID 를 무-억제 audit 이 아직 내는가 ⑶ID 마다 §D 행이 있는가, 셋 다 red. 음성 대조 실측: pr-audit 에서 하나를 빼면 ⑴ red, 없는 ID 로 바꾸면 ⑵⑶ red(rc=1). 기존 기대 목록과 같은 비대칭이다 — 해제 경로는 «줄을 지우는 것» 뿐이다.

**사용자 영향**: 없다. 배포 WASM 에 들어가는 크레이트는 zip 하나뿐이고 판본은 그대로다. 바뀐 4종은 네이티브 sentry 경로에서만 쓰거나 아예 컴파일되지 않는다.

**실측**:
- `Cargo.lock` **20줄**(10+/10−, 5 패키지 × 판본+체크섬) · `Cargo.toml` 손대지 않음.
- `cargo audit --ignore RUSTSEC-2026-0367 --ignore RUSTSEC-2026-0368 --ignore RUSTSEC-2026-0369` **rc=0**(`2 allowed warnings`) · 무-플래그 rc=1(zip 3건 — 의도된 것) · `node scripts/check-audit-warnings.mjs` rc=0 · `check-ledger-c-validity-scope.mjs` OK(C-5~C-9 추가).
- 4 게이트 + beta clippy green · `cargo test --all` 54 스위트 812 통과 0 실패 · 러너 블록 6줄 전부 PASS rc=0 · `npm run audit` PASS.
