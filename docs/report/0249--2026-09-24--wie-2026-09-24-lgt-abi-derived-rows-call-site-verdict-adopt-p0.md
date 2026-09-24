## [2026-09-24] java/lang/String 27 = substring(I) — 일지매영웅전기 FAIL(paints 0) → PASS · Runtime 12 · Thread 13 은 미확정 기록 (wie-2026-09-24-lgt-abi-derived-rows-call-site-verdict-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` String 블록에 `substring(I)Ljava/lang/String; = 27` 1행과 근거 주석,
Runtime 12 · Thread 13 옆에 「관측으로 미확정 — 재측정 트리거」 주석 1줄씩(ABI 값 무변경), ABI 핀 테스트
(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 27 행 1줄.

**왜**: indexOf(I)=21(#267) 착지 뒤 일지매영웅전기의 벽이 `java/lang/String vtable index 27` 로 옮겨졌다.

**사용자 영향**: 일지매영웅전기가 부팅 후 화면을 그린다(종전 검은 화면). 화면 내용의 시각적 정확성은 재지 않았다.

### ⒜ 현 main 에서 이 칸이 이름을 대는가 — 예

`origin/main` `c3a8c992` 빌드, `wie_validate --inject`:
`FAIL` · `no frame rendered (hang/black screen)` · ticks 870 · paints 0 ·
`java_exceptions.first` = `net/wie/WieError: Unimplemented: java/lang/String vtable index 27`.

### ⒝ 27 이 substring(I) 인 근거 — 호출부 직접 디스어셈블(임시 계측 · 커밋 안 함)

missing-entry 스텁에 lr·r1·r2 와 lr-0x200 부터 0x300 바이트 코드 덤프를 달아 Thumb 로 풀었다
(vtable 오프셋 = 4 + 4×index: 0x58=21 · 0x70=27 · 0x74=28).

```
0x383dc ldr r6,[sp,#0x24]          ; s  (루프 문자열 슬롯)
0x383de ldr r4,[sp,#8]             ; '|'
0x383e8 ldr r3,[r3,#0x58] ; bl     ; i = s.indexOf('|')        (21)
0x383f0 str r0,[sp,#0x1c] ; blt …  ; i < 0 이면 루프 탈출
0x38406 ldr r3,[r3,#0x74]          ; s.substring(0, i)          (28)  r1=0 · r2=i
0x38422 ldr r4,[sp,#0x20] ; adds r4,#1   ; r4 = i + 1
0x3843a ldr r3,[r3,#0x70]          ; s.<27>(i + 1)              r0=s · r1=r4
0x38442 str r0,[sp,#0x24]          ; 결과 → 루프 문자열 슬롯
0x38450 b 0x383dc                  ; 루프 머리에서 그 값을 21 의 «수신자»로 다시 쓴다
```

- 인자: r1 = i + 1 하나(실측 r1 = 0x2, r2 는 이전 호출의 힙 포인터 잔여).
- 반환: 루프 문자열 슬롯에 저장되고 다음 바퀴에서 String vtable 디스패치의 수신자가 된다 ⇒ 객체.
- 15..28 run 에서 int 1개·객체 반환은 substring(I) 하나다(indexOf(I)·lastIndexOf(I) 는 int 반환). 위치도 규칙과 같다.

### ⑴ 전/후 + 양방향 변이 (`--inject` · 기본 예산)

| 트리 | result | reason | ticks | paints | last_frame_content | java_exceptions.first |
|---|---|---|---|---|---|---|
| main `c3a8c992`(행 없음) | FAIL | no frame rendered | 870 | 0 | false | `…String vtable index 27` |
| 이 브랜치 run 1 | **PASS** | booted + rendered + survived input | 3,121,688 | 43 | true | `java/io/FileNotFoundException: File not found` |
| 이 브랜치 run 2 | **PASS** | 〃 | 185,963 | 2 | true | 〃 |
| 변이: 27 행만 제거 | FAIL | no frame rendered | 246 | 0 | false | `…String vtable index 27` |

paints 43 ↔ 2 차는 부하(loadavg 28→125)다 — 판정은 두 번 다 PASS · content true 다(AGENTS.md 「count 는 floor」).
남은 FileNotFoundException 은 1건이고 게스트가 잡아 계속 진행한다(판정 무관 · 이번 범위 밖).

### ⑵ Runtime 12 · Thread 13

값 무변경. 두 행 머리 주석 끝에
`# Unresolved by observation (N) — re-measure trigger: a title observed dispatching this slot.` 1줄씩.
근거는 원 제안 그대로다(Runtime 12 반환값은 다음 함수 첫 명령에서 덮임 · Thread 13 은 100타이틀 0회 디스패치).

### 회귀

- 네 게이트: `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` 통과 ·
  wasm clippy 통과 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — **46 스위트 · 432 passed · 0 failed**
  (`abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 27 행 포함 ok). `cargo +beta clippy --all -- -D warnings` 통과.
- `cargo clippy --workspace --all-targets` 의 `^warning` 줄: 변경 전 16 · 후 16(증가 0).
