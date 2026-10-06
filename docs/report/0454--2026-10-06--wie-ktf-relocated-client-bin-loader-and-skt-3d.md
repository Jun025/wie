## [2026-10-06] 큰 벽 2 — SKT 3D(`m/XO_World`) 렌더러 · KTF 재배치 client.bin 형식 판정 (wie-ktf-relocated-client-bin-loader-and-skt-3d)

**무엇을**: ⑴ SKT `71d1d8235bd1` 이 쓰는 3D API `m/XO_World`·`m/A3`·`m/V3` 를 소프트웨어 래스터로 구현했다(MBAC 모델 · MTRA 동작 · 8비트 BMP 텍스처).
⑵ 같은 타이틀의 리소스 적재가 `XFile.buf` 를 직접 읽다가 NPE 로 죽던 것을 고쳤다.
⑶ KTF 재배치 client.bin 3종(`1d5831e42a8a` `83fc429f9cbe` `b907b0faf483`)의 이미지 형식을 실측으로 더 밝혔다. 적재기는 넣지 않았다(§3).

**왜**: 운영자 지시(2026-09-30 · 10-06) — 미지원 게임의 큰 벽 둘. 출처는 wave7 회신 §000-4 후속 표.

**사용자 영향**: `71d1d8235bd1` 이 이제 켜진다. 타이틀·메뉴·옵션·이야기 화면을 지나 1스테이지 화면까지 간다. 그런데 1스테이지가 시작되기 직전에 멈춘다(§2-3). KTF 3종은 바뀌지 않았다.

### 1. SKT 3D — 무엇이 필요했나 (실측)

정적 호출 지점 수(`javap -c` 전수 · 이 타이틀의 클래스 6개):

| 클래스·멤버 | 호출 지점 | 뜻(근거) |
|---|---|---|
| `V3.x/y/z` 필드 | 88 · 82 · 115 | 정수 벡터. 게임이 직접 읽고 쓴다 |
| `A3.trans(V3,V3)` | 22 | `dst = this·src`. `scaleVector.z` 를 깊이로 쓴다(400 미만 절삭) |
| `XO_World.getMaxFrame(I)` | 17 | 프레임 수 16.16 고정소수점. 게임이 `frame += 경과ms × 1966`(= 30fps · 1966/65536)로 넘긴다 |
| `A3.mul(A3,A3)` | 12 | `this = a·b` |
| `XO_World.rotY(I,A3)` | 11 | 회전부만 덮는다. `ident` 없이 `rotY(-170)` 직후 `rotY(170)` 을 다른 행렬에 쓰므로 «곱하기»가 아니다 |
| `XO_World.getViewTrans(pos,look,up,A3)` | 10 | micro3D `lookAt`. `look` 은 방향이다(호출 직전 `m0 = d1 − d0`) |
| `XO_World.setPosture(action,frame)` | 9 | |
| `XO_World.setView(A3,sx,sy,cx,cy)` | 8 | 평행 투영. 게임이 원근 배율을 직접 계산해 넣는다(`10000·cy/(cy + z·1696/4096)`) |
| `A3.ident` · `A3.set` · `<init>` | 7 · 3 · 10 | |
| `XO_World.draw(Graphics)` | 6 | |
| `setVram` · `loadMBAC/BMP/MTRA` · `dispose` · `shareData` · `setClip` · `sin/cos` · `rotZ` | 5 · 4×3 · 4 · 1 · 1 · 3 · 1 | 각도 4096 = 한 바퀴 |

자료(이 타이틀): MBAC v3 1개(정점 573 · 삼각형 192 · 사각형 9 · 뼈 26) · MTRA v4 4개(동작 6 · 뼈 26 · 프레임 49~120) · 8비트 BMP 5개(128×128).
코퍼스 전체에서 `m/XO_World`·`com/mascotcapsule` 를 쓰는 타이틀은 **이 1종**이다.

