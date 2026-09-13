## [2026-09-13] `83 paints`·`30 paints` — 검토하고 «기각»한다(주석 무접촉) (wie-two-more-single-value-paint-counts-in-comments)

- **무엇을**: 코드·주석 **변경 0**. 채택 제안 `2026-09-13-keydraw-paint-count-is-a-floor#p0` 을
  ★**«검토하고 기각»(examined and declined)** 으로 처분하고 그 판정을 원장에 남긴다
  (이 저장소가 `AGENTS.md` §완료 결정에서 이미 쓴 형태다). ★**티켓이 그 종결을 명시 허용**했다
  (ⓑ: 「후자면 고치지 말고 그 판정을 회신에 적고 닫아라 — 그것도 정당한 완주다」).

- **ⓐ 두 자리 실재**(★**줄번호를 적지 않는다** — 이 리니지가 그 함정을 이미 밟았다. 문구로 지목한다):
  ⒜`wie_lgt/tests/test_key_reach.rs` — 「the 2026-09-05 black screen on THIS carrier … **ran at 83 paints**
  with a blank final frame」 ⒝`wie_cli/src/bin/wie_validate.rs` — 「**Measured 2026-09-05 on LGT: 30 paints**
  arrived with an all-black image and the final frame was black」.

- **★★ⓑ 문장 단위 판정 — 둘 다 «그때의 관측»이다. 그리고 결정적인 것은 시제가 아니라 «잴 수 있는가»다.**

  | | 그 수가 서술하는 것 | 그 수가 «지금» 재현되는가 | 판정 |
  |---|---|---|---|
  | ⒜ `83` | **2026-09-05 LGT 흑화면 사건**(MIDP 가 좋은 WIPI 프레임을 덮어쓴 그 회차) | ★**아니다 — 그 버그는 고쳐졌다** | 사건 서술 ⇒ **무접촉** |
  | ⒝ `30` | 같은 사건(같은 날 LGT) | ★**아니다 — 같은 이유** | 사건 서술 ⇒ **무접촉** |

  ★★**`AGENTS.md` 건과 «무엇이 달랐나» — 이 한 줄이 판정 기준이다**:
  거기서 `paints 55` 는 ★**문서가 «치라»고 지시한 명령 바로 옆**에 있었다 ⇒ 독자가 실제로 치고 **비교한다**
  ⇒ 다른 수가 나오면 회귀로 오독한다(그 오독이 실제로 났다). 여기 두 수는 ★**칠 명령이 없다** —
  둘 다 **고쳐진 실패**를 서술하므로 ★**「다시 재니 다르더라」가 원리적으로 일어나지 않는다.**
  ⇒ ★**「단일 값이라 낡는다」는 «인용 형태»가 아니라 «재현 가능성»의 문제**이고, 그 축에서 둘은 안전하다.

- **★부수 판정 — ⒜ 안의 «현재형» 주장 하나는 따로 봤다**: 같은 문단이 「this test **composes 1 frame when
  healthy**」라고 **현재형**으로 적는다 ⇒ 이것만은 「지금의 등식」이라 부하로 흔들리면 낡는다.
  ★**그러나 흔들리지 않는다 — 예산 축이 다르다**(실측):
  ```
  test_key_reach.rs : const DELIVER_TICKS: usize = 400_000;  for _ in 0..DELIVER_TICKS { emulator.tick()?; }
                      → ★«틱 고정» = 벽시계 무관 · 결정적
  wie_validate --inject : StdInstant::now() + deadline_secs + boot_secs/action_secs 스케줄
                      → ★«시간 고정» = 선행 회차가 «부하 의존»을 «잰» 바로 그 축
  ```
  ⇒ ★**선행 회차의 발견은 «시간 예산» 축의 성질이고, 틱 고정 루프에는 적용되지 않는다.**
  그러므로 그 「1」도 갈아치울 이유가 없다.

- **ⓒ 판정 술어로 쓰이는 코드 — 0건**: `scripts/`·`.github/`·`wie_cli/src`·`wie_lgt` 전수에서 그 두 수를
  비교연산에 쓰는 자리 **없음**(선행 회차의 `55` 조사와 같은 결과) ⇒ 문면 외 파급 0.

- **★「그 사건은 고쳐졌다」의 근거**(이 판정의 유일한 전제이므로 실측으로 댄다):
  `scripts/contract-roundtrip.mjs` **Scenario F** 가 그 회귀를 **픽셀로 실단언**하고(그 헤더가 2026-09-05
  트레이스를 그대로 기록한다: 「the good WIPI frame was painted and then OVERWRITTEN by MIDP's blank
  screenImage」), 그 검사가 **모든 engine PR 과 main push 에서 돈다**. 오늘 `origin/main` CI **green**.
  ⇒ 83·30 을 다시 내려면 **그 픽스를 되돌려야** 한다.

- **계약 2(대가) — 「포괄 문장을 가리키는 쪽」도 «하지 않았다»**: 고칠 자리가 없으므로 가리킬 것도 없다.
  ★**사본 0 · 새 단일 값 0 · 검사기 0 · `AGENTS.md` 재수정 0**(이미 닫혔다) · 러너·픽스처·`paints` 산출 경로 **무접촉**.

- **★재개 조건 — 이 기각을 다시 열어야 하는 경우**(유보한 질문은 장애의 모습으로 돌아온다):
  ⑴**Scenario F/E 가 제거된다** — 그러면 그 회귀가 다시 가능해지고 두 수가 «재현 가능»해진다
  (⒜의 주석 자신이 「If those scenarios are ever removed, re-open this」라고 이미 적어 두었다).
  ⑵누군가 그 두 수를 **비교 술어로 코드에 넣는다**(ⓒ가 0건에서 깨진다).
  ⑶`test_key_reach` 의 루프가 **틱 고정에서 시간 고정으로 바뀐다** — 그 순간 「1 frame」이 부하 의존이 된다.

- **게이트**: ★**러스트 무접촉**(diff 3파일 전부 `.md`/`.json`) ⇒ 4게이트 전량 불요. 문서 검사기 5종 전건 통과.
  채택 기록 = worklog `adoptedProposals: ["2026-09-13-keydraw-paint-count-is-a-floor#p0"]`.
