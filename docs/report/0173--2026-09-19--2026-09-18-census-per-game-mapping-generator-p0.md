## [2026-09-19] 재실행이 per-game 행을 스스로 남기게 했다 — 그런데 원문이 지목한 «재실행 경로»는 repo 밖이었다 (2026-09-18-census-per-game-mapping-generator-p0)

### ⒜ 원문 대조 — 브리프와 채택 원문은 «글자 그대로» 같다

정본 = `docs/worklog/2026-09-18-census-per-game-mapping-generator.json` 의 `proposals[0]`
(ref `2026-09-18-census-per-game-mapping-generator#p0`). 브리프가 인용한 `title`·`plainSummary`
두 줄은 원문과 **바이트 동일**하다 — 어긋난 곳 0.

★**단 «정본 파일이 없다»는 착시가 한 번 있었다**: 이 워크트리가 기동 시점에 이미 머지된 옛 브랜치
(`wie-restore-lgt-key-and-resource-reach-tests-lost-in-rename`)에 서 있었고 그 트리에는 그 파일이
없다. `origin/main`(`a70a4d2a`)에는 있다. 브리프의 절대경로 표기는 **머지 후 트리 기준**으로 옳다.

★★**그러나 원문 `why` 의 처방 한 줄은 실측으로 성립하지 않는다 — 이것이 이 회차의 핵심 발견이다.**
원문: 「재실행 경로(**`classify.sh`** 또는 그것을 부르는 회차)가 `game_lab/reports/` 를 «갱신»하도록
하면 …」. 두 축에서 안 된다.

| 축 | 실측 |
|---|---|
| ★`classify.sh` 는 **repo 밖**이다 | `.gitignore:23` = `/game_lab/` ⇒ `git ls-files game_lab` = **0**. 거기 고친 것은 CI·리뷰·다른 체크아웃 어디에도 **안 보인다** — 형제 생성기가 `scripts/` 로 간 이유가 정확히 그것이다 |
| ★`classify.sh` 는 **ingest** 다 | `game_lab/inbox/` 를 읽고 각 파일을 `working/`·`broken/` 로 **`mv` 한다**. 이미 분류된 코퍼스를 다시 재는 일은 «파일을 옮기지 않는» 다른 동작이다 |
| ★「그것을 부르는 회차」도 없었다 | 2026-09-18 인구조사는 `classify.sh` 를 **부르지 않았다** — 검증기를 손으로 돌려 `mktemp -d` 에 썼다(그 회신 자체가 「`classify.sh` **0줄**」이라 적었다) ⇒ **고칠 «커밋된 재실행 경로»가 애초에 없었다** |

⇒ 원문의 **목적**(재실행이 행을 남긴다)은 그대로 채택하고, **수단**(classify.sh 개작)은 기각했다.
`classify.sh` **무접촉**이다.

### ⒝ 무엇을 바꿨는가 — 수로

**신규 3파일 · 기존 파일 수정 1**(`AGENTS.md` **+12/-0**) · 제품 Rust **0줄**.

| 파일 | 행 | 무엇 |
|---|---|---|
| `scripts/game-lab-recensus.sh` | **253/0**(신규) | 분류가 끝난 코퍼스를 **읽기 전용**으로 재측하고 per-game 행을 **날짜 디렉터리**에 남긴다 |
| `AGENTS.md` | **+12/-0** | §Web-surface 의 «local only» 목록에 이 쌍(재측 러너 + 매핑 생성기)을 등재 |
| `docs/report/0173` · `docs/worklog/…p0.json` | 신규 | 이 기록 · 채택 기록 |

동작 요약 — `--corpus`(기본 `game_lab/broken`) · `--out`(기본 `game_lab/reports-YYYY-MM-DD`) ·
`--timeout` · `--kill` · `--nice`(기본 **15**) · `--limit` · `--resume` · `--shots` · `--dry-run` ·
`--from-stdin`. 검증기 호출은 baseline 을 만든 `classify.sh` 와 **같은 인자**(`--inject --timeout N`)다.

★**원문 `tradeoff` 가 제안한 «날짜별 디렉터리»를 그대로 채택했다** — 기본 출력이
`game_lab/reports-YYYY-MM-DD/` 이고, 7월 기준선 `game_lab/reports/` 는 **손대지 않는다**.
두 열 비교는 생성기의 `--reports` 두 번이다.

### ⒞ 양방향 개악 — 제품 호출부(스크립트 자신)에 주입, 2축

