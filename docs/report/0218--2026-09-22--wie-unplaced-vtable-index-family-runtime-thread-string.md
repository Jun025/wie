## [2026-09-22] 미배치 vtable 인덱스 셋을 채웠다 — 체스마스터가 렌더 대열에 합류했고, 알아낸 것이 아니라 «무해하게 만든» 것이다 (wie-unplaced-vtable-index-family-runtime-thread-string)

`data/lgt_java_abi.toml` 에 **행 3개**(+시험 1건). 엔진 로직 변경 **0줄** — 바뀐 것은 **데이터**다.

## ⑴ 무엇을 했나

| 클래스·인덱스 | 넣은 것 | 막혀 있던 타이틀 |
|---|---|---|
| `java/lang/Runtime` 11·12·13 | `freeMemory()J` · `totalMemory()J` · **`gc()V`** | **배틀몬스터** · 학교가는길 · **체스마스터** |
| `java/lang/Thread` **13** | `isAlive()Z` | 메이플스토리2007(렌더 «후») |
| `java/lang/String` **19** | `startsWith(Ljava/lang/String;)Z` | 훼밀리마트타이쿤 |

## ⑵ ★「알아냈다」가 아니다 — 두 문장을 구분해서 읽어라

★**「인덱스 13 이 `gc` 임을 알아냈다」는 «쓰지 않는다».** LGT ordinal 표는 이 회차의 범위 밖이고,
없이는 어느 인덱스가 무엇인지 **확정되지 않는다.** 이 회차가 한 일은 둘이다:

⒜**표에서 규칙을 «도출»하고 기존 측정값으로 교차검증했다**(아래 ⑶).
⒝★**틀려도 다치지 않게 만들었다** — 후보 넷이 **전부 비파괴적**임을 코드로 확인했다(⑷).

⇒ ★**그래서 이 행들은 «확정»이 아니라 «안전한 최선»이다.** 그 구분이 흐려지면 다음 사람이
이 표를 실측으로 오해하고 위에 더 쌓는다.

## ⑶ 도출한 규칙과 그 교차검증

> 서브클래스의 «자기» 메서드는 **index 10 부터 연속**으로(=`java/lang/Object` 가 10칸),
> **CLDC 라이브러리 소스의 선언 순서**대로 들어간다. 상속 메서드의 **오버라이드는 부모 인덱스를
> 재사용**하고 **static 은 칸을 차지하지 않는다.**

★**이 규칙은 «한 점»이 아니라 «구간»으로 검증된다 — 그래서 추측과 구별된다.**

- `java/io/InputStream`: CLDC 선언 순서는 `read() read([B) read([BII) skip(J) available() close()`
  ⇒ 규칙의 예측 `10 11 12 13 14 15`. **표의 측정치 10/11/12/14/15 가 그대로 맞고**, 비어 있는 13 은
  아무 타이틀도 부르지 않은 `skip` 이다.
- `java/lang/String`: `getBytes()=14` 와 `substring(II)=28` **사이 13칸**을 규칙이 채워야 하는데,
  그 구간의 CLDC 메서드가 **정확히 13개**다(`equalsIgnoreCase compareTo regionMatches
  startsWith(S,I) startsWith(S) endsWith indexOf(I) indexOf(I,I) lastIndexOf(I) lastIndexOf(I,I)
  indexOf(S) indexOf(S,I) substring(I)`). 이어서 `trim()=33`·`toCharArray()=34` 도 맞는다.
  ★**여유칸이 없으므로 한 칸이라도 틀리면 양끝이 어긋난다.** 19 는 그 안의 `startsWith(S)` 다.
- `java/lang/Thread`: ★**측정치 `setPriority=14` 가 «자유변수가 아니라 제약»이다.** `run()` 과
  `setPriority` 사이에 **정확히 두 칸**이 필요한데 CLDC **1.1** 이 그 구간에 `interrupt()` 와
  `isAlive()` **둘**을 준다. CLDC **1.0** 은 둘 다 없어 `setPriority` 가 12 가 되고 **측정과 모순**된다.
  ⇒ 12·13 이 강제되고, 13 은 선언 순서상 `isAlive()` 다.

★**`java/lang/Runtime` 만은 앵커가 «없다»**(행 자체가 없던 클래스다). CLDC 선언 순서
`exit(I)V freeMemory()J totalMemory()J gc()V` ⇒ `10 11 12 13`. ★**이것만은 순수 도출이다.**