형식은 MascotCapsule micro3D v3 이다. JL-Mod 의 역공학 로더(`ru.woesss.j2me.micro3d` · Apache-2.0)를 근거로 Rust 로 다시 썼다(`wie-skvm/src/mascot.rs`).
범위는 MBAC v2/v3(정점 형식 1 · 다각형 형식 1 · 법선 없음)과 MTRA v2–v5 이다. 더 높은 MBAC 판은 `-1` 을 돌려주고 경고를 남긴다.
그리는 방식은 이렇다. 뼈 행렬을 부모·동작 순서로 곱해 정점을 옮긴다. 그다음 평행 투영하고 z-버퍼로 삼각형을 채운다. 텍스처 색 0번은 투명 다각형에서만 빠진다.
- 면 버리기 방향은 실측으로 골랐다. 실제 아바타를 같은 시점에서 «양면 전부»로 그린 것과 비교하면, 채택한 방향은 6,117픽셀 중 6,086(99.5%)이 일치하고 반대 방향은 2,411(39%)이다.
- 이 래스터의 상한: 원근 보정이 없고(평행 투영이라 불필요) 조명·반투명 혼합이 없다. 이 타이틀의 MBAC 에는 법선이 없고, 다각형 재질 비트에도 혼합이 없다(재질 마스크 `0xFFF9` · 실측).

### 2. `71d1d8235bd1` 전/후

| | boot | render | input | 도달 화면 |
|---|---|---|---|---|
| 전 (`544a0643`) | fail | none | — | 게임 캔버스 생성자에서 `m/V3` 클래스 없음 |
| 후 | ok | ok | ok | 1스테이지 화면 · 시작 직전에 멈춤 |

**2-1. `XFile.buf`**: 이 타이틀은 자기 `com.xce.io.XResource extends XFile` 을 들고 온다. 이 클래스는 `type == FILE_JAR(3)` 이면 보호 필드 `buf`·`offset` 을 직접 읽는다. 종전 `XFile` 은 `buf` 를 비워 두어 첫 `read` 가 `NullPointerException: Array is null` 이었다.
`READ_RESOURCE` 로 열 때 항목 전체를 `buf` 에 담고, `is` 도 같은 버퍼의 스트림으로 바꿨다. `XFile.buf` 를 읽는 타이틀은 SKT 87종 중 이 1종이다(정적 검사).

**2-2. 3D 확인**: 2-3 의 벽을 임시 패치로 넘긴 실행(커밋하지 않음)에서 다음을 확인했다. `XO_World.draw` 180회 · `setPosture` 168회. 버스 안에서 텍스처 입은 아바타가 동작 프레임에 따라 춤춘다. 스크린샷은 repo 밖 증적에 있다.

**2-3. 남은 벽 — SKT `AudioClip.play` 의 차단 의미 (3D 와 무관)**: 1스테이지 시작 시 게임의 소리 스레드 클래스 `play(7)` 이 영원히 기다린다.
- 게임의 소리 스레드는 `synchronized` 안에서 `while (!isPlaying) { if (isRepeat) sleep(100); else wait(); }` 를 돈다. 즉 반복 모드에서는 모니터를 쥔 채 잔다.
- 그 상태에서 `stop()` 이 `isPlaying` 만 내리고 `isRepeat` 는 그대로 두면, 다음 `play()` 는 모니터를 영원히 얻지 못한다.
- 실기에서 이 교착을 벗어나는 길은 하나다. 소리 스레드의 `clip.play()` 가 재생 동안 **블록**하고, 다른 스레드의 `stop()→close()` 가 그 `play()` 를 **예외로** 끝내는 것이다. 그러면 `catch` 가 `isRepeat = false` 로 만든다.
- 우리 `WieAudioClip.play` 는 즉시 돌아온다(`wie_audio_clip.rs` 머리 주석의 실측 선택). 그래서 이 스레드는 100ms 마다 열기·재생·닫기를 되풀이하다가 위 상태에 갇힌다.
- 크기: 소리 전용 스레드의 `run()` 에서 `AudioClip.play` 를 부르는 SKT 타이틀은 **84종 중 74종**이다(정적 · `javap`). 블록 의미로 바꾸면 이 74종의 소리·스레드 타이밍이 모두 바뀐다. 그래서 이 회차에서 바꾸지 않았다(§후속 1).
- 소리 옵션을 «끄기»로 해도 같은 곳에서 멈춘다(실측 — 옵션은 음량만 바꾸고 `play` 는 그대로 부른다).

### 3. KTF 재배치 client.bin — 형식 (0340 에 더한 실측)

