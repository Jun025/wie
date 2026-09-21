## [2026-09-21] 추적 6수에 측정 조건과 「절대값 재인용 금지」 한정을 붙였다 (wie-lgt-docs-cite-load-dependent-trace-counts-as-absolutes)

**무엇을.** 2026-09-21 에 착지한 `docs/lgt.md` §7 배너와 `docs/lgt_abi.md` 의 두 자리가 **부하 의존
추적 수를 캐럿 없는 절대값**으로 박고 있었다. 수를 지우지 않고 **조건 + 한정**을 붙였다(⑴), 그리고 같은
회차 산출물의 수사·한정·전사 오류 3건을 함께 고쳤다(⑵⑶⑷).

**왜.** 그 회차 자신이 「같은 타이틀·같은 바이너리에서 `distinct_colors` 가 2 ↔ 93 으로 움직인다 —
절대값으로 인용하지 마라」를 규율로 세웠는데, **그 규율이 `dc`·`ticks` 에만 적용되고 추적 수에는 걸리지
않았다.** 게이트② 재측이 `184`/`184`/`130`/`3,584`/`187` — **최대 18배** — 를 읽어 그것을 실측으로
드러냈다(소견 F2). ★**논지(「§7 이 영구 블록이라 한 루프가 실제로 돈다」)는 그대로 견고하다** — 흔들린
것은 수뿐이고, 그래서 처방은 «삭제»가 아니라 «조건 명기»다.

**사용자 영향.** 산문만 바뀐다. 엔진·빌드·웹 표면 무영향(변경 파일 3개 전부 `docs/`).

### 고친 네 자리 — before → after

| # | 자리 | before | after |
|---|---|---|---|
| ⑴ | `docs/lgt.md` §7 배너 | `` `getNextEvent` **73×**, `dispatchEvent` **73×**, `paint` **21×**, `drawImage` **193×**, `createImage` **44×** — `` (조건·한정 0) | 같은 다섯 수 + **같은 줄 괄호 한 구**: `(★one run — release wie_validate at 394fde8b, --inject --action-secs 0.3, RUST_LOG=debug, 2026-09-21, loadavg 47–130; load-dependent — do not re-cite as absolutes, see below)`. 뒤 문단에 게이트② 재측 `184×/184×/130×/3,584×/187×` **up to 18× higher** + 「fixed wall-clock budget 이라 tick 수에 비례」 기전 + ★**「두 측정이 갈렸다는 사실 자체가 남길 값」** + 「부하 불변인 주장은 전건 **non-zero** 뿐」 |
| ⑴′ | `docs/lgt_abi.md` §8 `per-frame render driver` 행(`SUPERSEDED 2026-09-21`) | 같은 형태(`73×`·`21×`·`193×`·`pushCard 1×`) | 같은 한정을 한 구로 압축해 부착(조건 · 18배 · non-zero) |
| ⑵ | `docs/lgt_abi.md` §7b 배너 | `Two of them are named in this section's own buckets` — **바로 다음 줄이 셋을 나열하고 `all three` 라 적는다** | `All three of them are named …` (사실은 맞았고 수사만 틀렸다. `both bucket labels` 는 라벨이 둘이라 **옳아서 건드리지 않았다**) |
| ⑶ | `놈3` 렌더 주장 3곳 | `놈3` (69 paints / 93 colours) — 한정 0(같은 문장에서 메이플에는 `in-game intro scene` 을 붙였다) | `lgt.md` 배너 = `splash only, and shallow`(「시보구게임연구실」 로고 + 초록 스프라이트 · 예산 안에 메뉴 미도달) · `lgt_abi.md` §7b 배너 = `splash only and shallow` · §8 재기준선 행 = `splash only / shallow` |
| ⑷ | `docs/report/0210` §한계 | `PASS 후보만 7회` · `별도로 5회` · `loadavg 72~130` | `6~7회` · `4회(50초 3회 + 113초 1회 = 표의 +0/4 큰예산)` · `loadavg 47~130` + 정정 사유 각주 |

### ⑷ 는 «전사 오류»다 — 판정을 뒤집지 않았다

세 수 모두 회신
(`~/orchestrator/reports/wie-aot-java-render-0-premise-is-superseded-by-upstream-rebaseline.done.md`)과
어긋났고, ★**앞의 둘은 `docs/report/0210` 자신의 전수표와도** 어긋났다 — 표는 현영맞고 `6/6` · 메이플
`5/6`+추적 1회(※주석: 7회차 중 1회가 `--action-secs 0.3`) · 놈3 `5/7` · 일지매 `0/3 (+0/4 큰예산)` 이다.
⇒ 「7회」는 세 PASS 후보의 최대만 적은 것이고 「5회」는 자기 표의 4를 잘못 옮긴 것이다. 판정·전수표·§7
결론·cp 사료는 한 줄도 건드리지 않았다.

