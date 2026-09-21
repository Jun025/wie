## [2026-09-19] 센서스 생성기의 «입력»과 «조회» — 경로 키를 1순위로 올렸고, 문서가 시키던 awk 를 도구 안으로 들였다 (wie-census-generator-input-precedence-and-signature-lookup)

채택 제안 **2건**을 한 파일(`scripts/game-lab-census-map.mjs`)에서 이행한다 —
`2026-09-19-census-rerun-leaves-per-game-rows#p0`(입력 우선순위) ·
`2026-09-19-census-map-staleness-is-loud#p0`(`--signature` 조회 모드).

### ⒜ 축 A — 입력 우선순위: ★**데이터가 «지금» 사라지고 있었고, 그것을 돌려서 봤다**

**⑴ 겹치는 3건을 이름으로 지목한다**(내 실측 2026-09-19 · `game_lab/broken/**`):
**파일 187 · 고유 stem 184 · 겹치는 stem 3** — 제안의 수와 일치한다.

| stem | 두 캐리어 |
|---|---|
| `놈3` | `broken/ktf/놈3.zip` · `broken/lgt/놈3.zip` |
| `다크슬레이어2` | `broken/ktf/다크슬레이어2.zip` · `broken/skt/다크슬레이어2.zip` |
| `이노티아연대기2` | `broken/ktf/이노티아연대기2.zip` · `broken/unknown/이노티아연대기2.zip` |

**⑵ 덮어쓰기를 재현했다 — ★그리고 제안이 말한 것보다 나쁘다.**
그 6파일만 스크래치 코퍼스로 복사해 러너를 실제로 돌렸다
(`game-lab-recensus.sh --corpus <scratch> --out <scratch>` · **실 `game_lab/` 쓰기 0**):

