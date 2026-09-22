## [2026-09-22] `no frame rendered` 넷은 한 병이었다 — 삼켜진 `java/io` 스트림 예외 (wie-aot-java-boots-but-never-paints-hang-cluster)

### 무엇을

`no frame rendered (hang/black screen)` 로 모여 있던 LGT-AOT 4타이틀(7파일)의 근인을 이름으로 확정하고,
런타임이 문면으로 지목한 `java/io/DataInputStream` 세 칸을 채웠다. 넷째 칸(`java/io/DataOutputStream`
index 19)은 **일부러 비워 두었다** — 근거는 아래 §비워 둔 칸.

`wie-lgt/data/lgt_java_abi.toml` 한 파일 · 데이터 3행 + 주석. `.rs` **0줄**.

### 왜 — 0이 되는 축은 `Display::pushCard` 였고, 그 원인은 그 위에 있었다 (그리고 이 회차가 그 0을 옮겼다)

`RUST_LOG` 로 JVM 호출 축을 셌다. ★**세 열 모두 실측이고, 각 열이 어느 트리·어느 예산인지 적는다**
(배틀몬스터 before 는 로그 전량이 **278줄** · 그 전부가 약 0.2초 안에 끝나므로 예산을 늘려도 줄이 늘지 않는다):

| 축 | 배틀몬스터 **before**<br>(`dd88d4fc` · 25 s) | 배틀몬스터 **after**<br>(이 브랜치 · 60 s) | 체스마스터 **대조군**<br>(이 브랜치 · 60 s) |
|---|---|---|---|
| `org.kwis.msp.lcdui.Display::getDefaultDisplay` | 5 | 7 | 2 |
| `javax.microedition.lcdui.Image::createImage` | 17 | 21 | 100 |
| `org.kwis.msp.lcdui.Display::pushCard` | **0** | ★**1** | 1 |
| `net.wie.CardCanvas::pushCard` | **0** | ★**1** | 1 |
| `net.wie.CardCanvas::paint` | **0** | **0** | 2 |
| `net.wie.EventQueue::getNextEvent` | **0** | **0** | 3 |
| `net.wie.EventQueue::dispatchEvent` | **0** | **0** | 2 |
| `Graphics::drawImage` | **0** | **0** | 4 |
| 던져진 java 예외 | **1** | 289 | **0** |

**before 에서 0이 되는 지점은 `Display::pushCard`** 다. 카드가 등록되지 않으므로 `CardCanvas::paint` 도
이벤트 루프도 불릴 길이 없다.

**그런데 근인은 `pushCard` 가 아니라 그 위였다.** 278줄 전량이 **약 0.2초 안에** 끝나고 남은 예산
전체가 침묵인데, 그 마지막에 예외가 하나 있었다:

```
INFO jvm::jvm: throwing java exception: net/wie/WieError
                Unimplemented: java/io/DataInputStream vtable index 25
```

게스트는 리소스를 `DataInputStream` 으로 읽다가 미배치 vtable 칸을 밟고, **그 예외를 자기 catch 로
삼킨 뒤** 화면을 등록하지 않은 채 조용히 돈다. 그래서 검증기에는 크래시가 아니라 「한 프레임도 안 그림」
으로 보인다. ★**`no frame rendered` 가 이 계급을 감춘 것이 아니라, 삼켜진 예외가 감춘 것이다.**

4타이틀 전건이 같은 형태였고, 지목된 칸만 달랐다:

| 타이틀 | 삼켜진 예외 |
|---|---|
| 배틀몬스터(2파일) | `java/io/DataInputStream` index **25** |
| 훼밀리마트타이쿤 | `java/io/DataInputStream` index **28** |
| 서든어택포켓(2파일) | `java/io/DataInputStream` index **28** |
| 턴(2파일) | `java/io/DataOutputStream` index **19** |

대조군 체스마스터는 같은 트리에서 **예외 0건**이다. 이 한 줄이 「부하 탓인가」를 끊는다.

