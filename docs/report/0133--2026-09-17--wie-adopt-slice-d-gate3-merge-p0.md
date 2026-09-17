## [2026-09-17] 배포 수반 착지의 self-verify — 「이미 하고 있다」를 실측으로 갈아 «지시»로 바꿨다 (wie-adopt-slice-d-gate3-merge-p0)

**무엇을**: `AGENTS.md` 의 `verify-browser` 절에서 ★**「게이트③ 회차는 merge-contract 4-C 때문에 이미 그 스크립트를
프로덕션에 돌리고 있다」는 «단언»을 제거**하고, 그 자리에 ★**실측 비율 + 명령 + 합격 판정**을 넣었다.
★**제품 코드 0줄** · ★**머지 티켓 템플릿 무접촉**(repo 밖 · 아래 이유).

**왜**: 제안 `2026-09-16-slice-d-gate3-merge#p0`. 「`web.yml` 의 배포·검증 스텝이 전부 `event_name == 'push'` 뒤라
PR 런에서 원리적으로 안 돈다 ⇒ 착지 후 확인을 계약으로 못박아라」.

---

## ⓐ 지금도 참인가 — ★**템플릿 쪽은 참, repo 쪽은 이미 완비**

| 축 | 실측 |
|---|---|
| 템플릿 `~/orchestrator/templates/merge-ticket.tpl` §4-C | 「main 자동배포 수반 시 self-verify(운영 URL·콘솔 0에러) 증빙」 ★**한 줄 · 명령 없음** ⇒ **제안 그대로 참** |
| 이 repo 의 `AGENTS.md` | ★**명령을 «이미» 적고 있다** — `WIE_BASE=https://wie-web.pages.dev node scripts/verify-browser.mjs …` |
| 소유자 규칙 | `AGENTS.md` 가 「그 스텝이 red 면 착지시킨 게이트③ 회차가 owner」로 ★**이미 정해 두었다** |

## ⓑ 같은 축이 있는가 — ★★**있고, 쓰이고 있다. 그래서 «왜 안 쓰였나»가 진짜 문제였다**

`reports/` 의 `wie-*merge*.done.md` **131건** 전수:

| 축 | 수 |
|---|---|
| `WIE_BASE` 인용 | **53** |
| 「`Deploy to Cloudflare Pages`」 인용(= ★**배포 수반**) | **33** |
| ★**배포 수반 ∩ `WIE_BASE`** | ★**15** |
| ★**배포 수반인데 미인용** | ★**18 (55%)** |

⇒ ★★**`AGENTS.md` 의 「is already running … for merge-contract 4-C」는 «단언»인데 실측은 45% 다.**
★**그 18건 중 5건이 2026-09-16/17 한 레인이고, 13건은 그보다 앞선다** — ⇒ ★**한 회차의 실수가 아니라 상시 간극**이다.

★**18건이 «아무것도 안 한» 것은 아니다** — 전부 self-verify 를 **보고했다**: CI 스텝 결론 인용 + 별칭 `curl`.
그것은 4-C 의 「운영 URL」은 충족하지만 ★**「콘솔 0에러」를 충족하지 못한다** — 콘솔 오류와 off-origin 요청은 **브라우저 실행**이 있어야 보인다.

## ⓒ 제안이 틀렸는가 — ★**틀리지 않았다. 다만 «어디가 비었는지»를 한 칸 넓게 잡았다**

제안: 「템플릿 쪽에 **무엇을 어떻게 재는지가 없다**」 ⇒ ★**템플릿에 대해서는 참**이다.
그러나 ★**repo 쪽 `AGENTS.md` 에는 명령이 이미 있었다** — 그래서 53건이 그것을 찾아 썼다.
⇒ ★**빈 것은 «명령»이 아니라 «그 의무가 지시가 아니라 관찰로 적혀 있었다»는 점**이다.

---

# 고친 자리 — ★**「왜 여기인가」**

★**템플릿은 `~/orchestrator` 소관이고 repo 밖이라 이 회차가 고칠 수 없다**(제안 자신이 `target: orchestrator` 라 적었다).
★**`AGENTS.md` 의 그 절이 «템플릿 한 줄이 가리키는 자리»**이고, 거기에 ★**측정 가능한 거짓 단언**이 있었다.
⇒ 그 단언을 실측으로 바꾸고, ★**소유자 규칙 «옆»에 명령과 합격 판정을 붙였다**(찾아 헤매지 않도록).

- 제거: 「it is already running the same script against production for merge-contract 4-C」
- 추가: 실측 **15/33** · 5 vs 13 분해 · 「18건도 보고는 했다 — 다만 «콘솔 0에러» 절반이 빠졌다」
- 추가: ★**명령 + 합격 판정** — `NO-LEAK AUDIT: ✅` · `off-origin requests: 0` ·
  `requests whose body contains the game header bytes: 0` · ★`nonBlack: 0` **도 합격**(helloworld 규격)
- 추가: ★**CI 스텝 인용만으로는 부족한 이유** — 그 스텝은 **per-deploy URL** 을 읽으므로
  ★**«별칭이 안 넘어온» 경우를 원리적으로 못 본다**(이 손실행이 존재하는 이유가 그것이다)
- 추가: 템플릿이 repo 밖이라는 사실 + ★**복사 금지 · 이 절을 «가리켜라»**(§Constraints 「An external contract is referenced, never copied」)

