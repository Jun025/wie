## [2026-10-03] 진도 벽 2차 — 검증기 화면이 `DisplaySize` 를 무시했다 · `TextComponent` 글자는 `m_td` 에 · 셸의 크기는 화면 크기 · LGT ABI 3행 (wie-progression-engine-walls-r2-null-image-dialog-npe-key)

**무엇을**: 0413 §6 이 남긴 벽 셋을 쟀다. ① `9789fec50f39` 는 «게임시작» 직후 null `Image` 로 게임 스레드가 죽는다. ② `73f3a21e981c` 는 «서버 접속 실패» 대화상자 OK 에서 `c_bf` NPE 가 난다. ③ `be08d047cbae` 는 선택 화면에서 키가 안 먹는다.
**판정**: ① **엔진 탓이 아니라 검증기 탓이었다.** `wie_validate` 의 화면이 `resize` 를 버려서, `DisplaySize:176*220` 패키지를 240×320 으로 돌렸다. 게임은 화면 크기를 읽고 240폭 패키지에만 있는 `52.m` 을 찾았다. 화면이 크기를 따르게 고쳤다. 그 뒤의 벽은 `TextComponent.m_td` 가 null 인 것과 `setString` 이 값을 버리는 것이었다. 둘 다 고쳤다. ② 0413 의 `c_bf.keyNotify` NPE 는 다시 나오지 않았다. 같은 클래스의 `c_bf.paint` NPE 는 전·후 모두 간헐적으로 나온다. 원인은 찾지 못했다(§3). ① 의 `getString` 수정 뒤 이 타이틀은 `StringBuffer.insert(II)` 에 닿았고, 그 행을 넣었다. ③ 셸의 `getWidth/getHeight` 가 0 이라 `repaint(0, 0, 0, 0)` 이 아무것도 다시 그리지 않았다. 그 뒤의 벽 `String.replace(CC)`·`Object.hashCode()` 행을 넣었다.
**사용자 영향**: ③ 은 선택 화면에서 키가 먹고 다음 화면으로 간다. ① 은 «게임시작» 뒤 본편 화면과 이름 입력까지 간다. 이름 칸은 숫자를 받지 않는다(§2). ② 는 달라진 것이 없다. 다만 600초 진도 축으로는 세 종 모두 아직 `stuck` 이라 `compat.json` 은 바꾸지 않았다(§9).

증적: `~/orchestrator/reports/evidence/wie-progression-engine-walls-r2-null-image-dialog-npe-key/`. 타이틀은 sha12 로만 적는다.

### 1. ① `9789fec50f39` — null `Image` 의 출처

임시 계측을 넣었다(커밋 안 함). `java_throw` 에서 게스트 스택과 해석된 참조를 찍었다.

- 죽는 곳은 `n` 의 메서드 `0x1200ac` 이다. `obj.bl`(`n` 의 `Image` 필드, 오프셋 `0x16c`)을 읽고 그 값의 `Image` 메서드를 부른다. 그 값이 null 이다.
- `bl` 에 쓰는 putfield 는 하나뿐이다(`0x135104`). 그 참조는 죽는 시점까지 해석되지 않았다. 즉 그 대입은 한 번도 돌지 않았다.
- «게임시작» 뒤 이미지 로드 순서에서 `52.m` 만 `FileNotFoundException` 을 냈고 게임이 받아 삼켰다. jar 에는 `50.m`·`51.m`·`53.m` 은 있고 **`52.m` 은 없다.**
- `__adf__` 는 `DisplaySize:176*220` 이다. 그런데 검증기 화면은 240×320 이었다. `KtfEmulator::from_archive` 는 `platform.screen().resize(176, 220)` 을 부르지만, `HeadlessScreen::resize` 가 `Ok(())` 만 돌려줬다.
- 화면이 크기를 따르게 하자 게임은 `52.m` 을 찾지 않았고, `bl` 대입이 돌았다(putfield 참조 해석됨). 셸·브라우저 호스트는 원래 크기를 따른다(Scenario G).
- **이 결함의 범위**: 말뭉치 KTF 중 `176*220` 을 선언한 것이 **93종**(working 65 · broken 28)이다. 모두 지금까지 240×320 으로 측정됐다. 퇴행 확인은 §5.

다음 벽 둘:

