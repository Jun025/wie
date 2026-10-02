## [2026-10-02] 4차 군집 — 잠긴 파일 6종 안내 · KTF GMenubarForm · SKVM XDisplay.clear · 소리 판정용 타이틀별 키 (wie-census-wave4-remaining-walls-and-locked-titles)

**무엇을**
- 전수 도구(`scripts/playability-census.mjs`)가 **잠긴 파일**을 판별해 지원 현황 문장을 바꾼다: OMA DRM(`odcf`) = «암호로 잠긴 파일» · SKT 구매 단말 인증(XCE `SecureUtil`)이 시작 때 실패해 스스로 꺼진 것 = «구매한 휴대폰에서만 켜지도록 잠긴 파일». `compat.json` 6행.
- 전수 도구의 장시간 실행(`L` — 소리 축을 정하는 실행)이 `--titles` 의 타이틀별 키(레시피)를 앞에 붙인다. 키 파일은 `game_lab/` 에 두고 커밋하지 않는다.
- `wie-wipi-java`: `com/ktf/kfc/GMenubarForm`(부모 lwc `FormComponent`). `wie-skvm`: `com/xce/lcdui/XDisplay.clear(Graphics, Image, II)V`(로그만 남기는 무동작 — `copyLCD` 와 같은 급).

**왜**: 3차 군집 회신의 후속·다음 벽 절과 총괄 정책 판정(2026-10-02 · 인증 우회·번호 역산·DRM 복호화 금지).

**사용자 영향**: 지원 현황 playable 371 → **379** · limited 38 → **31** · not-yet 20 → **19**. 이 PR 코드 몫은 `bfa8ec352451`(not-yet → playable) 1종이고, 7종(limited → playable)은 main 에서 이미 풀려 있던 것을 다시 재서 반영했다(§3). 잠긴 파일 6종은 상태는 그대로이고 «아직 안 됨» 대신 실행할 수 없는 이유를 말한다. 소리 축 2종 no → ok(측정 정정).

### 1. 잠긴 파일 — 규칙과 전수 재스캔
| 종류 | 규칙 | 왜 이 모양인가 |
|---|---|---|
| `drm` | jar 자리에 OMA DRM 컨테이너(`odcf` 매직) | 정적으로 결정된다. 암호화 콘텐츠다 |
| `phone` | jar 안 한 클래스가 `MIDlet-Key`·`SERVICE_ID=`·`MIDlet-Jar-URL` 셋을 다 갖고 **그리고** A·B 두 실행이 한 장도 못 그린 채 하나라도 `clean exit` | ★클래스만으로는 틀린다 — 아래 |

전수 재스캔(코퍼스 429종 · 3c34efee 전수의 A·B):
- 검사 클래스 보유 **41종** — 그중 **36종은 playable**(검사가 시작 때 돌지 않거나 통과 경로가 아니다). 그래서 «클래스 있음»만으로 잠김이라 하면 36종을 틀리게 막는다.
- 규칙 결과: `phone` **4** · `drm` **2** = 총괄 판정 6종과 **정확히 같다. 자동 판별로 찾은 추가분 0.**
- 클래스는 있는데 잠김이 아닌 나머지 1종(`71d1d8235bd1`)은 다른 벽이다(`NoClassDefFoundError`, 그린 적 없음 · `stop=error`).
- 잠긴 행은 다른 안내 줄(«화면이 아직 나오지 않아요» 등)을 지우고 이 한 줄만 둔다 — 그 줄들은 «고칠 예정»으로 읽힌다.

### 2. 고친 것 — 전/후(6축 = boot/render/input/longplay/sound/speed)
| sha12 | 무엇 | 전(origin/main) | 후 |
|---|---|---|---|
| `bfa8ec352451` | KTF `com/ktf/kfc/GMenubarForm` 없음 | not-yet · 부팅 panic · paints 0 | **playable** · ok/ok/ok/ok/ok/ok · 600초 854키 무오류 · 재생 488 |
| `6e93f26fa2f5` | SKT `XDisplay.clear` 없음 — 메뉴에서 게임을 시작하면 `NoSuchMethodError` | 게임 시작 키에서 FAIL | 120초 무오류 · paints 2,210 · 게임 화면(달리기·점수) |

