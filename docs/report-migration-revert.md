# `REPORT.md` 회차 파일 이관 — **되돌리는 절차** (명령)

> 티켓 `wie-report-md-per-round-files-port-from-otterpebble`(2026-09-07).
> 형제 저장소 `otterpebble` 의 착지분(`636bb8e7`)을 포팅한 것이고, ★**이 문서도 함께 포팅했다** —
> 그 리니지가 **네 회차**(반려 3 + approve 1)를 쓴 자리가 대부분 «되돌리기·이행 레시피»였기 때문이다.
> ★**「되돌릴 수 있다」고 쓰지 않는다 — 아래를 그대로 실행하면 된다.**

## 0. 무엇이 바뀌었나 (되돌릴 대상)

| # | 자리 | 바뀐 것 |
|---|---|---|
| 1 | `docs/report/*.md` **51파일** | `REPORT.md` 원문을 회차별로 가른 것(요약·편집 0) |
| 2 | `REPORT.md` | 643줄 원장 → **자라지 않는 고정 안내** |
| 3 | `AGENTS.md` §Landing paperwork | 「`REPORT.md` 상단 append」 → 「`docs/report/` 새 파일」 |

★**`STATE.md` 는 대상이 아니다** — 가르지 않았다(현재 상태 파일이라 회차별 파일이 성립하지 않는다).

## 1. 가장 싼 되돌림 — 이관 커밋을 revert

```bash
# 이관 PR 의 착지 커밋을 찾는다(꼬리표로 찾는다 — 리터럴 -F)
git log --oneline --first-parent origin/main \
  | grep -F '[wie-report-md-per-round-files-port-from-otterpebble]'

git revert --no-commit -m 1 <그 sha>   # ★-m 1 — 이 저장소는 upstream-sync 라 «머지 커밋»으로 착지한다
git commit -m 'revert(repo): REPORT.md 회차 파일 이관 되돌림'
```
★★**위 명령이 답하는 것은 «머지 커밋»이지 «이관 커밋»이 아니다** — `--first-parent` 가 브랜치 안쪽을 걷지 않기 때문이다.
`revert -m 1` 에는 그 머지 sha 가 **맞다**. ★**그러나 §2 의 `$MIG`(이관 커밋)에 그것을 대입하지 마라** —
`<머지>^` 는 **착지 직전 `main`** 이라 이관 이후 회차를 이미 먹은 원문이 나온다. 이관 커밋은 이렇게 짚는다:
```bash
git log --format=%H --grep='wie-report-md-per-round-files-port-from-otterpebble' | tail -1   # ★--first-parent 없음
```
⇒ `REPORT.md` 가 643줄 원장으로 돌아오고 `docs/report/` 가 사라진다.
★**이관 뒤 새로 쓰인 회차 파일은 이 revert 로 지워지지 않는다**(그 커밋에 없던 파일이다) — 2 절로 합쳐라.

## 2. 이관 «후» 회차까지 합쳐 원장 한 파일로 복원

★★**파일명 앞 «연번»이 순서의 정본이다** — 날짜가 아니다.
```
docs/report/NNNN--YYYY-MM-DD--<slug>.md      NNNN = 전역 연번(오래된 것이 0001) · slug = 회차 티켓 id
```
★**이관 시점 `wie` 원장은 날짜 단조였다**(비단조 **0**지점 · 51절 실측). ★★**그런데도 연번을 넣었다** —
형제 저장소는 같은 자리에서 **비단조 12지점**이었고 거기서 「이름 오름차순 = 바이트 동일」이라 적었다가
**거짓으로 판명**됐다(424/436절이 어긋났다). ⇒ ★**단조성은 보장이 아니라 우연이다. 우연에 기대지 마라.**

