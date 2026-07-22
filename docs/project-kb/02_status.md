# wie 현재 상태 (02_status)

> **기준일 2026-07-14** · 레포 실측 기반(워크플로 실행 로그·커밋·베이스라인). 이 파일이 KB 의 최신 현황이다 — 매 작업 세션(main 반영) 시 재생성.

## 자동발행 파이프라인 (연방 ① 발신부) — **완전 라이브(dispatch 포함)**

`.github/workflows/publish-artifact.yml` (2026-07-08 라이브, dispatch 는 2026-07-09 PAT 재주입으로 활성):

- main push(엔진 소스 경로: `**/*.rs`·`Cargo.toml`·`Cargo.lock`·`build-wasm.sh`) → **fresh** `wie_web` WASM 빌드(wasm-bindgen 0.2.108 핀 + wasm-opt) → GitHub Release `engine-<shortsha>` 에 `wie_web_bg.wasm` + `wie_web.js` 발행(sha256 메타 포함, 같은 커밋 재실행 멱등).
- 최신 릴리스: **`engine-d7b5b02`**(2026-07-10).
- 이어서 otterpebble 에 `repository_dispatch`(event `wie-artifact-published`, payload: version·wieHead·wasmUrl/glueUrl·sha256·confirmedPlatforms `["KTF","SKT","LGT"]`) → 리시버(otterpebble 소유 `wie-artifact-receive.yml`)가 featurephone 재배포. (SoT: `publish-artifact.yml` dispatch 스텝 payload.)
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
- **LGT 컴파일모델**: `lgt_compile_model(): string | null` — **additive getter(2026-07-13, featurephone 옵션1 확정·셸소유 계약변경)**. LGT 타이틀이면 `"clet"`(WIPI-C, wie 렌더 가능) 또는 `"aot-java"`(AOT-Java, 부팅되나 §7 벽으로 미렌더), **비-LGT(KTF/SKT/J2ME)는 `null`**(개념 부적용 — wasm-bindgen `Option<String>`→JS null, 셸이 `=== "aot-java"` 한 줄로 차단 판정·나머지는 falsy 통과라 구분 최단). 판별근거=앱 자신의 import thunk 정적 스캔(`binary.mod` ELF 실행섹션에서 `bl`+`.word 0x64` java-interface thunk 유무; 0x64⇒aot-java, 부재⇒clet). **생성자에서 1회 산정, 인스턴스 수명 내 불변 — 첫 `tick()` 전 즉시 유효**(로드 성공 직후 호출 가능). read-only 조회, 런타임 무영향. 코퍼스 실측: working/lgt 54 전부 clet, broken/lgt 24 aot-java(deep-assets 알려진 24종과 정확 일치)+22 clet, ambiguous 0. `platform_kind()` 등 기존 표면 전부 무변경 순수 additive. 권고 셸 패턴: `"aot-java"` 업로드 시점 실행 차단+"준비 중" 안내, `"clet"`만 실행.

## CI 현황

