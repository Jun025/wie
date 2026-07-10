# wie 현재 상태 (02_status)

> **기준일 2026-07-10** · 레포 실측 기반(워크플로 실행 로그·커밋·베이스라인). 이 파일이 KB 의 최신 현황이다 — 매 작업 세션(main 반영) 시 재생성.

## 자동발행 파이프라인 (연방 ① 발신부) — 빌드·릴리스 라이브, dispatch 만 권한 이슈

`.github/workflows/publish-artifact.yml` (2026-07-08 라이브):

- main push(엔진 소스 경로: `**/*.rs`·`Cargo.toml`·`Cargo.lock`·`build-wasm.sh`) → **fresh** `wie_web` WASM 빌드(wasm-bindgen 0.2.108 핀 + wasm-opt) → GitHub Release `engine-<shortsha>` 에 `wie_web_bg.wasm` + `wie_web.js` 발행(sha256 메타 포함, 같은 커밋 재실행 멱등).
- 최신 릴리스: **`engine-1483146`**(2026-07-08, 무인증 curl + 해시 일치 실증).
- 이어서 otterpebble 에 `repository_dispatch`(event `wie-artifact-published`, payload: version·wieHead·wasmUrl/glueUrl·sha256·confirmedPlatforms `["KTF","SKT"]`) → 리시버(otterpebble 소유 `wie-artifact-receive.yml`)가 featurephone 재배포.
- **잔여(사람 1스텝)**: `OTTERPEBBLE_DISPATCH_TOKEN` 시크릿은 주입돼 있으나(2026-07-08) dispatch 단계가 **HTTP 403**("Resource not accessible by personal access token") — PAT 에 Jun025/otterpebble **Contents: read/write 권한(레포 접근 허가 포함)이 부족**. fine-grained PAT 재발급·재주입 시 즉시 활성(워크플로 수정 불요). 빌드·릴리스는 dispatch 실패와 무관하게 정상.

## 엔진 웹 계약 (소비자 featurephone 이 의존 — 변경 = 기획 사안)

원본: `wie_web/src/lib.rs`(wasm-bindgen 표면). 2026-07 현행:

- **생성자**: `new WieEmulator(filename, data: Uint8Array, canvas, audioCtx?, gain?, width, height)` — 7인자, 오디오 2개는 옵션(무음 시 undefined).
- **렌더**: 엔진이 **캔버스에 직접 blit**(더블버퍼 putImageData→drawImage, CSS pixelated 스케일). 소비자는 `requestAnimationFrame` 에서 `tick()` 만 호출.
- **입력**: `key_down/key_up/key_repeat(code)` — 키코드 문자열: `UP DOWN LEFT RIGHT OK LEFT_SOFT_KEY RIGHT_SOFT_KEY CLEAR CALL HANGUP VOLUME_UP VOLUME_DOWN NUM0`~`NUM9 HASH STAR`(미지 코드는 무시).
- **종료**: `has_exited` **없음(폐지)**. 정상 종료 = 콘솔 로그(`[wie] emulator requested exit`)뿐, 런타임 실패 = `tick()` 이 예외를 던짐(소비자는 예외로 감지).
- **오디오**: JS 가 사용자 제스처에서 만든 `AudioContext`+`GainNode` 를 주입, PCM 은 WebAudio 로 갭리스 스케줄, MIDI 는 무음 스텁.
- **세이브**: `has_saves()` / `export_saves()`(불투명 `WIESAV01` 블롭: RMS+FS) / `import_saves(blob)` / `export_fs`·`import_fs`. 해제는 `free()`.
- **로더**: `.zip`→KTF→LGT→SKT 순 판별, `.jar`→KTF→LGT→SKT→J2ME 폴백, `.jad` 는 거부(.jar 요구). `platform_kind()` 가 `"KTF"|"LGT"|"SKT"|"J2ME"` 반환.

## CI 현황