0340 의 결론(진입 = `+0x24` · GOT 두 번째 재배치 · 인터페이스 이름 `MNInterface`)은 그대로다. 아래는 이번에 새로 잰 것이다. 3종 모두 같은 모양이다.

| 위치 | `1d5831e42a8a` | `b907b0faf483` | `83fc429f9cbe` | 뜻 |
|---|---|---|---|---|
| `client.bin<N>` · 재배치 수 | 64 · 3,271 | 40 · 6,940 | 64 · 3,028 | `[u32 bss][u32 count][count×u32 오프셋][이미지]` |
| `hdr[0]` → | `{0x5e73c, 22, 0x20, fn 0x651, fn 0x651, self, …}` | `{0x4a3b0, 58, 0x80, …}` | `{0x5d30c, 12, 0x20, …}` | **클래스 등록부**: 첫 칸 = 클래스 서술자 배열의 끝, 둘째 = 개수 |
| 서술자 배열 | `0x5e424..0x5e73c` (22) | `0x49b88..0x4a3b0` (58) | `0x5d15c..0x5d30c` (12) | 칸 크기 0x24 |
| 헤더 재배치 칸 | `+0x00 08 0c 10 14 18 1c 24` | 같음 | 같음 | `+0x20/+0x28 = 0x13580001` · `+0x2c = 0x6ada465b` 는 상수 |

- **클래스 서술자(0x24)**: `[이름 ptr][0][부모][메서드 ptr][인터페이스 ptr][필드 ptr][u16 메서드 수][u16 필드 크기][u16 접근 플래그][u16 인터페이스 수][u16][u16]`.
  표준 KTF `JavaClassDescriptor` 와 칸 순서가 같다. 다른 점은 하나다. **부모가 포인터가 아니라 `+0x10` 이름표의 색인**이다(메인 클래스의 부모 `0x75` · 다른 클래스 대부분 `0x4b` = `java/lang/Object` 자리).
- **필드 정의**는 표준 `JavaFieldDefinition [access][class][name][value]` 와 같다(`KEYCODE_LEFT` = `0x19 · class · name · 2`).
- **이름 문자열**도 표준 `JavaFullName` 부호화와 같다(`"\xcc()Z+cancel"` = 해시 바이트 + 서술자 + `+` + 이름).
- `+0x10` 표는 클래스 이름과 메서드 이름을 섞어 담은 상수표다(`1d5831e42a8a` 첫 40칸 실측). 클래스 객체 표가 아니다.
- **`init(param)` 은 `get_interface("MNInterface")` 만 부르고 0 을 돌려준다.** 64칸짜리 기록용 표를 넘겨 실행했을 때 init 동안 불린 칸은 **0개**였다. `MNInterface` 의 칸은 init 이 아니라 그 뒤 클래스 실행 중에 쓰인다.
- 헤더 뒤 `0x30..0x480` 은 런타임 도우미 코드다. `mov r3, fp; ldr r3, [r3, #0x34]; mov sp, r3` 꼴이 되풀이된다. 즉 **`fp`(r11) = 스레드 문맥, `+0x34` = 스택**이라는 호출 규약이 있다.

**판정 — 이 회차에 적재기를 넣지 않는다.** 데이터 구조는 표준 KTF AOT 와 거의 같다. 이것은 0340 이 몰랐던 사실이고, 기존 `KtfJvmSupport` 를 재사용할 수 있다는 근거다. 그러나 부팅까지 남은 것은 셋이다.
⑴ 서술자 배열에서 클래스를 만드는 두 번째 적재 경로(색인 → 클래스 해석 포함).
⑵ `MNInterface` 칸들 — 0340 이 정적으로 센 10개 이상. 그중 뜻을 아는 것은 `+0x20` throw · `+0x40` 클래스 적재 둘뿐이다.
⑶ `fp` 문맥 배치.
⑵·⑶ 은 참고 자료가 여전히 0이라 칸마다 실행 추적으로 풀어야 한다. 크기는 L 이고 이 회차 시간에 들어가지 않았다. 앞부분만 넣으면 오류 문구만 바뀐다(0340 과 같은 이유). 그래서 코드는 바꾸지 않았다. 측정에 쓴 임시 패치(재배치 적용 + 기록용 `MNInterface`)는 되돌렸다.