```bash
# 복원 — ★연번 «내림차순»이 원문 배열(최신 → 과거)이다. 머리글 두 줄이 앞에 붙는다
{ printf '# REPORT\n\n'; ls docs/report/*.md | sort -r | xargs cat; } > /tmp/REPORT.restored.md
```
※`wie` 의 회차 파일은 **51개 전부 꼬리 개행이 있다**(실측 0/51 결손) ⇒ `cat` 으로 이어도 절이 붙지 않는다.
★**형제 저장소는 436개 중 10개가 개행 없이 끝나 `cat` 을 금지했다** — 이 저장소에 그 조건이 생기면
(회차 파일을 손으로 만들다 개행을 빠뜨리면) 같은 금지가 되살아난다. **되돌리기 전에 위 실측을 다시 하라**:
```bash
c=0; for f in docs/report/*.md; do [ "$(tail -c1 "$f" | od -An -c | tr -d ' ')" = '\n' ] || c=$((c+1)); done; echo "$c"
```

### ★검산 — ★**순서를 «보는» 것으로 한다**(줄 수 대조는 부적격)
★★**⒜ 의 기준 sha 는 «이관에 든 `0001`~`0051`» 의 것이다 — 위 복원본은 «이관 이후 회차»까지 포함하므로 그대로 대조하면 «반드시» 어긋난다.**
★그 어긋남은 결함이 아니라 **범위 차이**다. sha 축으로 보려면 **범위를 잘라서** 재라.
```bash
# ⒜ 기준값과 sha256 대조 — 이관 시점 원문: 4f85ec30…  213,785B  51절
#    ★sed 로 «이관분만» 남긴다. 빼면 이관 이후 회차를 함께 먹어 틀린 답이 나온다
{ printf '# REPORT\n\n'; ls docs/report/*.md | sort -r | sed -n '/\/0051--/,$p' | xargs cat; } | shasum -a 256

# ⒜-b 독립 축 — 이관 «부모 커밋»에 직접 물으면 범위 지정이 필요 없다
# ★★--first-parent 를 붙이지 마라 — 착지 뒤에는 그것이 «머지 커밋»을 집어 답을 뒤집는다(아래 참조)
MIG=$(git log --format=%H --grep='wie-report-md-per-round-files-port-from-otterpebble' | tail -1)
git show "$MIG^:REPORT.md" | shasum -a 256
git show "$MIG^:REPORT.md" | grep -c '^## \['     # ★51 이어야 한다. 53 이면 부모를 잘못 짚었다

# ⒝ 더 확실한 축 — §1 의 revert 결과와 «내용 그대로» 대조한다(★복원본 전체를 본다 · 같은 $MIG 를 쓴다)
git show "$MIG^:REPORT.md" > /tmp/REPORT.orig.md
diff /tmp/REPORT.orig.md /tmp/REPORT.restored.md && echo "★순서까지 동일"
```
★★**`--first-parent` 를 붙이면 «착지 뒤» 뒤집힌다 — 브랜치 위에서는 돌기 때문에 눈에 안 띈다.**
이 저장소는 **upstream-sync 라 머지 커밋으로 착지**한다(§1 이 `-m 1` 로 적은 그 사실이다) ⇒
`main` 의 first-parent 경로에 **이관 커밋이 없고**, 남는 매치는 머지 커밋뿐이며 `$MIG^` = **착지 직전 `main`** 이 된다.
★그것은 «이관 이후 회차를 이미 먹은 원문»(절 **53**)이라 sha 가 어긋난다 — ★**`--first-parent` 없는 `tail -1`
= 최고령 매치 = 이관 커밋**이고, 그 형태는 **착지 전에도 후에도 옳다.** ★위 절수 한 줄이 그 갈림을 즉시 가른다.
★★**「줄 수가 같다」로 검산하지 마라 — 형제 저장소에서 그 검산이 «통과해 버렸다».**
줄 수는 **순서에 무감**이라 424절이 뒤섞여도 초록이 난다.
※이관 «후» 회차가 쌓였으면 ⒝ 의 `diff` 는 그 회차만큼 차이가 난다 — **그 차이가 «새 회차뿐»인지** 보라.

```bash
mv /tmp/REPORT.restored.md REPORT.md && rm -rf docs/report
```