- **`TextComponent.m_td` 가 null**: 이름 입력 대화상자의 게스트 클래스 `k` 는 `TextFieldComponent` 를 상속한다. `k.keyNotify` 가 먼저 `m_td.length` 를 읽고(`0x17d150` = arraylength 도우미) 부모로 넘긴다. 첫 키에서 NPE 가 났다. 해석된 참조가 `TextComponent.m_td [C` 임을 런타임에 확인했다.
- **`setString` 이 값을 버림**: `m_td` 만 채우면 게임이 매 프레임 `setString("")` 을 부르고 `getString()` 으로 다시 읽는다. 스텁은 값을 버리고 늘 `"temp"` 를 돌려줬다. 그래서 둘이 끝내 같아지지 않았고, 43초 만에 게스트 힙이 바닥났다(`guest heap allocation … failed` · 문자열 할당 4만 회).
- **처방**(`text_component.rs`): `<init>` 에서 `m_td` 를 빈 배열로 둔다. `setString` 은 `m_td` 에 저장한다(null 이면 빈 배열). `getString` 은 `m_td` 를 문자열로 돌려준다. `"temp"` 대체값은 없앴다. 그 값은 입력 수단이 없던 시절 upstream 스텁의 자리표시였다(`49db5171`).

### 2. ① 의 다음 자리 — 이름 칸이 숫자를 받지 않는다

본편 지도 화면 → 비서 대사 → «시장님 존함» 입력 대화상자까지 간다. 숫자 키는 `m_td` 에 붙는다. 그런데 게임이 그 직후 `setString("")` 로 지운다. 숫자 5회 입력, 5회 모두 그 뒤에 `setString ""` 이 왔다(계측 로그). 이름은 한글 등 글자만 받는 것으로 보인다. 글자 입력기(멀티탭 한글)는 KTF lwc 에 없다. 크기 L 이다. 이번 회차 범위 밖이다.

### 3. ② `73f3a21e981c`

