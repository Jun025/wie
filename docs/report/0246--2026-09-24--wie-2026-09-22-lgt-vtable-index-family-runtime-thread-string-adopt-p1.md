## [2026-09-24] 학교가는길 `ax vtable index` — 앱 클래스 표가 샌 것도, ABI 행이 빠진 것도 아니었다. 런타임이 링크 때 «부모에만» 덧붙인 칸이 자식에게 안 갔다 (wie-2026-09-22-lgt-vtable-index-family-runtime-thread-string-adopt-p1)

## 반증 먼저

- ⒜ **현 main(`3d7ac24c`)의 벽은 `ax vtable index 30` 이 아니라 `29` 다**(FAIL · tick 3 · paints 0). 번호가 움직인 것은
  «고정 ABI 인덱스»가 아니라는 첫 증거다.
- ⒝ `ax` 는 `build_from_compiler_vtable` 경로를 **타지 않는다** — 그 경로 덤프(`ptr_vtable != 0`)에 나온 클래스는 `w`·`u`·`ak` 셋뿐이다.
  `ax` 는 `ptr_vtable == 0` 이라 `build_methods`(부모 표 복사) 경로다. ⇒ 가설 ⑴(그 함수의 구멍)은 **기각**.
- 상속 사슬(계측): `ax → aa → an → org/kwis/msp/lwc/ShellComponent → ContainerComponent → Component → Object`.
  서술자 `vtable_count` = an 125 · aa 143 · ax 143. ShellComponent 표 = 39칸. **29 는 플랫폼 부모 범위 안** ⇒ 모양은 ⑵.

## 그런데 처방은 ⑵의 «부모 행을 측정으로 채운다»가 아니다

링크 로그(`virtual_method_index`)가 29 의 정체를 말한다:

```
VTLINK APPEND org/kwis/msp/lwc/Component.getHeight()I -> 28
VTLINK APPEND org/kwis/msp/lwc/Component.getWidth()I  -> 29
```

게스트는 `Component.getWidth` 를 **이름으로 링크**하고, 런타임은 ABI 행이 없으니 Component 표 끝(당시 길이 29)에 덧붙여
**29 를 돌려준다.** 그런데 ShellComponent·an·aa·ax 는 **그보다 먼저** 만들어져 부모 표를 «복사해» 가진 상태라 그 칸이 없다 —
ax[29] 는 ShellComponent 가 자기 미배치 메서드용으로 예약한 **빈 칸 스텁**이었다. 그래서 `ax vtable index 29`.

⇒ 29 는 LGT 의 고정 ABI 값이 **아니라 우리 런타임이 그 순간 고른 번호**다. `lgt_java_abi.toml` 에 `Component getWidth = 29` 를 넣으면
이 타이틀은 통과하겠지만, 그것은 «측정한 ABI»가 아니라 **우리 버그의 부산물을 ABI 로 박제**하는 것이다. 그래서 행을 넣지 않았다.

★더 나쁜 형태도 같은 뿌리에 있다: 부모가 자기 길이에 덧붙이면 그 번호가 **자식이 이미 «다른 메서드»로 쓰는 칸**일 수 있다.
그때는 스텁이 아니라 **엉뚱한 메서드가 조용히 호출된다.** 이번 타이틀은 빈 칸이라 시끄럽게 죽었을 뿐이다.

## 한 일(엔진)

1. `LgtJvmSupport::virtual_method_index` — 링크 때 덧붙이는 칸을 **이미 로드된 모든 하위 클래스에도** 쓴다(하위가 오버라이드했으면
   그 구현). 번호는 ⒜그 하위들 표 길이의 최대 ⒝아직 안 로드된 «컴파일된 표» 하위 클래스의 `vtable_count` 최대 — 둘 다 넘는 곳으로
   잡아 **충돌이 없게** 한다. 하위 열거 = 게스트 생성 클래스 표(`LgtClassLoader.generatedClasses`) + wipi-java·midp 플랫폼 클래스.
