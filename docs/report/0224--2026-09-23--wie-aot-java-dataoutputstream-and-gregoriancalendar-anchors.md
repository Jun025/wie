## [2026-09-23] `DataOutputStream 19` · `GregorianCalendar 29` — 닻을 «측정»으로 만들었다 (wie-aot-java-dataoutputstream-and-gregoriancalendar-anchors)

### 무엇을

`wie-lgt/data/lgt_java_abi.toml` 한 파일. 직전 회차(PR #255)가 **일부러 비워 둔** `java/io/DataOutputStream`
index 19 를 포함해, 세 클래스에 행을 넣었다. 전부 이번 회차에서 게스트가 스스로 드러낸 값으로 **측정**한 것이고,
선언 순서에서 **추측으로 파생한 칸은 하나도 넣지 않았다**.

| 클래스 | 채운 칸 | 무엇으로 쟀나 |
|---|---|---|
| `java/io/OutputStream` | 11 `write([B)V` | 인자 1개 · 그 인자의 클래스가 게스트 힙에서 `[B` 로 읽힘(서든어택포켓 9회 전건) |
| `java/io/DataOutputStream` | 15 `writeBoolean(Z)V` 16 `writeByte(I)V` 17 `writeShort(I)V` 19 `writeInt(I)V` 20 `writeLong(J)V` | index 20 만 **r2 가 0 으로 지워진다** = 64비트 인자. run 시작점이 15 로 강제된다 |
| `java/util/Calendar` | 19 `get(I)I` 29 `setTime(Ljava/util/Date;)V` | 19 의 인자가 `1 2 5 11 12 13`(Calendar 필드 상수 Y/M/D/h/m/s) · 29 의 인자 클래스가 `java/util/Date` |

**18(`writeChar`)은 비워 뒀다** — 디스패치한 타이틀이 없다(이 파일이 `skip(J)`=13 에 쓰는 것과 같은 규율).
run 은 이제 측정됐으므로, 어떤 타이틀이 요구하면 한 줄이면 된다.

### 왜 — ⑴ 읽기-쪽 오라클은 서지 않았다. 그래서 다른 것을 쟀다

티켓이 제안한 오라클은 「index 19 로 쓴 뒤 `readShort`(25)로 읽는가 `readInt`(28)로 읽는가」였다.
**라운드트립은 관측되지 않았다.** 훼밀리마트타이쿤의 실행은 읽기(리소스)가 앞, 세이브 쓰기가 끝이고,
쓴 것을 같은 실행에서 되읽지 않는다. `wie_validate` 의 파일·DB 는 **메모리 전용**이라 실행 간 보존도 없다.
티켓의 규정대로라면 여기서 「채우지 말고 그 사실을 적어라」로 끝나야 했지만, **다른 축이 그 칸을 강제했다.**

**게스트의 호출 규약이 곧 오라클이다.** LGT 컴파일러는 `java.*` 호출의 인덱스를 인라인하지만(임포트 테이블에
이름이 **한 번도** 나오지 않는다 — 실측), 인자를 싣는 방식은 ABI 를 따라야 한다. 훼밀리마트타이쿤 세이브
기록부의 **한 연속 구간**, 같은 스트림 객체, 연속 호출:

```
index 16 ×13   r2 = 0x490503c0, c1, c2 ... cb   ← 게스트가 1씩 증가시키는 문자열 커서
index 19 ×1    r2 = 0x490503cb                  ← 그대로다. r2 는 인자가 아니다
index 20 ×5    r2 = 0x0                         ← 커서를 «지웠다». r2 가 둘째 인자 워드다
```

실행 전체로도 같다 — r2 가 0 인 적이 **단 한 번도 없는** 15(230회·서로 다른 값 114개) 16(151/58)
17(69/34) 19(24/7) 에 대해, 20 은 **10회 전건 0**.

⇒ index 20 은 64비트를 받고 15..19 는 32비트를 받는다. `DataOutputStream` 선언 순서
(`writeBoolean writeByte writeShort writeChar writeInt writeLong …` — 이 파일의 **측정된** `DataInputStream`
run 이 따르는 바로 그 소스 순서)에서 **첫 두-워드 항목은 `writeLong`, 시작점에서 +5** 다.
그래서 시작점은 15 이고, **index 19 = `writeInt(I)V`** 다.

**다른 시작점은 같은 데이터가 전부 반증한다** — 11·12·14·16·17·18 은 한-워드로 측정된 칸에 두-워드 메서드를
올리고, 13 은 `OutputStream` 의 선언 5개에 3칸만 남긴다. **15 만 남는다.**
직전 회차가 걱정한 후보(시작점 17 ⇒ 19 = `writeShort`)는 «덜 그럴듯해서»가 아니라 **측정으로 반증**됐다.

### 왜 — ⑵ 비워 두게 만든 «설명되지 않는 두 칸»은 계산 착오였다

직전 회차의 주석은 「`PrintStream.println(String)V = 34` 인데 시작점 15 셈은 32 까지만 닿는다 ⇒ 두 칸이
설명되지 않고, 그 두 칸이 `OutputStream` 의 것이면 시작점은 17」이라고 적었다.
**그 두 칸은 `PrintStream` 자신의 것이다** — 세던 것이 print/println 18개뿐이라 `checkError()Z` 와
`setError()V` 를 빠뜨렸다(CLDC 소스는 print 블록 «앞»에 선언하고, rustjava 도 둘 다 선언한다 — 확인함).
15 부터 다시 세면 checkError 15 · setError 16 · print ×9 = 17..25 · println() 26 · println ×7 = 27..33 ·
**println(String) 34**. 정확히 맞는다. ⇒ **그 측정은 시작점 15 를 반증하는 것이 아니라 «확증»한다.**

### 왜 — ⑶ 배치 규칙 자체를 처음으로 «측정»했다

이 파일의 규칙(「자식의 자기 메서드는 10 부터 CLDC 선언 순서로」)은 그동안 파생이었다. 이번에 **게스트가
직접 만든 컴파일러 vtable** 에서 두 번 쟀다 — 디스패치가 아니라 클래스 등록 시점의 표에서:

- 놈3 의 AOT 클래스 `f extends java/io/InputStream` — `vtable_count=19`, 자기 슬롯 `10 12 13 14 15 16 17 18`
  (11 = `read([B)` 는 상속 그대로). ⇒ **InputStream 은 정확히 19칸** = 10 + CLDC 인스턴스 메서드 9개.
  `readByte()B=23` 앵커가 독립적으로 가리키던 바로 그 값이다.
- 스파이더맨3 의 `c extends java/lang/Thread` — `vtable_count=21`, 자기 슬롯 `11 18 19 20`
  (11 = `run()` 오버라이드, 18..20 = 새 메서드 3개). ⇒ **Thread 는 18칸** = 10 + 8.
  `setPriority=14` 앵커와 일치.

두 클래스, 두 개수, 둘 다 같은 규칙에 떨어진다. `java/io/OutputStream` 은 선언이 5개
(`write(I) write([B) write([BII) flush() close()`)이므로 **15칸** — 위 레지스터 측정이 독립적으로
확증한 바로 그 값이다.

### 왜 — ⑷ GregorianCalendar 는 위치가 아니라 «인자»가 이름을 말했다

`java/util/Calendar` 의 크기를 고정하는 앵커는 없고, 코퍼스에 Calendar 서브클래스도 없다.
그래서 파생으로는 한 칸도 못 넣는다. 대신 훼밀리마트타이쿤이 두 칸을 **스스로 이름 붙였다**:

- **index 19** — `Calendar.getInstance()` 직후 연속 6회, 인자 `1 2 5 11 12 13` =
  `YEAR MONTH DAY_OF_MONTH HOUR_OF_DAY MINUTE SECOND`. 이 순서를 받는 메서드는 `get(I)I` 뿐이다.
- **index 29** — 그 바로 앞의 단 1회, 인자의 클래스를 게스트 힙에서 읽으면 **`java/util/Date`**.
  이 계층에서 `Date` 하나를 받는 메서드는 `setTime(Ljava/util/Date;)V` 뿐이다.

둘 다 `Calendar` 선언이므로 행은 **부모에** 넣었다(`vtable.rs` 의 조상 순회가 자식 빌드에서 집어 간다).

### 3타이틀 전/후 벽

| 타이틀 | 전 | 후 |
|---|---|---|
| 턴 (`턴.zip` · `(LGT)턴.zip`) | `Unimplemented: java/io/DataOutputStream vtable index 19` | `… java/io/ByteArrayOutputStream vtable index 16` |
| 서든어택포켓 (`서든어택포켓.zip` · `lgt 서든어택 포켓.zip`) | 같은 칸 | `… java/io/ByteArrayOutputStream vtable index 16` + `FileNotFoundException`/`Invalid filename` |
| 훼밀리마트타이쿤 | `Unimplemented: java/util/GregorianCalendar vtable index 29` | `… java/lang/StringBuffer vtable index 22` |

다섯 파일 전건이 벽 너머로 갔다. **아직 아무도 paint 하지 않는다** — 새 벽이 그 앞에 있다.

### `FileNotFoundException` 의 판정 — vtable 문제가 «아니다», 그리고 ABI 행을 채운 뒤에도 남는다

티켓의 가설(「index 19 가 막혀 쓰기가 실패해 파일이 안 생긴 것」)은 **틀렸다**. 실측: 서든어택포켓은
`org.kwis.msp.io.File::<init>(name, mode=1, 0)` 으로 연다. **mode 1 = `READ_ONLY`** 이므로 쓰기 경로가 아예
개입하지 않고, 첫 부팅에 존재한 적 없는 세이브를 **읽으려다** 나는 정상적인 부재다
(`wie_validate` 의 `MemoryFilesystem` 은 실행마다 비어 있다).

이름을 붙일 결함은 따로 있다 — **`wie-wipi-java/src/classes/org/kwis/msp/io/file.rs` 의 `init_with_flag` 가
`RandomAccessFile` 실패를 종류와 무관하게 `IOException("Invalid filename")` 으로 덮는다**(바로 옆에
`// TODO check exception type` 이 있다). 게스트가 받는 문면은 「파일 이름이 잘못됐다」인데 실제 조건은
「파일이 없다」이고, 예외 종류로 «없으면 만든다» 를 가르는 게스트는 그 분기를 놓친다.
고치지 않았다 — 이 회차 밖이고, 후속 제안으로 올렸다.

### 라이브 4종 회귀 — 퇴행 없음. 그리고 «짝짓지 않은 비교»에 한 번 속을 뻔했다

`놈3` · `메이플스토리2007` · `현영맞고2006` · `체스마스터`. 뒤 셋은 양쪽 트리에서 **3/3 PASS**
(`content=true`, `distinct_colors` 불변).

`놈3` 은 처음에 **퇴행처럼 보였다** — 변경 트리 3회 중 1회 FAIL(load ~41), 원본 트리 3/3 PASS(load ~32).
AGENTS.md §Definition of Done 이 이름 붙인 «짝짓지 않은 표본»이 그대로 재현된 것이다. 같은 분에
**교차 실행**으로 다시 재니:

| | FAIL | PASS |
|---|---|---|
| before (`origin/main` 바이너리) | **3/6** | 3/6 |
| after (이 브랜치 바이너리) | **2/6** | 4/6 |

실패 모양도 동형(`paints=0` · `content=false`). ⇒ **부하 기아이지 이 변경의 퇴행이 아니다.**
기아임은 CPU 로도 보인다 — FAIL 회차는 벽시계 12.66s 에 user 0.89s(7%), PASS 회차는 20.39s 에 7.12s.
예산을 10→20s 로 늘리면 같은 파일이 PASS 로 돌아온다(선형).

`놈3` 은 이 회차의 **측정 대상이기도 하다**(`f extends java/io/InputStream` 이 InputStream=19칸을 쟀다).

### 게이트

`cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0 ·
`RUST_MIN_STACK=4194304 cargo test --all` **0 failed**(clet 회귀 `wie-lgt` 포함) ·
러너 블록 6종 전건 PASS(`keydraw_ktf`/`keydraw_lgt` 둘 다 `paints=55` · `last_frame_content=true` · rc=0,
문서화된 유휴 구간 48–55 안) · `check-worklog-json` OK · `check-worklog-coverage` **10/10 = 100%, 기한 내**.

### 부수: 코퍼스 전수 조사에서 나온 벽 목록 (LGT 100개 · 예산 20s)

`broken/lgt` 46 + `working/lgt` 54. vtable 벽은 이것이 전부다 — **`DataOutputStream` 의 «두 번째 측정점»은
없었다**(티켓의 보조 축 ⒜ 는 서지 않았고, 그래서 위 호출 규약 축으로 갔다).

| 벽 | 타이틀 |
|---|---|
| `java/io/DataOutputStream 19` | 턴 ×2 — **이 회차가 닫음** |
| `java/util/GregorianCalendar 29` | 훼밀리마트타이쿤 — **이 회차가 닫음** |
| `java/lang/String 21` | 일지매영웅전기 — **새로 발견** |
| `Game 5` | 배틀몬스터 ×2 — 별 티켓(`wie-battlemonster-…`) |
| `ax 30` | 학교가는길 — 이 회차 밖 |

### 사용자 영향

세이브를 쓰는 LGT 타이틀 3종(파일 5개)이 멈춰 있던 칸을 지났다. 화면은 아직 안 나온다 — 그 다음 벽이 남아 있다.
대신 **조용히 잘못된 폭으로 쓸 위험이 사라졌다**: index 19 가 `writeShort` 였다면 게스트의 세이브가 2바이트씩
어긋난 채 «정상처럼» 저장됐을 텐데, 그 후보는 이제 측정으로 배제됐다.

### 계측에 대해

vtable 스텁·호출 프록시·클래스 등록 지점에 붙인 로그는 전부 **`option_env!` 로 감싼 일회용**이었고,
최종 diff 에 **한 줄도 남기지 않았다**(변경 파일은 ABI toml · 이 문서 · 워크로그 셋뿐).
게임 바이트·`game_lab/` 커밋 **0**, 새 검사기·CI 스텝 **0**.

uptime: `load averages: 25.16 32.42 37.98` (측정 종료 시점) · 회차 내내 25–107 사이를 오갔다.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus <코퍼스>` 를 **실행해서** 적는다(손으로 세지 않았다).
`--corpus` 를 준 것은 코퍼스가 이 워크트리 밖(`~/work/otterpebble/wie/game_lab`)에 있기 때문이다.

**BOUNDED 62회 / 31쌍** · **SUFFIX-ATTACHED 21회 / 6쌍** · PREFIX-EMBEDDED 1회 / 1쌍.
★**0건이 아니다** — 이 회차는 어느 타이틀이 어느 칸에서 멈추는지가 결론이라 제목을 **일부러** 쓴다
(`lgt_java_abi.toml` 이 이미 같은 이유로 턴·서든어택포켓·훼밀리마트타이쿤을 적고 있다).
SUFFIX-ATTACHED 6쌍은 눈으로 갈랐다 — **5쌍은 «더 긴 다른 제목» 안에 짧은 stem 이 들어앉은 것**
(`서든어택` ⊂ `서든어택포켓`, `훼밀리마트타이쿤` 자기 자신), **1쌍만 조사가 붙은 진짜 언급**
(`서든어택포켓은`). 게임 **바이트**나 `game_lab/` 경로는 커밋에 **0건**이다.

<!-- corpus-name-inflow v1 subjects=3 tree=d1a6be1abf642b4f B=62/31 P=1/1 S=21/6 -->
