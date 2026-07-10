# wie 현재 상태 (02_status)

> **기준일 2026-07-10** · 레포 실측 기반(워크플로 실행 로그·커밋·베이스라인). 이 파일이 KB 의 최신 현황이다 — 매 작업 세션(main 반영) 시 재생성.

## 자동발행 파이프라인 (연방 ① 발신부) — **완전 라이브(dispatch 포함)**

`.github/workflows/publish-artifact.yml` (2026-07-08 라이브, dispatch 는 2026-07-09 PAT 재주입으로 활성):

- main push(엔진 소스 경로: `**/*.rs`·`Cargo.toml`·`Cargo.lock`·`build-wasm.sh`) → **fresh** `wie_web` WASM 빌드(wasm-bindgen 0.2.108 핀 + wasm-opt) → GitHub Release `engine-<shortsha>` 에 `wie_web_bg.wasm` + `wie_web.js` 발행(sha256 메타 포함, 같은 커밋 재실행 멱등).
- 최신 릴리스: **`engine-d7b5b02`**(2026-07-10).
- 이어서 otterpebble 에 `repository_dispatch`(event `wie-artifact-published`, payload: version·wieHead·wasmUrl/glueUrl·sha256·confirmedPlatforms `["KTF","SKT"]`) → 리시버(otterpebble 소유 `wie-artifact-receive.yml`)가 featurephone 재배포.
- ~~PAT 403 잔여~~ — **해소 확인(2026-07-10)**: d7b5b024 발행 런에서 "Dispatch to otterpebble" 스텝 success. 구 403(2026-07-08)은 fine-grained PAT 권한 부족이었고 재발급·재주입으로 종결.

## 엔진 웹 계약 (소비자 featurephone 이 의존 — 변경 = 기획 사안)

원본: `wie_web/src/lib.rs`(wasm-bindgen 표면). 2026-07 현행:

- **생성자**: `new WieEmulator(filename, data: Uint8Array, canvas, audioCtx?, gain?, width, height)` — 7인자, 오디오 2개는 옵션(무음 시 undefined).
- **렌더**: 엔진이 **캔버스에 직접 blit**(더블버퍼 putImageData→drawImage, CSS pixelated 스케일). 소비자는 `requestAnimationFrame` 에서 `tick()` 만 호출.
- **입력**: `key_down/key_up/key_repeat(code)` — 키코드 문자열: `UP DOWN LEFT RIGHT OK LEFT_SOFT_KEY RIGHT_SOFT_KEY CLEAR CALL HANGUP VOLUME_UP VOLUME_DOWN NUM0`~`NUM9 HASH STAR`(미지 코드는 무시).
- **종료**: `has_exited(): boolean` — **additive getter(2026-07-10, featurephone 요청·셸소유 확정 계약변경)**. 코어가 정상 종료를 요청한 순간(= `[wie] emulator requested exit` 로그 경로, 현행 도달점 WIPI `MC_knlExit`) true 로 플립, **sticky**(한번 true 면 인스턴스 수명 내내 유지). 플립 후 `tick()` 은 **안전한 no-op**(코어 미전진, `Ok` 반환 — rAF 루프가 플래그 관찰 전에 더 돌아도 무해). 런타임 실패 = 기존대로 `tick()` 이 예외를 던짐 → 셸 구분법: **clean exit = getter true·throw 없음 / 실패 = throw**. 세이브는 플립 후에도 읽기 가능 — 매 `tick` 후 getter 폴링, true 시 `export_saves()` 로 persist 후 `free()`. 구(폐지) has_exited 의 제거를 revert 한 것이 아니라 현 모델 위 순수 additive(기존 소비자 무손상).
- **오디오**: JS 가 사용자 제스처에서 만든 `AudioContext`+`GainNode` 를 주입, PCM 은 WebAudio 로 갭리스 스케줄, MIDI 는 무음 스텁.
- **세이브**: `has_saves()` / `export_saves()`(불투명 `WIESAV01` 블롭: RMS+FS) / `import_saves(blob)` / `export_fs`·`import_fs`. 해제는 `free()`.
- **로더**: `.zip`→KTF→LGT→SKT 순 판별, `.jar`→KTF→LGT→SKT→J2ME 폴백, `.jad` 는 거부(.jar 요구). `platform_kind()` 가 `"KTF"|"LGT"|"SKT"|"J2ME"` 반환.

## CI 현황