증적 = `~/orchestrator/reports/evidence/2026-09-18-census-per-game-mapping-generator-p0/`.

**개악 1 — fail-closed 기준선 가드**(`mutation-1-guard.txt`).
`--out game_lab/reports` 는 7월 열을 **제자리에서 덮는** 유일한 불가역 동작이라 경고가 아니라 거절이다.

```
원형  --out game_lab/reports --dry-run  → rc=3 (거절)
개악  가드 분기를 `if false` 로            → rc=0 ★green (= 덮으러 간다)
원복                                      → rc=3
```

> ★★**[게이트② 반려 승계 `-fix` · 2026-09-19] 이 개악은 «틀린 것을 쟀다» — 그리고 가드도 «fail-closed 가
> 아니었다». 둘 다 철회한다.** 이 개악은 분기를 `if false` 로 바꿔 **«분기가 도달하는가»** 를 쟀을 뿐,
> ★**«가드가 지키려는 속성을 지키는가»는 재지 않았다**(이 저장소가 반복해 이름 붙인 「구현을 쟀지 속성을
> 안 쟀다」 그대로다). 실제로 검수자가 잰 결과 `./game_lab/reports` · `game_lab/./reports` ·
> `game_lab/reports/../reports` · **절대경로** 가 전부 **rc=0 통과**였고, 같은 구멍이 **코퍼스 가드에도**
> 있었다(`./game_lab/broken/x` 통과). ⇒ ★**「유일한 불가역 동작을 막았다」가 «점 하나»로 뚫렸다.**
> 처방과 «속성을 재는» 새 개악은 아래 **§`-fix`** 에 있다.

**개악 2 — 내구성: 행이 살아남는가**(`mutation-2-durability.txt`). 개악은 2026-09-18 의 «그 형태»다
— 출력 디렉터리를 `mktemp -d` + `trap rm -rf` 로 되돌린다.

```
원형  --limit 1 실행 → 남은 .json 1 · 생성기 「reports matched 1」
개악  OUT="$(mktemp -d)" + trap rm → 남은 .json 0 · 생성기 rc=2 「does not exist」 ★red
원복  → 남은 .json 1 · 「reports matched 1」
```

**왕복 실측**(`run-a-noshots.txt`): 재측 3건 → `reports-ab-noshots/` 3 json →
`game-lab-census-map.mjs --reports <그 디렉터리>` → **매핑 3행**(파일 경로 + 버킷)이 나온다.
인구조사가 잃은 바로 그 자료를 **명령 하나**로 낸다.

**모집단 일치**: 이 스크립트가 세는 코퍼스는 **파일 187 · 중복 stem 3** 으로,
인구조사·생성기의 실측(187/184/3)과 **정확히 같다**.

### ⒟ 대가 — 숨기지 않는다

1. ★**45분을 싸게 만들지 않는다.** 전수 재측은 여전히 전수 재측이고, 이 맥은 self-hosted 러너를
   겸한다(2026-09-18 회차가 그 45분에 열린 PR 5건을 세웠다). 바뀐 것은 **한 번만 치른다**는 것뿐이다.
2. ★**CI 가 이것을 못 본다.** 코퍼스가 실제 게임 바이트라 Constraint 9 상 어떤 검사도 이 경로를
   돌릴 수 없다 ⇒ 이 스크립트가 **썩어도 red 가 나지 않는다.** `scripts/smoke_gate.sh` 와 같은 계급이다.
   `checker-census.mjs` 의 «호출자 0» 바구니가 하나 늘어난다(그 계수는 결함이 아니라 질문이다).
3. ★★**stem 충돌은 여전히 자료를 잃는다.** per-game JSON 은 생성기가 찾는 키가 stem 이라 `<stem>.json`
   이고, 두 캐리어가 같은 제목을 가지면 **나중 실행이 앞엣것을 덮는다**(코퍼스 실측 3건). 없애지 못했고
   **시끄럽게만** 만들었다 — 시작할 때 그 수를 출력하고, `summary.tsv` 를 **경로 키**로 써서 파일 단위
   진실은 살린다. ⇒ 「조용한 손실」이 「알려진 손실」이 됐을 뿐이다.
4. ★**디스크.** 기준선의 shot 디렉터리는 **286 MB / 6,069 PNG**(실측)이고 그것이 **한 회차분**이다.
   그래서 `--shots` 를 opt-in 으로 뒀다 — 그러면 baseline 과 **인자가 달라진다**(아래 ⒡).