- **Rust CI**(`rust.yml`): 3-OS 매트릭스, `fmt --check` + `clippy -D warnings`(wasm32 타깃 포함) + 전체 테스트 — **green**(clippy red 해소됨).
- **coverage**(`coverage.yml`): `CODECOV_TOKEN` 없으면 업로드만 skip(fork-safe) — green.
- **Web**(`web.yml`): wasm 빌드 + Cloudflare Pages 배포, Pages 프로젝트(`wie-web`) 부재 시 자가 재생성(`pages project create || true`).
- **Security audit**(`rust-audit.yaml`, 매일 schedule + `workflow_dispatch`): **schedule 경로 정정(2026-07-22, `[wie-security-audit-schedule-red]`)**. 종전 KB 의 "green(2026-07-10 해소)" 서술은 **실측과 불일치한 오독이었다**: `gh run list -w "Security audit"` 보존 이력의 **schedule 런 28건 전부(2026-06-25~07-22) failure**(예: 29887989781·29798146955), 유일 success 는 2026-07-10 **workflow_dispatch** 1건(같은 날 같은 커밋의 schedule 런 29067652864 는 failure). 원인은 취약점이 아니라 **fork+Issues 비활성**: 이 repo 는 `dlunch/wie` fork 라 Issues 가 기본 off 이고 repo-레벨 disable 은 토큰 권한으로 못 넘긴다. 그런데 종전 `rustsec/audit-check` 액션이 경고 2건을 **GitHub Issue 로 올리려다** `Issues has been disabled in this repository.` 로 매 schedule 런에서 죽었다. 선행 커밋 `7405c50b` 이 붙인 `issues: write` 는 애초에 성립 불가한 처방이었고, 같은 커밋이 KB 를 red→green 으로 바꾼 것이 오독의 출처. **정정 조치**: 리포팅 경로를 `cargo audit` 직접 실행으로 전환(Issue 생성 안 함 → fork 에서 성립), 무효 권한 `issues/checks: write` 제거. **부류 분리 실증(로컬)**: 경고만 있는 현 상태 = exit 0(green), 취약점 존재 시(quick-xml ignore 제거) = exit 1(red) — 종료코드로 구분. ★정확한 성격 규정: 이 잡은 **공급망 차단 게이트가 아니다**(`.github/workflows/` 전수에 `workflow_run` 소비 참조 0건, publish-artifact 독립 구동). 탐지(cargo audit) 자체는 정상 동작하며 Actions 탭에 red 로 보였으므로 무성 실패도 아니었다 — 실제로 죽었던 것은 **알림 채널(Issue 생성)과 신호 대 잡음비**다. **PENDING 2건**: ① `quick-xml` 0.39.2(RUSTSEC-2026-0194·0195, XML DoS) — 패치는 0.41.0에만 존재하나 유일 소비자 `wayland-scanner`(최신 0.31.10)가 `^0.39` 요구로 상위 차단. 빌드타임 proc-macro 가 vendored 신뢰 XML 만 파싱해 공격면 없음 — 근거 명시 후 개별 `--ignore` 처리, wayland-scanner 가 ≥0.41 채택 시 ignore 제거. ② `ttf-parser` 0.25.1 unmaintained(RUSTSEC-2026-0192, patched 버전 없음)·`spin` 0.12.0 yanked — 둘 다 **비게이팅 경고**(로그에는 보이나 exit 0). skrifa 이행은 `ab_glyph`(최신 0.2.32도 ttf-parser 의존) 교체가 선행돼야 해 보류. **로드맵에 미래 트랙으로 등재(아래 5번)** — 매일 audit 경고가 잊히지 않게 추적.

## 두 트랙 — 엔진 정상화 현황

**트랙 ① 타이틀 회수(모드 A — 자율주행 진행 중)**
- **회귀 베이스라인**(`scripts/smoke_gate_baseline.tsv`): **292 타이틀 부팅+렌더 PASS — KTF 190 / LGT 52 / SKT 50**(2-run 교집합 검증, 게임파일 미포함·식별자만). 게이트는 부팅+렌더만 판정(입력 생존은 비게이팅 어드바이저리). 구 스냅샷(2026-07-02) 202 대비 **+90**.
- **커버리지 오딧 승격(2026-07-13)**: 코퍼스 전량(ktf190/lgt54/skt50=294) 대비 기존 261 등재분을 대조해 "PASS-both 이나 미등재" 후보 **31종(전부 SKT)** 특정 → 각 타이틀 SKT 2-run 독립 실행(A/B 각 SKT50/50 전수 PASS)으로 교집합 승격, **261→292**. 후보가 SKT 뿐인 이유: KTF 코퍼스 190=베이스라인 190(전량 등재 완료), LGT 코퍼스 54 중 52 PASS 전량 등재(FAIL 2 제외). 승격 후 전체 코퍼스 2-run 재검증 — 두 런 모두 **292 전수 회귀-0·absent 0**. baseline.tsv 데이터 등재만(엔진 런타임 무변경, 회귀-0 자명).
- **제외 2건(등재 금지)**: `lgt/놈ZERO` = 기지의 per-game FAIL(누락 blit SVC 아님, 게임별 near-blank). `lgt/하이브리드` = 선재 핀 이슈(널점프 inject runaway) **PENDING·미접촉** — 승격/수정 금지.
- **d7b5b024(sec/audit-green 머지) 게이트 소급 확정(2026-07-10)**: 코퍼스 복귀 후 2-run 실측 — 두 런 모두 **베이스라인 261 전수 회귀-0**(294 중 292 PASS, FAIL 2건은 두 런 동일한 비-베이스라인 LGT 타이틀 놈ZERO·하이브리드). crossbeam-epoch 0.9.20 등 lockfile 패치 상향의 회귀-블라인드 해소. ② has_exited getter 변경은 wasm32 전용 `wie_web` 한정 — 네이티브 `wie_validate` 바이너리 sha256 동일 실증(재컴파일 미발생, wie_cli 의존트리 무관)으로 동일 2-run 이 변경 후 트리에도 유효.
- 최근 리듬(git log): WIPI-Java/MIDP 메서드 보강 · RustJava 포크 핀 상승(트랙2 클러스터 다수 귀속: readUnsignedByte·TimeZone·Byte 등) · 결정적 실행기(BTreeMap 폴링·스레드 스케줄링 = 구 트랙1 반영)로 232→261.
- dispatch 의 `confirmedPlatforms` 는 **KTF·SKT·LGT**(2026-07-14 LGT 승격). SKT 는 코퍼스 50종 전량 베이스라인 등재. **LGT 는 clet 52종 confirmed** — 셸이 `lgt_compile_model()==="aot-java"` 로 AOT-Java 24종을 사전 제외하므로 confirmed 는 **clet 서브셋 정식 지원**을 의미(AOT 24종은 §7 동결·"준비 중", 렌더 가능 승격 아님). J2ME 는 웹 로더 폴백 지원.

