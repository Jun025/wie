## [2026-09-28] KTF 0103BF27 — «monitor_exit 뒤 멈춤»은 11초 sleep 이었다 · call_native 가 스레드 컨텍스트 결과 칸을 읽는다 (wie-2026-09-28-wie-census-blank-screen-14-and-invokespecial-lookup-adopt-p1)

**무엇을**
- `wie-ktf` `call_native`(JB 인터페이스 슬롯 0x30): 네이티브가 결과를 스레드 컨텍스트 `+0x24`(태그) · `+0x28`(값)에 실으면
  그 값을 반환한다(태그 2 = int). 호출 전에 칸을 비우고 호출 뒤 원래 값을 되돌려, 안쪽 호출의 태그가 바깥 호출의 답이 되지 않는다.
  태그 0 은 종전대로 r0:r1 · 그 밖의 태그는 경고 후 r0:r1.
- `KtfJvmThreadContext` 를 0x24 → 0x30 바이트로 늘렸다(`native_result_type` · `native_result[2]`).
  종전에는 게임의 `+0x24`/`+0x28` 쓰기가 **할당 밖**으로 넘치고 있었다.
- 시험 1(`test_call_native_reads_result_published_in_thread_context`): 손으로 조립한 Thumb 네이티브 2개(칸에 싣는 것 · r0 만 쓰는 것).

**왜**: 0334 제안 `#p1` — 로딩 스레드가 `monitor_exit` 뒤 멈춘다는 관측의 원인.

**사용자 영향**: 대상 2종(`4120f27288ab` `739c7657c1f2`)의 로딩 루프가 11초 주기에서 제 주기로 돈다(20초 paints 5~6 → 380~399).
**화면은 아직 흰색이다** — 다음 벽(아래)이 남아 있어 이용자에게 보이는 변화가 없으므로 `docs/player-updates/` 기록은 두지 않았다.

### 원인 — 잰 것

측정 = release(LTO 끔) `wie_validate` + 임시 계측(커밋 0).

1. 로딩 스레드는 멈추지 않았다. `monitor_exit` 다음이 `Thread.sleep(10026)` · 다음 회 `sleep(11027)` — 20초 창에서 두 번 잠든다.
   `sleep` 값 + 그 회의 경과 = **11,036ms 로 일정**(8638+2398 · 10985+51 · 11029+7).
2. 호출부 `0x108e26`(돌아올 주소 `0x108e2b`)의 루프: `t0 = currentTimeMillis()` → `period = <네이티브 0x1058bd>()` → 그리기 →
   `sleep(max(period − (now − t0), 10))`.
3. `0x1058bc` 의 끝: `*(base+0x2b1c)` 를 `(*G)+0x28` 에, `2` 를 `(*G)+0x24` 에 쓰고 **r0 에는 상수 `0x2b1c`(=11036)** 를 남긴 채 반환.
   `G` 는 우리가 넘기는 «현재 스레드 컨텍스트» 칸이다 — 같은 이미지가 `(*G)+0x20` 에 예외 핸들러를 쓰고(`0x108e60`),
   그 자리가 `KtfJvmThreadContext::current_java_exception_handler` 와 같다.
4. 같은 이미지에서 이 칸에 태그를 쓰는 곳 전수(Thumb 스윕): **3곳, 전부 태그 2**. `0x129e18` 는 칸에도 쓰고 r0 에도 같은 값을 남겨 종전에도 맞았다.

### KTF 전수 짝 재측(같은 시각 · 비-LTO release 전/후 · 두 소스 파일만 다름)

| 회 | 조건 | load1 | PASS→PASS | FAIL→FAIL | 나빠진 쪽 | 좋아진 쪽 |
|---|---|---|---|---|---|---|
| r1 | 20초 · 키 없음 · 266종 | 171~274(중앙 221) | 238 | 27 | 1(`65bace1623a7`) | 0 |
| r2 | 같음(재측) | 282~433(중앙 378) | 229 | 34 | 1(`7ec716a0cec9`) | 1 |
| k1 | `--inject` · 266종 | 261~361(중앙 304) | 204 | 35 | PASS→FAIL 1 · PASS→UNMEASURED 5 · 끝 화면 빔 3 | 14 |

- 나빠진 쪽 전건을 짝 3회 재측: `65bace1623a7` `7ec716a0cec9` `c94d64777926` 은 전/후 모두 3/3 PASS(첫 회 paints 0~2 = 굶음).
  끝 화면 빔·UNMEASURED(깨끗한 종료)는 **전/후 양쪽에서** 나온다 — 키 도착 시점에 따라 종료 메뉴까지 가는 경주다.
- `89c214dbd15d`(commonRes 아님)만 쏠림이 보였다: 짝 10회 UNMEASURED 전 **3/10** · 후 **6/10**(Fisher p≈0.37 — 유의 아님).
  이 타이틀은 commonRes 틀이 아니다(칸에 태그를 쓰는지는 재지 않았다). 알려진 경로 차이는 컨텍스트 할당이 12바이트 커져 힙 배치가 밀린 것뿐이다.
- `js_commonResInvokeNative` 틀을 쓰는 KTF 19종 중 r1 에서 바뀐 것은 대상 2종뿐이다(나머지 17종 paints·색 수 동일 범위).

### 다음 벽 — 흰 화면이 남는 이유(못 고침 · 제안 등재)

고친 뒤 20초에 `MC_grpFillRect` 10,013 · `MC_grpDrawString` 1,445 가 **전부** `MC_grpCreateOffScreenFrameBuffer(240,320)` 버퍼로 간다.
게임의 `Card.paint(g)` 는 `g` 에 아무것도 그리지 않고(reset·translate 만), `MC_grpFlushLcd`·`MC_grpCopyFrameBuffer` 는 0회다.
단서 하나: 첫 paint 에서 `org.kwis.msp.lcdui.Graphics.getPixels(0,0,1,1,byte[],0,4)` 를 한 번 부른다(현재 스텁).

### 검증
- 시험 1 · 변이 3(칸 무시 · 호출 전 비우기 삭제 · 되돌리기 삭제) 전건 red · 원상 green.
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable·beta) · wasm32 clippy · `RUST_MIN_STACK=4194304 cargo test --all`(581 passed) rc=0.
- 러너 블록: `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS · `keydraw_ktf` `keydraw_lgt` `--inject --expect-last-frame` PASS rc=0(paints 55·55).