- **Rust CI**(`rust.yml`): 3-OS 매트릭스, `fmt --check` + `clippy -D warnings`(wasm32 타깃 포함) + 전체 테스트 — **green**(clippy red 해소됨).
- **coverage**(`coverage.yml`): `CODECOV_TOKEN` 없으면 업로드만 skip(fork-safe) — green.
- **Web**(`web.yml`): wasm 빌드 + Cloudflare Pages 배포, Pages 프로젝트(`wie-web`) 부재 시 자가 재생성(`pages project create || true`).
- **Security audit**(`rust-audit.yaml`, 매일 + `workflow_dispatch`): **green**(2026-07-10 해소) — `crossbeam-epoch` 0.9.18→**0.9.20**(RUSTSEC-2026-0204) · `anyhow` 1.0.102→**1.0.103**(unsound) · `memmap2` 0.9.10→**0.9.11**(unsound) 은 lockfile 패치 상향으로 제거(호출부 무수정 — 전부 semver-호환 패치 라인). 워크플로에 `permissions: issues/checks: write` 추가로 이슈 생성 권한 복구. **PENDING 2건**: ① `quick-xml` 0.39.2(RUSTSEC-2026-0194·0195, XML DoS) — 패치는 0.41.0에만 존재하나 유일 소비자 `wayland-scanner`(최신 0.31.10)가 `^0.39` 요구로 상위 차단. 빌드타임 proc-macro 가 vendored 신뢰 XML 만 파싱해 공격면 없음 — 근거 명시 후 workflow `ignore` 처리, wayland-scanner 가 ≥0.41 채택 시 ignore 제거. ② `ttf-parser` 0.25.1 unmaintained(RUSTSEC-2026-0192, patched 버전 없음) — 대안 skrifa 이행은 `ab_glyph`(최신 0.2.32도 ttf-parser 의존) 교체가 선행돼야 해 보류. 경고는 audit 비게이팅. **로드맵에 미래 트랙으로 등재(아래 5번)** — 매일 audit 경고가 잊히지 않게 추적.

## 두 트랙 — 엔진 정상화 현황

**트랙 ① 타이틀 회수(모드 A — 자율주행 진행 중)**
- **회귀 베이스라인**(`scripts/smoke_gate_baseline.tsv`): **261 타이틀 부팅+렌더 PASS — KTF 190 / LGT 52 / SKT 19**(2-run 교집합 검증, 게임파일 미포함·식별자만). 게이트는 부팅+렌더만 판정(입력 생존은 비게이팅 어드바이저리). 구 스냅샷(2026-07-02) 202 대비 **+59**.
- **d7b5b024(sec/audit-green 머지) 게이트 소급 확정(2026-07-10)**: 코퍼스 복귀 후 2-run 실측 — 두 런 모두 **베이스라인 261 전수 회귀-0**(294 중 292 PASS, FAIL 2건은 두 런 동일한 비-베이스라인 LGT 타이틀 놈ZERO·하이브리드). crossbeam-epoch 0.9.20 등 lockfile 패치 상향의 회귀-블라인드 해소. ② has_exited getter 변경은 wasm32 전용 `wie_web` 한정 — 네이티브 `wie_validate` 바이너리 sha256 동일 실증(재컴파일 미발생, wie_cli 의존트리 무관)으로 동일 2-run 이 변경 후 트리에도 유효.
- 최근 리듬(git log): WIPI-Java/MIDP 메서드 보강 · RustJava 포크 핀 상승(트랙2 클러스터 다수 귀속: readUnsignedByte·TimeZone·Byte 등) · 결정적 실행기(BTreeMap 폴링·스레드 스케줄링 = 구 트랙1 반영)로 232→261.
- dispatch 의 `confirmedPlatforms` 는 **KTF·SKT** — LGT 는 clet 52종이 부팅+렌더하나 아직 "확정" 승격 전. J2ME 는 웹 로더 폴백 지원.

**트랙 ② §7 벽 — LGT AOT-Java 렌더(모드 B — 외부 산출물 대기)**
- LGT AOT-Java 24종은 렌더 0 유지. 바이너리-측 조사는 cp59 로 완결: per-frame 구동은 TIMER_EVENT(21) 모델로 확정(구현 가능), 유일 블로커는 **0x64 ordinal→native 등록표**. 오프라인 획득 소진 증명(AromaWIPI 비공번호 — `docs/reference/lgt_0x64_ordinal_table.md`) → **실기 트레이스 필요**. 도착 시 4단계 즉시 활성화 스캐폴드 커밋됨(기본 비활성·회귀 0). 요약: `10_deep-assets.md`, 원문: `docs/lgt_abi.md` §7·§8.

## 로드맵 위치 · 잔여

1. ~~dispatch PAT 권한 수정~~ — **완료(2026-07-10 확인)**: d7b5b024 발행 런 dispatch success, 자동 전파 완전 라이브.
2. ~~security audit red 해소~~ — **완료(2026-07-10)**. 잔여 PENDING(quick-xml 상위 차단·ttf-parser unmaintained)은 CI 현황 참조. ~~코퍼스 복귀 시 261 재확인 권장~~ → **소급 확정 완료(2026-07-10, 트랙① 참조)**.
3. 트랙 ① 지속(261+) · LGT clet 확정 승격 · 플레이키 타이틀(입력 타이밍) 분류.
4. 트랙 ② 는 실기 트레이스 확보(사람/외부) 전까지 동결 — 재조사 금지 목록 준수(`10_deep-assets.md` 가드레일).
5. **[미래 트랙 — 착수 아님] ab_glyph→skrifa 폰트스택 이행**: `ttf-parser` RUSTSEC-2026-0192(unmaintained) 해소의 선행요건. `ab_glyph` 가 최신(0.2.32)까지 ttf-parser 에 의존하므로 폰트 렌더 스택 자체를 skrifa 계열로 교체해야 경고가 사라진다. 지금 구현 착수 금지 — 매일 audit 의 비게이팅 경고를 이 항목으로 추적(ab_glyph 의 탈-ttf-parser 릴리스 또는 skrifa 직접 이행 타당성 재평가 시 활성화).