## ★명령을 «인라인»으로 적었다 — 그 자체가 판단이고, 근거는 실측이다

처음에 ```` ```sh ```` 펜스로 적었다. ★**그러면 `doc-liveness` parity 가 새 의무를 만든다**(이 파일의 규약:
「새 fenced `sh` 블록은 parity-checked — 같은 PR 에서 job 에 넣거나 `NOT-RUN` 을 선언하라」).
그런데 그 명령은 ★**주간 job 이 이미 돌리는 `npm run verify` 의 «별칭 변종»**이라 중복 의무가 된다.

★★**그리고 «블록인용 안 펜스»는 통과하지만 그것이 더 나쁘다** — 추출 정규식이
`scripts/check-doc-liveness-parity.mjs` 의 **`/^\s*```(\S*)\s*$/`** 라 ★**`>` 는 whitespace 가 아니어서 «보이지 않는다».**
⇒ 오늘은 rc=0 이지만 ★**누군가 들여쓰기를 푸는 순간 red 가 된다 — 다음 사람을 위한 트랩**이다.
⇒ ★**인라인으로 적고, 왜 인라인인지를 그 줄 옆에 적었다.**

**개악 대조(양방향 · 제품 호출부 = `AGENTS.md` 의 그 줄)**

| 형상 | `node scripts/check-doc-liveness-parity.mjs` |
|---|---|
| ★**인라인**(채택) | ★**rc=0** — `25 documented line(s) … nothing extra` |
| ★**펜스화**(개악) | ★**rc=1** — `+ WIE_BASE=…` · `add the line to a DOC-COPY region, or a '# NOT-RUN: …'` |
| 복원 | ★**rc=0** |

## ★내가 건너뛴 검증을 «이 회차에서 실제로 돌렸다»

위 18건 중 **5건이 내 회차**다. ⇒ ★**변명하지 않고 지금 실행해 discharge 했다**(04:55 KST · 프로덕션 대상):

```
NO-LEAK AUDIT: ✅ no game bytes left the browser, no off-origin requests
off-origin requests: 0 · requests whose body contains the game header bytes: 0
canvas pixel stats: nonBlack 0 / total 76800   ← helloworld 는 blank 로 끝나는 것이 규격(합격)
```
★**최근 배포(#167 착지분)가 프로덕션 별칭에서 실제로 부팅되고, 게임 바이트가 브라우저 밖으로 나가지 않는다**를 확인했다.

# 게이트

`fmt` OK · `clippy` **0** · ★`+beta clippy` **0** · wasm **0** ·
`cargo test --all` ★**42 타깃 전건 합산 384 passed · 0 failed · 0 ignored** ·
검사기 **7종 rc=0**(doc-liveness-parity 25줄 · serial 128건·중복 0 · worklog-json 113 ·
linux-system-deps · parity-lock-wired · parked-workflows · worklog-coverage).

## ★대가 — Contract 2

⒜**무엇을 잃는가**: ⑴★**기계 강제는 여전히 0** — 게이트③이 그 명령을 돌렸는지 «검사»하는 축은 없다.
바뀐 것은 ★**문서가 «관찰»에서 «지시»가 된 것**뿐이고, **45%** 라는 수가 그 한계를 그대로 보여 준다.
⑵★**게이트③ 회차가 배포 완료를 기다리게 되면 슬롯을 그만큼 문다**(제안이 적은 그 대가 — CI 한 바퀴).
★**다만 이 회차가 실제로 재 보니 그 실행 자체는 «분» 단위**다(배포 완료를 기다리는 시간이 대부분이고, 그건 이미 하던 일이다).
⑶★**45% 는 «오늘의 수»다** — 상수로 인용하지 마라. 술어(`Deploy to Cloudflare Pages` ∩ `WIE_BASE`)가 문구 기반이라 문구가 바뀌면 다시 재야 한다.
⑷★**그 술어는 «회신에 적었는가»를 세지 «돌렸는가»를 세지 않는다** — 돌리고 안 적은 회차는 미실행으로 계상된다
(그 경우도 **증빙 부재**라 계약 미충족이지만 원인은 다르다).

⒝**안 하면 무엇이 나쁜가**: ⑴★**소유자 규칙이 «전제 위에» 서 있었다** — 「red 면 그 라운드가 **re-runs it**」은
그 라운드가 **이미 돌릴 줄 안다**를 가정하는데, 실측 55% 는 그 가정이 절반 이상 깨졌음을 보인다.
⑵★**더 나쁜 형태는 «조용한 통과»다**: CI 스텝 인용 + `curl` 만으로 「self-verify 했다」로 적히면
★**콘솔 오류·off-origin 요청이 있는 배포가 green 으로 보고된다.** 제안이 막으려던 것이 정확히 그것이다.

## 사용자 영향

**없다**(문서만). 다만 ★**다음 배포 수반 착지부터는 «무엇을 돌리고 무엇이 합격인지»가 한 자리에 있다.**

## 한계 — 숨기지 않는다

- ★**템플릿 §4-C 는 그대로다** — repo 밖이라 못 고친다. ⇒ **에스컬레이션**(그 한 줄이 이 절을 가리키게 하는 것이 남은 절반).
- ★**검사 축 0** — 후속 제안으로 뺐다(먼저 「문서만으로 얼마나 고쳐지나」를 다음 33건으로 재는 편이 싸다).