- `bfa8ec352451` 후 측정은 전수 도구 그대로(A·B 30초 + L 600초 · `--jobs 1`)다. 다른 레인의 전수가 호스트 락을 잡고 있어 이 1종만 락 경로를 따로 두었다(0388 과 같은 처리).
- ★`GMenubarForm` 의 부모(`FormComponent`)는 **추론**이다. 이 repo 에 API 문서가 없다. 게임이 부르는 메서드는 `<init>()V` 하나뿐이었고 그 뒤 오류 0 이다.
- ★`XDisplay.clear` 는 무엇을 지우는지 모른다. 무동작으로 두었고 화면 이상은 보이지 않았다(샷 확인). `6e93f26fa2f5` 는 compat 상 이미 playable 이었다 — 공용 키 루프가 게임 안으로 못 들어가서 이 벽을 못 봤다. 레시피(§4)가 들어가자 드러났다. 게임 안에서 재생은 여전히 0 이다.

### 3. 조작 «화면 1~3장 멈춤» 9종 — 흔들림 먼저
전수 도구 `--only probe` 를 **독립 2회**(main 빌드 · 같은 9종) 돌렸다.
| 결과 | 수 | sha12 |
|---|---|---|
| 두 번 다 `input ok` — 0388 이후 main 에서 이미 풀렸다 | 7 | `0eb19d9bbe7a` `2cbd63e4427a` `4a7e489bf6ab` `55aadf368b8e` `86132b2ed76f` `9e16cc54d0ab` `d9afc4db742c` |
| 두 번 다 1~2장에서 멈춤 | 2 | `38277d63b0ba`(SKT · paints 2) `6e9991f08650`(SKT · paints 1) |
- 7종의 장시간(600초 · 같은 main 빌드): 7/7 실패 줄 0(6종 854/900 키 · `86132b2ed76f` 259키에서 게임 쪽 종료 — 전수 정의상 ok) ⇒ **limited → playable 7**. `compat.json` 은 이 7행 전체를 옮겼다.
- ★이 PR 의 코드가 바꾼 것이 아니다(두 실행 모두 main 빌드). 0388(핀 `bd2337ff`) 이후 착지한 형제 회차의 몫이다 — 그래서 이용자 소식은 내지 않았다.
- 남은 2종(SKT · 두 번 다 1~2장)은 키 이전 렌더 벽이다(0388 분류 그대로) · 미조사.

### 4. 소리 판정 불가 15종 — 타이틀별 키
공용 키 루프가 `UP UP OK` 로 «게임 시작»이 아닌 메뉴 칸에 들어가거나, 숫자·`CLR`·`1.예/2.아니오` 를 요구하는 화면에서 멈췄다. 타이틀별 키를 `game_lab/recipes-sound/<sha12>.keys` 에 두고(비커밋) 전수 도구로 다시 쟀다(`run --only long --titles` — `L.json` 에 `recipe` 가 남는다).
| 분류 | 수 | sha12 |
|---|---|---|
| **소리 남(측정 정정)** | 2 | `01e2715ba07a`(재생 41 · MIDI 6,697) `0865be217bde`(재생 3 · MIDI 5,062) |
| 게임 안에 들어감 · 재생 0 | 5 | `34ab350dc98a` `51011b242bb5` `6c9f969f089f` `7089dec0e8df` `6e93f26fa2f5`(§2 수정 후) |
| 네트워크로 데이터를 받아야 진행(«다운로드중 1/11» · «CONNECTING» · «추가 다운로드») | 3 | `1793f87924d4` `74cb49013d64` `f44271803135` |
| 로고에서 멈춤(어떤 키로도 안 넘어감) | 2 | `73f3a21e981c` `be08d047cbae` |
| 메뉴 다음 빈/검은 화면 | 2 | `c361632541a7` `ccb45e6b8d80` |
| 숫자 입력 칸에 첫 글자만 들어감(생년 입력) | 1 | `1045007289d8` |
- `compat.json` 은 소리 2행만(no → ok · 안내 줄 제거). 측정 정정이라 이용자 소식은 내지 않았다(0388 과 같다).
- «게임 안 재생 0» 5종을 «원래 조용함»으로 판정하지 않았다. 근거 없이 그렇게 쓰지 않는다.

