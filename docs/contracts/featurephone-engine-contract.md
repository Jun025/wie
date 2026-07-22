# 엔진↔featurephone 소비 계약 (engine-side drift guard)

정본(기계판독): [`featurephone-engine-contract.json`](./featurephone-engine-contract.json)
검사기: [`scripts/check-engine-contract.mjs`](../../scripts/check-engine-contract.mjs)(정적) ·
[`scripts/contract-roundtrip.mjs`](../../scripts/contract-roundtrip.mjs)(브라우저 왕복)
CI: [`.github/workflows/engine-contract.yml`](../../.github/workflows/engine-contract.yml)(PR/push) ·
[`publish-artifact.yml`](../../.github/workflows/publish-artifact.yml)(릴리스 fail-closed 게이트)

## 배경 (2026-07-22)

otterpebble `apps/featurephone` 는 wie 엔진을 **빌드 산출물 경계**로 소비한다:
`publish-artifact.yml` 이 main 푸시마다 fresh `wie_web.js`(glue) + `wie_web_bg.wasm` 을
GitHub Release 로 발행하고 `repository_dispatch` 로 수신부를 깨우면, 수신부가 sha256 검증 후
핀을 범프해 배포한다. 웹 셸에 있던 부팅 왕복 셀프테스트는 2026-07-20 제거됐고(ROM ZERO·BYOF
원칙), 그 커버리지를 **엔진 레포 CI 가 인수**하기로 결정됐다(운영자 지시, 제안
`2026-07-20--featurephone-selftest-removal-seo#p2` 채택). 이 디렉터리가 그 인수분이다.

## 계약 표면 (소비자 실측 — otterpebble `apps/featurephone/lib/engine.ts`)

| 표면 | 내용 | 검사 |
|---|---|---|
| 아티팩트 쌍 | `web/src/wasm/wie_web.js` + `wie_web_bg.wasm`, glue 는 `wie_web_bg.wasm` 를 **이름으로** 같은 폴더에서 fetch | 정적 + 왕복 B(무인자 `default()`) |
| glue export | `default`(init, `WebAssembly.Module` 인자 = 셸 1차 경로/무인자 = 폴백), `init`(panic hook), `WieEmulator` | 정적 + 왕복 A/B |
| 생성자 7-인자 | `(filename, data:Uint8Array, canvas, audioCtx?, gain?, width, height)` — 오디오 미전달 = 무음 모드 | 정적(arity) + 왕복(실호출) |
| 메서드 10종 | `tick`·`key_down`·`key_up`·`platform_kind`·`lgt_compile_model`·`has_exited`·`export_saves`·`has_saves`·`import_saves`·`free` | 정적 + 왕복(실호출) |
| 키 어휘 18종 | `UP/DOWN/LEFT/RIGHT/OK/LEFT_SOFT_KEY/RIGHT_SOFT_KEY/CLEAR/NUM0–9` (셸 `KEY_MAP` 송신분) | 정적(소스 핀 — 미매핑은 **무음 no-op 이라 JS 에서 관측 불가**) + 왕복(no-throw) |
| 정상 종료 체인 | 코어 exit 요청 → `has_exited()` sticky true → 이후 `tick()` 안전 no-op → 세이브는 계속 읽힘 | 왕복(픽스처가 실제 clean-exit 수행) |
| 세이브 블롭 | `WIESAV01` 매직의 불투명 블롭, 왕복 가능, 불량 블롭은 `false`(throw 아님) | 정적(소스 핀) + 왕복(export→import 실측) |
| LGT 판별 | `lgt_compile_model()` ∈ {"clet","aot-java",undefined}, 생성 직후 유효 — 셸은 "aot-java" denylist | 왕복(KTF=undefined·LGT=clet 실측) |
| dispatch payload | `wie-artifact-published` + `version/wieHead/wasmUrl/glueUrl/wasmSha256/glueSha256` — 수신부는 키 누락 시 fail-closed | 정적(워크플로 텍스트 핀) |

## 커버리지 정직 고지 (못 잡는 것)

- **렌더링/blit 회귀**: 픽스처(`test_data/helloworld_*.zip`)는 콘솔 출력 후 즉시 정상 종료하고
  **화면을 그리지 않는다** — 캔버스 self-blit 은 검사 불가(픽셀 수는 info 로만 기록).
  실게임 렌더 회귀는 기존 292종 smoke_gate(네이티브)와 수동 검증 몫이다.
- **실게임 런타임 회귀**: 상용 게임 파일은 ROM ZERO 원칙상 레포·CI 에 없다. 특정 게임만 깨지는
  회귀(타이밍·API 구현 디테일)는 이 계약검사가 잡지 못한다.
- **행동 시맨틱의 전수 보장 아님**: 왕복은 "그 계약대로 한 번 동작함"의 증거이지 전 입력 공간
  증명이 아니다. 예: `has_exited` sticky 는 픽스처 1회 exit 로 관측하며, 키 어휘는 no-throw 만
  관측한다(미매핑 키는 소스 핀으로만 잡는다).
- **소비자 쪽 드리프트**: otterpebble 쪽이 계약을 바꾸는 방향(새 메서드 기대 등)은 엔진 CI 가
  알 수 없다 — 그 방향은 otterpebble CI/리뷰 몫.

## 계약을 의도적으로 바꿀 때

1. 이 디렉터리의 JSON(+필요시 검사기)을 같은 PR 에서 갱신한다 — 검사 실패가 "사람이 눈치채는
   지점"이 되도록 설계됐다.
2. otterpebble 소비자(`lib/engine.ts`·수신 워크플로)와 **같은 롤아웃**으로 조율한다(핀 순서:
   엔진 하위호환 추가 → 소비자 채택 → 구 표면 제거).
