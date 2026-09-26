## [2026-09-26] DataInputStream 19 = readFully([B)V — 스파이더맨3(LGT) 첫 렌더 PASS (wie-2026-09-25-spiderman3-datainputstream-19-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 의 `java/io/DataInputStream` 블록에 `readFully([B)V = 19` 한 행과 근거 주석을
더했다. 코드 변경 0. 채택 제안 `2026-09-25-datainputstream-readchar-readutf#p0`.

**왜**: 스파이더맨3 가 27 을 넘긴 뒤 이 빈 칸에서 `Unimplemented` 로 죽었다. 19 는 선언 순서 규칙의 첫 자기 칸이고
(20 = readFully([BII) 바로 앞), 호출부 인자가 그 자리와 맞는다(아래 ⒝).

**사용자 영향**: 스파이더맨3(LGT)가 **처음으로 화면을 그리고 입력 27/27 을 버틴다**(`wie_validate` PASS · 시각 정확성은 미확인).

### ⒜ 반증 시도 — `origin/main` `e5fa2af3`
- 칸 19 행: 없음(블록은 20 22 23 25 27 28 32).
- release `wie_validate --inject` ×3: `stop error` · paints 1 · `Unimplemented: java/io/DataInputStream vtable index 19` **3/3**. 전제가 선다.

### ⒝ 판정 — 호출부(binary.mod `.text` VA · Thumb)
누락 칸 핸들러에 임시 덤프를 붙인 빌드(커밋하지 않았다)로 잰 값:

| 축 | 측정 |
|---|---|
| LR | `0x1b8b` 3/3 → 호출 `0x1b86`, 디스패치 `0x1b82 ldr r3, [r3, #0x50]`(= 4 × (19+1)) |
| 인자 | r0 = 스트림 · r1 = `0x1b46` 에서 방금 할당한 배열(길이 = readShort(`#0x68`) 결과 − 4) · r2 = 할당 때 남은 `8`(낡은 값) |
| 게스트 힙 | r1 의 클래스 = **`[B`** · 길이 **30** — 3/3 |
| 반환값 | 버린다(`0x1b8a` 부터 r0 재설정) |

byte[] 하나만 받고 결과를 버리는 DataInputStream 메서드는 readFully([B)V 뿐이다 — 20(`[BII`)·21(skipBytes(I)I)은 인자 모양이 다르다.

★**두 번째 호출부 — 코퍼스에 없다.** 제안의 tradeoff 가 요구한 교차는 수행했고 결과는 «부재»다:
- 정적: 로컬 LGT 91개 binary.mod 에서 `ldr rX, [rX, #0x50]` + bl 디스패치를 DataInputStream 오프셋(`#0x54`~`#0x74`) 이웃 기준으로
  훑었다. 스파이더맨3 의 다른 `#0x50`(`0x375fe`)은 r1·r2 두 인자라 아니다. 다른 타이틀 후보(게임빌2010프로야구 `0xb4064`,
  블레이드마스터3 `0x3ce78`, SD한국전쟁 `0x316bc`)는 열어 보니 스트림이 아니었다(정수 인자·정적 필드 수신자).
- 동적: 무행 빌드로 88개 실행(20초씩) — `DataInputStream vtable index 19` 에 닿는 타이틀은 스파이더맨3 **하나**.
- ⇒ 이 행은 **호출부 1곳 + 규칙 자리 + 게스트 힙의 인자 타입** 위에 선다. 주석에도 그렇게 적었다.

### ⒞ before / after (release · `--inject`)
| | stop | input | paints | content | colors | 죽음 |
|---|---|---|---|---|---|---|
| before ×3 | error ×3 | — | 1·1·1 | — | — | DataInputStream **19** 3/3 |
| after ×3 | deadline·max-ticks·deadline | 27·26·27 | 237·234·227 | true | 512 | 없음 · java_exceptions 0 |
| after `--max-ticks 200000000` | deadline | 27/27 | 226 | true | — | 없음 |

after 2회차 `UNMEASURED`(max-ticks, 26/27)는 AGENTS.md 대로 `--max-ticks` 를 올려 재실행해 PASS. **다음 벽: 예산 안에서는 없다.**
stub_hits 는 Font 계열(`Font::<init>`·`getDefaultFont` 1194 · `charsWidth` 112) — 글자 폭이 스텁이라 문자열 배치는 틀릴 수 있다.

### 무회귀 — 같은 시간대 base/after 짝 실행
| 타이틀 | base | after |
|---|---|---|
| 슈퍼액션히어로 | FAIL · `java/util/Random vtable index 10` | 동일 |
| 간호사타이쿤2 | FAIL · paints 0 | 동일 |
| 배틀몬스터 | PASS · 141 | PASS · 181 |

### 게이트
- `fmt` rc0 · `clippy --all -D warnings` 출력 0 · wasm clippy 출력 0 · `cargo +beta clippy --all -D warnings` 출력 0.
- `RUST_MIN_STACK=4194304 cargo test --all`: **454 passed / 0 failed**(결과 블록 47).
- `clippy --workspace --all-targets` 경고 13 — diff 의 Rust 줄 0 이라 증가 0.
- runner: `helloworld_lgt` PASS · `keydraw_lgt --inject --expect-last-frame` PASS · paints 55 · rc0.
- 부하: loadavg 60~119(측정 내내).

유입(corpus-name-inflow): BOUNDED 25건 + SUFFIX-ATTACHED 4건 — 전부 이 리니지가 이미 쓰던 타이틀명(스파이더맨3·슈퍼액션히어로·간호사타이쿤2·배틀몬스터 등)이고 게임 바이트 0.

<!-- corpus-name-inflow v1 subjects=3 tree=73e626014cecafd8 B=59/25 P=0/0 S=9/4 -->
