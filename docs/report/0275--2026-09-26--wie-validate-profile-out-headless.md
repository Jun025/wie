## [2026-09-26] wie_validate `--profile-out` — 헤드리스 프로파일 · 놈3 no-frame 1건 포착: 게스트 공회전이 아니다 (wie-validate-profile-out-headless)

**무엇을**: 제안 `2026-09-25-nom3-no-frame-low-load-split#p0` 채택. `wie_validate` 에 `--profile-out <path>`(기본 꺼짐)를
열었다 — `ArmCore::sample_profile` 의 PC + R7 사슬을 GUI `wie --profile-out` 과 **같은 folded 형식**으로 쓴다.
꺼져 있으면 파일을 만들지 않고 JSON 줄도 그대로다(러너 줄 불변). KTF/LGT 만 샘플이 생긴다(J2ME/SKT 는 ARM 코어 없음).

**왜**: 0265 의 FAIL B(부트 뒤 guest `Card.paint` 로 들어가 16.5s 네이티브 호출 0)에서 guest 위치를 못 쟀다 —
`build_emulator` 가 `profile: None` 을 박아 두었기 때문이다.

**사용자 영향**: 없음(검증 도구 플래그 · 기본 꺼짐).

### 시험

`wie_cli/tests/validate_profile_out.rs` — 실제 바이너리를 `helloworld_ktf.zip` 로 두 번 돌린다: 끄면 파일 **무생성**,
켜면 파일 생성 · 모든 줄이 `0x<hex>(;0x<hex>)* <count≥1>` · JSON 줄 필드 수 동일.

### 놈3 측정 (릴리스 · 순차 · 첫 FAIL 에서 정지)

- 바이너리: 이 브랜치 `e30db802` release `wie_validate` 사본 · 타이틀 `game_lab/broken/lgt/놈3.zip` sha256 `b475b639…`(0265 와 같은 값).
- 명령: `wv --inject --profile-out p<i>.folded <놈3.zip>` · RUST_LOG 없음 · load1 43.6–121.2.
- 결과: **25회 중 1 FAIL**(#25 · load1 117 · `paints 0` · `input_steps 27/27` · `stop deadline` · ticks 813) — 0265 의 no-frame 과 같은 모양.

### 프로파일이 말하는 것

샘플은 **엔진 실행 조각이 끝날 때마다 1개**다(`core.rs` 의 `run` 루프). 조각은 SVC(네이티브 호출), 함수 반환
(`RUN_FUNCTION_LR` `0x7f000000`), 또는 명령 예산 소진(`INSTRUCTIONS_PER_YIELD` 10,000)에서 끝난다. ⇒ leaf 가
`0x8`(SVC 벡터) · `0x7f000000` 가 아닌 샘플만 «guest 코드 한가운데서 예산이 떨어진» 샘플이다.

| | 샘플 | 조각 끝 `0x8` | `0x7f000000` | guest PC | guest PC, 마지막 ¼ |
|---|---|---|---|---|---|
| FAIL #25 | 57,000 | 43,888 | 13,030 | **80** | **14** |
| PASS #3 | 243,000 | 207,556 | 34,887 | 555 | 155 |

- ★**FAIL 은 guest 공회전이 아니다.** guest 가 네이티브 호출 없이 도는 루프라면 10,000 명령마다 guest PC 샘플이
  하나씩 쌓여야 한다 — 16.5s 면 수천~수만 개다. 실측은 판 전체 **80**, 마지막 ¼ 에 **14** 다.
  마지막 배치(998/1,000)는 거의 전부 SVC 벡터다 — 0265 의 «thread 2 의 `Thread.sleep(100)` 만 깬다»와 맞는다.
- ⇒ 그리던 스레드는 **돌고 있는 것이 아니라 서 있다**(무언가를 기다리거나 다시 스케줄되지 않는다). 0265 의 B 를
  «`Card.paint` 안에서 guest 가 헤맨다»로 읽으면 틀린다 — 고칠 곳은 guest 코드가 아니라 **대기/깨움 경로** 쪽이다.
- ★**이 판이 A 인지 B 인지는 가르지 않았다** — 스레드 로그 없이 돌렸다(로그가 부트를 늘려 A 를 키운다는 0265 의 지적 때문).
  ticks 813 은 0265 의 B(973)에 가깝고 A(48)와 멀지만, ticks 는 판정 축이 아니다(0265). A 였더라도 «guest 공회전 아님»은 그대로다.

### 한계

- 샘플은 **스레드를 구분하지 않는다** — 조각 끝의 PC 만 있다. «어느 스레드가 무엇을 기다리나»는 이 도구로 답하지 못한다.
- 샘플 간격은 시간이 아니라 조각 수다. «마지막 ¼»은 배치 순서(1,000 샘플씩 flush)로 자른 것이다.
- FAIL n=1. 재현율 1/25 는 0265 의 5.6%(6/107)와 같은 크기다.

### 검증

- 네 게이트 + beta clippy · `RUST_MIN_STACK=4194304 cargo test --all` 48 스위트 전부 0 failed(회신에 원문).
- 증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-validate-profile-out-headless/` —
  `runs.tsv`(25회) · `o25.json` · `p25.folded.gz`(FAIL) · `p3.folded.gz`(PASS 대조) · `loop.sh`.

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <game_lab>`): BOUNDED 11회/5쌍 — `놈3` 9회(이 회차의 대상 · 리니지 0229·0265 가 이미 적은 라이브 타이틀) · `놈ZERO` 2회(`wie_validate.rs` 의 기존 주석 1 — 이 회차가 쓴 줄 아님 · 이 문장 1) · SUFFIX-ATTACHED 0.
