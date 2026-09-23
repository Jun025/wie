## [2026-09-23] `--max-ticks` 백스톱이 `--inject` 축을 무효화하는 결함 — 조건은 재현됐고 결함은 이미 닫혀 있었다 (wie-validate-max-ticks-backstop-silently-voids-inject-axis)

**무엇을**: 채택 제안 `2026-09-23--featurephone-register-chessmaster-aot-java#p0`(원 worklog 는 otterpebble repo)이
지목한 결함을 재측정했다. ★**결과는 「재현 불가」다 — 처방은 약 5시간 «먼저» 착지해 있었다**(PR #256 ·
머지 `cb2e6dca` · 2026-09-23 07:39 KST · 티켓 `wie-validate-inject-can-pass-without-injecting-anything-fix`).
티켓 계약 1⒝이 정한 분기대로 **고치지 않았다**. 코드 변경 0.

**왜**: 제안은 정확했다. 틀린 것은 「아직 열려 있다」는 전제뿐이고, 그 전제는 제안이 쓰인 시점(featurephone
회차가 함정을 실제로 밟은 시점)에는 **참이었다**. 이 회차가 한 일은 그 사실을 기계로 확정한 것이다.

**사용자 영향**: 없다(코드 무접촉). 확정된 것은 「버튼을 한 번도 눌러보지 않은 타이틀이 «확인됨»으로 등재되는
경로가 지금은 막혀 있다」는 것이고, 그 근거가 이 문서다.

---

### ⑴ 재현 실측 — 조건은 100% 재현되고 판정만 다르다

★**제안이 쓴 명령을 글자 그대로**, 제안이 지목한 그 타이틀(sha256 `4ece6eee…e276` — 제안 본문의 등재 해시와
동일 · ★**파일명은 적지 않는다**, Constraint 9)에 대해 **release 빌드**로 돌렸다. 기본 `--max-ticks` 유지:

```
--inject --boot-secs 15 --action-secs 1.2 --timeout 240 --shotdir <임시>
```

| 축 | 고침 «전» (`6e194590`) | 고침 «후» (현 HEAD `6935e829`) |
|---|---|---|
| `result` / rc | ★**`PASS` · rc=0** | ★**`UNMEASURED` · rc=2** |
| `reason` | `booted + rendered + survived input sequence (visual correctness NOT checked)` | `--inject delivered 0/27 input steps (run ended: max-ticks) — input survival NOT measured (otherwise: …)` |
| `ticks` | **50,000,000**(백스톱 정확히 소진) | **50,000,000** |
| `ms` | **6,679**(= 6.7초) | 8,084 |
| shotdir | **0장** | **0장** |
| `stop` · `input_steps` | ★**필드 자체가 없다** | `max-ticks` · `0/27` |

원문 — 고침 «전»:

```
{"file":"<게임파일 경로 생략>","platform":"lgt","result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)","ticks":50000000,"paints":2,"content":true,"last_frame_content":true,"distinct_colors":36,"nondominant_pct":17.6,"center_nonuniform_pct":9.6,"last_frame_distinct_colors":36,"last_frame_nondominant_pct":17.6,"last_frame_center_nonuniform_pct":9.6,"ms":6679}
shotdir 장수: 0
```

원문 — 고침 «후»:

```
{"file":"<게임파일 경로 생략>","platform":"lgt","result":"UNMEASURED","reason":"--inject delivered 0/27 input steps (run ended: max-ticks) — input survival NOT measured (otherwise: booted + rendered + survived input sequence (visual correctness NOT checked))","stop":"max-ticks","input_steps":0,"input_steps_total":27,"ticks":50000000,"paints":2,"content":true,"last_frame_content":true,"distinct_colors":36,"nondominant_pct":17.6,"center_nonuniform_pct":9.6,"last_frame_distinct_colors":36,"last_frame_nondominant_pct":17.6,"last_frame_center_nonuniform_pct":9.6,"ms":8084}
shotdir 장수: 0
```

⒜ **`--max-ticks` 기본값 = 50000000** — `--help` 실측(`[default: 50000000]`). 제안의 「5,000만」 일치.