### 세지만 고치지 않은 것 (범위 밖 — 티켓 Contract 4)

- **`docs/project-kb/10_deep-assets.md:13`** — `놈3`(69 · 93) 에 splash 한정 **부재**. 이 티켓의 파일
  범위(`docs/lgt.md`·`docs/lgt_abi.md`) 밖이라 **세고 두었다.**
- **두 문서에 남은 caret-free 부하 의존 절대값 5건** — `lgt.md:101` 의 `154 paints / 512 colours`·
  `69 paints / 93 colours`, `lgt_abi.md:1493` 의 `121 paints / 512 colours`·`154 / 512`·`69 / 93`.
  (`lgt.md:100` 의 메이플은 `up to 121` 로 이미 캐럿이 있다.) 같은 병이지만 티켓이 지목한 네 자리가
  아니라 **고치지 않았다** — 범위 확대는 재검수 사유다.
- ★**게이트② 재측의 loadavg 는 어디에도 기록돼 있지 않다.** 그래서 「18배」는 **두 측정이 갈렸다**까지만
  말할 수 있고 **부하에 «얼마나» 비례하는가**는 말할 수 없다. 문서에 그 한계를 그대로 적었다.

### 대가 — 있다, 숨기지 않는다

한정을 괄호로 붙이니 `lgt.md` §7 배너의 그 줄이 **한 줄에 다 들어가면서 길어졌다**(약 390자). ★**「그래서
몇이냐」가 흐려지는 대가**가 실제로 있다 — 다섯 수와 조건이 한 호흡에 섞인다. 수를 빼면 그 대가는
없어지지만 티켓 대전제 ⓑ 가 금지한 처방이고, 이 저장소가 반복해 세운 형태(「측정 시각·조건을 붙여
보존한다」)와도 어긋난다. ⇒ **길이를 택했다.** 한 줄로 몰은 이유는 Acceptance ⒜ 가 「그 수가 한정 구와
**같은 줄**에 있어야 한다」를 요구하기 때문이다(문단 분리로는 그 기계 판정이 서지 않는다).

### 개악 대조

**해당 없음 — 산문 변경 · 실행 로직 0.** 변경 파일 3개가 전부 `docs/`(`.rs`·`.mjs`·워크플로 무접촉)이고,
문면을 sed 로 되돌려 관문이 지는지를 볼 «관문»이 이 변경에는 존재하지 않는다.

### 티켓 내부 충돌 — 해소하고 적는다

티켓 Contract 2 와 Acceptance ⒟ 는 ⑷(장부 3건)을 **명시 범위**로 요구하는데, Contract 4 는
`docs/report/0210` 을 **무접촉**으로 적었다. 실측하니 ⑷ 의 세 값은 ★**`docs/report/0210` §한계에만
존재한다**(`docs/lgt.md`·`docs/lgt_abi.md` 에는 `loadavg`·`일지매`·`113` 문자열이 **0건**). ⇒ 두 조항은
그대로는 양립하지 않는다. **⑷ 를 이행하는 쪽을 골랐다**: ⓐ 티켓이 ⑷ 를 두 곳(Contract 2 · Acceptance ⒟)에서
요구하고 ⓑ 세 수 중 둘이 **0210 자신의 표와도** 어긋나 «전사 오류»이며 ⓒ 판정·표·결론·cp 사료를
건드리지 않으므로 「그 회차의 원장이다」라는 무접촉 사유(= 판정을 남이 고쳐 쓰지 마라)를 침범하지 않는다.
⇒ 고친 것은 **세 토큰과 그 정정 각주**뿐이다.

### 검증

- **카고 4관문 + beta** — `fmt --all -- --check` · `clippy --all -D warnings` ·
  `clippy --target wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` ·
  `+beta clippy --all -D warnings`: 전건 rc=0(`AGENTS.md` 가 「docs-only 는 면제가 아니다」로 못박았다).
- **문서 검사기** — `check-worklog-json` · `check-docs-report-serial` · `check-doc-liveness-parity` ·
  `check-worklog-coverage`: 전건 rc=0.
- **연번** — `--next-serial` 이 처음 `0211` 을 답했으나 그 사이 **PR #247 이 머지돼** origin/main 이
  `4aee5248` 로 움직였다(0211 선점). base 를 당기고 다시 물어 **0212**. ★**이 도구의 답은 「묻는 순간의
  참」이다** — 도구가 이미 적어 둔 그 한계가 이 회차에서 실제로 발화했다.
