# wie 현재 상태 (02_status)

> **기준일 2026-07-09** · 레포 실측 기반(워크플로 실행 로그·커밋·베이스라인). 이 파일이 KB 의 최신 현황이다 — 매 작업 세션(main 반영) 시 재생성.

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
- **Security audit**(`rust-audit.yaml`, 매일): **red** — 취약 3건: `quick-xml` 0.39.2 (RUSTSEC-2026-0194·0195, XML DoS — ≥0.41.0 필요) · `crossbeam-epoch` 0.9.18 (RUSTSEC-2026-0204 — ≥0.9.20). 경고: `ttf-parser` unmaintained · `anyhow` <1.0.103 unsound · `memmap2` <0.9.11 unsound. 부가로 audit 액션의 이슈 생성 권한 부족(workflow permissions) — 알림 경로도 손봐야 함.

## 지원 플랫폼 실상

- **회귀 베이스라인**(`scripts/smoke_gate_baseline.tsv`): **261 타이틀 부팅+렌더 PASS — KTF 190 / LGT 52 / SKT 19**(2회 독립 전수 실행 검증, 게임파일 미포함·식별자만). 게이트는 부팅+렌더만 판정(입력 생존은 비게이팅 어드바이저리).
- dispatch 의 `confirmedPlatforms` 는 **KTF·SKT** — LGT 는 상당수 부팅되나 아직 "확정" 아님(ABI RE 진행 중: `docs/lgt_abi.md`·`docs/lgt.md`). J2ME 는 웹 로더 폴백으로 지원.
- 최근 흐름(git log): WIPI-Java/MIDP 메서드 보강·RustJava 포크 핀 상승·결정적 실행기(BTreeMap 폴링)로 타이틀 복구가 주 리듬(232→261).

## 로드맵 위치 · 잔여

1. **dispatch PAT 권한 수정**(사람 1스텝, 위) → 자동 전파 완전 라이브.
2. **security audit red 해소** — quick-xml ≥0.41 / crossbeam-epoch ≥0.9.20 업그레이드 + audit 워크플로 이슈 권한.
3. 타이틀 복구 지속(261+) · LGT 확정 승격 · 플레이키 타이틀(입력 타이밍) 분류.
