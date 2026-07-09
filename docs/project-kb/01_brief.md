# wie 엔진 브리프 (01_brief)

> 기획-계층 요약 · 비권위. 기술 원본: `README.md` · `docs/architecture.md` · `AGENTS.md`.

## wie 는 무엇인가

한국 피처폰 시대(WIPI · SKVM · J2ME) 모바일 앱을 현대 웹/데스크톱에서 되살리는 **Rust 에뮬레이터**다. upstream 은 dlunch/wie 이고, 이 레포(Jun025/wie)는 **웹 서비스화 + 검증 체계 + 연방 자동발행**을 얹은 포크다. JVM 런타임은 RustJava(자체 포크 핀)를 upstream 의존성으로 소비한다. **업스트림 dlunch/wie 는 직접 접촉하지 않는다** — 푸시는 오직 origin(Jun025/wie).

- 코어: `wie_core_arm`(ARM 에뮬), `wie_jvm_support`, `wie_backend`(시스템 서비스)
- 플랫폼: `wie_ktf` · `wie_skt` · `wie_lgt`(통신사 3사) · `wie_j2me`, API 계층 `wie_midp` · `wie_wipi_*` · `wie_skvm`
- 소비 형태: `wie_cli`(로컬/검증 — 헤드리스 검증기 포함) · `wie_web`(wasm-bindgen 브라우저 어댑터 — 산출물의 원천)

## 이중 역할 — ① 엔진 정상화 트랙 + ② featurephone 엔진 공급자

wie 의 기획은 두 축으로 굴러간다:
1. **엔진 정상화(기존 트랙)**: 부팅+렌더 게이트로 동작 타이틀 수를 단조 증가시키는 회수 작업(모드 A 자율주행)과,
   LGT AOT-Java 렌더의 **§7 벽**(외부 산출물 대기 — 모드 B) 돌파. 상세: `10_deep-assets.md` · `02_status.md`.
2. **연방 공급자(신규 트랙, 2026-07-08~)**: main 반영 시 fresh WASM 을 자동발행해 otterpebble featurephone 에
   전파하는 산출물 경계의 발신부(아래).

## 연방 내 위치 — 독립 git #3 · 빌드 산출물 경계

wie 는 4-레포 연방(otterpebble·qts·wie·RustJava)의 **엔진 레포**다. 경계 타입은 **빌드 산출물 경계**: 엔진 소스는 이 레포에만 살고, 소비자에게는 **fresh WASM 아티팩트(GitHub Release)로만** 전달된다. 벤더링·서브모듈·병합은 안티패턴(경계 SoT: otterpebble `.claude/rules/repo-boundaries.md`).

- **공급자(wie)**: main 반영 → `.github/workflows/publish-artifact.yml` 이 fresh WASM 빌드 → `engine-<shortsha>` Release 발행 → otterpebble 에 `repository_dispatch`(event `wie-artifact-published`, sha256 핀 포함).
- **소비자(featurephone.otterpebble.com)**: otterpebble 소유 리시버가 dispatch 를 받아 해시 검증 후 fresh 엔진을 정적 서빙. BYOF(게임파일은 사용자 브라우저에만) + per-user vault 는 otterpebble 소관이나, **엔진 웹 계약(WieEmulator API — `02_status.md`)이 접점**이다. 계약이 바뀌면 소비자가 깨진다 — 계약 변경은 기획 사안.
- 공개 레포라 Actions 무료 + Release 자산 무인증 다운로드(리시버가 curl 로 받는 전제).

## wie 자체 웹서비스 (featurephone 과 별개)

이 레포는 자체 프론트(`web/`, React+Vite)와 Cloudflare Pages Functions+D1 백엔드(`functions/` — 계정·세이브 동기화)를 가진 독립 서비스이기도 하다(wie-web Pages 프로젝트). **BYOF 원칙**: 게임 바이트·파일명·해시·보유목록은 서버에 절대 안 감(`scripts/audit-no-leak.sh` 로 강제). 서버엔 계정·불투명 세이브·문의 텍스트만. 상세: `docs/web.md` · `docs/COMPLIANCE.md`.

## 기획/사업 맥락

- **미션**: 소멸한 한국 피처폰 게임 유산의 보존·재생. 게임 파일을 호스팅하지 않는 BYOF 모델로 권리 리스크를 구조적으로 회피(법적 고지: README — 법률 자문 아님).
- **품질 축**: "부팅+렌더" 회귀 게이트(`scripts/smoke_gate.sh` + 커밋된 베이스라인)로 동작 타이틀 수를 단조 증가시키는 것이 핵심 지표. 현황 수치는 `02_status.md`.
- **운영 모델**: 3-역할(운영자=결정·승인 / Claude Code=구현+제안 / 외부 Claude Project=(a)~(d) 선택적 조언). 이 KB 는 그 조언자용 입력이다.