### 새 회차의 연번
**가장 큰 수 + 1**을 쓴다(재번호 0). 두 회차가 같은 수를 골라도 **파일명이 slug 로 갈려 충돌은 나지 않고**,
잃는 것은 그 둘 «사이»의 상대 순서뿐이다(같은 시각에 난 회차라 원장 의미상 무해).
```bash
printf '%04d\n' $(( $(ls docs/report | sed -E 's/^([0-9]{4})--.*/\1/' | sort -n | tail -1 | sed 's/^0*//') + 1 ))
```

## 3. 쓰는 쪽 원복 지점 (한 곳뿐)

```bash
git diff <이관 sha>^ <이관 sha> -- AGENTS.md      # 바뀐 문구 확인
```
- `AGENTS.md` §Landing paperwork — 회차 기록 줄을 **`REPORT.md` 상단 append** 로 되돌린다.
★`.claude/rules/` 는 이 저장소에 **없다**(형제 저장소에는 있어 원복 지점이 둘이었다).

## 4. 읽는 쪽 — ★**원복할 것이 없다**

이관 시점 실측(추적 파일 **485개** 모집단): `REPORT.md`·`STATE.md` 를 **프로그램으로 읽는 자리 0건** —
`.github/` **0** · `scripts/` **0**. 이름이 나오는 곳은 전부 산문·이력이다
(`AGENTS.md` 1 · `docs/upstream-realign-verdict.md` · `docs/worklog/*.json` 10 · 두 파일의 상호 언급).
```bash
# 재산출
git grep -lIE 'REPORT\.md|STATE\.md' -- .github scripts | wc -l     # ★0 이어야 한다
git grep -lIF 'REPORT.md' -- . ':!REPORT.md'
```
※`.gitignore` 의 `REPORT` 히트 두 줄은 `/STEP2_REPORT.md`·`/BRIDGE_REPORT.md` 로 **다른 파일**이다(오탐).
⇒ 게이트가 이 파일에 의존하지 않으므로 되돌려도 **깨질 소비처가 없다**.
★**단 이 사실은 «이관 시점»의 것이다** — 되돌리기 전에 위 명령으로 **다시 세라**.

## 5. 되돌려야 할 신호 (판단 재료)

- `docs/report/` 파일 수가 너무 많아 디렉터리 탐색이 실용적이지 않다 → 되돌림이 아니라 **연/월 하위 디렉터리**가 답이다.
- 색인을 커밋해야 할 요구가 생겼다 → ★**그때가 이 구조의 이점을 잃는 순간이다**(색인이 다시 «추가 지점»이 된다).
- ★**`.gitattributes` `merge=union` 으로 갈아타고 싶어졌다** → ★**그것이 이 이관이 거부한 방향이다.**
  union 은 충돌을 «없애는» 것이 아니라 «숨기는» 것이고, 그 결과 **충돌이 안 나면서 의미가 깨진다.**

---

# 부록 — 이관 «직후» 열린 PR 5건을 푸는 법 (한시적 · 이 절은 다 풀리면 지운다)

★**이관은 열린 PR 을 «일시적으로 더 나쁘게» 만들지 않는다 — 이 저장소는 이미 전건 충돌이었다.**
이관 시점 실측(`git merge-tree --write-tree` vs `origin/main`):