2. `JavaVtable::build_from_compiler_vtable` — 부모 표가 컴파일된 개수보다 길면(런타임이 덧붙인 칸) 그 꼬리를 **상속**한다(종전엔 잘렸다).
3. `ArmCore::make_shared_svc_stub` + `JavaVtable::write` — 빈 칸 스텁을 **(category, index)당 1개**로 공유.
   ★1·2 만 넣었을 때 3종이 **`SVC stub space exhausted` → 스택 오버플로(abort)** 로 죽었다(당신은골프왕·서든어택포켓·붕어빵타이쿤3).
   스텁 공간은 4,096개(64KiB/16B)인데 학교가는길은 수정 **전에도** 약 2,200개를 쓰고 있었다 — 표를 새로 쓸 때마다 빈 칸마다 새 스텁을
   찍었기 때문이다. 스텁은 (category, id)의 순수 함수이고 핸들러는 인스턴스에서 클래스 이름을 읽으므로 공유해도 문면이 같다.

## 측정

| | before(main 동일 동작) | after |
|---|---|---|
| 학교가는길 | FAIL · t3 · p0 · `Unimplemented: ax vtable index 29` | FAIL · t1231~2198 · p0 · `no frame rendered (hang/black screen)` |

★**완주는 아니다** — 벽을 넘어 «부팅은 되는데 안 그리는» 계급(배틀몬스터·서든어택포켓과 같은 벽)으로 옮겼다.

**LGT 짝 회귀**(release · `--inject` · 91파일 · 파일마다 before→after 연속 · 6병렬 · loadavg 22~34):
PASS→PASS 45 · FAIL→FAIL 41 · UNMEASURED→UNMEASURED 2 · NOJSON→NOJSON 1 · **PASS→UNMEASURED 2** · **PASS→FAIL/NOJSON 0**.
그 2건(테일즈위버 이스핀편·현영맞고2006)은 `max-ticks` 소진이고 3회 재측에서 before 도 같은 결과를 냈다
(테일즈위버 before PASS 1/3 · after 3/3 · 현영맞고 2/3 대 2/3) ⇒ **부하 노이즈이고 회귀가 아니다.**
★1차 짝 회귀(3번 없이)는 **NOJSON 3건**을 냈고 그것이 위 3번을 찾게 했다 — 회귀 검사가 실제로 무언가를 잡았다.

**양방향 변이**(`virtual_method_appended_to_a_parent_reaches_subclasses_built_before_it`):
- 원본 → **ok**
- M1 하위 전파 제거 → **FAILED** `ShellComponent vtable index 34` 불일치
- M2 충돌 회피 제거(부모 길이에 덧붙임) → **FAILED** `index … collides with the subclass's … slots`

**게이트**: fmt ✅ · `clippy --all -D warnings` ✅ · wasm clippy ✅ · `+beta clippy` ✅ · `cargo test --all` **431 passed · 0 failed**(46 suites) ·
`clippy --workspace --all-targets` 경고 16건 전부 이 diff 밖 파일(`wie-backend/canvas.rs`·`hardening.rs`·`dod_ci_parity.rs`) ⇒ 증가 0.
러너 블록: draw_j2me·helloworld_ktf·helloworld_lgt·text_j2me **PASS** · keydraw_ktf/lgt(`cargo run` 문서 명령) **PASS · paints 55 · content true**.
※release 바이너리로 keydraw 를 돌리면 before/after **둘 다** `UNMEASURED(max-ticks)` 다 — release 가 틱을 더 빨리 태우기 때문이고 이 diff 와 무관하다.

## 남는 것(숨기지 않는다)

- 하위 열거는 «생성 클래스 + wipi-java·midp 플랫폼 클래스»다. `java/lang/*`(rustjava) 쪽의 플랫폼 하위 클래스는 **안 본다** —
  그쪽 부모에 링크로 덧붙는 경우는 이번에 관측되지 않았다.
- 덧붙인 칸 번호가 커졌다(예: `Graphics.setColor` 67 → 컴파일 하위 경계 이상). 의미는 같고 표가 길어질 뿐이다.
