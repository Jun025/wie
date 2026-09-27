## [2026-09-27] 전수 점검 소형 결손 묶음 — lwc·SKT 클래스·RMS 열거·LGT stdlib 4행·한 tick 미종료 (wie-census-small-stub-bundle)

**무엇을**: 전수 점검(`docs/report/0321`)이 «S급»으로 모은 첫 벽 7군을 고쳤다. 영향 타이틀은 sha 앞 12자로만 적는다.
**왜**: 7군 모두 엔진 쪽 결손이다(없는 메서드·클래스·import 행, 그리고 협력형 스레드에서 끝나지 않는 틱).
**사용자 영향**: 부팅이나 첫 키에서 죽던 **8종**이 화면을 띄우고 27키를 버틴다(아래 표 PASS 전환). 키를 오래 누르면 죽던 1종이 버틴다. 3종은 첫 벽을 넘었지만 다음 벽에서 멈춘다. 6종은 그대로다(식별 못 한 KTF ABI 2군 5종 · kfc 툴킷 1종).

### 무엇을 바꿨나

| 군 | 고친 곳 | 근거 |
|---|---|---|
| lwc 메서드 | `LabelComponent.<init>(String)` · `Component.getX/getY/layout/validate` | `getWidth/getHeight` 와 같은 답: 이 층은 배치를 하지 않는다(0, no-op) |
| KTF 클래스 | `org/kwis/msp/handset/LED.set(I)V`(정적 · no-op) | 빈 클래스를 넣고 잰 다음 벽이 `set(I)V` 하나였다 |
| SKT RMS | `RecordStore.enumerateRecords(RecordFilter,RecordComparator,Z)` + 세 인터페이스 + `net/wie/RecordEnumerationImpl` | 필터·비교자는 게스트로 되부른다. 비교자 없으면 id 오름차순. `keepUpdated`·`rebuild` 는 받고 무시(`ponytail:` 주석) |
| SKT com/xce | `io/ByteToCharConverter` · `io/ByteToCharEUC_KR` · `lcdui/TextComponentHandler.isLoaded()=false` · `XFile.fsavail` 0 → 1,000,000 | 바이트코드 실측: `convert` 는 끝 인덱스가 아니라 **길이**를 받는다(`end-start`, `out.length-outOff`). `TextComponentHandler` 의 유일한 호출부는 `isLoaded` 가 거짓이면 곧바로 `return` 한다. `fsavail()<0x2000` 검사가 네 곳 — 0 이면 «저장공간 부족»으로 첫 키에 종료했다 |
| LGT stdlib | `0x3f9 vsprintf` · `0x408 strncat` · `0x426 malloc` · `0x428 free` | 아래 «ABI 식별» |
| `f6fe2adc8cce` 한 tick 미종료 | `WieAudioClip.play` 가 끝에 `Thread.yield` · 아직 소리 나는 클립의 `play()` 는 다시 시작하지 않는다(`soundingUntil`) · LBMP type 2(2비트 회색) 해독 | 아래 «원인 판정» |