5. ★**날짜 디렉터리는 스스로 쌓인다.** 회수·회전 장치를 넣지 않았다(`game_lab/` 은 gitignore 라 아무도
   안 본다). 전수 1회 = json 187 + (shots 켜면) 약 286 MB.
6. ★**`classify.sh` 는 여전히 `game_lab/reports/` 에 쓴다** — 새 게임을 ingest 하면 기준선 디렉터리에
   **행이 는다**. 기존 187행은 그대로이므로 7/7 재현은 안전하지만, 「기준선은 불변」이 **아니다**.
   고치려면 repo 밖 파일을 고쳐야 하고 그건 위 ⒜의 이유로 안 했다.

### ⒠ 게이트 — 내 출력

`gates.txt` 참조. 이 회차의 diff 에 **Rust 0줄**이지만 `AGENTS.md` §Definition of Done 이
「무엇을 바꿨든 넷 다 돌린다」라 전부 돌렸다.

### ⒡ 부수 실측 — 적어 두지 않으면 다음 사람이 다시 밟는다

★★**같은 게임 3건을 두 번 돌렸더니 버킷이 갈렸다**(load 170 → 202 · `run-a`/`run-b`):
`UNCLASSIFIED ×3` ↔ `only blank/uniform` · `panic-unwrap` · `no frame rendered`.
엔진은 **한 글자도 안 바뀌었다**. `ticks` 는 A 에서 2~3, B 에서 3~39 다.
⇒ ★**`AGENTS.md` 가 이름 붙인 «짝짓지 않은 표집» 그대로다** — 두 팔이 `--shots` **와** 부하 분(minute)
둘 다에서 갈렸고 부하 쪽이 훨씬 크게 움직인다. ⇒ ★**`--shots` 가 판정을 움직이는지는 «재지 못했다»**
(그 A/B 는 한산한 기계와 짝짓기가 필요하다). 스크립트 주석에 그 한계를 그대로 적었다.
★그리고 이 관측은 이 회차의 논거를 **강화한다**: 판정이 부하에 묶이므로 per-game 행은 **날짜와 함께**
남아야 하고, 「어느 게임이었나」를 잃으면 저부하 재측이 **구조적으로 불가능**해진다(게이트②가 그 자리에서 막혔다).

★**작은 결함 1건 — 고치지 않고 적는다**: 생성기 헤더의 `★reports mtime range` 는 `toISOString()` =
**UTC** 인데 디렉터리 이름은 **로컬 날짜**다. 실측: 05:26 KST(=전날 20:26 UTC)에 쓴
`reports-2026-09-19` 를 생성기가 `2026-09-18` 로 표시한다. **하루 어긋날 수 있다**.
이번 회차에서 고치지 않은 이유 = 그 줄은 형제 파일의 출력 포맷이고 모든 기존 TSV 의 그 줄이 같이 바뀐다
⇒ 후속 제안(`#p1`)으로 남긴다.

### 범위·경계

`classify.sh` **0줄** · 제품 Rust **0줄** · `.gitignore` **무접촉** · 인구조사 표·`docs/report/0165`
**무접촉** · `STATE.md` **무접촉**(2026-09-18 이후 일상 착지는 이 파일을 건드리지 않는다) ·
새 검사기·새 워크플로 **0** · `AGENTS.md` 에 **fenced `sh` 블록 0**(그것을 더하면 `doc-liveness.yml`
동기 의무가 붙는데 이 명령들은 코퍼스가 필요해 주(週) 러너가 돌릴 수 없다 — 인라인 코드로 적었다).
코퍼스 **쓰기 0**(스크립트는 코퍼스 디렉터리에 절대 쓰지 않고, `--out` 이 코퍼스 안이면 rc=3).
★**게임 파일명 유입 0** — 이번에 손댄 4파일(신규 3 + `AGENTS.md`)을 코퍼스 고유 stem **184**개와 NFC 완전일치로 전수 대조해 **0건**을 실측했다
(형제 회차가 「0」을 잘못 주장한 자리라 세어서 적는다).


---

## §`-fix`(2026-09-19) — ★가드를 «문자열 비교»에서 «해석된 경로 비교»로