★★**그리고 이 회차의 3행이 그 0을 «옮겼다»** — 위 표 after 열에서 `pushCard` 와 `CardCanvas::pushCard` 가
**0 → 1** 이다. 화면은 이제 **등록된다.** 0이 되는 자리는 그 다음 칸, `CardCanvas::paint` 와
`EventQueue::getNextEvent` 로 내려갔다. ⇒ 검증기 문면(`no frame rendered`)은 전후가 같은데 **안에서는
한 칸 전진했다** — 문면만 보면 놓치는 종류의 진척이고, 그래서 이 표를 남긴다.

※`CletWrapper*` 축은 **세 열 전부 0** 이다. 이 네 타이틀과 체스마스터는 네이티브 clet 이 아니라
**AOT Java** 라 `org/kwis/msp/lcdui/Card` 로 직접 가고, `net/wie/CletWrapper` 경로를 타지 않는다.
그 축이 0인 것은 결함이 아니다 — 대조군도 같다.

### 기아가 아니다 — 예산이 아니라 «칸»이었다

`no frame rendered` 는 「안 그린다」와 「아직 못 그렸다」를 구별하지 못하는 버킷이라, 이 리니지가 세 번
밟은 함정이다(현영맞고 `--timeout 25` FAIL → `30` PASS). 그래서 먼저 예산을 9배로 늘려 보았다:

| 예산 | ticks | paints | CPU(user) | ticks / CPU-s |
|---|---|---|---|---|
| `--timeout 20` | 1,116 | 0 | 7.84 s | **142.3** |
| `--timeout 180` | 10,484 | 0 | 73.21 s | **143.2** |

★**tick 은 CPU 시간에 정확히 선형**이고(142.3 ↔ 143.2), 9배 예산은 9.4배 tick 을 샀는데 **paints 는
0에서 움직이지 않았다.** 같은 트리·같은 분에 체스마스터는 `--timeout 30` 으로 **PASS**(t50,000,000 =
tick 상한 도달 · p1 · content true)다. ⇒ 멈춘 원인은 예산이 아니다.

`ticks` 를 처방의 증거로 쓰지 않은 이유도 여기 적는다 — 이 값은 **예산이 다르면 비교할 수 없고**
(AGENTS.md 가 「throughput 측정치가 아니다」라고 이미 못박았다), 부하에 따라 한 파일이 0.43M~50M 로
움직인 선례가 있다. 이 회차의 판정은 전부 **벽 문면**으로 했다.

### 채운 칸 — 파생이 아니라 «측정된 닻»에서 나온다

`readByte()B = 23` 은 **측정값**이고, 이 파일의 규칙(하위 클래스 고유 메서드는 10부터 CLDC 선언 순서로
연속, 오버라이드는 부모 인덱스 재사용, static 은 칸 없음)을 여기에 걸면 run 이 **강제된다**:

```
InputStream  10 read()  11 read([B)  12 read([BII)  13 skip(J)  14 available()  15 close()
             16 mark(I)  17 reset()  18 markSupported()
DataInputStream 고유 run 은 19 부터:
             19 readFully([B)  20 readFully([BII)  21 skipBytes(I)  22 readBoolean()  23 readByte() ←닻
             24 readUnsignedByte()  25 readShort()  26 readUnsignedShort()  27 readChar()  28 readInt()
```

