## [2026-09-18] keydraw 생성기의 호출처 0 — ★**declare(⒝) 로 닫았다. ⒜ 는 «비싸서»가 아니라 «불가능»해서 탈락했다** (wie-keydraw-fixture-generator-is-never-run-against-its-committed-zip)

**무엇을**: `AGENTS.md` §Web-surface commands 에 **NOT-RUN 선언 1블록**(`smoke_gate.sh` 선례와 같은 형식).
★**코드 0줄 · 새 워크플로 0 · 커밋된 zip 무접촉 · 형제 생성기 무접촉 · `game_lab/` 무접촉 · `STATE.md` 무접촉.**

**왜**: `scripts/make-wipi-keydraw-fixture.sh` 의 **실행 호출처가 0** 이고, 커밋된
`test_data/keydraw_{ktf,lgt}.zip` 이 그 산출물인데 CI 도 로컬 러너도 그것을 **다시 만들지 않는다**.

**사용자 영향**: 없다(문서 1파일).

### ⓐ 호출처 0 — ★**재측 확인. 단 `.github/` 히트는 «있고» 그것은 실행이 아니다**

추적 파일 전수(`git ls-files | xargs /usr/bin/grep -n`)에서 `.github/` 히트는 **1건**:
`engine-contract.yml:330` = ★**잡-내부 관련성 필터의 «경로»** 이지 실행이 아니다(선행 census 가 이미 그렇게 분류했다).
⇒ ★**실행 호출처 0 은 참이다.** 남은 히트는 전부 `docs/`·`STATE.md`(서술)과 `contract-roundtrip.mjs`(★**읽기** 의존).

### ⓑ ★★**급소 — byte-stable 이 아니다. 그리고 «기계»를 내가 봤다**

★**스크립트 자신이 선언한다**(머리주석): 「the output is **not byte-reproducible** (the build embeds
paths) … Verify a regeneration by re-running the tests, **not by diffing the zips**」.
★**선행 실측도 있다**(`docs/worklog/2026-09-05-ktf-lgt-key-reach-fixtures.json` `reproducibility`):
「★**md5 는 달라진다**(빌드가 경로를 박는다)」.

★★**그런데 나는 «주장»을 인용하는 데서 멈추지 않고 그 기전을 커밋된 산출물에서 직접 봤다** —
zip → `00000000.jar` → ARM 바이너리를 풀어 `strings` 로 세니 **경로형 문자열이
`keydraw_ktf` 41건 · `keydraw_lgt` 39건**이고, 그중 하나가
★**`/Users/jun0m1/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src`** 다.
⇒ ★★**`ubuntu-latest` 는 `/home/runner/...` 를 박는다 ⇒ 재생성-비교는 «flaky» 가 아니라 «구조적으로 영구 red»다.**
★**이것이 ⒜ 를 탈락시킨 «측정»이다** — 제안이 경고한 「상시 red 는 꺼진다」가 아니라 그보다 강하다:
★**러너는 이 바이트를 «만들 수 없다».**
※무거운 재생성(네트워크 클론 + nightly 빌드)을 돌리지 않고 이 결론에 닿았다 — ★**같은 답에 더 싼 자가 있었다.**

### ⓒ 도구 가용성 — ★**CI 는 `@stable` «전건»이다**

필요: ⑴`dlunch/wipi` **네트워크 클론** ⑵**nightly + `rust-src`**(`-Zbuild-std` 가 둘 다 요구) ⑶`thumbv4t-none-eabi`(빌드 std 가 소스에서 만든다).
실측: 워크플로 **5개 전건이 `dtolnay/rust-toolchain@stable`** 이다(nightly 0). 로컬엔 nightly+rust-src 가 **있다**.
⇒ ⒜ 를 올리려면 러너에 **nightly 툴체인 설치 + rust-src + 외부 클론**이 붙는다 — ★그런데 ⓑ 때문에 **그 비용을 치러도 red 다.**

### ⓓ ★**「zip 안의 바이트」는 «아무도 안 보는» 것이 아니다 — 티켓의 값이 그만큼 줄고, 그 사실을 적는다**

| zip | `cargo test --all`(6다리 · 무조건) | `contract-roundtrip`(engine 필터 뒤) | 로컬 러너 | doc-liveness(주간) |
|---|---|---|---|---|
| `keydraw_ktf.zip` | ★**본다** — `wie-ktf/tests/test_key_reach.rs`·`test_resource_reach.rs` 가 `include_bytes!` 후 **부팅** | 본다 (Scenario **E**·**E-res**) | 본다 | 본다 |
| `keydraw_lgt.zip` | ★★**안 본다**(아래 유실) | 본다 (Scenario **F**·**F-res** · `:606-610`) | 본다 | 본다 |

