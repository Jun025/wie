## [2026-10-28] SKT 구매 휴대폰 잠금 5판 해제 — 클래스 해시 표로 `isValid` 만 참으로 (wie-skt-five-purchase-lock-normalize)

**무엇을**: 0386 이 「설계대로 도는 DRM」으로 분류하고 둔 SKT `SecureUtil` 계열 잠금 5판
(`0ed66634d3dc` `a42f77f44955` `c33090c12755` `eefc947d8337` `44b6356d13f8`)을 켰다. 바꾼 것은 둘이다.
- `wie-jvm-support/src/class_patches.rs` — **클래스 파일 MD5 로 걸리는 표** 5행. 맞는 클래스의
  `isValid(MIDlet)` 한 메서드만 본문을 돌리지 않고 `true` 를 돌려준다.
- `JvmRuntime::define_class` 가 게스트 클래스를 정의하면서 그 표를 조회한다(`InheritedMethods` 가
  그 메서드만 `ForcedBoolMethod` 로 감싼다).

`compat.json` 5행을 실측값으로 고치고 이용자 소식 1건을 더했다. census 코드는 바뀌지 않았다(주석만).

**왜**: 운영자 지시 2026-10-28 「SKT 휴대폰 잠금 5종도 게임빌과 같은 조건으로」. 2026-10-02 「인증 우회 0」은
**판마다 운영자가 푼다**(2026-10-26 LGT `01f05f8231f4` · 0540 에 이어 두 번째). 다른 잠금 판
(KTF OMA DRM 2종 · KTF 이노티아2 2판 · 운영자가 이름대지 않은 SKT 판)의 정책은 그대로다.

**사용자 영향**: 다섯 판이 「지원 안 됨」에서 「플레이 가능」이 된다. 거절 상자 대신 시작 화면을 지나
각자의 본 게임 화면까지 들어간다.

### 1. 다섯 판은 전부 같은 잠금이다 — 번호를 몰라도 되는 자리

0386 §1 이 읽은 XCE `SecureUtil` 계열 하나다. `validate` 는 `isValid` 가 거짓일 때만 4줄
(「인증 되지 않은 / 컨텐츠 입니다. / …」)을 그리고 `System.exit(-1)` 하며, 참이면 **아무 일도 하지 않는다**.
그래서 고치는 자리는 `isValid` **한 곳**이고, 돌려주는 것은 번호가 아니라 **판정**이다.
`isValid` 안의 `MIN`·`MIDlet-Key`·MD5 는 호출되지도 않는다.

| sha12 | 클래스(그 판의 jar 안) | 참으로 돌리는 메서드 | `validate` 자리 |
|---|---|---|---|
| `0ed66634d3dc` | `b` | `l(Ljavax/microedition/midlet/MIDlet;)Z` | `b.m(MIDlet)` |
| `a42f77f44955` | `j` | `if(…)Z` | `j.a(MIDlet)` |
| `c33090c12755` | `com/xce/security/SecureUtil` | `isValid(…)Z` | `SecureUtil.validate` |
| `eefc947d8337` | `SecureUtil`(기본 패키지) | `isValid(…)Z` | `SecureUtil.validate` |
| `44b6356d13f8` | `a` | `a(…)Z`(private) | `a.a(MIDlet)` |

★**`44b6356d13f8` 의 벽은 `install.dat` 가 아니다 — 0418 §표(57행)의 귀속이 틀렸다.** 바이트코드
(`q` 의 `static {}`)를 다시 읽으면 `install.dat` 는 **설치 표식**이다: `DataInputStream.available() >= 12`
이면 `readLong`(시각)·`readInt`(`m.MIN` 의 `hashCode`)를 읽고, 없으면 그 자리의
`catch` 가 받은 뒤 **두 줄 아래에서 `writeLong(currentTimeMillis)` + `writeInt(hashCode)` 로 새로 만든다**.
종료는 그다음 줄의 `a.a(SF.a)` — 곧 위 표의 `validate` — 에서 난다. 그래서 티켓 2의 ⒜·⒝ 중
**어느 쪽도 아니고**, 나머지 4판과 똑같은 1번 길로 처리했다(가상 `install.dat` 를 내줄 필요가 없다).

### 2. 왜 «클래스 해시»인가 — 범위가 새지 않는 이유

- **메서드 이름 규칙은 쓸 수 없다**: 5판 중 3판이 난독화돼 `l`·`if`·`a` 다. 이름으로 걸면 같은 글자를 쓰는
  남의 클래스에 걸린다.
- **jar 해시는 자리가 없다**: `define_class` 는 클래스 한 장만 받는다. jar 바이트를 거기까지 끌고
  내려가야 하는데, 클래스 해시가 더 좁으면서 공짜다.
- 표에 **해시 없는 행은 없다**. 조회가 빗나가면 바뀌는 것이 0이다(`an_unlisted_class_is_untouched`).

**전수 실측(2026-10-28 · SKT 코퍼스 84 패키지의 모든 `.class` 엔트리 · 내부 jar 포함)**:
이 5개 MD5 에 맞는 클래스는 **정확히 5개, 정확히 그 5 패키지 안**이다. 같은 계열(salt `a0a535ef35b`)을
가진 패키지는 **41개**이고 그중 36개는 전부터 playable 이다 — 해시가 다르니 **한 판도 걸리지 않는다.**