- **Rust CI**(`rust.yml`): 3-OS 매트릭스, `fmt --check` + `clippy -D warnings`(wasm32 타깃 포함) + 전체 테스트 — **green**(clippy red 해소됨).
- **coverage**(`coverage.yml`): `CODECOV_TOKEN` 없으면 업로드만 skip(fork-safe) — green.
- **Web**(`web.yml`): wasm 빌드 + Cloudflare Pages 배포, Pages 프로젝트(`wie-web`) 부재 시 자가 재생성(`pages project create || true`).
- **Security audit**(`rust-audit.yaml`, 매일 + `workflow_dispatch`): **green**(2026-07-10 해소) — `crossbeam-epoch` 0.9.18→**0.9.20**(RUSTSEC-2026-0204) · `anyhow` 1.0.102→**1.0.103**(unsound) · `memmap2` 0.9.10→**0.9.11**(unsound) 은 lockfile 패치 상향으로 제거(호출부 무수정 — 전부 semver-호환 패치 라인). 워크플로에 `permissions: issues/checks: write` 추가로 이슈 생성 권한 복구. **PENDING 2건**: ① `quick-xml` 0.39.2(RUSTSEC-2026-0194·0195, XML DoS) — 패치는 0.41.0에만 존재하나 유일 소비자 `wayland-scanner`(최신 0.31.10)가 `^0.39` 요구로 상위 차단. 빌드타임 proc-macro 가 vendored 신뢰 XML 만 파싱해 공격면 없음 — 근거 명시 후 workflow `ignore` 처리, wayland-scanner 가 ≥0.41 채택 시 ignore 제거. ② `ttf-parser` 0.25.1 unmaintained(RUSTSEC-2026-0192, patched 버전 없음) — 대안 skrifa 이행은 `ab_glyph`(최신 0.2.32도 ttf-parser 의존) 교체가 선행돼야 해 보류. 경고는 audit 비게이팅.

## 두 트랙 — 엔진 정상화 현황

**트랙 ① 타이틀 회수(모드 A — 자율주행 진행 중)**
- **회귀 베이스라인**(`scripts/smoke_gate_baseline.tsv`): **261 타이틀 부팅+렌더 PASS — KTF 190 / LGT 52 / SKT 19**(2-run 교집합 검증, 게임파일 미포함·식별자만). 게이트는 부팅+렌더만 판정(입력 생존은 비게이팅 어드바이저리). 구 스냅샷(2026-07-02) 202 대비 **+59**.
- 최근 리듬(git log): WIPI-Java/MIDP 메서드 보강 · RustJava 포크 핀 상승(트랙2 클러스터 다수 귀속: readUnsignedByte·TimeZone·Byte 등) · 결정적 실행기(BTreeMap 폴링·스레드 스케줄링 = 구 트랙1 반영)로 232→261.
- dispatch 의 `confirmedPlatforms` 는 **KTF·SKT** — LGT 는 clet 52종이 부팅+렌더하나 아직 "확정" 승격 전. J2ME 는 웹 로더 폴백 지원.

**트랙 ② §7 벽 — LGT AOT-Java 렌더(모드 B — 외부 산출물 대기)**
- LGT AOT-Java 24종은 렌더 0 유지. 바이너리-측 조사는 cp59 로 완결: per-frame 구동은 TIMER_EVENT(21) 모델로 확정(구현 가능), 유일 블로커는 **0x64 ordinal→native 등록표**. 오프라인 획득 소진 증명(AromaWIPI 비공번호 — `docs/reference/lgt_0x64_ordinal_table.md`) → **실기 트레이스 필요**. 도착 시 4단계 즉시 활성화 스캐폴드 커밋됨(기본 비활성·회귀 0). 요약: `10_deep-assets.md`, 원문: `docs/lgt_abi.md` §7·§8.

## 로드맵 위치 · 잔여

1. **dispatch PAT 권한 수정**(사람 1스텝, 위) → 자동 전파 완전 라이브.
2. ~~security audit red 해소~~ — **완료(2026-07-10)**. 잔여 PENDING(quick-xml 상위 차단·ttf-parser unmaintained)은 CI 현황 참조. ※ 이번 세션은 game_lab 코퍼스 부재(BYOF)로 smoke_gate 미실행 — 의존성 패치 상향 3건뿐이라 전체 테스트 + 3-OS CI 로 회귀 검증, 코퍼스 복귀 시 261 베이스라인 1회 재확인 권장.
3. 트랙 ① 지속(261+) · LGT clet 확정 승격 · 플레이키 타이틀(입력 타이밍) 분류.
4. 트랙 ② 는 실기 트레이스 확보(사람/외부) 전까지 동결 — 재조사 금지 목록 준수(`10_deep-assets.md` 가드레일).