### ABI 식별 — 호출부 디스어셈블(추측 등재 0)
도구: 임시 계측(미커밋)으로 import 트랩 시점의 `r0-r3`·`lr` 앞 128바이트·호출 슬롯 표를 떠 `capstone`(Thumb)으로 읽었다.
- **0x426 malloc / 0x428 free** (`b7699c10dfd1`) — 0x426 을 부르는 함수는 C++ `operator new` 그대로다: `if (size==0) size=1; p=f(size); if (p) return p;` 아니면 예외 경로. 바로 앞 함수는 `operator delete`(`if (p) g(p)`)이고 그 `g` 의 지연 결합 슬롯 id 가 **0x428** 이다(슬롯 = `push {lr}; bl resolver; 1; id`).
- **0x3f9 vsprintf** (`2dbde9acca99`) — 가변 인자 함수(`push {r0-r3}` 진입)가 자기 `fmt, ...` 을 스택 버퍼에 포맷해 경로로 연다. `r2 = sp+0x120` = 저장된 `fmt` 바로 다음 워드 = 첫 가변 인자 주소(`va_list`). 이 이미지의 슬롯 표는 조밀하다: `0x3f8` 다음 슬롯이 호출된 슬롯이고 그다음이 `0x3fa`.
- **0x408 strncat** (`4fdbd64c9fbd`, 키 처리 중) — `memset(dst,0,0x42)` → 두 인자 문자열 호출 → `if (n) f(dst, local, n)`. 이 이미지의 슬롯 표는 희소·오름차순이고 이 슬롯은 그 두 인자 호출의 슬롯 바로 다음, `0x415 memmove` 앞이다. 표 안에서 문자열 함수는 «X, 개수형 X» 쌍으로 늘어선다(`0x405 strcpy`·`0x406 strncpy`, `0x407 strcat`). **모양 + 위치이지 심볼 이름은 아니다** — `svc_ids.rs` 주석에 그 한계를 적었다.
- **식별 못 한 것(등재 0)**: KTF `MC_dbSortRecords`(slot 8 · 2종)와 미식별 WIPI-C 표 selector 5 fn 0(3종). 이 회차는 새 관측을 얻지 못했다 — 선행 회차의 모양(`0176`: `f(이름, 크기, 플래그, 1)` → 양수 핸들 · `-12`)과 H2 미결(`0207`)이 그대로다. 표는 fail-closed 로 둔다.

### 원인 판정 — `f6fe2adc8cce` 한 tick 미종료
1. 이미지 해독 실패 `Unsupported grayscale type 2` → 그 `Image` 가 null 이라 `paint` 가 매 프레임 NPE. LBMP type 2 는 LCD 페이지 배치 1비트 평면 2장(평면 0 = 상위 비트, 3 = 검정) + `mask` 면 투명 평면 1장이다. 평면 순서는 사진 퍼즐 4장 모두에서 «평면 0 = 상위»의 전변동(TV)이 21~32% 낮아 골랐다.
2. 그것을 고쳐도 틱이 안 끝났다. 인터프리터 추적(`RUST_LOG=trace`)의 마지막 루프는 두 명령뿐이다 — `tk/Kingdoms$1` 의 `do { this$0.e.play(); } while (a);`. 협력형 스레드에서 `play()` 가 즉시 돌아와 이 스레드가 영원히 양보하지 않았다. `play()` 끝에 양보를 넣자 틱이 끝났지만 30초 프로브에 **Play 146,619회**가 나갔다(곡을 매번 처음부터). 그래서 아직 소리 나는 클립의 `play()` 는 무시한다(시퀀스 길이 = 마지막 이벤트 시각). 같은 프로브에서 Play **2회**다.
   대가: 한 클립을 길이보다 빨리 다시 치는 게임은 이제 소리가 끊기지 않고 끝까지 간다. SKT 소리 타이틀 6종의 짝 재측에서 Play 수는 전·후 같은 자릿수다(아래 표).