### 5. 못 고친 것
| 항목 | 상태 | 크기 |
|---|---|---|
| `a16f08d025eb`(LGT) 한 색 화면 | 원인까지 좁혔다(진단 빌드 · 커밋 0): 게임 스레드 `GameCanvas.run` 이 `0x64` 로 **`PlayGuide as IEventHandler`**(게임이 선언한 인터페이스) 표를 받고, 그 표의 칸이 0 이라 게임 자신의 «메서드 없음» 경로인 import `0x40` 을 부른다 → `Unknown lgt java import: 0x40` 로 죽고 주 스레드는 `max-ticks` 까지 돈다. 표는 21칸 = `Object` 10칸 + **이름 없는 11칸**. ★인터페이스 서술자의 메서드 목록이 **비어 있다**(0개) ⇒ 지금 읽는 자료로는 그 11칸이 수신자의 어느 메서드인지 알 수 없다. 칸 ↔ 메서드 대응이 LGT 클래스 배치 어디에 있는지 찾는 것이 다음 일이다. `0x40` 처리기만 넣는 것은 증상 처리다(칸이 여전히 비어 있다). 같은 표에 처리기 없는 `0x38` 도 있다 | M |
| KTF AOT 소리 3종(`1b3b4868d46e` `7218e8720f8c` `8bfd08fe4370`) | 60초 실행 로그: `Clip.<init>`·`setBuffer`·`setListener`·`Player.stop` 만. `Volume` 조회 0 회 ⇒ `Volume.getDefaultVolume` 이 0 을 돌려주는 stub 이지만 이 3종의 조건은 아니다. 조건은 AOT 코드 안 — ARM 추적이 필요하다 | M |
| `1eaa92092bee` `fe76e641bb3d` 힙 헤더 | 형제 회차가 착지(#434 · 0402) — 이 회차 대상 아님 | — |
| `59263295de74` · LGT 마젠타 · `ChoiceText` | 형제 회차가 착지(0398 · 0399 · 0397) | — |

### 6. 되돌리면 red
- `g_menubar_form_is_an_lwc_form` — `get_protos()` 에서 빼면 red.
- `clear_is_resolvable_with_the_descriptor_a_title_calls` — 메서드 행을 빼면 red.
- 전수 `selftest` 44/44: ⑴`phone` 에서 실행 조건 제거 → 41/43 ⑵앞에 붙은 바이트 보정 제거 → 42/43 ⑶`odcf` 판정 제거 → 42/43 ⑷세 속성 중 하나만 → 42/43 ⑸장시간 키에서 레시피 제거 → 43/44.

### 7. 지원 현황 갱신 방법
전수 도구 `report` → `player-data.mjs` `fromCensus`(공개 어휘) → 이 회차가 잰 행만 옮겼다(0391 §5 와 같은 방식 · 원인이 main 인 7행은 §3 에 따로 밝혔다). `bfa8ec352451`·조작 7종은 행 전체, 소리 2종은 소리 축만, 잠긴 6종은 `knownIssues_ko` 만. 최상위 `enginePin` 은 그대로 둔다(나머지 행의 핀).

### 8. 가드
라이브·가드 7종(`49ade89578c5` `ddd885583b15` · LGT `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) 전·후 빌드 30초 짝: 결과·재생 수 같음(`a30bbe008b5e` 만 전 `clean exit` · 후 PASS — 첫 실행 안내로 끝나는 기존 흔들림이고 좋은 쪽이다).