**트랙 ② §7 벽 — LGT AOT-Java 렌더(모드 B — 외부 산출물 대기)**
- LGT AOT-Java 24종은 렌더 0 유지. 바이너리-측 조사는 cp59 로 완결: per-frame 구동은 TIMER_EVENT(21) 모델로 확정(구현 가능), 유일 블로커는 **0x64 ordinal→native 등록표**. 오프라인 획득 소진 증명(AromaWIPI 비공번호 — `docs/reference/lgt_0x64_ordinal_table.md`) → **실기 트레이스 필요**. 도착 시 4단계 즉시 활성화 스캐폴드 커밋됨(기본 비활성·회귀 0). 요약: `10_deep-assets.md`, 원문: `docs/lgt_abi.md` §7·§8.

**LGT confirmed 승격 선결조건 = AOT-Java graceful 제외 신호 (2026-07-13 조사, 미구현)**
- **Q1 실패모드 실측**(배틀몬스터 2빌드, wie_validate=웹과 동일 `LgtEmulator::from_archive` 경로): 진짜 AOT-Java 는 **silent blank(throw 없음)** — paints=1·content=false·max_ticks(5천만) 완주, `run_err` 미발생(reason="only blank/uniform frames"). 부팅은 성공(`registered 20 app classes` + import table `0x64` 다수)하나 §7 렌더 드라이버 부재로 조용히 검은화면. **최악 케이스**: 셸의 현행 실패감지(tick() throw / has_exited)로 감지 불가. ※ 대조: broken/lgt 의 clet 미완성분(영웅서기4=WIPIC SVC 111, 붉은보석=stdlib 0x3f7)은 **throw** 하고, 제노니아2 는 이제 PASS — 즉 broken 폴더는 AOT 전용 아님, 실패모드는 서브셋별로 갈림.
- **Q2 판별 신호 가용성**: clet↔AOT 구분은 컨테이너/파일명으론 불가(양쪽 다 jar 안 `binary.mod` + `app_info`, MClass 필드는 대부분 공란이라 비신뢰). **판별점 2개 실재**: ⓐ **정적(로드 전)** — `binary.mod` ELF import thunk 의 `0x64`(Java-interface) 참조 유무. deep-assets 가 16바이트 thunk 패턴으로 24/102 정적 특정 완료해 **로드 전 파일 바이트만으로 판별 가능** 실증됨. ⓑ **부팅 극초기(첫 tick 전, `load_native` 내)** — `register_app_classes` 반환 non-empty(AOT 는 `.data` 에 class descriptor, clet 은 없음) + 첫 import table `0x64`(AOT) vs `0x1fb`(WIPI-C clet). 현 로더는 `loadable_archive`(app_info 존재)·`loadable_jar`(binary.mod 존재)로 **LGT 판정만** 하고 컴파일모델은 표면에 미노출 — `platform_kind()="LGT"` 한 단계 아래 정보는 내부에 존재하나 셸이 못 봄.
- **최소 additive 제안(구현 금지·형태만)**: `platform_kind()` 무변경 유지 + ▸옵션1(선호, 정적) 별도 판별 getter 예 `lgt_compile_model() -> "clet"|"aot-java"` — 로더가 binary.mod 의 0x64 thunk 정적 스캔(추출기 기존)으로 셋, 셸이 "aot-java"면 사전 "미지원 서브셋" 안내 후 제외. ▸옵션2(로드실패 명시화) AOT 감지 시(class descriptor non-empty && 렌더드라이버 부재) 로드 단계에서 명시적 `WieError` 반환해 현행 silent-blank 를 explicit-throw 로 승격 — 단 런타임 동작 변경이라 계약·회귀 검토 필요. 둘 다 기존 표면 무변경 후방호환.
- **승격 안전성 판정**: Q1(AOT=silent blank, 감지불가) + Q2(판별신호 명확히 가용) 종합 → **"명시적 caveat + graceful 제외 신호 선행 필요"**. additive 신호를 노출하고 셸이 AOT 서브셋을 사전 제외하면 승격 안전(그때 clet 서브셋만 confirmedPlatforms 승격).
- **LGT confirmed 승격 4단계 = 전체 완료(2026-07-14)**: **① 엔진 getter 노출 = 완료(2026-07-13, #33)** — `lgt_compile_model()` 라이브(엔진 웹 계약 참조, 옵션1 정적 0x64 thunk 스캔). **② 셸 배선(featurephone "aot-java" 차단+안내) = 완료**. **③ clet-only 재검증 = 완료**. **④ confirmedPlatforms 에 LGT 승격 = 완료(2026-07-14)** — `publish-artifact.yml` dispatch payload `["KTF","SKT"]`→`["KTF","SKT","LGT"]`(발신부 메타데이터 한정, 런타임 무변경). **범위 불변식**: clet 52종의 platform-level confirmed 선언일 뿐 AOT-Java 24종을 렌더 가능으로 만들지 않음 — 셸이 aot-java 를 사전 제외하므로 confirmed=clet 서브셋 정식 지원, AOT 24종은 §7 동결 유지.

## 로드맵 위치 · 잔여

1. ~~dispatch PAT 권한 수정~~ — **완료(2026-07-10 확인)**: d7b5b024 발행 런 dispatch success, 자동 전파 완전 라이브.
2. ~~security audit red 해소~~ — **schedule 경로 정정 완료(2026-07-22, `[wie-security-audit-schedule-red]`)**: 2026-07-10 "해소" 는 dispatch 런 오독이었고 schedule 은 전건 failure 였다(원인 fork+Issues 비활성). `cargo audit` 직접 실행으로 전환해 알림 채널 의존 제거 — 취약점=red / 경고=green 부류 분리 확립. 잔여 PENDING(quick-xml 상위 차단·ttf-parser unmaintained·spin yanked)은 CI 현황 참조. ~~코퍼스 복귀 시 261 재확인 권장~~ → **소급 확정 완료(2026-07-10, 트랙① 참조)**.
3. 트랙 ① 지속(292+) · **LGT clet 확정 승격은 AOT-Java graceful 제외 신호 선행 필요**(트랙② 하단 조사 참조 — Q1 silent blank·Q2 판별신호 가용, 유보 권고) · 플레이키 타이틀(입력 타이밍) 분류.
4. 트랙 ② 는 실기 트레이스 확보(사람/외부) 전까지 동결 — 재조사 금지 목록 준수(`10_deep-assets.md` 가드레일).
5. **[미래 트랙 — 착수 아님] ab_glyph→skrifa 폰트스택 이행**: `ttf-parser` RUSTSEC-2026-0192(unmaintained) 해소의 선행요건. `ab_glyph` 가 최신(0.2.32)까지 ttf-parser 에 의존하므로 폰트 렌더 스택 자체를 skrifa 계열로 교체해야 경고가 사라진다. 지금 구현 착수 금지 — 매일 audit 의 비게이팅 경고를 이 항목으로 추적(ab_glyph 의 탈-ttf-parser 릴리스 또는 skrifa 직접 이행 타당성 재평가 시 활성화).
