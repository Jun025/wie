## [2026-09-16] 조각 D — base swap 을 «하지 않았다»: 중심 결정이 도구로 닫히지 않는다 (wie-p3-slice-d-merge-upstream-main-as-base)

**무엇을** — ★**머지하지 않았다.** 이 회차의 산출은 «왜 지금 못 하는가»를 **수와 실행 출력**으로 고정한 것이다.
제품 코드 **0줄** · 머지 예행은 격리 worktree 에서 돌고 **제거**됐다. 채택 제안 `2026-09-16-p3-remaining-slices-plan#p3`.

**왜** — 티켓 Contract 4 가 「★막히면 «멈추고 보고»하라. 억지로 밀지 마라」를 명시했다.
★**막힌 자리가 «크기»가 아니라 «결정»임**을 아래가 보인다.

### 막은 것 ⑴ — 중심 결정이 «양쪽 다 검증 불가»다

조각 A 가 원인을 `wie-lgt/src/runtime/wipi_c.rs` 의 **graphics 27줄 배선**으로 좁혔고, 선택지를 둘로 적었다.
★**이 회차가 그 둘을 다시 재서 «어느 쪽도 지금 검증할 수 없다»를 확인했다**:

| 선택지 | 얻는 것 | 잃는 것 | 검증 가능? |
|---|---|---|---|
| ⒜ upstream LGT 전용 유지 | ★제안이 명시한 **headline benefit**(「LGT 전용 그래픽 0→1,095줄」) | ★**`keydraw_lgt` FAIL**(조각 A 실측 3/3) | ✗ 코퍼스 부재 |
| ⒝ 공용 구현으로 복귀 | `keydraw_lgt` PASS(조각 A 실측 2/2) | ★**그 1,095줄을 버린다 = 제안의 benefit 을 스스로 취소** | ✗ 코퍼스 부재 |

★★**그리고 이 회차가 «새로» 쟀다 — ⒜ 를 고르면 그 테스트를 «다시 만들 수도 없다».**
게스트 SDK(`dlunch/wipi`)에 `lgt` feature 가 **있고** `wipic-sys/src/lgt/graphics.rs` 도 **있다**. 그런데:

```
$ ledger-grep -c -i 'lgt' <sdk>/wipi/src/framebuffer.rs
0
```

★**고수준 `Framebuffer` 타입에 `lgt` 분기가 «0건»이다** — `width`/`height`/`bpl`/`bpp`/`buf` 를
**feature 와 무관하게** 공용 레코드 배치로 직접 읽는다(`read_fb()`).
⇒ ★**`--features lgt` 로 다시 빌드해도 upstream 의 `LgtFramebuffer` 와 여전히 안 맞는다.**
⇒ ★★**⒜ 를 고르면 우리의 «유일한 LGT 그리기 픽스처»를 잃고, 가용 도구로는 대체품을 만들 수 없다.**
※이것은 **upstream 자신의 SDK 가 가진 공백**이다 — upstream 도 그 경로를 SDK 게스트로 시험한 적이 없어 보인다(추정 · 무는 것 없음).

★**부수로 하나 닫았다**: 조각 A 가 남긴 「upstream 이 픽셀을 한 겹 아래로 내줄 수 있다」는 **참이다** —
upstream `get_framebuffer_pointer` 가 `resolve_framebuffer(handle).framebuffer.0.buf.0` 를 **실제로 준다**.
⇒ upstream ABI 는 ★**자기완결적**이다. ★**그러나 그 사실이 결정을 풀지 «않는다»** — 우리 SDK 게스트는
그 접근자를 **부르지 않고**(`framebuffer.rs` 가 레코드를 직접 읽는다) 실게임이 어느 쪽인지는 **여전히 코퍼스 문제**다.

### 막은 것 ⑵ — 선행 둘이 «done 인데 착지하지 않았다»

| 선행 | done | PR | main 반영 |
|---|---|---|---|
| 조각 A | Y | **#160 OPEN** | ✗ |
| 조각 C | Y | **#159 OPEN** | ✗ |

★**조각 C 는 머지를 «줄이려고» 있는 조각이다**(74 → 67 · 그 PR 제목이 그렇게 적는다).
실측: 오늘 `main`(`28fb4364`) 기준 머지 예행 미해결 = ★**74**(A 167 · R 146 · D 104 · UU 35 · AU 19 · UD 17 · AA 3).
★**그 74 중 «8건»이 정확히 #159 가 지우는 파일이다**(정규화 교집합 · underscore→hyphen):