⒝ ★**조건은 재현되고 결함은 재현되지 않는다.** 백스톱이 `--boot-secs 15` 가 끝나기 «전»에 6.7초로 소진되고,
키는 **0/27**, shotdir **0장** — 제안이 적은 「6.2초 · shotdir 0장」과 **같은 형상**이다(6.2 ↔ 6.7 차이는 호스트
부하. 이 회차 load1 34.6 · idle 17.9%). ★**달라진 것은 그 형상에 붙는 «이름»뿐이다: `PASS` → `UNMEASURED`.**
★**두 축의 픽셀 지표가 전건 동일하고**(paints 2 · colors 36 · nondominant 17.6% · center 9.6%)
★**그 값이 제안 ⑶의 «부팅 축» 수치와도 바이트 동일**하다 ⇒ 런이 실제로 부팅에서 끝났다는 교차 증거다.

⒞ ★**「부팅 축과 구별되지 않는다」는 제안 문면을 그대로 옮기지 않는다 — 어긋난 대로 적는다.**
실측하면 **부팅 축**(`--inject` 없음)의 `reason` 은 `booted + rendered` 로 **다른 문자열**이다.
★**실제 구별 불가 상대는 «정상 주입 PASS»이고, 그쪽이 더 나쁘다** — 고침 전의 0스텝 PASS 는
`keydraw_ktf --inject`(27/27 실주입)의 `reason` 과 ★**바이트 동일**하다. 그리고 고침 전 JSON 에는
`stop`·`input_steps` 가 **없었으므로** 두 회차를 가를 필드가 ★**하나도 없었다**(`paints` 2 ↔ 55 는 수이지 판정이 아니다).

⒟ ★**「못 쟀다 ≠ 통과」 관용은 이 저장소에 «이미» 있었고 착지한 처방이 그것을 재사용했다 — 새 이름 0.**
정본 인용: `scripts/ktf-image-sweep.py:27`(「★Exit 2 means "could not measure" … never」) ·
`scripts/checker-census.mjs:57` · `scripts/check-ledger-c-validity-scope.mjs:48` ·
`scripts/check-upstream-new-workflows.mjs:78`(ci-presence 규율 — 「fail-closed: "could not measure" is not
"nothing new"」) · `scripts/cargo-metadata.mjs:8`.
★**착지한 처방이 그 중 첫째를 «이름으로» 인용한다** — `wie_cli/src/bin/wie_validate.rs:26-28`:
「Exit 2 for it is this tree's existing convention for that state (AGENTS.md, `ktf-image-sweep.py`: …)」.
문서 축은 `AGENTS.md:187`.

### ⑵ 처방 전/후 — 그리고 「먼저」를 무엇으로 판정하는가

전/후는 ⑴의 표다. 착지한 처방의 실체(`git diff 6e194590 cb2e6dca -- wie_cli/src/bin/wie_validate.rs`):
`Outcome` 에 `unmeasured`/`stop`/`input_steps`/`input_steps_total` 4필드 + `verdict()`/`exit_code()` +
`inject_unmeasured()`/`stop_cause()` 2함수 + 단위테스트 4개.

★**「먼저」를 «시간 비교»로 추정하지 않는다 — 계약 2⒞가 금지한 그 형태를 피해 있다.**
판정자는 **주입 루프가 실제로 돌린 횟수를 센 카운터**다:

```rust
fn inject_unmeasured(inject: bool, passed: bool, input_steps: u64) -> bool {
    inject && passed && input_steps == 0
}
```

`input_steps` 는 스케줄 이벤트의 **DOWN 반쪽에서 dispatch «전»에** 증가한다(`wie_validate.rs` 의 `if *down
{ input_steps += 1; }`) ⇒ ★**「백스톱이 먼저 걸렸다」를 «벽시계 대소»로 추론하는 대신 «키가 0개 나갔다»는
직접 관측으로 대체**했다. 부수 효과로 이 술어는 원인을 가리지 않는다 — `clean exit`·`deadline` 으로 0스텝이
나도 같은 판정이다(그쪽이 옳다. 어느 쪽이든 입력 축은 안 쟀다). ★**어느 것이 끝냈는지는 `stop` 이 따로 말한다.**

계약 2⒜(`--inject` 없는 회차 불변)도 술어의 첫 항 `inject &&` 으로 구조적으로 보장되고, 단위테스트
`zero_injected_steps_is_not_a_pass_test` 가 플래그 off × (passed 2 × steps 3) **6조합 전건**을 잠근다.

### ⑶ 개악 대조 4종 — 출력 원문

★**계약 3 의 네 칸을 «현 트리»에 대해 실행했다.** 즉 이 절은 새 처방의 검증이 아니라
★**착지한 처방이 그 합격선을 실제로 넘는지**의 검증이다.