- 0413 이 본 `NullPointerException at c_bf.keyNotify` 는 이번 정책 키 경로에서 다시 나오지 않았다(300초 · 전 1판, 후 3판).
- 같은 클래스의 **`c_bf.paint` NPE** 는 판마다 0~2회 나온다. 전 1판에서 1회, 후 판들에서 0·2·0회였다(마지막 0회는 150초 계측 실행). 「`m_td` 가 원인」은 이 수로 뒷받침되지 않는다 — 초안에서 그렇게 적었다가 철회했다. 원인은 찾지 못했다. 간헐이라 재현 비용이 크다(크기 M · 다음 회차는 NPE 순간의 게스트 lr 를 찍는 계측부터).
- `getString` 이 실제 글자를 돌려주자 새 경로가 열렸다. `java/lang/StringBuffer vtable index 34` 에서 게임 스레드가 죽었다(lr `0x5cf68`).
- **호출부**(binary.mod 파일 `0x5bf74..0x5bf98`): `r6 = (r7 / 10) * 10` 다음에 `mov r1, #0` 을 한다(리터럴 0). `r2 = r7 - r6` 이고, `ldr ip, [r3, #0x8c]` 로 StringBuffer 에 디스패치한다. 즉 `sb.insert(0, n % 10)` 이다(오른쪽부터 숫자 쓰기). CLDC 순서 delete(II) 27 → deleteCharAt 28 → insert(I,Object/String/[C/Z/C/I) 29..34 와 맞는다. 핀이 `insert(II)` 를 구현한다.
- 0413 이 말한 «서버 접속 실패» 대화상자의 OK 경로는 이번 정책 키로 다시 밟지 못했다. 가짜 서버 응답은 만들지 않았다. 진도는 전·후 모두 300초 `ok`(새 화면 4 · 마지막 130초)다. compat 의 `stuck` 은 600초 값이다.

### 4. ③ `be08d047cbae`

- 계측 로그: 키마다 `ShellCard.keyNotify` → 게스트가 `Component.repaint(0, 0, 0, 0)` 을 부른다(60초에 504회). 그 뒤 `ShellCard.paint` 가 오지 않는다.
- 게스트는 셸을 만들 때 `getWidth()`·`getHeight()` 를 한 번 읽어 둔다. `Component` 의 스텁은 0 을 돌려준다. 0 × 0 영역 다시 그리기는 아무것도 덮지 않는다.
- **처방**(`shell_component.rs`): `ShellComponent` 가 `getWidth/getHeight` 를 재정의해 기본 Display 의 크기를 돌려준다. 셸은 자기가 보이는 화면이다. 필드는 더하지 않았다(LGT AOT 하위 클래스의 필드 오프셋이 밀린다). `(IIII)` 로 만든 셸도 화면 크기를 답한다(코드에 `ponytail:` 로 적었다).
- `wie-lgt` 시험 `virtual_method_appended_to_a_parent_reaches_subclasses_built_before_it` 는 `ShellComponent` 를 하위 클래스 예로 쓴다. 이제 그 칸에 재정의가 들어간다. 그래서 «부모의 메서드» 대신 «각 클래스 자신의 답»을 단언하게 고쳤다. 뒤늦게 연결된 칸이 하위 클래스에 닿는다는 단언 자체는 그대로다.
- 그 뒤 벽 둘:
  - `java/lang/String vtable index 30`(lr `0x8c168` · 파일 `0x8b168..0x8b198`): 직전 index-11(charAt) 디스패치 결과가 r1, `mov r5, #0x20` 이 r2, 그리고 `ldr ip, [r3, #0x7c]` 다. 즉 `s.replace(s.charAt(i), ' ')` 이다. CLDC substring(II) 28 · concat 29 · **replace(CC) 30** · toLowerCase · toUpperCase · trim 33 으로, 측정된 28·33 과 맞는다.
  - `java/lang/String vtable index 2`(lr `0x8c890` · 파일 `0x8b8a0..0x8b8c0`): r0 = 키 하나만 넣고 `ldr ip, [r3, #0xc]` 를 부른다. 결과는 바로 앞에서 색인한 버킷 루프에 쓰인다. 즉 `key.hashCode()` 다. Object 의 측정 행 getClass 1 · equals 3 사이 빈칸이다.
- 남은 벽: `java/util/Stack vtable index 25`(lr `0x1388e0` · r1 `0x32` · r2 `0x4d`). 보조 스레드가 18초마다 죽는다. 게임 화면은 계속 바뀐다. 이번에 역어셈하지 않았다(크기 S~M).

### 5. 전/후

바이너리: 전 = `origin/main aaf22218` release, 후 = 이 브랜치 release. census 진도 정책 **v2 인자 그대로**를 썼다(키 순환, `--stall-secs 60` 과 탈출 7종, `--restart-at 300`, `--relaunch 8`, 총 420초, 곡선은 300초 창). 판정은 census `fingerprint`/`progressCurve` 를 잘라 썼다. `build-slot` 경유 1잡씩 돌렸다.

| sha12 | 막힌 지점(전) | 다음 지점(후) | 진도 300초(전 → 후) |
|---|---|---|---|
| `9789fec50f39` KTF | 메뉴 «게임시작» → 게임 스레드 NPE(uncaught 2) | 본편 지도 · 비서 대사 · «시장님 존함» 입력(숫자 거부 §2) · uncaught 0 | stuck(새 화면 1) → stuck(2) |
| `73f3a21e981c` LGT | `c_bf.paint` NPE 1 | 예외 0 | ok(4 · 130초) → ok(4 · 130초) |
| `be08d047cbae` LGT | 선택 화면 · 키 무반응 | 선택 화면이 키마다 바뀐다 → 다음 화면 · `Stack 25` 벽(보조 스레드) | **stuck(1) → ok(4 · 250초)** |

- **`be08d047cbae` 600초**(축 정의와 같은 창 · 후): PASS · 새 화면 4 · 마지막 새 화면 80초 ⇒ stall 520초 = **stuck**. 300초 표의 `ok`(마지막 250초)는 창이 짧아서 나온 값이다. 0413 의 `d448aee68157` 과 같은 함정이다.
- 전 바이너리는 `aaf22218` 이다. 측정 뒤 `origin/main` 이 7회 착지해 `a3b5988e` 가 되었고 그 위로 rebase 했다(충돌 0). 그 7회는 `lgt_java_abi.toml` +8 · `wie_validate.rs` +19 를 바꿨고, 이번 표를 다시 재지는 않았다.

### 6. 퇴행

census A 프로브(30초 · `--relaunch 1 --pacing 8`)를 전·후 같은 순서로 교대해 돌렸다. **100종** 전부 짝이 있다.

| 묶음 | 수 | FAIL 전 → 후 | content 전 → 후 | 뒤집힌 행 |
|---|---|---|---|---|
| 라이브 LGT 5 + 가드 `49ade89578c5` `ddd885583b15` | 7 | 0 → 0 | 7 → 7 | 0 |
| KTF `176*220` 선언 전부(이번에 크기가 바뀐 묶음) | 93 | 3 → 3(같은 세 종) | 91 → 91 | 0 |

- ★한 번 퇴행을 잡고 고쳤다. 셸 크기 수정의 첫 판은 `AnnunciatorComponent`(`ShellComponent` 하위)에도 화면 크기를 답했다. 그러자 `b475b6399684`(라이브 LGT)가 30초에 그림 239 → **2**(2판 모두)로 떨어졌다. 이 타이틀은 부팅 때 상태줄의 높이를 두 번 읽는다. 상태줄은 그리지 않으므로 0 을 답하게 했다. 고친 뒤 209·226 이고, 위 표가 그 최종 바이너리의 값이다. 시험 단언도 넣었다.
- 판정이 `UNMEASURED` 인 행(전·후 54쌍)은 census A 인자에 `--max-ticks` 가 없어 `max-ticks` 에 닿은 것이다. 전·후 같은 조건이다(0413 §5 와 같다).

- 게이트(최종 코드 · rebase 전): `cargo fmt --check` 와 `cargo clippy --all -D warnings`·wasm32 clippy 가 rc=0 이다. `RUST_MIN_STACK=4194304 cargo test --all` 은 660 통과 0 실패다. rebase 뒤(`a3b5988e` 위): 같은 게이트 rc=0 · `cargo test --all` 669 통과 0 실패 · `build:wasm` rc=0 · engine contract 113/0.
- 러너 블록(rebase 뒤 release): draw · helloworld ×2 · text 모두 PASS다. keydraw ×2 는 `--max-ticks 1e9` 에서 PASS 27/27 이다(0413 과 같은 조건).

### 7. 되돌리면 red(실측)

| 수정 | 시험 | 되돌림 결과 |
|---|---|---|
| `HeadlessScreen::resize` 저장 | `wie_validate` `headless_screen_follows_resize` | FAILED |
| `<init>` 의 `m_td` 빈 배열 | `wie-wipi-java` `text_lives_in_m_td_from_construction_and_set_string_keeps_it` | FAILED(`m_td is null`) |
| `setString` 저장 | 같은 시험 | FAILED(`getString` ≠ `"ab"`) |
| 셸 `getWidth/getHeight` | `shown_shell_component_is_painted_through_the_display` | FAILED(`shell getWidth = 0`) |
| `insert(II)` 34 · `replace(CC)` 30 · `hashCode` 2 | `abi_rows_cover_the_indexes_titles_actually_dispatch_on` | 행마다 FAILED(3/3) |

### 8. 측정 조건

- `host-load-guard --status --recovered` 는 회차 내내 거의 rc≠0 이었다(idle 0~8% · load 15~250 · 다른 레인 census·빌드). 조항 ⒞ 의 «폭을 줄인다(≤2)» 쪽을 택해 **1잡**으로 돌렸다. 실행마다 가드 상태와 loadavg 를 증적 로그에 남겼다. 전·후는 같은 몇 분 안에 교대로 돌렸다.
- census 락은 다른 레인의 실행(5시간 넘게 보유)이 쥐고 있었다. 그래서 `census.mjs run` 은 쓰지 못했다. 같은 인자로 `wie_validate` 를 직접 돌렸다. 처음 띄운 census 는 락을 기다리며 build-slot 한 칸을 차지하고 있어서 내가 멈췄다.
- ★한 번은 내 측정 체인 둘이 겹쳤다(멈춘 줄 알았던 체인이 살아 있었다). 겹친 동안의 결과는 버리고 다시 쟀다. 위 표는 모두 단독 실행이다.
- `nohup … &` 는 쓰지 않았다. 회차 끝에 자기 프로세스는 0 이다.

### 9. compat.json

바꾸지 않았다. `progress` 축은 600초 정의다. `9789fec50f39` 는 이름 칸에서 멈추므로 `stuck` 이다. `be08d047cbae` 는 600초에서 `stuck` 이다(§5). `73f3a21e981c` 는 600초를 다시 재지 않았다. 여섯 축(`status`)은 이번 회차가 census 로 재지 않았다(락 · §8). 게임별 `changes` 는 `docs/player-updates/2026-10-03-{small-screen-game-start,choice-screen-keys}.json` 에서 빌드 때 파생된다.

`HeadlessScreen` 수정은 KTF `176*220` 93종의 **다음 census 값**을 바꿀 수 있다. 이번 A 프로브에서는 판정이 바뀐 행이 0 이다. 진도·긴 실행 축은 재지 않았다.

### 10. 게임 이름 유입

INFLOW_SECTION