## ⑷ ★`exit(I)V` — 무엇을 했고, 왜 그렇게 했나

★★**티켓은 「`exit` 를 치명적이지 않게(기록+무동작) 만들라」고 했는데, 그 조치가 «불필요»했다 —
이 JVM 에는 `Runtime.exit` 가 «아예 없다».**
핀 고정된 rustjava(`dlunch/RustJava@5b84dd1`)의 `java/lang/Runtime` 선언은
`<init>` · `getRuntime`(static) · `totalMemory` · `freeMemory` · `gc` 로 ★**가상 메서드가 «셋»**이다.
직전 회신과 티켓이 적은 「가상 메서드 넷(… `exit` 포함)」은 **사실과 다르다**.

⇒ ★**위험이 «제거된» 것이 아니라 «처음부터 없었다».** 그래서:

- 네 슬롯 중 **10 은 비워 뒀다**(= 도출상 `exit` 자리). 게스트가 거기로 분기하면
  `Unimplemented: java/lang/Runtime vtable index 10` 이라는 **자기 이름을 말하는 스텁**을 만난다.
- ★**무동작 `exit` 를 «심지 않은» 것은 선택이고, 사유를 적는다**: `exit` 는 계약 전체가
  «프로세스를 끝내는 것»인 유일한 메서드다. 조용히 아무것도 안 하는 가짜를 놓으면 **게임이 끝나야 할
  자리에서 안 끝나고** 아무도 그것을 모른다. 시끄러운 스텁이 낫다.
- ⇒ ★**어느 순서가 맞든 치명적인 착지점이 없다**: 남은 셋은 전부 무인자이고, 둘은 **같은 상수**
  `0x100000` 을 돌려주며(`totalMemory`↔`freeMemory` 는 **서로 바뀌어도 관측 불가**), `gc` 는 수집을 한 번 돌 뿐이다.
  최악은 **long 하나가 틀리거나 불필요한 수집 1회**이고, **종료되는 경우는 없다.**
- ★**실측이 그 판단을 지지한다**: 46파일 전수에서 `vtable index 10` 을 부른 타이틀은 **0건**이다.

## ⑸ 측정 — before/after 를 «짝지어» 같은 분에 번갈아 (release · `--inject`)

★before 바이너리는 이 회차가 직접 빌드했다(변경 2파일 `git stash` → 빌드 → 복원).

| 타이틀 | before | after |
|---|---|---|
| **배틀몬스터**(2파일) | FAIL **t2** p0 · `Unimplemented: java/lang/Runtime vtable index 13` | FAIL **t1894 / t974** p0 · ★`no frame rendered (hang/black screen)` |
| ★**체스마스터** | FAIL **t1** p0 · 같은 벽 | ★★**PASS · t12641687 · paints 12** |
| 학교가는길 | FAIL t3 p0 · 같은 벽 | FAIL t3 p0 · ★`Unimplemented: **ax** vtable index 30`(앱 고유 클래스 — 새 계급) |
| 훼밀리마트타이쿤 | FAIL **t2** p0 · `String vtable index 19` | FAIL **t1056** p0 · `no frame rendered` |
| 메이플스토리2007 | ★**7회 중 1회** FAIL · `Thread vtable index 13` | ★**7회 중 0회** · 전건 PASS |

★★**「한 칸으로 3종이 동시에 간다」는 주장은 «절반만» 참이었다 — 실측이 그렇게 말한다.**
셋 다 그 벽을 **넘었지만**, 화면까지 간 것은 **체스마스터 하나**다. 배틀몬스터는 tick 2 → **약 1,000배**
전진한 뒤 «부팅은 통과하는데 한 프레임도 안 그리는» 계급(서든어택포켓·턴과 **같은 벽**)에 들어갔고,
학교가는길은 **세 틱 만에 다음 vtable 구멍**을 만난다. ★**진척은 실재하고, 완주는 아니다.**

★**`Thread` 건은 «되는 게임을 더 멀리» 보냈다**: 메이플스토리2007 의 그 벽은 렌더 **후**에 나므로
막을 때도 paints 196~206 이었다. 채운 뒤 그 문면은 **구조적으로 도달 불가**하다(슬롯이 더는 비어 있지 않다).

## ⑹ 회귀 — `game_lab/broken/lgt` **46파일 전건 짝지어** 실행