**⒜ 주입 0스텝 + 백스톱 선착 = PASS 가 아니다** — 커밋 fixture(`draw_j2me.jar` · 부팅 중 페인트 1장)로
`--max-ticks` 를 낮춰 같은 형상을 만들었다. 4자리 전건 동일:

```
N=1000    rc=2 result=UNMEASURED stop=max-ticks steps=0/27 paints=1 content=true ticks=1000
N=10000   rc=2 result=UNMEASURED stop=max-ticks steps=0/27 paints=1 content=true ticks=10000
N=100000  rc=2 result=UNMEASURED stop=max-ticks steps=0/27 paints=1 content=true ticks=100000
N=1000000 rc=2 result=UNMEASURED stop=max-ticks steps=0/27 paints=1 content=true ticks=1000000
```

**⒝ 정상 주입 회차 = 여전히 PASS**(기본 백스톱 · `--expect-last-frame` 까지 켜서):

```
--- keydraw_ktf rc=0
{"file":"test_data/keydraw_ktf.zip","platform":"ktf","result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)","stop":"deadline","input_steps":27,"input_steps_total":27,"ticks":26573429,"paints":55,"content":true,"last_frame_content":true,...,"ms":20104}
--- keydraw_lgt rc=0
{"file":"test_data/keydraw_lgt.zip","platform":"lgt","result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)","stop":"deadline","input_steps":27,"input_steps_total":27,"ticks":22786311,"paints":55,"content":true,"last_frame_content":true,...,"ms":20079}
```

★`paints 55` 는 AGENTS.md 가 적은 유휴 범위(48~55) 안이다 — load 34.6 에서도 판정·카운트 둘 다 정상.

**⒞ `--inject` 없는 회차 = 전과 동일**(백스톱이 걸려도 PASS 유지 · 같은 fixture · 같은 `--max-ticks 1000`):

```
rc=0
{"file":"test_data/draw_j2me.jar","platform":"j2me","result":"PASS","reason":"booted + rendered","stop":"max-ticks","input_steps":0,"input_steps_total":0,"ticks":1000,"paints":1,"content":true,"last_frame_content":true,...,"ms":332}
```

★**0스텝인데 PASS 이고 그것이 옳다** — `--inject` 가 없으면 잴 입력 축이 애초에 없다. `input_steps_total` 이
**27 이 아니라 0** 인 것이 그 상태의 지문이다.

**⒟ 그 판정을 되돌리면 red** — 두 가지로 개악했다. ★**둘 다 총계 15 가 유지되므로 구문 파손 red 가 아니라
단언 실패 red 다**(저장소가 경계한 오독을 이 수로 가른다).

M1 — 0스텝을 PASS 로 되접기(`inject && passed && input_steps == 0` → `false`):

```
M1 cargo test rc=101
tests::zero_injected_steps_is_not_a_pass_test --- FAILED
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

★그 개악 바이너리의 **런타임**이 곧 고침 전 동작이다 — 계약이 금지한 그 PASS 가 그대로 돌아온다:

```
rc=0
{"file":"test_data/draw_j2me.jar","platform":"j2me","result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)","stop":"max-ticks","input_steps":0,"input_steps_total":27,"ticks":1000,"paints":1,...,"ms":294}
```

M2 — 게이트에 «배달된 수» 대신 «총계»를 먹이기(`outcome.input_steps` → `outcome.input_steps_total`).
★총계는 `--inject` 회차에서 항상 27 이라 게이트가 **영원히 발화하지 않는다**:

```
M2 cargo test rc=101
tests::the_gate_is_handed_the_delivered_count_test --- FAILED
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

복원 후 `git diff` **0줄**(바이트 동일 · sha16 `98527cf0116e5ae2`) · `cargo test -p wie_cli --bin wie_validate`
**rc=0 · 15 passed**.

### ⑷ 계약 4 — 기각된 대안이 어디에도 없다

`scripts/`·`.github/` 전수 `grep -rn 'max-ticks'` → ★**0건**. 호출부 3곳(`scripts/smoke_gate.sh` ·
`scripts/lgt_render_probe.sh` · AGENTS.md 러너 블록) 중 `--max-ticks` 를 올려 부르는 곳은 없다 ⇒
★**「기억에 얹힌 가드」 관례는 채택되지 않았고, 이 회차도 만들지 않았다.**
※`AGENTS.md:187` 의 「on `max-ticks` raise `--max-ticks`」는 이 대안이 «아니다» — 도구가 UNMEASURED 로
말해 «준» 다음에 사람이 읽는 후속 안내이고, 도구 판정을 대체하지 않는다.