**변경 = 제품 1파일 + 문서 1.** `scripts/game-lab-recensus.sh` 의 `norm()`(1줄)을 `canon()` 으로 바꾸고
★**두 가드가 «같은 술어»를 쓴다** — 술어 2벌 금지(한쪽만 고치면 다른 쪽이 남는다는 것이 반려 사유의 절반이었다).
`AGENTS.md` 의 그 문장도 **같은 PR 에서** 넓혔다(검수자가 「문서가 코드보다 넓게 약속한다」로 지목한 줄).

**⒜ 심링크 — «따라간다»(`cd -P`/`pwd -P`).** 이 가드가 지키는 것은 기준선의 **바이트**이므로 그 바이트에
닿는 심링크는 같이 거절돼야 한다. ★대가도 적는다: **다른** 디렉터리를 가리키는 별칭을 일부러 쓰는 호출자는
«친 문자열»이 아니라 «가리키는 곳»으로 판정된다. 실측 — 기준선을 가리키는 심링크 **rc=3** ·
딴 곳을 가리키는 심링크 **rc=0**.

**⒝ 아직 없는 경로 — «정규화된다».** 출력 디렉터리는 나중에 `mkdir -p` 로 생기므로 «존재해야 하는» 가드는
여기서 쓸모가 없고 `realpath`/`fs.realpath` 는 부재 경로에서 실패한다. ⇒ **①어휘 정규화**(`//`·`.`·`..` 를
접는다 — 만들어진 적 없는 꼬리에서도 된다) → **②존재하는 최장 접두만 물리 해석** → 남은 꼬리를 붙인다.
분할 중 글로브가 도는 것은 `set -f` 로 막았다.

**측정 — ★양쪽 다 낸다**(한쪽만이면 그냥 다 막은 것이다) · 증적 `evidence/…-p0-fix/guard-normalization-after.txt`:

| 거절돼야 하는 것 | rc | 허용돼야 하는 것 | rc |
|---|---|---|---|
| `game_lab/reports` | **3** | `game_lab/reports-2026-09-19` | **0** |
| `game_lab/reports/` | **3** | `./game_lab/reports-2026-09-19` | **0** |
| `./game_lab/reports` | **3** | `<abs>/game_lab/reports-2026-09-19` | **0** |
| `game_lab/./reports` | **3** | `game_lab/./reports-ab` | **0** |
| `game_lab/reports/../reports` | **3** | `game_lab/reports2` | **0** |
| `<abs>/game_lab/reports` | **3** | `game_lab/reportsX/../reportsX` | **0** |
| `game_lab/broken/x` | **3** | `game_lab/broken2/x` | **0** |
| `./game_lab/broken/x` | **3** | `<scratch>/out` | **0** |
| `game_lab/broken/../broken/x` | **3** | `<scratch>/link-to-elsewhere` | **0** |
| `<abs>/game_lab/broken/deep/x` | **3** | `game_lab/reports-mut/sub/deep` | **0** |
| `<scratch>/link-to-baseline`(심링크) | **3** | | |

★**`reports2`·`broken2`·`reportsX` 를 일부러 넣었다** — 접두 비교로 대충 막으면 그 셋이 함께 걸린다.

**양방향 개악 — ★이번에는 «속성»을 잰다**(`evidence/…/mutation-normalisation.txt`):
```
원형   ./game_lab/reports · game_lab/./reports · …/../reports · 절대경로 · ./game_lab/broken/x  → 전건 rc=3
개악   canon() 첫 줄에 `printf '%s' "${1%/}"; return 0`(= 정규화 제거)                          → 전건 rc=0  ★red
원복                                                                                            → 전건 rc=3
대조군 정규 철자(`game_lab/reports`·`game_lab/broken/x`)는 세 상태 «모두» rc=3
```

**남는 구멍 — 적어 둔다.** 기준선 가드는 여전히 **«같음»** 이지 **«안쪽»** 이 아니다
(`--out game_lab/reports/sub` 는 통과한다). ★그것이 옳다고 판단했다 — 러너가 쓰는 것은 `$OUT/<stem>.json`
이라 **하위 디렉터리는 기준선의 판정 파일을 덮지 않는다**. 다만 그 디렉터리 «안에 새 하위 트리»가 생기는 것은
사실이고, 그래서 여기 적는다.

**끝에서 끝까지 확인**: 고친 뒤 `--limit 1 --out game_lab/reports-fixsmoke` 가 정상 동작해 **json 1건**이
남고 생성기가 「reports matched 1」을 낸다 ⇒ ★**가드를 조이면서 정상 경로를 막지 않았다.**