- 판정·벽 **유지 40/46** · 변경 **6건**(전부 위 표의 대상)
- ★**`PASS → non-PASS` 0건** · `non-PASS → PASS` **1건(체스마스터)** ⇒ PASS **8 → 9**
- ★**렌더 대열 무사**: 놈3(p129→p189) · 메이플스토리2007(p97→p99) · 현영맞고2006(p161→p155) ·
  붉은보석(p455→p492) 전건 **양쪽 PASS**. ★`paints` 를 절대값으로 읽지 마라 — 같은 판정 안의 부하 변동이다.
- 변경 6건 중 하나는 **스파이더맨3**인데, 다른 것은 `InvalidMemoryAccess(3340648398 ↔ 3340649941)`
  **포인터 숫자뿐**이다(선재 패닉 · 같은 계급). ★**「바뀌었다」로 세지 않는다.**
- `LGT 크로이센`·`크로이센` `NO-JSON` 은 **양쪽 다** — 선재이고 무관.
- ★**영웅서기4 는 양쪽에서 흔들린다**(전수 스윕 1회차 PASS ↔ 짝지은 회차 before/after 둘 다 FAIL t639/t683).
  ★**짝지어 재지 않았으면 이것을 회귀로 오독했을 것이다.**

## ⑺ ★측정 위생 — 이 회차가 실제로 밟은 함정 하나 (다음 사람이 또 밟는다)

★★**`target/` 에 남아 있던 낡은 바이너리 때문에 «직전 회차의 결론이 재현되지 않는다»고 1.5시간 믿었다.**
회차 시작 시 `cargo build --release` 가 **9.32초에 "Finished"** 를 찍었고(= 아무것도 다시 짓지 않았다),
그 바이너리는 배틀몬스터에 대해 직전 회신이 적은 `Unimplemented: java/lang/Runtime vtable index 13` 이
아니라 ★**옛 벽 `Invalid memory access; address: 0`** 을 냈다.
⇒ 「직전 회차의 ⑸ 표가 틀렸다」는 가설을 세우고, `handle_missing_java_vtable_entry` 가 **자기 진단 중에
널을 역참조한다**는 (그럴듯한) 근인까지 만들어 **수정 코드를 썼다.**

★**그 가설은 틀렸다.** 같은 소스를 **새로 빌드**하니 원래 핸들러가 **5/5 정확히 이름을 말했고**,
짝지은 스윕의 before 열도 전건 그 문면이다. ⇒ ★**직전 회신 ⑸ 는 옳았고, 내 관측이 낡은 것이었다.**
⇒ **쓴 수정 코드는 버렸다**(값을 못 한 변경은 남기지 않는다 — 직전 회차가 `unwrap` diff 를 버린 그 규율).

★**남길 규율**: 「벽이 내 예상과 다르다」의 첫 용의자는 **코드가 아니라 바이너리**다.
`stat -f %Sm target/release/wie_validate` 와 **빌드 소요 시간**을 먼저 보라 — 수 초짜리 "Finished" 는
«최신»이 아니라 **«아무것도 안 했다»**일 수 있다.

## ⑻ 게이트

4게이트 + `cargo +beta clippy --all -- -D warnings` 전건 rc=0. 엔진 크레이트의 데이터·시험을
만졌으므로 **러너 블록**도 돌렸다(§Definition of Done). 수치는 done 회신에 있다.

시험 `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 은 **개악 대조로 진다** — 행을 지우면
「vtable index N is empty」, 인덱스를 옮기면 이름·디스크립터 불일치로 FAIL.
★**이 시험이 필요한 이유**: 이 표는 파싱 오류를 내지 않는다. 행이 틀리면 **조용히 다른 메서드를 부른다.**

## ⑼ 손대지 않은 것

- ★**배틀몬스터의 새 벽**(`no frame rendered`) — 서든어택포켓·턴과 **같은 계급**이고 티켓이 범위 밖으로 명시했다.
- ★**학교가는길의 `ax vtable index 30`** — 앱 고유 클래스의 구멍이라 **다른 계급**이다(플랫폼 ABI 가 아니다).
- `impl ClassInstance` 의 `unwrap()` 무리 · 스파이더맨3 · SD한국전쟁 · `0x64` ordinal 표 획득 ·
  §7 재조사 — 전부 티켓이 금지했거나 범위 밖. 게임 바이트 커밋 **0** · `game_lab/` 커밋 **0** ·
  새 검사기·CI 스텝 **0** · `STATE.md` 무접촉 · main 무접촉 · **머지 0**.