### ⑸ 이 회차가 «하지 않은» 것과 그 이유

- **코드 수정 0** — 계약 1⒝(「⒝가 재현 안 되면 그것이 결론이다 — 고치지 말고 적어라」).
- **부분 주입(1~26스텝) 축으로 넓히지 않았다.** 백스톱이 «몇 개 쏜 뒤» 걸리면 여전히
  `PASS … survived input sequence` 다. ★그러나 그것은 이 티켓의 합격선(3⒜ = 0스텝)이 아니고,
  착지한 처방이 그 경계를 **의도로 적어 두었다**(`inject_unmeasured` 주석: 「Zero steps plus a PASS is the
  only combination where the verdict is about an axis the run never touched」). `input_steps: 5/27` 은
  JSON 에 **정직하게 나온다** ⇒ 침묵이 아니라 «판정에 반영되지 않음»이다. ★넓히려면 별 티켓이다.
- **`upstream/main` 머지 0 · `wie_web.js`/`wie_web_bg.wasm` 무접촉 · RustJava 핀 무접촉 ·
  otterpebble 트리 무접촉**(원 worklog 는 `git show origin/main:` 으로 **읽기만** 했다) ·
  게이트·워크플로 무접촉 · 시크릿 출력 0.

### ⑹ 부수 관측 — 다음 사람에게 값하는 것 하나

★**`--max-ticks` 백스톱이 «얼마나 빨리» 타는지의 실측 두 점**(release):
지목된 LGT 타이틀은 **6.7초에 5,000만 틱** · `keydraw_ktf --inject`(debug)는 20.1초에 **2,657만 틱**
⇒ ★**같은 기본값이 타이틀에 따라 «부팅 전»에도 타고 «20초 예산 안»에 안 타기도 한다.**
그래서 제안이 적은 대로 **기본값 상향은 처방이 아니다**(더 빠른 다음 타이틀이 같은 자리를 다시 연다).
지금 그 자리를 막는 것은 상향이 아니라 «0스텝을 PASS 로 부르지 않는 것»이다.

---

### ⑺ 게이트

코드 무접촉 회차지만 AGENTS.md 가 정한 대로 **전건 돌렸다**(「a docs- or `web/`-only diff is not an exemption」):

| 게이트 | rc |
|---|---|
| `cargo fmt --all -- --check` | **0** |
| `cargo clippy --all -- -D warnings` | **0** |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | **0** |
| `RUST_MIN_STACK=4194304 cargo test --all` | **0** |
| `cargo +beta clippy --all -- -D warnings` | **0** |

AGENTS.md 러너 블록도 전건 PASS: `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` ·
`text_j2me.jar --timeout 5` · `keydraw_{ktf,lgt}.zip --inject --expect-last-frame`(rc=0 · 27/27 · paints 55).

진행 중 PR 목록은 `gh pr list -R Jun025/wie`.

### ⑻ 코퍼스 게임명 유입

★**손으로 술어를 다시 만들지 않고 도구를 돌렸다** — `node scripts/corpus-name-inflow.mjs --corpus <코퍼스>`
(★이 레인 worktree 에는 `game_lab/` 이 없어 기본 경로로는 rc=2「못 쟀다」가 난다. 코퍼스는 메인 체크아웃에
있으므로 `--corpus` 로 가리켰고, 그 rc=2 를 「0건」으로 읽지 않았다).
모집단 고유 stem **451개** · 대상 `origin/main...HEAD` 추가·수정 **2파일**(텍스트 2 · 건너뜀 0):

★**유입 0건(BOUNDED 0회/0쌍)** · **판단 필요 0건(SUFFIX-ATTACHED 0회/0쌍)** · PREFIX-EMBEDDED 0회/0쌍.
★두 바구니가 «함께» 0 이라 그 0 은 이 리니지가 반려됐던 그 0 이 아니다.
이 문서가 지목 타이틀을 **이름이 아니라 sha256 앞 8자**로만 가리킨 것이 그 0 의 이유다.

<!-- corpus-name-inflow v1 subjects=2 tree=da4c0897e3832343 B=0/0 P=0/0 S=0/0 -->
