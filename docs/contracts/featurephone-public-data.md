# featurephone 공개 데이터 계약 — `compat.json` · `updates.json`

셸(otterpebble `apps/featurephone`)이 이용자에게 보여 주는 «지원 게임 목록»과 «업데이트 소식»의 원천이다.
엔진 아티팩트와 **같은 릴리스**에 실려 같은 `repository_dispatch` 로 넘어간다(핀 범프와 원자적).
검사: `node scripts/player-data.mjs`(PR 마다 `engine-contract.yml` 의 `contract` 잡) · 생성: `… build --out <dir>`(`publish-artifact.yml`).

★게임 바이트·게임 화면 캡처·게임 파일을 구하는 곳은 어느 파일에도 없다 — 제목, **파일 내용 sha256**, 문장뿐이다.

## 1. `compat.json` — 스키마는 새로 정하지 않았다

전수 조사(`scripts/playability-census.mjs report` · PR #354)가 내고 셸 가져오기(`apps/featurephone/scripts/compat-import.mjs` ·
otterpebble #1141)가 받는 **그 스키마 그대로**다. 두 소비자가 이미 쓰는 것을 두 벌로 만들지 않기 위해서다.

```
{ schema: 1, generatedAt: "YYYY-MM-DD…", enginePin: <측정한 wie 커밋 40hex>,
  entries: [{ sha256: <64hex>, platform: "KTF"|"SKT"|"LGT"|"J2ME", model: "clet"|"aot-java"|null,
              title: <표시 이름>, fileTitle: <파일에서 온 원 이름(검색·추적용)>, status: "playable"|"limited"|"not-yet",
              axes: { boot, render, input, longplay, sound, speed: "ok"|"partial"|"no"|"unknown" },
              knownIssues_ko: [쉬운 한국어 문장],
              changes: [{ date, enginePin, summary_ko, kind, pr }] }] }
```

총괄 초안과 다른 점과 이유:

| 초안 | 계약 | 이유 |
|---|---|---|
| `id` 슬러그 | `sha256` | 셸의 파일 정체성 규약이 내용 해시다(보관함 배지가 그것으로 매칭). 같은 이름이 통신사마다 다른 파일이고, 한 게임의 판이 여럿이다 |
| 등급 6개(`menu`·`intro`·`boots`…) | 3개 + 축 6개 | 헤드리스로 «메뉴»와 «인트로»를 가를 술어가 없다. 대신 축별로 무엇이 됐는지를 따로 싣는다 |
| `api` | `platform` + `model` | 전수 조사가 재는 값이 이것이다. `api`(WIPI-C/Java)는 측정 안 한 값이라 넣지 않았다 |
| `verifiedAt`·`verifiedWieHead` 항목별 | 최상위 `generatedAt`·`enginePin` | 한 회차가 전건을 같은 핀으로 잰다 |

★**표시 이름 규칙**(`stripMarkers`·`retitle` · 검사기가 «표식 잔존 0»과 «통신사 안 이름 중복 0»을 거부로 잡는다):
파일 표식 — 대괄호 태그·재다운로드 `[1]`·통신사 표기(`kt`·`KTF`·`lgt`·`skt`·`SKVM` — 통신사는 `platform` 이 따로 말한다)·
`에디트`/`수정판`/`추가다운완료`·파일 판 번호(`1.04`·`01.00.05`)·숫자 id 접두 — 는 `title` 에서 빼고(`+` 는 띄어쓰기로) 원 이름은 `fileTitle` 에 둔다.
**뺀 뒤 같은 통신사 안에서 겹칠 때만** 구별 표기를 붙인다: 뺀 표식 하나(화면 크기가 있으면 그것, 없으면 판 번호)가 그 묶음에서 유일하면 ` (작은화면)`처럼, 아니면 ` (2)`·` (3)`(표식 짧은 순 → sha256 순).

`changes` 의 `kind`·`pr` 과 `fileTitle` 은 셸 가져오기가 요구하지 않는 **추가 필드**다(모르는 키는 무시된다).

### 등급 술어 — 무엇을 봤으면 그 등급인가

축(전수 조사 `judge()` · 정본 서술 `docs/report/0321`). 프로브 A = 27키 주입 30초, B = 키 없이 30초.

| 축 | `ok` | `no` | `unknown` |
|---|---|---|---|
| boot | A·B 중 하나라도 그렸다, 또는 A 가 실패하지 않았다 | 둘 다 한 장도 못 그리고 A 실패 | – |
| render | 두 가지 색 이상인 화면이 한 장이라도 있다 | 한 색뿐(uniform) 또는 그린 적 없음 | – |
| input | A 에 B 가 한 번도 안 보인 화면이 있다(키가 화면을 바꿨다) | 없다, 또는 키 입력 중 panic | 앞 두 축이 ok 가 아니다 |
| longplay | 600초 반복 키 조작에서 실패 줄이 없다 | 실패(또는 30초 안에 그린 뒤 오류) | 앞 셋이 ok 가 아니라 재지 않았다 |
| sound | 이벤트가 든 재생이 엔진 → 싱크까지 왔다 | 30초 안에 한 번도 없다 | 부팅 안 됨 |
| speed | 늦음 비율 ≥ 0.9(헤드리스 하한, 또는 브라우저 2회가 서로 10% 안) | 브라우저 2회가 서로 10% 안이고 < 0.9 | 그 밖 — 헤드리스 하나로는 «느림»을 판정하지 않는다(부하 속 측정은 하한) |

| progress(선택 · **공개 값 `ok`·`stuck`**) | `ok`: 진도 정책 키로 N분 조작하는 동안 마지막 1/3 안에 새 화면(16×16 휘도 지문)이 나왔다(정책 v2 = 새 화면이 60초 없을 때만 뒤로·오른쪽 소프트키·다음 항목을 섞는다 · 2026-10-02~) | `stuck`: 마지막 1/3 동안 새 화면 0 — 짝 재측 2회가 같을 때만(실행 FAIL 도 `stuck`) | 키 없음 — 재지 않았다(대부분) |

등급 = `playable`: boot·render·input·longplay 넷 다 `ok` · `limited`: boot·render 만 `ok` · `not-yet`: 그 밖.
★예외 하나 — **통신망 벽**(전수 도구 `netWall`): 한 실행에서 접속을 20회 이상 되풀이한 타이틀(옛 통신사 서버 없이는 못 넘어간다)은 넷이 `ok` 여도 `limited` 이고, `knownIssues_ko` 는 그 안내 한 줄뿐이다(`docs/report/0416`).
★`playable` 은 «키를 눌러 화면이 바뀌고 10분 조작에서 안 멈췄다»까지다 — 사람이 끝까지 해 본 것이 아니다.
★측정 안 한 축은 `unknown` 이다. 추측으로 올리지 않는다.

가져오기(`player-data.mjs import`)가 전수 조사의 축 어휘를 계약 어휘로 바꾼다:
`ok→ok` · `n/a→unknown` · `fail`·`none`·`uniform`·`error`·`silent`→`no`(여섯 축). progress 만 `ok→ok` · `stuck`·`error`→`stuck` · `n/a`→키 없음.
`progress` 는 **선택 축**이다 — 전수 조사가 잰 행에만 실리고(`n/a` 면 키 없음), 셸 가져오기는 여섯 축만 검사하며 나머지 키는 그대로 통과시킨다(otterpebble `compat-import.mjs` 의 `...x`). ★값 어휘가 여섯 축과 **다르다** — 셸 `PROGRESS_UI`(otterpebble `lib/compat.ts` · #1244)는 `ok`·`stuck` 만 그리고 모르는 값은 칸을 숨기므로, `no` 로 실으면 막힌 행만 사라져 «좋은 쪽만 보이는» 표시가 된다. 그래서 `stuck` 을 그대로 싣는다(`scripts/player-data.mjs` 의 `EXTRA_AXIS_VALUES`). 레시피(제목별 키)로 잰 결과는 싣지 않는다 — 이 축의 정의는 «정책 키». 등급(`status`)에는 넣지 않았다. `partial` 은 지금 아무 측정도 내지 않는다.

## 2. `updates.json` — 한 항목 한 파일

원천 = `docs/player-updates/<YYYY-MM-DD>-<slug>.json`. 한 파일에 쌓으면 형제 PR 이 전부 충돌한다(`STATE.md` 가 적어 둔 병).
YAML 이 아니라 JSON 인 이유: 파서 의존을 새로 들이지 않는다(node 표준만).

```
{ date: "YYYY-MM-DD"(= 파일 이름 앞 10자), kind: "new-support"|"fix"|"improvement"|"sound"|"speed",
  titles: [compat.json 의 sha256…](빈 배열 = 특정 게임이 아닌 전체 소식),
  summary_ko: 쉬운 한국어 1~2문장, pr: "https://github.com/Jun025/wie/pull/<n>",
  enginePin?: 40hex }
```

- `summary_ko` 에 `—―–·・ㆍ‧∙•` 를 쓰지 않는다 — 셸 사이트 문구 래칫(otterpebble `scripts/user-copy-glyph-check.mjs`)이 거부하고 가져오기가 파일째 막힌다. `player-data.mjs` 가 검사한다.
- `enginePin` 을 생략하면 빌드가 **그 파일을 main 에 들여온 first-parent 커밋**(= 머지)으로 채운다. 그래서 새 항목은 고친 PR 안에서 함께 쓰면 된다.
  못 채우면(아직 main 에 없음) 그 항목은 이번 빌드에서 빠진다 — 지어낸 핀으로 싣지 않는다.
- 빌드 산출: `{ schema: 1, generatedAt, wieHead, entries: [{ id(파일 이름), …위 필드, enginePin }] }` 날짜 최신 먼저.
- 게임별 `compat.json.changes` 는 **이 파일들에서 파생**한다(원천은 하나). `titles: []` 항목은 소식 피드에만 나온다.
- ★`build` 는 **산출물 전체**를 셸 가져오기와 같은 규칙으로 다시 잰다 — `compat.json` 의 `changes[].enginePin` 은 40hex 또는 null, `updates.json` 의 `wieHead`·`enginePin` 은 40hex. 하나라도 어기면 **아무것도 쓰지 않고 실패**한다(셸은 위반 1건에 파일 전체를 거부하므로, 거부될 데이터를 릴리스에 싣지 않는다). 2026-09-29 `git log --diff-merges` 가 패치를 함께 켜 핀 자리에 `+}` 가 실렸고 셸이 compat 35건·updates 18건 위반으로 둘 다 버렸다.

## 3. 배달

`publish-artifact.yml` 이 엔진 릴리스 `engine-<sha>` 에 `compat.json`·`updates.json` 을 wasm·glue 와 함께 올리고,
`client_payload.publicData = { compatUrl, compatSha256, updatesUrl, updatesSha256 }` 를 싣는다
(최상위 키 10개 제한 때문에 한 객체로 묶었다). 셸 수신부가 wasm 과 같은 방식(URL + sha256 대조)으로 받는다.
데이터만 바뀌면 릴리스가 자동으로 안 돈다(경로 필터가 엔진 파일만 본다) — 다음 엔진 릴리스에 실리거나 `workflow_dispatch` 로 보낸다.

## 4. 갱신

- 전수 조사를 다시 돌린 뒤: `node scripts/player-data.mjs import <report 가 쓴 compat.json>` → 커밋.
- 게임 동작을 바꾸는 PR: 항목 파일 1개를 같은 PR 에 넣는다(`AGENTS.md` §Landing paperwork).
