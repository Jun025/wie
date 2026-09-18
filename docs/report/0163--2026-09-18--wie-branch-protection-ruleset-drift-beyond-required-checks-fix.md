## [2026-09-18] 드리프트 경로가 「라이브 형상을 복사해 넣어라」면서 **그것을 안 찍었다** — `--print-current` 로 그 자리에서 준다 (wie-branch-protection-ruleset-drift-beyond-required-checks-fix)

**무엇을**: 게이트② 반려(PR #197) 승계. **C2**(갱신 절차의 재료 부재)를 처방하고, **C1**(critical)을 검수자가 허용한
**선언 경로**로 닫았으며, **C3·C4**(minor · 둘 다 «출력» 축이라 범위 안)를 함께 닫았다.
★**판정 술어·지문 산출 방식 무접촉**(계약 3) · 새 워크플로 **0** · required-checks 축 **무접촉**.

## ⓐ 두 경로를 내가 직접 냈다 — 검수자 말이 맞다
| 경로 | rc | 출력 | 라이브 형상 전문 |
|---|---|---|---|
| 정상 | 0 | **5줄** | — |
| 베이스라인 **부재** | 2 | **62줄** | ★**있다** |
| **드리프트** | 1 | **9줄** | ★**없다** — 「`copy the LIVE shape`」라 말하면서 차이 **한 줄**만 찍는다 |

⇒ ★**「의도한 변경이면 복사해 넣어라」를 받은 사람이 복사할 원본을 못 얻는다.** 재현됐다.

## ⓒ 민감 정보 — 전수 확인. ★**전문 출력이 누출이 될 수 없다**
정규화 산출물의 **잎 필드 27개 전수**를 값까지 열거해 확인했다(이름·target·enforcement·ref 조건·rule 타입·
머지 방식·승인 설정·check 컨텍스트). 어휘 계수: `token` `secret` `email` `@` `actor_id` `integration_id`
`node_id` `password` `http` `url` `_links` `current_user_can_bypass` — ★**전부 0**.
★**결정적 논거**: `--print-current` 의 출력은 **커밋된 `.github/branch-protection-expected.json` 과 바이트 동일**(`cmp` 확인)
⇒ ★**이미 repo 에 공개된 내용**이다. 마스킹 불요.
※`bypass_actors` 가 채워지는 날 `actor_id`/`actor_type` 이 이 파일에 들어온다 — **시크릿은 아니나** 그때 한 번 보라(오늘 `[]`).

## ⓑ 왜 «전문 항상 출력»이 아니라 `--print-current` 인가 — 셋을 재서 골랐다
| 안 | 고르지 않은 이유(=대가) |
|---|---|
| ⑴전문 항상 출력 | 실패 메시지가 **9줄 → 약 64줄**. ★**사람이 실제로 읽는 것이 그 실패 메시지**인데 55줄 객체를 거기 묻으면 **안 읽게 된다** — 이 도구의 값어치는 「어느 줄이 움직였나」를 읽히는 것이다 |
| ⑵파일로 떨구기 | **CI 아티팩트 수명**에 묶인다. ★그리고 이 도구는 **CI 호출자가 0**(손으로 돈다)이라 아티팩트라는 소비 경로 자체가 없다 |
| ★⑶`--print-current` | **골랐다** — 한 줄로 **파일에 바로 쓴다**(55줄 복사·붙여넣기 훼손 위험 0) · 정규화를 **한 벌로** 유지 |

★**⑶의 대가도 적는다**: ⒜`gh api` **왕복 1회 추가** ⒝★**실패한 run 과 재시딩 run 사이에 라이브 형상이 움직일 수 있다**
⇒ 그래서 실패 메시지가 재시딩 **다음**에 **평문 재실행**을 요구한다(그 재실행이 곧 검증이다).
★**「잃는 것이 없다」가 아니다.**

## ★수락 — 드리프트를 만들고 «그 출력만 보고» 재시딩해 봤다
```
line 34:  expected  "required_approving_review_count": 1,  ↔  live  "required_approving_review_count": 0,
  ⇒ If the operator meant it, reseed in the same PR and say why in the round's report:
       node scripts/check-branch-protection-claim.mjs --print-current > .github/branch-protection-expected.json
       git diff -- .github/branch-protection-expected.json   # ← read THIS as the change, then re-run …
```
그 두 줄을 **그대로** 실행 → 재실행이 **`OK — …` rc=0**. ⇒ ★**출력만으로 갱신이 된다**(통과 조건 충족).
★`--print-current` 의 **stdout 은 JSON 만**이라 `> 파일` 이 안전하다(안내문은 stderr) — `JSON.parse` 로 확인.

## ★양방향·안정성
- ⒜**드리프트 있으면** 재료가 나온다(위) · ⒝**없으면 조용하다** — 정상 경로 **5줄, 변경 전과 동일**(시끄러워지지 않았다)
- ★**지문 안정성 불변**: 평문 연속 2회 `cmp` **동일** · `--print-current` 연속 2회 `cmp` **동일**

## C1 — 검수자가 허용한 **선언 경로**로 닫았다(코드 축은 제안으로 올렸다)
반려 사유는 「`any field, present or future, moves the fingerprint` 가 거짓이고 **그 맹점이 선언 목록에 없다**」이고,
검수는 ★**「코드를 그대로 두고 ⑵⑶의 선언 목록에 명시해도 반려는 해소된다」**를 명시했다. 이 회차의 계약 3 이
**지문 술어 무접촉**이므로 그 경로를 골랐다:
⑴**거짓 문장 삭제·교체** — 이제 「**rule `parameters` 의 최상위**에 필드가 생기면 지문이 움직인다」로 좁히고,
  ★그것이 「any field」와 **다르다**는 것과 **실측(`integration_id` → rc=0)**, **실노출 0**, **알려진 1줄 처방**을 그 자리에 적었다.
⑵**버리는 목록에 편입** — `_links` 와 ★**rule-parameter 배열 안 객체의 `context` 외 전 키**.
⑶**`AGENTS.md`·`STATE.md`·worklog `limits` 3곳 전부**에 같은 사실을 적었다(C1 이 「셋 다 안 적었다」고 지적한 그 셋).
★**코드 1줄 처방은 검수자가 이미 검증해 두었다**(11축 재실행) ⇒ **worklog 제안 `#p0`** 로 올렸다. ★**베이스라인 재시딩이 동반돼야 한다**는 것까지 적었다.

## C3 — «완전히» 고치지 않았다(그 사실을 적는다)
줄 짝짓기는 **여전히 위치 기반**이다. 바꾼 것은 ★**줄 수가 다르면 그 사실을 먼저 말한다**는 것뿐이다.
실측(검수자의 E축 = `non_fast_forward` 제거): `★fields were ADDED or REMOVED (baseline 52 lines ↔ live 55) — … Diff the reseeded file instead of trusting these pairs.`
★진짜 diff(LCS)를 넣는 대신 **재시딩 후 `git diff`** 를 읽게 안내했다 — 그쪽이 사람이 이미 쓰는 도구이고, 새 알고리즘을 들이지 않는다.

## C4 — 축 ⑴ 이 축 ⑵ 를 가리지 않는다
조기 `process.exit(1)` 을 **판정 기록(`failed`)**으로 바꾸고 exit 를 끝으로 옮겼다. ★**술어는 무변경**이다.
실측: 축⑴ 개악 + 축⑵ 드리프트 동시 ⇒ 축 신호 줄 **2 → 3**(`understate`·`overstate`·`DRIFTED`) · **rc=1 불변**.

## ★이 회차가 «안» 한 것
- ★**base 를 당기지 않았다** — 계약 3 이 「ⓒ(머지 가능성)는 이 티켓이 풀지 않는다」로 못박았다.
  검수 회신은 base 당김이 CONFLICTING + 상속 red 를 **함께** 없앤다고 적었고 그것은 맞다 — ★**그러나 이 티켓의 범위가 아니다.**
- ★**`STATE.md` 를 «만졌다»** — 계약 3 의 「가능한 한 만지지 마라」를 어겼고 **그 사실을 적는다**:
  C1 ⑶ 이 요구한 정정(「정규화가 버리는 것」 목록)이 **바로 그 파일에 있었다**. ★**새 항목을 «추가»하지 않고 이 PR 이 이미 가진
  항목 «안»을 고쳤다** ⇒ 새 삽입점을 만들지 않았다(충돌면은 이 PR 이 이미 갖고 있던 것과 같다).
- ruleset 쓰기 **0** · PAT **0** · 새 CI 스텝 **0** · 제품(엔진) 코드 **0줄**.