```
wie-wipi-java/src/classes/java.rs                                  wie-wipi-java/src/classes/java/lang.rs
wie-wipi-java/src/classes/java/io.rs                               wie-wipi-java/src/classes/java/lang/out_of_memory_error.rs
wie-wipi-java/src/classes/java/io/interrupted_io_exception.rs      wie-wipi-java/src/classes/java/lang/virtual_machine_error.rs
wie-wipi-java/src/classes/java/io/unsupported_encoding_exception.rs wie-wipi-java/src/lib.rs
```

⇒ 지금 머지하면 ★**조각 C 가 이미 «재서» 내린 결정을 손으로 다시 내리고**, 그 뒤 #159 와 **같은 8파일에서 충돌**한다.
※조각 C 자신이 「그 8건은 D 가 upstream 쪽을 고르면 해소된다」고 적었다 — **결정은 이미 있다**.
★**그래서 이 항은 «정확성» 블로커가 아니라 «중복·충돌» 블로커다.** ⑴과 계급이 다르다는 것을 흐리지 않는다.

### 규모 (참고 — 이것만으로는 멈출 사유가 아니다)

`UU` 35파일에 ★**충돌 헝크 75개** · `UD`+`AU`+`AA` ★**39파일**이 add/delete 판단 대상 ⇒ 합 **114 결정**.
최대 헝크: `Cargo.lock` 13 · `wie-wipi-c/src/api/graphics.rs` 4 · `wie-lgt/src/runtime/svc_ids.rs` 4.
★**티켓이 `size: L` · `risk: high` · 180분으로 이미 그것을 알고 발권했다** ⇒ ★**크기는 블로커로 쓰지 않는다.**

### 이 회차가 «하지 않은» 것

★머지 **0**(예행은 `--abort` + worktree 제거) · 제품 코드 **0줄** · `compile_model.rs` 무접촉 ·
조각 E 무접촉 · upstream 발신 **0** · ★**27줄 결정을 «임의로» 내리지 않았다.**

### ★부수 발견 — 연번 `0113` 을 두 열린 PR 이 «둘 다» 쓴다

```
$ node scripts/check-docs-report-serial.mjs --next-serial
check-docs-report-serial: 디스크 기준 0113 · 열린 PR claim [0113(#159) 0113(#160)] ⇒ 다음 빈 번호 0114
```

★**착지 «전»이라 검사기는 `OK — 중복 연번 0` 을 낸다**(그것은 머지된 트리만 본다) ⇒
★**둘 다 착지하면 그때 `main` 이 red 가 된다.** `AGENTS.md` 규율은 「**착지하지 않은 쪽을 옮겨라**」인데
★**지금은 «둘 다» 미착지**다. ⇒ ★**이 회차는 남의 PR 을 고치지 않았다** — 총괄 판단으로 올린다(둘 중 하나가 0114 로 이동하면 닫힌다.
★이 리포트가 0114 를 쓰므로 그 회차는 **0115** 를 잡아야 한다).

### 사용자 영향

없음(문서 전용). ★**다음 회차가 «무엇을 먼저 정해야 하는지»가 한 줄로 좁혀졌다** — 27줄 배선의 ⒜/⒝.

### 재개 조건 — ★이 중 하나가 서면 조각 D 가 돈다

1. ★★**292 코퍼스(특히 LGT 52건)** — ⒜/⒝ 를 «재서» 고를 수 있게 된다. **가장 곧은 길**이다.
2. ★**운영자·총괄이 ⒜ 또는 ⒝ 를 «명시로» 고른다** — 그러면 검증 불가를 **알고 받는** 결정이 되고 D 는 즉시 집행 가능하다.
   (⒜면 `keydraw_lgt` 를 **기대 실패로 재분류**해야 하고, ⒝면 제안의 benefit 한 줄을 **철회**해야 한다.)
3. ★**upstream SDK 가 `framebuffer.rs` 에 LGT 분기를 넣는다** — 그때는 픽스처를 다시 만들 수 있어 ⒜의 대가가 사라진다.

★**그리고 «순서»는 셋 중 무엇이든 #159 착지 «뒤»다**(위 8건 중복 때문 — 정확성이 아니라 비용 축).
