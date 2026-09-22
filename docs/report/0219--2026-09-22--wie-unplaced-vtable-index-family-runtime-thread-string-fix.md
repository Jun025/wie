## [2026-09-22] 제품 데이터 파일 안의 «거짓 안전 근거»를 지웠다 — `Runtime.exit` 는 있고, 안전한 이유는 다른 것이다 (wie-unplaced-vtable-index-family-runtime-thread-string-fix)

게이트②(PR **#254** · `request-changes`) 승계. ★**고친 것은 «문장 둘»이고 diff 는 문서·주석뿐이다** —
`lgt_java_abi.toml` 의 **비-주석 줄은 바이트 동일**(sha256 대조 `db5598ca…` ↔ `db5598ca…`)이고 `.rs` diff 는 **0줄**이다.
검수자가 「데이터 3행 «자체»는 옳다 · 시험·개악 대조·CI 는 재실행 요구 없음」으로 닫은 범위를 그대로 지켰다.

## ⑴ ★차단 1 — 제품 데이터 파일 «안»의 문장이 거짓이었다

`wie-lgt/data/lgt_java_abi.toml` 의 `java/lang/Runtime` 주석이 이렇게 적고 있었다:

> ~~`the JVM this runs on has no Runtime.exit at all, so there is nothing destructive for a
> misplaced dispatch to land on`~~

★**거짓이다.** 실측 둘:

```
$ sed -n '19,35p' ~/.cargo/registry/src/*/rustjava-runtime-0.1.1/src/classes/java/lang/runtime.rs
  JavaMethodProto::new("exit", "(I)V", Self::exit, MethodAccessFlags::PUBLIC),   ← 있다

$ grep -n -A3 'name = "rustjava-runtime"' Cargo.lock
3377:name = "rustjava-runtime"
3378-version = "0.1.1"
3379-source = "registry+https://github.com/rust-lang/crates.io-index"
```
그리고 그 `exit` 는 무해하지도 않다 — `wie-jvm-support/src/runtime.rs:188` 이
`self.system.platform().exit()` 로 **WIPI `MC_knlExit` 와 같은 종료 경로**를 탄다(주석이 자인한다).

★★**그런데 «결론»은 산다 — 기전이 다르다.** `wie-lgt/src/runtime/java/jvm_support/vtable.rs`:
```
:181   unplaced += 1;  continue;                                        ← 행이 없으면 index 를 못 받는다
:190   methods.resize(methods.len() + unplaced, { target: 0, method: None });   ← 꼬리에 빈 칸으로만 예약
```
⇒ `exit` 는 **호출 가능한 target 으로 표에 올라오지 않는다.** 10~13 중 어디로 잘못 분기해도 닿지 못한다.

★**왜 이것이 critical 이었나**(검수자 판정 그대로): 거짓 문장이 **하필 그 표 바로 위, 제품 데이터 파일 «안»**
에 있다. 다음 사람이 index 10 을 채우려 할 때 **가장 먼저 읽는 자리**이고, 「`exit` 가 아예 없다」를 믿으면
★**10 을 채우는 것이 «공짜»라고 결론 낸다.** ★**근거 없이 비우는 것보다 «거짓 근거»로 비우는 것이 나쁘다 —
자신감을 주기 때문이다.**

★**그래서 새 주석은 «반대 방향»을 가리킨다**: 행을 넣으면 그 메서드가 **꼬리 예약에서 «도달 가능한 target»으로
옮겨가고**, 도출이 맞다면 그 target 이 게임을 끝낸다. ⇒ **index 10 을 비워 둔 결정은 이 정정 뒤에 «강해진다».**
★**index 10 은 채우지 않았다**(승계 범위 밖 · 티켓 명시).

★**핀 표기도 정정했다**: `dlunch/RustJava@5b84dd1` 은 **현 트리의 핀이 아니다** —
`Cargo.toml` 이 「upstream consumes RustJava from crates.io, so the git `rev` pin is gone」이라 적는다.

★**퍼진 세 곳을 모두 고쳤다**: `lgt_java_abi.toml` 주석 · `docs/report/0218` ⑷ · `.done.md` ⑴(정정 표기).
★**지우지 않고 «정정 표기»로 남겼다** — 무엇이 틀렸는지가 남아야 같은 자리를 다시 밟지 않는다.

## ⑵ ★차단 2 — 「렌더 대열 합류」의 근거 축 (전문 = `0218` ⑸-a)

★**PASS 의 술어는 `content ∨ paints>=1` 이다** ⇒ 「PASS · paints 12」로는 **«그렸다»와 «12번 빈 프레임을
칠했다»가 구별되지 않는다.** 표제가 «렌더»를 주장하는데 그 축이 없었다.

| 타이틀 | 판정 | paints | `content` | `last_frame_content` | `distinct_colors` | `nondominant_pct` | `center_nonuniform_pct` |
|---|---|---|---|---|---|---|---|
| ★**체스마스터** | **PASS** | 10~12 | ★**true** | ★**true** | ★**41** | ★**84.7%** | ★**83.0%** |
| 현영맞고2006 | PASS | 95 | true | true | 512 | 90.3% | 85.1% |
| 메이플스토리2007 | PASS | 106 | true | true | 255 | 8.8% | 18.5% |
| 놈3 | PASS(5회 중 3) | 178~201 | true | true | 99 | 33.5% | 22.2% |
| 붉은보석 | PASS | 377 | true | true | **2** | **1.9%** | **4.2%** |

★★**판정 — 값이 표제를 «지지한다». 그래서 낮추지 않았다**(합격선 ⑶의 반대 분기).
결정적인 것은 **`center_nonuniform_pct` 83.0%** 다: 이 축은 소스 주석이 밝히듯 **크롬(상태바·소프트키·테두리)을
제외한 중앙**의 균일도를 묻고, ★**«UI 껍데기만 그린 빈 캔버스»를 «진짜 게임 프레임»과 가르려고** 만들어졌다.
껍데기는 ~0 에 머문다. 체스마스터는 83% 이고, 색 41종·비배경 84.7% 로 **기존 대열 안에서도 진한 쪽**이다
(`붉은보석`은 색 **2**·1.9% 로 PASS 한다).

★**약한 축도 적는다**: `paints 10~12` 는 대열 최저이고 **최종 프레임의 중앙은 15.7%** 로 떨어진다.
⇒ ★**「렌더한다」는 참이고 「끝까지 게임 화면을 유지한다」는 이 축으로 주장하지 않는다.**

## ⑶ ★재측을 «한» 이유 — 검수자는 불요로 닫았다

검수자 문안은 「그 스윕 JSON 에 **이미 있던** 값을 옮겨 적으면 끝 · 재측 불요」였다.
★**그 JSON 이 없었다** — 직전 회차의 46파일 스윕 출력이 임시 디렉터리와 함께 사라졌다(`game_lab/reports/`에
남은 것은 09-21 배틀몬스터 2건뿐). ⇒ 옮겨 적을 원본이 없으므로 **해당 타이틀만** 다시 돌렸다.

★**그래도 범위는 지켰다**: 같은 커밋 `894ee5d3` 을 **새로 release 빌드**(8m13s — ★캐시 적중이 아니다,
`0218` ⑺ 이 남긴 그 규율)해 `--inject` 로 5타이틀만 실행했다. **데이터·시험·엔진 무접촉** · 46파일 스윕
재실행 **안 함** · 개악 대조 재실행 **안 함**.

★**richness 3축은 3회 실행에서 값이 «완전히 동일»했다**(41 / 84.7 / 83.0) — 판정·`ticks`·`paints` 와 달리
부하에 흔들리지 않는다. ⇒ **절대값으로 인용해도 되는 몇 안 되는 축**이다.

★**놈3 5회 중 2회 FAIL**(`panic during '08_OK'/'09_UP': InvalidMemoryAccess(273416884)`) — ★**선재 흔들림이고
이 회차와 무관하다**(이 회차 diff 는 주석뿐 · 같은 실행에서 richness 3축은 PASS/FAIL 양쪽에서 동일). AGENTS.md
⑴~⑷ 대로 재실행해 3/5 PASS 를 확인했다. `impl ClassInstance` 의 `unwrap()` 무리는 **PR #253** 소관이다.

## ⑷ 비차단 2건도 고쳤다

- 「**행 3개**」 → ★**«3 클래스 · 행 5개»**(`Runtime` 블록 신설 11·12·13 + `Thread` 13 + `String` 19).
  본문·시험은 처음부터 다섯을 전부 다뤘으므로 **틀린 것은 «수 표기»뿐이다.** worklog `summary` 와 제안 4 제목
  (「네 인덱스」 → 「다섯 인덱스」)도 같이 고쳤다.
- 「**엔진 로직 0줄**」 — 참이다. `.rs` 가 0줄은 «아니지만»(+49 −5) 바뀐 두 훅이 **전부 `#[cfg(test)] mod tests`
  안**이다. ⇒ ★**「시험 1건」이 곧 그 `.rs` 변경**임을 `0218` 머리에 한 줄로 적었다.

## ⑸ 손대지 않은 것

데이터 5행 · 시험 · 엔진 · index 10 · `#252` 축 · `STATE.md` · 게임 바이트 · `game_lab/` 커밋 — **전부 0**.
`--amend`·force-push **없음**(후속 커밋 1개). 머지 **0**.
