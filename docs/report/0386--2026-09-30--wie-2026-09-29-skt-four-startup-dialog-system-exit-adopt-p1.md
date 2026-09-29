## [2026-09-30] SKT 4종 시작 종료 — 구매자 전화번호에 묶인 인증키 · 흉내 낼 값 없음 (wie-2026-09-29-skt-four-startup-dialog-system-exit-adopt-p1)

**무엇을**: 시작하자마자 스스로 꺼지는 SKT 4종(`0ed66634d3dc` `a42f77f44955` `c33090c12755` `eefc947d8337`)의 종료 분기를 바이트코드로 읽었다. 코드는 바꾸지 않았다.
**왜**: 2차 전수 검수 제안 `2026-09-29-census-wave2-recensus#p1`(Tower 채택). 흉내 낼 수 있는 값이면 구현하고, 아니면 판정 근거를 남기는 조사 회차다.
**사용자 영향**: 없음. 4종은 그대로 «지원 안 됨»이다. 인증 검사를 통과시킬 방법은 원 구매자의 전화번호를 알아내거나 검사를 우회하는 것뿐이다. 둘 다 이 회차에서 할 일이 아니다(§3).

증적: 별도 파일은 없다. 아래 내용은 각 타이틀 jar 에 `javap -c -p -constants`(openjdk 26)를 돌려 다시 확인할 수 있다. sha12 만 적고 게임 제목·바이트는 0 이다.

### 1. 무엇이 끄나 — 4종 모두 XCE `SecureUtil`
네 타이틀의 `startApp` 첫 호출은 모두 같은 인증 검사다. 두 종은 클래스 이름이 난독화돼 있다.

| sha12 | `startApp` 첫 호출 | 검사하는 속성 |
|---|---|---|
| `0ed66634d3dc` | `b.m(MIDlet)`(난독화) | `MIDlet-Key` |
| `a42f77f44955` | `j.a(MIDlet)`(난독화) | `MIDlet-Key` ~ `MIDlet-Key4` 중 하나 |
| `c33090c12755` | `com/xce/security/SecureUtil.validate` | `MIDlet-Key` · `MIDlet-Key2` |
| `eefc947d8337` | `SecureUtil.validate`(기본 패키지) | `MIDlet-Key` · `MIDlet-Key2` |

`validate` 는 `isValid` 가 거짓이면 4줄을 그린다. 그다음 `XDisplay.refresh` → `Thread.sleep(2000)` → `System.exit(-1)` 순서다. 4줄은 상수 풀에서 읽었다.
- 키가 맞지 않을 때: 「인증 되지 않은 / 컨텐츠 입니다. / 프로그램을 종료 / 합니다.」
- 키 속성이 없을 때: 「인증키가 / 존재 하지 않습니다. / 프로그램을 종료 / 합니다.」

### 2. `isValid` 가 계산하는 값
```
key = hex(MD5( 통신사접두 + MIN + SERVICE_ID + "a0a535ef35b" ))  ==  MIDlet-Key
```
- **MIN**: `System.getProperty("com.xce.wipi.version")` 이 null 이면 `m.MIN` 을 그대로 쓴다. 아니면 `MIN` 을 읽는다. 첫 글자가 `0` 이면 앞 3자를, 아니면 앞 2자를 떼고, 7자리면 앞에 `0` 을 붙인다. 즉 **단말의 전화번호**다.
- **통신사 접두**: `m.CARRIER` 값에 따라 붙는다. SKT→`011` · STI→`017` · KTF→`016` · HSP→`018` · LGT→`019` · `010`→`010`.
- **SERVICE_ID**: `MIDlet-Jar-URL` 에서 `SERVICE_ID=` 뒤 5자를 건너뛰고 10자를 쓴다(다운로드 서비스 번호).
- 해시는 MD5 다. 초기값 `0x67452301 0xefcdab89 0x98badcfe 0x10325476` 과 첫 라운드 상수 `0xd76aa478` 을 확인했다. 출력은 소문자 16진수 32자다.

엔진 값(`wie-skt/src/emulator.rs` · `MIN`/`m.MIN` = `01000000000`, `com.xce.wipi.version` = `""`)이면 MIN 은 `00000000`, 입력은 `011`+`00000000`+SERVICE_ID+salt 가 된다. 4종 모두 결과가 msd 의 키와 **다르다**(파이썬 `hashlib.md5` 로 재현함).

### 3. 판정 — 구현하지 않는다
- 키는 **구매 단말의 전화번호**로 만들어진다. 우리가 되돌려 줄 수 있는 «그럴듯한 값»이 없다. 기기 모델이나 버전 문자열과 달리, 맞는 값은 타이틀마다 한 개(`a42f77f44955` 는 최대 4개)의 실제 전화번호뿐이다.
- 그 번호는 해시를 전수 대입해야 나온다(접두 6종 × 7~8자리 ≈ 6×10⁸). 결과는 **실존 인물의 개인정보**다. 이 저장소에 넣을 수 없다.
- 남는 길은 `isValid` 를 참으로 만드는 우회뿐이다(클래스 패치 · `String.equals` 가로채기 등). 그것은 콘텐츠 보호 우회이고, 정책 결정이다. 엔진 호환성 작업이 아니다. 이 회차는 판단하지 않는다.
- ⇒ 이 군집(4종)은 **결함이 아니라 설계대로 도는 DRM** 으로 분류한다. 다음 전수에서 같은 제안이 다시 나오지 않도록 여기 남긴다.

### 4. 곁가지 관측 — 대화상자는 화면에 나오지 않는다
`wie_validate`(`3c34efee` 전수 빌드)로 4종을 돌리면 `paints 0` · `distinct_colors 0` · `clean exit` 이다. 대화상자는 `com.xce.lcdui.Toolkit.graphics` 에 그리고 `XDisplay.refresh` 로 내보낸다. 그런데 엔진의 `XDisplay::refresh` 가 stub(`wie-skvm/src/classes/com/xce/lcdui/x_display.rs`)이라 이용자는 검은 화면만 보고 앱이 끝난다. 그래서 캡처가 없다. 문구는 §1 처럼 상수에서 읽었다. 이 stub 를 채워도 앱은 계속 종료되므로 후속 제안으로 올리지 않았다.

### 5. 게이트
PR 본문과 done 회신에 적는다.

게임 파일명 유입: BOUNDED 0 · SUFFIX-ATTACHED 0.

<!-- corpus-name-inflow v1 subjects=2 tree=6501bd0a66a89d30 B=0/0 P=0/0 S=0/0 -->