### 전/후 — 같은 시각 짝 재측
- 전: `origin/main`(`16803a4a`) + #350(`ff40c906`) 병합 스크래치. 후: 같은 스크래치 + 이 PR. 둘 다 release 빌드.
- 명령: `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(전수 점검 프로브 A와 같다). 타이틀마다 전·후를 4병렬로 나란히 돌렸다. 틱이 끝나지 않는 경우를 위해 벽시계 200초 상한.
- 호스트 부하 load1 **174~232**(표의 마지막 열). 벽시계 축은 덜 진행된 상태로 쟀다.

| sha12 | 전 | 후 |
|---|---|---|
| `ae749cc5a777` | FAIL boot · `ShellComponent.getX` | **PASS** |
| `ae212f5390ce` | FAIL boot · `LED` 없음 | **PASS** |
| `1e43e2e0055f` | FAIL boot · `LED` 없음 | **PASS** |
| `66959afab216` | FAIL boot · `enumerateRecords` | **PASS** |
| `fb80e97cbc57` | FAIL boot · `ByteToCharEUC_KR` | **PASS** |
| `f6fe2adc8cce` | 200초 안에 끝나지 않음 | **PASS**(Play 2) |
| `2dbde9acca99` | FAIL 27_OK · import 0x3f9 | **PASS** |
| `b7699c10dfd1` | FAIL boot · import 0x426 | **PASS** |
| `4fdbd64c9fbd` | PASS(27키 안에는 0x408 미도달) · 장시간 키 루프 120단계: **FAIL 44_NUM4 · import 0x408** | PASS · 같은 루프 120단계 완주 |
| `9a2cf5ffc9d3` | PASS(30초) · 장시간 루프 5회 중 1회 FAIL(7단계 · 자바 예외 1 · 사유 문자열은 못 남겼다) | PASS(30초) · 장시간 루프 5회 모두 예외 0 |
| `33f3e7669599` | FAIL · `LabelComponent(String)` | FAIL · 다음 벽 `ButtonComponent` 없음 → paint `java.lang.Error` |
| `ca7fa8ade8ad` | FAIL boot · `LabelComponent(String)` | FAIL boot · 다음 벽 `ContainerComponent.getComponent(I)` |
| `a10a1f02b41b` | FAIL · `layout()` | FAIL · 다음 벽 잘못된 메모리 접근(`validate` 뒤) · 한 판은 `org/kwis/msf/core/ProgramExitException` 없음 |
| `59263295de74` `e085e193211d` | FAIL · slot 8 | 같음(미식별) |
| `6af589a88cf9` `7da00ecd4804` `4166acd8fc62` | FAIL · selector 5 fn 0 | 같음(미식별) |
| `f07cbc782828` | FAIL · `com/ktf/kfc/GForm` | 같음 — 아래 |

`9a2cf5ffc9d3` 의 벽(전수 점검 600초 실행의 `06_OK`)은 이 부하에서 잘 재현되지 않았다 — 고침의 근거는 바이트코드(호출부가 `isLoaded` 하나로 막힌다)와 단위 시험이다.

**퇴행 0**: 라이브 LGT 5종(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) · 퇴행 가드 2종(`49ade89578c5` `ddd885583b15`) · SKT 소리 6종(`02bc66e85cf4` `090877d7a3e0` `1367261bc3ee` `1f0d7e81336a` `2d66945008c1` `2f84b8cc870d`) 모두 전·후 부팅·화면 ok. `4ece6eeeaa04` 는 두 빌드 모두 키 스크립트로 스스로 종료(clean exit, UNMEASURED)한다 — 짝 3회 추가 재측에서 전·후 같았다.

### 되돌리면 red — 18개 변이 전건
등록 행 교체(배열 길이 유지) 4 · 동작 행 14. 모두 변이 red · 원상 green. 등록 변이가 잡히도록 SKT 시험은 크레이트 `get_protos()` 를 쓴다. 목록은 회신 참조.

### 남긴 것
- `com/ktf/kfc` 는 스텁이 아니라 툴킷이다: 한 타이틀이 `GForm`·`GMenubarForm`·`GMsgBox`·`GTextField`·`GTextListener` 를 쓴다. `GForm(IIII)` 만 넣고 재면 벽이 `GTextField` 로 옮겨갈 뿐이라 넣지 않았다.
- lwc 가 위젯 툴킷으로 쓰이는 타이틀(자식 저장 `getComponent`, `ButtonComponent`)은 «배치하지 않는다»는 이 층의 전제를 넘는다 — 후속 제안.

### 게이트
- `cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`(1,386초) · `npm run build:wasm` · `check-engine-contract` · `npm run audit` 전부 rc=0.
- 러너 줄(엔진 변경): `draw_j2me` · `helloworld_ktf/lgt` · `keydraw_ktf/lgt --inject --expect-last-frame` · `text_j2me --timeout 5` 전부 PASS · rc=0.
- 게임 파일명 유입: 이 회차가 **더한 줄**에는 0건이다. 도구 표기 BOUNDED 20회/13쌍은 고친 파일에 **이미 있던** 주석이다(SUFFIX-ATTACHED 0).

<!-- corpus-name-inflow v1 subjects=28 tree=6e13fcfc9af3a1f6 B=20/13 P=0/0 S=0/0 -->