★**두 zip 다 «부팅되고 정확한 정수로 단언된다»**(`key:<code>` · `res:<size>:<sum>`)
⇒ ★**의미 있는 드리프트는 «조용하지 않다».** 티켓 전제(「바로 드러나지 않는다」)는 **절반만 참**이다.
★**그리고 그것이 옳은 축이다** — 생성기 주석이 요구하는 검증이 정확히 「시험을 다시 돌려라」이고, 그 시험들이 그 일을 한다.

★★★**함께 발견한 «진짜» 결손 — `keydraw_lgt.zip` 은 `cargo test` 커버리지가 «0» 이다.**
`b52ed661` 이 `wie_ktf/tests/test_key_reach.rs` 와 **`wie_lgt/tests/test_key_reach.rs` 를 둘 다** 만들었는데
(그 회차 회신이 「두 경로 다 … **부분 완료가 아니라 전건**」이라 적었다), ★**2026-09-16 크레이트 개명
(`wie_lgt` → `wie-lgt`)에서 lgt 쪽이 «따라오지 않았다»** — 현 `wie-lgt/tests/` 에는 `test_helloworld.rs` 뿐이고,
`wie-lgt/src/` 의 `keydraw_lgt` 히트 2건은 **주석**이다. `cargo metadata` 의 시험 타깃 전수에도 없다.
⇒ ★**LGT 의 zip 은 「paths 필터 뒤(Scenario F)」와 「주간·로컬」만이 덮고 «무조건 도는 축»이 0 이다.**
★**이 회차가 고치지 않았다** — 시험 «파일 신설»은 declare 가 아니고 티켓 범위 밖이다. ⇒ **별 회차 축**으로 선언에 적었다.

### ⑶ ★남은 방어선이 «산다» — 개악 대조 2칸

`contract-roundtrip.mjs:262-287` 이 그 `.sh` 를 **텍스트로 파싱**해 `BAR_H`·`KeyCode::X => N` 표·`res.bin` payload 를
읽고 계약과 대조한다. 내가 일부러 뒤집었다(백업+`trap` 자동 원복 · 커밋 0):

| 개악 | 결과 |
|---|---|
| `BAR_H` → `BAR_H_RENAMED`(위치자 표류) | ★**rc=1** `Error: keydraw fixture: cannot read const BAR_H…` |
| `KeyCode::Key1 => 999`(계약 불일치) | ★**rc=1** `Error: keydraw fixture paints 999 …` |
| 원복 후 | `git diff --stat` **공집합** |

⇒ ★**fail-closed 가 공허하지 않다.** 이것이 declare 가 «가리킬» 대상이다.

### ⑷ 고른 갈래 = ★**⒝ declare(NOT-RUN)** · 탈락 둘의 이유

| 갈래 | 판정 | 이유 |
|---|---|---|
| ⒜ 재생성-비교 CI 배선 | ★**탈락 — «불가능»** | ⓑ: 러너가 이 바이트를 만들 수 없다(경로 박힘) ⇒ **영구 red** ⇒ 꺼진다. ⓒ 의 nightly+클론 비용을 치러도 마찬가지다 |
| ⒞ 간접 축 강화(zip «안»을 보게) | ★**탈락 — «틀린 축»** | 생성기 주석이 「zip 을 diff 하지 말고 시험을 다시 돌려라」로 **명시 기각**한 방법이고, 행동은 ⓓ 가 **이미** 단언한다. zip 파싱 코드만 늘고 잡는 것은 «무의미한 바이트 차이»다 |
| ★**⒝ declare** | ★**채택** | `AGENTS.md` 의 `smoke_gate.sh` 선례와 **같은 형식** — 「local only, and structurally so」 + 이유 + ★**대신 무엇이 지키는가** + ★**무엇을 여전히 못 잡는가** |

### 대가 — ★**declare 는 «안전»이 아니다**

- ★**드리프트가 여전히 조용한 구간이 있다**: 바이트가 바뀌었는데 **게스트의 출력·픽셀이 안 바뀌면** 아무도 모른다.
  ★**그것은 설계다**(행동을 지키는 축이니) — 그러나 「declare 했으니 안전」이 아니다.
- ★**`keydraw_lgt.zip` 의 무조건 축이 0 이다**(위 유실). ⇒ **별 회차**가 그 시험을 되살려야 한다.
- ★**선언은 기계가 아니다** — `check-engine-runner-fixtures` 는 러너 «픽스처»를 보고 이 «생성기»는 보지 않는다.
  ⇒ 이 블록이 낡으면 **아무도 울지 않는다**(선례 `smoke_gate.sh` 도 같은 성질이다).