| | REPORT.md 충돌 | 충돌 없음 |
|---|---:|---:|
| **이관 «전»** | **5**(#98·#99·#100·#105·#106) | **0** |

⇒ ★**새로 나빠지는 PR 은 없다.** 다만 **해소 방식이 바뀐다** — 아래 레시피를 쓴다.
★그리고 **`STATE.md` 충돌은 그대로 남는다**(이관 대상이 아니다) — 그쪽은 종전대로 합집합 해소다.

## 해소 레시피 (그 PR 의 게이트③ 회차가 `git merge origin/main` 직후에 친다)

```bash
# 1) 내 브랜치가 REPORT.md 에 «추가한 절 전건»을 떼어 각각 새 파일로 옮긴다
#    ★★기준은 :1(merge-base)이다 — :3(theirs)로 재지 마라.
#      이 레시피가 쓰이는 시점의 :3 = 착지 후 main 의 REPORT.md = «고정 안내» = 절 0개다.
#      그러면 `mine ∖ theirs` = 내 절 «전건»이 되어 원장을 통째로 복제한다
#      (형제 저장소 게이트② 실측: 417건 복제가 «검산 통과»로 넘어갔다).
#    ★★한 절만 뜨면 안 된다 — 브랜치가 2절 이상 더한 경우가 흔하다(형제 실측: 13건 중 9건).
git show :1:REPORT.md > /tmp/base.md            # :1 = merge-base(공통 조상) ★차집합의 기준
git show :2:REPORT.md > /tmp/mine.md            # :2 = ours(내 브랜치 쪽)
python3 - <<'EOF'
import os, re, hashlib
def secs(p):
    t=open(p,encoding="utf8").read().split("\n"); f=False; st=[]
    for i,l in enumerate(t):
        if l.startswith("```"): f=not f
        if not f and l.startswith("## ["): st.append(i)
    return ["\n".join(t[s:(st[k+1] if k+1<len(st) else len(t))]) for k,s in enumerate(st)]
# ★★비교는 반드시 `rstrip()` 위에서 한다 — `secs()` 가 절을 「헤딩 ~ 다음 헤딩 직전」으로 자르므로
#   슬라이스에 **경계 빈 줄이 포함**된다. 새 절이 삽입되면 «그 앞 절»의 꼬리 빈 줄이 1개 달라져
#   원문 그대로 비교하면 **기존 절이 «추가»로 오인**된다(형제 실측: 충돌 PR 14건 중 ★4건 = 29%).
mine, base = secs("/tmp/mine.md"), {b.rstrip() for b in secs("/tmp/base.md")}
added=[b for b in mine if b.rstrip() not in base]  # ★merge-base 에 없던 = 내가 «추가한» 절
os.makedirs("docs/report", exist_ok=True)
def norm(x): return hashlib.sha256(x.rstrip().encode("utf8")).hexdigest()   # ★가드도 같은 정규화
have={norm(open(f"docs/report/{f}",encoding="utf8").read()) for f in os.listdir("docs/report")}
dup=[b for b in added if norm(b) in have]
if dup:                                          # ★독립 축 — 절 «수» 산술이 아니라 «내용»을 본다
    raise SystemExit(f"★중단 — 이미 docs/report 에 있는 절 {len(dup)}건을 복제하려 한다."
                     " 기준이 :1 인지 확인하라(아무것도 쓰지 않았다).")
nxt=max([int(f[:4]) for f in os.listdir("docs/report") if f[:4].isdigit()], default=0)
for b in reversed(added):                        # 오래된 것부터 = 작은 연번부터
    nxt+=1
    d=re.match(r"^## \[(\d{4}-\d{2}-\d{2})\]", b).group(1)
    n=f"docs/report/{nxt:04d}--{d}--<slug>.md"   # ★<slug> 를 회차 티켓 id 로 바꿔라
    open(n,"w",encoding="utf8").write(b if b.endswith("\n") else b+"\n")
    print("옮김:", n, "·", b.split("\n")[0][:60])
print(f"★추가 절 {len(added)}건 전건 이동")
EOF

# 2) REPORT.md 는 «main 쪽(고정 안내)»을 그대로 받는다 — 내 절은 이미 1)로 옮겼다
git checkout --theirs -- REPORT.md 2>/dev/null || git checkout origin/main -- REPORT.md

# 3) 검산 — ★«절 수»로 본다(줄 수는 순서·누락에 둔감하다). ★기준은 여기서도 :1 이다
/usr/bin/grep -c '^## \[' /tmp/mine.md                     # :2 내 브랜치 절 수
/usr/bin/grep -c '^## \[' /tmp/base.md                     # :1 merge-base 절 수
ls docs/report/*.md | wc -l                                # ★(:2 − :1) 만큼 늘었어야 한다

git add docs/report REPORT.md && rm -f /tmp/mine.md /tmp/base.md
```
★★**이관 PR «자신»의 머지 회차는 방향이 반대다.** 그 PR 은 `:2`(ours)가 이미 «고정 안내»(절 0개)이고
새 절은 **main 쪽(`:3`)에** 쌓인다 ⇒ 1단계에서 **mine 을 `:2:` 대신 `:3:` 로** 읽는다(기준 `:1` 은 그대로,
2단계 `--theirs` 는 **`--ours`** 로 뒤집는다). ★중복 가드는 양쪽 방향 모두에서 그대로 지킨다.

★★**`rstrip()` 정규화가 «안전한» 이유 — 절 본문의 유의미한 꼬리 공백이 원장에 없기 때문이다.**
`rstrip()` 은 **꼬리 공백만 다른 두 절을 같다고 본다** ⇒ 그 차이가 유의미하면 **진짜 새 절을 버린다**.
그래서 전제를 수로 확인한다 — 이관 시점 `docs/report` 51건을 `rstrip` 정규화해 세면 **중복 0**이다(실측).
★**바꾸기 전에 다시 세라**(0 이 아니면 이 정규화를 쓰면 안 된다):
```bash
python3 -c 'import os,hashlib,collections
d="docs/report";c=collections.Counter(hashlib.sha256(open(os.path.join(d,f),encoding="utf8").read().rstrip().encode()).hexdigest() for f in os.listdir(d))
print("파일",sum(c.values()),"· rstrip 정규화 후 중복",sum(v-1 for v in c.values() if v>1))'
```

★★**3단계 검산이 red 면 — «새로 쓴 파일을 먼저 지우고» 다시 판단하라.**
1단계는 검산 **«전»에 이미 파일을 쓴다** ⇒ red 시점에 그 파일들이 디스크에 남아 있다. 회수:
```bash
git status --short docs/report          # 이번 실행이 만든 것 확인(?? 또는 A)
git clean -f docs/report                # 미추적분 제거 · 이미 add 했으면 git rm --cached 후 삭제
```
★**지우지 않고 기준만 고쳐 다시 돌리면** 남은 파일 때문에 연번이 어긋나고 중복 가드가 «자기가 만든 것»을 문다.

★★**전제 — 원장은 append-only 다.** `:1` 기준 차집합은 「**내가 더했다**」와 「**상대가 지운 것을 내가 안 지웠다**」를
구별하지 못한다. 회차 절은 지우지 않는 것이 이 원장의 규율이라 실무상 무해하지만, ★**그 전제가 깨지면 이 기준도 깨진다** —
그때는 `diff <(/usr/bin/grep '^## \[' /tmp/base.md) <(/usr/bin/grep '^## \[' /tmp/mine.md)` 로 **사라진 절이 있는지** 먼저 보라.

★★**왜 «검산»도 이 모양인가 — 형제 저장소에서 두 회차 연속으로 «초록인데 틀렸다»가 났다.**
옛 3단계는 `(내 절 수 − theirs 절 수)` 였는데 그것은 **1단계가 쓴 것과 같은 양**이라 1단계가 무엇을 하든 항상 일치한다(항진식).
그래서 417건 복제를 PASS 시켰다. 지금은 두 축이 서로 다른 것을 본다 —
⑴3단계 검산이 `:1` 기준이라 1단계가 `:3` 으로 돌아가면 **불일치가 난다**
⑵1단계 자신이 **내용 동일 중복**을 보고 **쓰기 전에 멈춘다**(절 수 산술과 무관한 축).

★**`--theirs` 를 «내용 판단 없이» 쓰는 유일한 자리다** — 여기서만 안전한 이유는 **내 기여를 1)에서
이미 다른 파일로 옮겼기** 때문이다. ★다른 충돌에 이 패턴을 옮겨 쓰지 마라(원장 규율: 한쪽 전량 채택 금지).
