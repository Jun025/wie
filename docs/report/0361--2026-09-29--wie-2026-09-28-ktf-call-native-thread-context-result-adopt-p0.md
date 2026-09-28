## [2026-09-29] KTF 0103BF27 — 그림은 오프스크린이 아니라 «화면 프레임버퍼»로 간다 · 화면에 올리는 것은 Java paint 주기다 (wie-2026-09-28-ktf-call-native-thread-context-result-adopt-p0)

**무엇을**: 판정만. 코드 변경 0 — 대상 2종(`4120f27288ab` `739c7657c1f2`)이 그린 그림을 화면으로 옮기는 경로를 식별했다.

**왜**: 0353 제안 `#p0` — 고친 뒤에도 흰 화면이 남는 이유.

**사용자 영향**: 없음(이 회차는 코드를 바꾸지 않는다). 아래 시제품으로 대상이 실제 게임 화면(인트로 그림 + 한글 자막)을 그린다는 것은 확인했다.

### 결론

1. ★**0353 의 전제 «그리기가 전부 `MC_grpCreateOffScreenFrameBuffer` 버퍼로 간다»는 틀렸다.**
   20초 한 회(대상 `739c7657c1f2`): `MC_grpFillRect` 10,159 · `MC_grpDrawString` 2,686 · `MC_grpPutPixel` 452 가
   **전부 한 핸들 `0x49052ce0`** 로 가고, 그 값은 `MC_grpGetScreenFrameBuffer(0)` 이 준 핸들(`0x7fff1000` 칸의 값)과 같다.
   `MC_grpCreateOffScreenFrameBuffer(240,320)` 가 준 핸들 `0x4904f0c0` 에는 그리기 **0회**.
   화면 프레임버퍼 내용은 실제로 차 있다(표본: 서로 다른 색 2~52 · 0 아닌 화소 70,080~72,904 / 76,800).
2. **한 프레임의 순서**(스레드 T2): `MC_knlCurrentTime` → 화면 프레임버퍼에 그리기 → `Card.repaint()` → `Card.serviceRepaints()`
   → `Card.paint(g)`(`g` 에는 `reset`·`translate` 만) → `Thread.sleep`. `MC_grpFlushLcd`·`MC_grpCopyFrameBuffer` 는 0회.
   ⇒ 게임은 **Java 화면과 네이티브 화면 프레임버퍼가 같은 메모리**라고 가정하고, «그린 뒤 repaint 를 부르면 시스템이
   그 메모리를 LCD 로 올린다»는 경로를 쓴다. wie 의 KTF Java 모드는 `Display` paint 뒤 **Java 쪽 `screenImage`** 만
   올리므로(`wie-midp` `display.rs` 의 `screen.paint(&*image)`), 네이티브 프레임버퍼는 한 번도 화면에 가지 않는다.
3. `org.kwis.msp.lcdui.Graphics.getPixels(0,0,1,1,byte[],0,4)` 는 **이 경로에 필요하지 않다** — 아래 시제품은 그 스텁을
   그대로 두고(배열은 0) 화면을 얻었다. 역할(화소 형식 탐지인지 등)은 판정하지 않았다.

### 시제품(임시 계측 · 커밋 0)

- ⑴`MC_knlCurrentTime` 에서 화면 프레임버퍼가 있으면 `screen.paint` · ⑵midp `Display` 의 `screenImage` 올리기를 끔.
  ⑴만: 대상 2종 20초 `PASS` · `content true` · `distinct_colors` 512(종전 1) — 단 마지막 프레임은 뒤이은 Java paint(한 색)가 덮는다.
  ⑴+⑵: 12초 스냅숏이 인트로 화면(그림 + 자막 + `#SKIP`)이다.
- ⑴은 위치가 틀렸다(커널 시계 호출은 우연히 프레임 머리에 있을 뿐) — **구현안이 아니다.**

### KTF 전수 — 화면 프레임버퍼를 누가 가져가나(비-LTO release · 12초 · 키 없음 · 266종 · `MClass`=`__adf__`)

| 모드 | `GetScreenFrameBuffer` | `FlushLcd` | 결과 | 수 |
|---|---|---|---|---|
| Java | 0 | 0 | PASS 179 · FAIL 13 | 192 |
| Java | ≥1 | 0 | FAIL | **2 (대상)** |
| `__adf__` 가 최상위에 없음 | 0 | 0 | PASS | 3 |
| `__adf__` 가 최상위에 없음 | ≥1 | 106·206 | PASS | 2 (`93d5b6b8ceb5` `974e0df9ab1e`) |
| Clet | ≥1 | ≥1 | PASS 52 · FAIL 2 | 54 |
| Clet | ≥1 | 0 | FAIL | 11 |
| Clet | 0 | 0 | FAIL | 2 |

- ⇒ Java 모드에서 화면 프레임버퍼를 가져가고 **스스로 올리지 않는** 타이틀은 대상 2종뿐이다. 구현이 Java paint 주기에
  네이티브 프레임버퍼를 올리게 하면 직접 영향권은 이 2종 + 스스로 올리는 2종(`FlushLcd` 로 이미 올리므로 순서만 관건)이다.
- Clet 모드 11종(가져가고 `FlushLcd` 0 · 전부 FAIL)은 같은 원인인지 **재지 않았다** — Clet 은 midp paint 가 꺼져 있어
  경로가 다르다(`disable_midp_paint`). 흰 화면 군으로 따로 남긴다.
- load1 120~170 에서 잰 1회다 — 수는 분류용이고 PASS/FAIL 판정 근거가 아니다.

### 구현이 고를 것(다음 회차)

- 실기에서는 한 메모리이므로 «나중에 쓴 쪽이 이긴다». Java 이미지를 통째로 네이티브 프레임버퍼로 바꾸면
  Java 로 그리면서 프레임버퍼만 받아 두는 타이틀이 흰 화면이 된다 — 위 표에서 Java 모드에 그런 타이틀은 없었지만
  12초·키 없음 1회의 관측일 뿐이다. 차이 기반 합성(마지막 동기 이후 네이티브 쪽에서 바뀐 화소만 Java 이미지 위에)이
  두 방향을 다 지킨다. 이 판단과 KTF 전수 짝 재측은 제안 `#p0` 의 몫이다.

### 검증
- 코드 변경 0. 이 가지(`origin/main` `270e4770` + 문서 2개)에서 `cargo fmt --all -- --check` ·
  `cargo clippy --all -- -D warnings`(stable·beta) · wasm32 clippy · `RUST_MIN_STACK=4194304 cargo test --all`(586 passed) rc=0.
- `node scripts/corpus-name-inflow.mjs`: 유입 **0쌍(BOUNDED)** + 판단 필요 **0쌍(SUFFIX-ATTACHED)**. 타이틀은 sha 앞 12자로만 적었다.

<!-- corpus-name-inflow v1 subjects=2 tree=e73d2eed868903ce B=0/0 P=0/0 S=0/0 -->