```
러너 출력 — 6게임이 «각각» 돌았다
   FAIL (ktf) ticks=0    paints=0   놈3.zip
   PASS (ktf) ticks=119  paints=46  다크슬레이어2.zip     ← ★PASS
   PASS (ktf) ticks=21   paints=1   이노티아연대기2.zip   ← ★PASS
   FAIL (lgt) ticks=1071 paints=0   놈3.zip
   FAIL (skt) ticks=0    paints=0   다크슬레이어2.zip
   FAIL (unknown) ticks=0 paints=0  이노티아연대기2.zip

남은 것: summary.tsv 6행(경로 키) ↔ <stem>.json ★3개(stem 키)
```
⇒ ★**stem 층은 한 벌을 잃는 정도가 아니라 «PASS 를 FAIL 로 바꾼다».**
같은 입력을 **변경 전 생성기**(`HEAD` = PR #218 head)와 **변경 후**로 각각 돌려 대조했다:

```
행 6 · 판정이 다른 행 2
   ktf/다크슬레이어2.zip     BEFORE FAIL UNCLASSIFIED → AFTER PASS PASS
   ktf/이노티아연대기2.zip   BEFORE FAIL UNCLASSIFIED → AFTER PASS PASS
```

**⑶ 처방은 제안 그대로** — `summary.tsv`(경로 키) **1순위** · `<stem>.json` **폴백**.
★**폴백은 지우지 않았다**(아래 ⑸의 이유로 7월 열은 그것으로만 읽힌다).

★**키를 «둘» 잡았다**: ⒜해석된 절대경로(정확) ⒝`<carrier>/<basename>`(꼬리).
⒝가 없으면 러너를 상대경로로, 생성기를 절대경로로 부른 흔한 조합에서 **한 행도 매치하지 않고
조용히 stem 폴백으로 떨어진다** — 고친 것처럼 보이면서 아무 일도 하지 않는 형태다.
꼬리 키가 187파일을 유일하게 식별하는 이유는 ★**겹치는 stem 들이 서로 다른 캐리어 밑에 있기 때문**이다.
꼬리가 둘 이상 겹치면 그 키는 **`null` 로 무효화**해 폴백으로 보낸다(순서로 한쪽을 고르지 않는다).
★`realpathSync` 가 아니라 `path.resolve` 다 — 같은 바이트를 가리키는 두 코퍼스 항목을 합치면
이 변경이 지키려는 «두 행»을 도로 하나로 만든다.

**⑷ ★«어느 입력이 이겼나»를 출력한다**(제안 `tradeoff` 가 이 축의 합격선으로 건 것):
- **행마다** — TSV **7열 `source`**(`summary.tsv` / `stem.json` / `none`).
  ★**6열 뒤에 «덧붙였다»** — 문서화된 조회가 `$6`·`$1` 이라 1~6열의 의미가 그대로다.
- **집계로** — 헤더 `# ★inputs: …` · stdout · `--bucket`/`--signature` 의 stderr 줄.

```
새 열 : ★inputs: summary.tsv=6 (summary.tsv had 6 row(s))
7월 열 : ★inputs: stem.json=186 · none=1 ★(summary.tsv has 73 row(s) and NONE of them
         matched a corpus file — its 'file' column is not corpus-relative; …)
```

**⑸ ★★대전제 정정 — 「7월 기준선에는 경로 키가 없다」는 «틀렸다»**(Contract 1: 적고 고치지 않는다).
`game_lab/reports/summary.tsv` 는 **실재한다**(73행 · 2026-07-01 · 같은 7열 헤더).
그런데 그 `file` 열은 **경로가 아니라 맨 베이스네임**(`(SKT) 교실이데아.zip`)이라
코퍼스 경로와 **조인되지 않는다** ⇒ 7월 187행은 **전건 `<stem>.json` 에서** 온다.
⇒ ★**제안의 결론(폴백을 남겨라)은 옳고, 그 이유가 달랐다.**
★**베이스네임을 세 번째 키로 받지 «않았다»**: 그것은 겹치는 stem 을 가르지 못해 **손실 있는 행에
`summary.tsv` 라는 라벨만 붙이고**, 7월 열 73행을 **소급 재분류**한다(Non-goal).
★대신 **「73행을 읽었는데 0건 썼다」를 큰 소리로 말한다** — 읽고 안 쓴 것과 없는 것은 다르다.

**⑹ ★고쳐지지 «않는» 것**(제안 `tradeoff` 그대로): 7월 열은 여전히 stem 키라
그 3건의 과거 판정은 **복원되지 않는다**(되돌릴 자료가 없다). 실물:

| 7월 행 | 판정 | source |
|---|---|---|
| `ktf/이노티아연대기2` | FAIL · UNCLASSIFIED | `stem.json` |
| `unknown/이노티아연대기2` | FAIL · UNCLASSIFIED | `stem.json` |

★오늘 재측이 말하듯 **`ktf` 쪽은 PASS 다** ⇒ 7월의 저 `ktf` 행은 **측정이 아니라 옆 행의 사본**이고,
그 사실은 이제 알 수 있지만 **되돌릴 수는 없다**.
⇒ ★**비대칭을 «헤더에 남기기로» 판단했다** — 겹치는 stem 이 있을 때만 한 줄
(`★3 stem(s) exist under 2+ carriers (…)`)이 헤더·stdout 에 나간다. 코퍼스의 속성이므로
그날 입력에 `summary.tsv` 가 있든 없든 참이고, 없을 때 침묵하면 **비교하는 사람이 그 3행을 1:1로 읽는다**.

### ⒝ 축 B — `--signature <regex>`: 문서가 시키던 조회가 네 경고면을 전부 우회하고 있었다

**⑴ ★「주석 줄이라 awk 가 우회한다」를 «쳐서» 확인했다.** 파일이 스스로 문서화한 그 한 줄 그대로:

```
$ awk -F'\t' '$6 ~ /no frame rendered/ {print $1}' <census-map.tsv>
rc=0 · stdout 1줄 · stderr 0줄
그 출력 안의 경고(STALE/★/#) 줄 수 = 0      ← ★네 면 전부 우회
대조: 같은 파일 «안»에는 STALE 문자열이 2줄 실재한다
```

**⑵ `--bucket` 의 형태를 그대로 따랐다** — 경로는 **stdout**, 나머지는 **stderr**, 0건도 **rc=0**
(「없음」은 오류가 아니라 답이다). 새 관용은 만들지 않았다.
★두 필터를 동시에 주면 **rc=2 로 거절**한다(무엇을 물었는지 추측해 부분집합을 답처럼 돌려주지 않는다) ·
★잘못된 정규식도 **rc=2**(리터럴 폴백은 다른 질문에 답하는 것이다).

**⑶ ★★한계 고지를 «함께» 출력한다 — 그리고 그 세 수를 «다시 쟀다».**
제안이 인용한 수치를 독립으로 재측 → **세 건 모두 정확히 일치**:

| 축 | 제안(2026-09-18) | 내 재측(2026-09-19) |
|---|---|---|
| 한 excerpt 값이 덮는 행 | 99 / 187 | ★**99 / 187**(`tick error during 'boot': Fatal error: `) |
| 여러 줄 이유 | 164 / 452 | ★**164 / 452** |
| 첫 줄 120자 초과 | 4 | ★**4** |

★**출력의 그 줄은 «인용»이 아니라 «그 입력에서 계산한 값»이다** — 코퍼스가 움직이면 같이 움직인다:

```
# ★LIMIT col 6 is one truncated line, not a search index — measured on THIS input:
  top excerpt covers 99/187 row(s) · 164/452 json reason(s) are multi-line ·
  4 first line(s) exceed 120 chars. A signature on line 2 cannot match; …
```
★json 이 없는 입력(= `summary.tsv` 만 있는 디렉터리)에서는 **「0」이라 적지 않고 «잴 수 없다»** 고 적는다.

**⑷ 행 안 표식(데이터 오염)은 되살리지 않았다** — 기각된 처방 그대로 둔다.
`--signature` 는 그 기각을 **우회하지 않고** 조회를 도구 안으로 옮긴 것이다.

**⑸ ★대가 — 분류기가 «조회 도구»로 넓어진다**: 이제 이 도구의 답에 도구의 권위가 붙는데
**6열은 그 권위를 감당하지 못한다**. 그래서 한계를 주석이 아니라 **매 호출 출력**에 넣었다.
★범위는 여기서 멈춘다 — **한 정규식 · 한 열 · 조인 없음 · 두 번째 색인 없음.**

**⑹ ★부수로 드러난 것**(고치지 않았다 · 기록만): 같은 6열이라도 **출처에 따라 뜻이 다르다.**
러너는 `summary.tsv` 를 쓰기 전에 줄바꿈을 공백으로 **평탄화**하므로, summary 출처 행의 6열은
「**이유 전체**의 첫 120자」이고 json 출처 행은 「**첫 줄**의 첫 120자」다. 한 줄짜리 이유에서는 동일하고,
여러 줄 이유에서는 summary 쪽이 **더 많이 담는다**(서명 검색에는 유리). 헤더에 그렇게 적었다.

### ⒞ 개악 양방향

**A(입력) — 제품 호출부에 주입, 원복 «바이트 동일» 확인**

```
BASELINE  6행 전건 source=summary.tsv · ktf 두 건 PASS
개악      lookup() 의 경로 키 2단계를 제거(=stem 만 본다)
          → 6행 전건 source=stem.json · ★판정이 달라진 행 2/6
            ktf/다크슬레이어2   PASS → FAIL
            ktf/이노티아연대기2 PASS → FAIL
RESTORED  cmp 바이트 동일 ✔
```

**B(조회) — ★무는 «검사»가 없다. 그렇게 적는다.**
`node scripts/checker-census.mjs` 기준 이 파일은 **callers 0** 바구니이고 `.github/` 참조 **0**
(repo 안의 세 «언급»은 전부 산문·census 목록이지 실행 호출부가 아니다).
⇒ 한계 고지를 지워도 **red 를 낼 검사가 없다.** 보인 것은 red/green 이 아니라 출력 차이다:

```
개악      console.error(columnSixLimits()) 1줄 제거 → stderr 의 LIMIT 줄 수 0
RESTORED  LIMIT 줄 수 1 · cmp 바이트 동일 ✔
```
★**검사를 새로 만들지 않았다** — 이 축의 `tradeoff` 가 「범위를 더 넓히지 마라」이고,
이 저장소의 `game_lab/` 계열은 **CI 가 영원히 못 도는** 자리(Constraint 9)라 새 검사기는
`checker-census` 의 «호출자 0» 바구니를 하나 더 늘릴 뿐이다.

### ⒟ 범위·경계

`game_lab/` **쓰기 0** — 재현·측정은 전부 `mktemp -d` 스크래치(6파일 복사 · 디렉터리째 삭제).
실 `game_lab/census-map.tsv`·`game_lab/reports/` **무접촉**(모든 실행에 `--out <scratch>`).
러너(`game-lab-recensus.sh`) **무접촉** · 행 안 표식 **0** · 7월 열 소급 복원 **0** ·
선행 티켓(PR #218) 몫인 헤더 날짜·`ENGINE_PATHS` **무접촉**(그 커밋 위에 얹었다).
★**게임 이름**: 이 회차가 문서에 더한 코퍼스 stem 은 **3종**(`놈3`·`다크슬레이어2`·`이노티아연대기2`) —
★**전부 새 이름이 아니다**: 세 이름은 `scripts/game-lab-census-map.mjs` 의 기존 주석과
`scripts/game-lab-recensus.sh` 에 **이미 있었다**(선행 회차가 같은 이유로 실어 두었다). 게임 **바이트는 0**.

### ⒠ 이 PR 이 PR #218 «위에» 있다는 사실

티켓은 「`origin/main` 을 다시 당겨 그 착지 위에서 작업하라」고 했는데, 선행 티켓의 PR **#218 은
착수 시점에 열려 있었고 머지되지 않았다**(같은 파일 `+104/-24`). ⇒ ★**그 브랜치 위에 쌓았다** —
`origin/main` 에서 분기하면 같은 파일에서 충돌하고 「그 회차의 변경을 되돌리지 마라」를 지킬 수 없다.
★따라서 이 PR 의 diff 는 **#218 이 머지되기 전까지 그 커밋을 함께 보여 준다**. #218 착지 후에는
이 회차의 커밋만 남는다. **#218 이 재작업되면 이 브랜치의 베이스는 낡는다** — 그때는 리베이스가 필요하다.