★**이 닻이 파일의 기존 주석을 한 군데 교정한다.** 그 주석은 InputStream 을 「read() read([B) read([BII)
skip(J) available() close() = 10..15」 여섯 개로만 적었는데, **그것이 전부라면** DataInputStream 고유
run 이 16부터 시작해 `readByte` 가 **20**에 앉는다 — 측정값 23과 어긋난다. 23이 되려면 상속 칸이 셋 더
있어야 하고, CLDC 1.1 InputStream 은 `close()` 뒤에 정확히 셋(`mark` `reset` `markSupported`)을 준다.
⇒ 16·17·18이 채워지고 고유 run 이 19에서 시작하는 것이 **측정으로 고정된다.**

그래서 채운 세 칸은 닻에서 각각 **3칸 앞(20) · 2칸 뒤(25) · 5칸 뒤(28)** 이고, 전부 **하나의 연속 run
안**이다. 이 파일의 어떤 파생 행보다 짧은 거리다.

★**세 칸 모두 «런타임이 먼저 이름을 불렀다».** 25·28을 채운 뒤 서든어택포켓이 **20**을 새로 요구해서
그때 20을 채웠다 — 요구되지 않은 칸을 미리 채운 적은 **없다**(§손대지 않은 것).

### 비워 둔 칸 — `java/io/DataOutputStream` index 19 (턴 · 서든어택포켓)

★**채우지 않았다. 이것은 미완이 아니라 판단이다** — `java/lang/Runtime` index 10 을 비워 둔 것과 같은 형태다.

출력 쪽에는 **자기 닻이 없고**, 가까운 유일한 측정값이 평범한 파생을 **확인해 주는 대신 반증한다**:
CLDC OutputStream 은 `write(I) write([B) write([BII) flush() close()` 라 하위 고유 run 이 15에서
시작해야 하고, 그러면 DataOutputStream 은 15 writeBoolean · 16 writeByte · 17 writeShort ·
18 writeChar · **19 writeInt** 다. 그런데 같은 파일의 `PrintStream.println(String)V` 는 **34**인데,
같은 셈으로 PrintStream 의 앞선 print/println 선언 18개를 세면 **32**까지밖에 닿지 않는다.
★**두 칸이 설명되지 않고**, 그 둘이 OutputStream 것이라면 DataOutputStream run 은 17에서 시작해
index 19 는 `writeInt` 가 아니라 **`writeShort`** 다.

두 후보는 **같은 스트림에 서로 다른 바이트 폭을 쓴다.** 하나가 다른 하나를 대신하면 게스트가 나중에
읽어 들이는 자기 데이터가 **조용히 깨지고**, 그 회차는 겉보기에 정상으로 끝난다. 지금은 그 칸이
`Unimplemented: java/io/DataOutputStream vtable index 19` 로 **자기 이름을 말한다** — 두 실패 중 나은 쪽이다.

### 옮겨 간 벽 — 4타이틀 전건 (전 `--timeout 30` / 후 `--timeout 120`)

| 타이틀 | 전 | 후 |
|---|---|---|
| 배틀몬스터 2파일 | `DataInputStream 25` · t315 / t409 · p0 | ★`Game vtable index 5`(**앱 고유 클래스**) + `IllegalMonitorStateException` ×288 · t7499 / t8193 · p0 |
| 훼밀리마트타이쿤 | `DataInputStream 28` · t233 · p0 | ★`java/util/GregorianCalendar 29` · t8395 · p0 |
| 서든어택포켓 2파일 | `DataInputStream 28` · t86 / t97 · p0 | ★`DataOutputStream 19` · t1275 / t2818 · p0 |
| 턴 2파일 | `DataOutputStream 19` · t412 / t481 · p0 | 불변 `DataOutputStream 19` · t6712 / t1440 · p0 |

⇒ ★**`java/io/DataInputStream` 은 7파일 전건에서 닫혔다** — 이제 그 클래스의 칸을 요구하는 타이틀이
**하나도 없다.** 그리고 서든어택포켓이 턴과 **같은 칸**으로 수렴했다 ⇒ 비워 둔 그 한 칸이 이제
**4타이틀 중 2타이틀의 유일한 벽**이다.

★**paints 는 전건 0 그대로다 — 어느 것도 «플레이 가능»해지지 않았다.** 진척은 실재하고 완주는 아니다.

### 사용자 영향

- **지금 당장은 없다.** 4타이틀은 여전히 `paints 0` 이고, 바뀐 것은 벽의 위치와 이름이다.
- **렌더 4종은 그대로 돈다**(featurephone.otterpebble.com 라이브분): 놈3 · 메이플스토리2007 ·
  현영맞고2006 · 체스마스터 **4/4 PASS**. 자세한 실측은 §회귀.
- 다음 한 칸(`DataOutputStream 19`)이 닻을 얻으면 턴·서든어택포켓 2타이틀이 동시에 움직인다.

### 회귀 — ★놈3 의 FAIL 한 건은 «기아»였고, 짝지어 재서 끊었다

첫 회차에서 **놈3 가 FAIL**(load1 46.43 · `--timeout 150`)했다. 라이브 타이틀이라 AGENTS.md
§The four gates 의 절차대로 **결정적 단계(손대지 않은 트리에서 재현)** 를 밟았다 — before 바이너리
(`dd88d4fc` = `origin/main` 이 실은 그것)와 after 를 **한 루프 안에서 교대**로 돌려 같은 부하 분을 보게 했다:

| | run1 | run2 | run3 |
|---|---|---|---|
| **before**(dd88d4fc) | PASS p1110 | PASS p242 | PASS p425 |
| **after**(이 브랜치) | PASS p602 | PASS p218 | PASS p535 |

★**6/6 PASS · 양쪽 다 3/3** 이고 paints 범위가 겹친다(242~1110 ↔ 218~602) ⇒ 첫 FAIL 은 **굶은 런**이고
이 변경의 회귀가 아니다. ★**교대 짝짓기 없이 before 만 따로 돌렸으면 이 판정을 못 했다** — 이 맥의 부하는
한 회차 안에서 loadavg 23~97 로 움직였다.

| 타이틀 | 결과 |
|---|---|
| 놈3 | **PASS** (교대 3/3 · 위 표) |
| 메이플스토리2007 | **PASS** t50,000,000 · p381 · dc 63 · nd 8.8% |
| 현영맞고2006 | **PASS** t50,000,000 · p139 · dc 62 · nd 5.8% |
| 체스마스터 | **PASS** t50,000,000 · p1 · dc 36 · nd 17.6% |

### 손대지 않은 것

- ★**`Game vtable index 5`**(배틀몬스터의 새 벽) — **앱 고유 클래스**라 ABI 표의 대상이 아니다
  (학교가는길 `ax index 30` 과 같은 계급 · 티켓이 범위 밖으로 확정한 것).
- ★**`IllegalMonitorStateException` ×288**(배틀몬스터) · **`GregorianCalendar 29`**(훼밀리마트타이쿤) ·
  **`FileNotFoundException`/`IOException Invalid filename`**(서든어택포켓) — 이 회차가 새로 **관측**했고
  각자 다른 계급이다. 이름만 붙이고 두었다(후속 제안).
- ★요구되지 않은 `DataInputStream` 칸(19·21·22·24·26·27·29~32) — 파생으로는 읽히지만 **채우지 않았다.**
- `#254` 재측정 **0** · `otterpebble`·`tower`·원장 **무접촉** · 게임 바이트 커밋 **0** ·
  `game_lab/` 커밋 **0** · 새 검사기·CI 스텝 **0** · 계측 코드 최종 diff **0줄**.

### 코퍼스 게임 파일명 유입

★**도구로 쟀다**(`node scripts/corpus-name-inflow.mjs` · 재유도하지 않았다):
**유입 26건(BOUNDED)** · **판단 필요 12건(SUFFIX-ATTACHED)** · PREFIX 0건. 주제 파일 3건.

★**0이 아니고, 0일 수 없다** — 이 회차의 산출물 자체가 「어느 타이틀이 어느 칸에서 멈추는가」라
타이틀 이름이 본문의 내용이다(직전 두 회차 `0218`·`0219` 도 같다). 들어온 것은 **이름뿐이고
바이트는 0** 이다: 게임 바이트 커밋 0 · `game_lab/` 커밋 0 · 경로 0. SUFFIX-ATTACHED 12건은 전부
`서든어택`→`서든어택포켓`, `턴`→`턴과`처럼 **더 긴 제목이거나 조사가 붙은 언급**이고 새 유입이 아니다.

### 전제 — `depends_on` 게이트

착수 시점(14:55 KST) `Jun025/wie#254` 는 **OPEN** 이었고 `origin/main` 에 vtable 행이 **없었다**
(`git log origin/main -1 -- wie-lgt/data/lgt_java_abi.toml` → `0c4706a2`). 그 사실을 적고, 측정은
**승인 핀 `dd88d4fc`**(그 PR 의 tip · 검수 `verdict: approve`) 트리에서 시작했다.
`#254` 는 **15:25 KST 에 머지**됐고(`9787eceb`), 이 브랜치는 그 뒤 `origin/main` 에서 갈라졌다.
`git merge-base --is-ancestor dd88d4fc origin/main` → **YES** ⇒ 「전」 실측과 「후」 실측이 **같은 계보**
위에 있다. ★낡은 트리에서 잰 수는 **없다.**

<!-- corpus-name-inflow v1 subjects=3 tree=dcb2d484556912d4 B=69/26 P=0/0 S=36/12 -->