### 4. 측정

`playability-census.mjs run --only probe --jobs 2`(long 풀 · 호스트 잠금)으로 같은 57종을 전(`544a0643`) → 후(이 브랜치) 순서로 쟀다.
대상 = `XFile` 을 참조하는 SKT 49종(이번 `XFile` 변경의 범위 · 정적 검사) + `71d1d8235bd1` + KTF 재배치 3종 + 라이브 LGT 5종 + 가드 2종. load1 은 전 8~32 · 후 20~90 이다.

| 묶음 | 종 | 축(status·boot·render·input)이 바뀐 것 | A/B 결과·stop·content 가 바뀐 것 |
|---|---|---|---|
| `71d1d8235bd1` | 1 | not-yet · fail · none · n/a → limited · ok · ok · ok | A·B 둘 다 FAIL(error) → PASS(deadline) |
| KTF 재배치 `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | 3 | 0 — 셋 다 boot fail(`Unsupported KTF client.bin layout`) 그대로 | 0 |
| 가드 `49ade89578c5` `ddd885583b15` | 2 | 0 | 0 |
| 라이브 LGT `13d7e3c21856` `b475b6399684` `4ece6eeeaa04` `1b107b96bf4e` `a30bbe008b5e` | 5 | 0 | 0 |
| `XFile` 참조 SKT(위 1종 제외) | 48 | 0 | 0 |

- 다른 칸은 하나다. `2d66945008c1` A 의 Java 예외 수가 5 → 4 로 줄었다. 첫 예외는 둘 다 같은 `RecordStoreNotFoundException` 이고 paints 는 146 ↔ 156 이다. `XFile` 경로와 무관한 시점 차로 본다.
- `71d1d8235bd1` 의 longplay·speed·progress 는 **재지 못했다.** 전체 실행을 걸었으나 census 호스트 잠금을 다른 레인 실행 둘이 번갈아 쥐어 약 1.5시간 동안 시작하지 못했고, 중단했다. 그래서 compat 행의 longplay·speed 는 `unknown` 이다.
  멈춤의 근거는 직접 실행 4회다(키 스크립트 · 소리 켬 3회 · 끔 1회). 4회 모두 1스테이지 화면 뒤 프레임이 바뀌지 않았고, 원인은 §2-3 이다.
- compat.json 은 이 1행만 바꿨다: status `not-yet → limited` · boot/render/input/sound `ok` · knownIssue 「첫 스테이지가 시작되기 직전에 멈춰요.」. sound `ok` 는 후 프로브의 audio plays > 0 이다.

### 5. 되돌리면 red — 6 변이 전건

면 버리기 방향 · 동작 행렬 곱 생략 · `lookAt` 외적 순서 · `draw` 의 캔버스 쓰기 생략 · `setView` 배율 축 뒤바꿈 · `XFile.buf` 미기입. 각각 해당 시험이 FAILED, 원상 green.

### 검증

`cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`(713 통과 · 0 실패) · `npm run build:wasm` · `check-engine-contract`(113 통과) · `npm run audit` 전부 rc=0.
러너 줄(엔진 변경 · 이 브랜치 release `wie_validate`): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 load1 63 에서 첫 실행이 `UNMEASURED · stop max-ticks` 였다. 전 빌드도 같은 값이었다. AGENTS 지시대로 `--max-ticks` 를 올리자 둘 다 PASS · rc=0(paints 80 · 55)이었다.

### 후속

| # | 대상 | 벽 | 크기 | 근거 |
|---|---|---|---|---|
| 1 | `71d1d8235bd1`(+ SKT 소리 스레드 74종) | `AudioClip.play` 가 소리 스레드에서 재생 동안 블록하고, `close` 가 그 `play` 를 예외로 끊어야 한다 | M — 74종 전/후 짝 필수 | §2-3 |
| 2 | KTF `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | 서술자 배열 적재기 + `MNInterface` 칸(10개 이상) + `fp` 문맥(+0x34 = 스택) | L | §3 |
| 3 | `71d1d8235bd1` | longplay·speed·progress 미측정 | S — 1이 착지한 뒤 census 한 번 | §4 |