### 3. 측정

release `wie_validate`. base = 이 브랜치에서 **표만 비운 빌드**(`forced_bools` 가 항상 빈 목록) —
엔진의 다른 차이는 없으므로 «개악 대조»와 «회귀 기준»을 겸한다. load1 은 측정 내내 9~41 이었다.

**⑴ 다섯 판 (census 형식 A·B 30초 · L 600초 · S)**

| sha12 | base A | head A | head L | head 판정 |
|---|---|---|---|---|
| `0ed66634d3dc` | UNMEASURED · clean exit · 그림 0 | PASS · deadline · 그림 34 | 키 854/900 · 그림 167 | **playable** |
| `a42f77f44955` | UNMEASURED · clean exit · 그림 0 | PASS · deadline · 그림 264 | 854/900 · 5,639 | **playable** |
| `c33090c12755` | UNMEASURED · clean exit · 그림 0 | PASS · deadline · 그림 2,708 | 854/900 · 51,660 | **playable** |
| `eefc947d8337` | UNMEASURED · clean exit · 그림 0 | PASS · deadline · 그림 62 | 854/900 · 235 | **playable** |
| `44b6356d13f8` | UNMEASURED · clean exit · 그림 8 | PASS · deadline · 그림 1,477 | 854/900 · 8,956 | **playable** |

축은 다섯 판 모두 boot/render/input/longplay/sound **ok** 다. speed 는 셋이 ok(0.997~0.999)이고
`0ed66634d3dc`·`44b6356d13f8` 은 **unknown** 이다 — 두 판은 `sleep` 을 한 번도 하지 않아 #347 비율의
분모가 비는 쪽이지 느린 것이 아니다.

**⑵ 개악 대조** — 표를 비운 base 가 바로 그 대조다. `report` 를 돌리면 다섯 판이 전부
`not-yet` + 「구매한 휴대폰에서만 켜지도록 잠긴 파일…」로 **되돌아간다**(소비 지점 = census 의 `lockVerdict`).
head 에서는 다섯 행 모두 `playable` · 안내 문장 0건이다.

**⑶ 범위 대조 · SKT playable 표본 15종** (0411 §1 의 `SecureUtil` 보유 playable 36종 중 표본 5종
`0262a4fe3389` `66959afab216` `78bd51675574` `a0436eb1ddbb` `fb7a86ce425b` + 일반 SKT 10종
`02bc66e85cf4` `2d66945008c1` `3de4bcb4f5e2` `4f99be2762e2` `7089dec0e8df` `976141a9525b`
`b42e4242866b` `ccdd9295c5c4` `d6abd6258d08` `f2ae515201f2`):
A·B **30쌍 전부 같은 판정 · 같은 종료**(PASS · deadline)다. 그림 수 차는 load 구간의 차다. 회귀 **0**.

### 4. 번호 유출 0

- 패치는 비교 **결과**만 돌려주므로 어느 단계에서도 번호를 읽지 않았다. MD5 전수 대입 0 · 역산 0.
- 엔진의 `MIN`/`m.MIN` 값(`01000000000`)은 그대로다.
- 변경분 diff · 새 파일 · 회신 · 측정 로그 · 캡처 파일명에서 11자리 이상 숫자열을 훑어 `01000000000`
  외 **0건**임을 확인했다(수치는 회신 파일).
- 캡처 5장은 본 게임 화면이고 번호를 그리지 않는다. 경로는 회신에 있다(repo 밖).

### 5. census 판별은 손대지 않았다

`lockVerdict` 는 「정적으로 검사를 가졌다 **그리고** 스스로 끝났다」이다. 풀린 판은 더 이상 스스로 끝나지
않으므로 **실측으로** 그 바구니에서 빠진다 — 규칙을 손보면 운영자가 이름대지 않은 판까지 함께 빠진다.
주석 두 곳만 갱신했다(판마다 푸는 정책 · 「41 중 36 play」→ 41).
0418 §91행이 적은 「판별에 설치 토큰 종료 추가」는 **이미 #455(0422)가 닫았고**, 그 귀속 자체가 §1 처럼
틀렸다 — 그 행은 이 회차로 소멸한다.

### 6. 겹침

- 이 회차가 손댄 `compat.json` 은 **그 5행뿐**이다. wie-2(이노티아2)·wie-3(LGT 서버벽 잔여)의 열린 PR 이
  다른 행을 건드리면 union 으로 풀린다.
- `scripts/playability-census.mjs` 는 주석 블록 2곳만 바뀌었다.

### 7. 게임 파일명 유입

표식은 커밋 뒤의 도구 출력 그대로다. 이 회차가 더한 줄의 게임 이름은 0이다(sha12 로만 적었다).

도구 표기: BOUNDED 330건 · SUFFIX-ATTACHED 17건 — 전부 계약상 제목 목록인 `compat.json` 과
`playability-census.mjs` 에 **이미 있던** 줄이다.

<!-- corpus-name-inflow v1 subjects=10 tree=3716106b9239962c B=718/330 P=0/0 S=37/17 -->
