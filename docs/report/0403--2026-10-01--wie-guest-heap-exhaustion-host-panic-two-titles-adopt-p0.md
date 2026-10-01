## [2026-10-01] 손님 힙이 차는 두 타이틀 — KTF 는 오프스크린 픽셀 누수, LGT 는 화면 버퍼 덮어쓰기 (wie-guest-heap-exhaustion-host-panic-two-titles-adopt-p0)

**무엇을**: 0389 §5 의 두 벽을 계측했다.
- `7e2247bdf565`(KTF)는 **누수**였다. `MC_grpDestroyOffScreenFrameBuffer` 가 레코드만 반납하고 픽셀 버퍼는 남겼다. 공유 함수를 고쳤다.
- `fe76e641bb3d`(LGT)는 **힙이 차지 않았다**. 게임이 화면 버퍼 끝 너머에 24줄을 써서 다음 블록의 할당기 헤더를 덮었다. 같은 날 #434 가 다른 처방으로 착지해, 이 PR 은 그쪽을 건드리지 않는다(§3).
- 그 밖에 KTF 클래스 정의의 `unwrap` 두 곳을 손님 예외로 바꿨고, 할당이 실패하면 힙 요약을 남기게 했다.

**왜**: Tower 채택 제안 `2026-09-30-census-wave3-singleton-tail#p0`.

**사용자 영향**: `7e2247bdf565` 가 4~5분 뒤 꺼지지 않는다. 지원 현황 limited → **playable**(`compat.json` 1행). 오프스크린을 지우는 다른 4종은 짝 재측에서 그대로다.

증적은 스크래치(`/tmp/wie-heap-p0/`)에 있다. 이 문서는 타이틀을 sha12 로만 적는다.

### 1. 계측 — 할당 실패 순간의 힙

탐침(미커밋)은 셋이다. ⑴`Allocator::alloc` 이 `AllocationFailure` 일 때 list 블록 목록과 버킷별 점유를 찍는다. ⑵`0x20000` 이상 할당과 129~256바이트 할당·반납을 주소·lr 과 함께 찍는다. ⑶특정 주소에 쓰는 호스트 `write_bytes` 를 찍는다. ⑴만 정리해 커밋했다(§4).

| sha12 | list(128MB) | 버킷 | 판정 |
|---|---|---|---|
| `fe76e641bb3d` | 18블록. 18번째(`0x403431d4`) 헤더가 `0xffffffff` — 크기 `0x7fffffff` · in_use | 전부 0.3% 미만(4B 404/1048576 … 512B 57/65536) | **덮어쓰기**. 힙은 비어 있다 |
| `7e2247bdf565` | 2110블록 · 최대 빈 구간 `0x6d2e944`(약 109MB) | **256B 131072/131072** · 나머지 4% 미만 | 256B 버킷만 가득. list 로 넘기지 않는다(`Allocator::alloc` 은 크기로만 고른다) |

### 2. `7e2247bdf565` — 누가 256B 버킷을 채우나(시간축)

129~256바이트 할당 134,564건 중 반납은 3,492건이다. 남은 131,072건(= 버킷 전체)의 출처:

| 크기 | 손님 lr | 건수 | 남음 |
|---|---|---|---|
| 210 | `0x115b47` | 126,832 | 126,832 |
| 152 | `0x115b47` | 3,515 | 3,515 |
| 174·141·166·175·197·236 | `0x10ffa3` | 3,600대 | 133 |

같은 런의 살아 있는 개수(30초 간격): 134 → 14,025 → 25,777 → 34,301 → 46,150 → 57,954 → 70,756 → 82,565 → 95,185 → 107,524 → 119,628 → 130,202(실패). **초당 약 400건으로 선형 증가**한다 = 누수다.

같은 타이틀의 디버그 로그(60초)에서 초당 수백 번 나오는 쌍은 `MC_grpCreateOffScreenFrameBuffer(9, 11)` → `MC_grpFillRect` → `MC_grpDestroyOffScreenFrameBuffer` 이다(create 20,887 · destroy 25,461 — 다른 크기 포함). 픽셀은 9×11×2 = 198바이트이고, KTF 간접 할당 머리를 더하면 210이다. 152 = 7×10×2 + 12 도 같은 함수다.

`destroy_offscreen_framebuffer` 는 `context.free(framebuffer)` 한 줄뿐이었다. `FrameBuffer::new` 가 픽셀을 **따로** 할당하는데 그것을 반납하지 않았다. ⇒ 레코드(20B · 32B 버킷)는 돌아오고 픽셀만 쌓였다. 버킷 표에서 32B 가 비어 있는 이유와 맞는다.

**고침**: 레코드를 읽어 `buf` 가 0 이 아니면 먼저 반납하고, 그다음 레코드를 반납한다(`wie-wipi-c/src/api/graphics.rs`). 널 핸들 경로는 그대로다.

**근거 — 횟수로 잰다(부하와 무관)**: 600초 동안 오프스크린 create 수를 센다.

| 빌드 | create | destroy | 결과 |
|---|---|---|---|
| 전 `origin/main` `c387adef` | 135,851 | 135,850 | `panic … unwrap() on AllocationFailure` |
| 후(이 브랜치, #434 병합 전) | 263,200 | 263,200 | 할당 실패 0 · deadline |
| 후(#434 병합 뒤 `43ccfe63` · guard rc=0) | 233,442 | 233,442 | 할당 실패 0 · panic 0 · deadline |

후 빌드는 전 빌드가 죽은 횟수의 약 2배를 지나도 할당 실패가 0이다.

**호스트 panic 경로**: 전 빌드의 panic 은 `wie-ktf/.../jvm_implementation.rs` 의 `define_class_rust` `unwrap()` 이다(0389 의 `:55`). 이제 `KtfJvmSupport::instantiation_error`(예약 `OutOfMemoryError` → 없으면 `WieError`)로 손님에게 넘긴다. `define_array_class` 의 `unwrap()` 도 같다. LGT 는 이미 `host_error` 로 넘기고 있었다(#424).
- ★**대조 시험이 없다.** 클래스 정의를 실패시키려면 첫 할당(4B 버킷 100만 칸)부터 채워야 한다. 버킷 할당기는 헤더를 선형으로 훑으므로 시험이 너무 느리다. 누수가 고쳐진 지금 이 타이틀은 그 경로에 닿지 않는다.

### 3. `fe76e641bb3d` — 덮어쓰기 지점(#434 와 같은 결론, 다른 처방 근거)

- 0x25804(= 240×320×2 + 4)는 `MC_grpGetScreenFrameBuffer` 의 픽셀이다. 그 끝 `0x403431d0` 을 호스트 쓰기 감시로 잡았다. `memcpy(dst=0x403431d0, src=0x4036ecb8, 0x1e0)` 이고 lr 은 `0x28685` 다.
- 그 함수(`binary.mod` Thumb `0x28560~0x2868e`)는 fill-rect 다. 클립(`bl 0x25380`)이 준 `y1..y2` 를 돌며 한 줄(폭×2)씩 `memcpy` 한다. 줄 주소는 `MC_GRP_GET_FRAME_BUFFER_POINTER` + `(y·stride + x)·2` 다.
- 같은 런에서 그 lr 의 `memcpy` 11,520건의 목적지 줄은 **24 ~ 343** 이다. 클립이 **위 24줄을 비우고** 320줄을 그린다.
- 부팅 순서는 `GetScreenFrameBuffer → WIDTH → HEIGHT → CreateOffScreenFrameBuffer(240, 344)` 이다. `clet_register(_, 1)` 로 어넌시에이터를 켠 타이틀이다.

#434 는 같은 벽을 «알려준 높이 아래 소프트키 24줄» 로 읽고, 화면 픽셀을 24줄 더 잡아 착지했다(0402). 위 fill-rect 는 **위쪽 24줄을 건너뛰고** 그 아래에 «알려준 높이» 만큼 그린다. 이 관찰은 «높이를 320 − 24 = 296 으로 알려준다(어넌시에이터 아래 영역)» 쪽 해석도 지지한다. 이 회차는 그 처방도 만들어 쟀다(미커밋 · 아래). 그러나 #434 가 먼저 착지했으므로 **처방을 둘 다 싣지 않았다**.

| 296 처방의 짝 재측(전 `c387adef` · 60초 · `--inject --pacing 8 --relaunch 1`) | 결과 |
|---|---|
| 어넌시에이터 켠 LGT 14종(`clet_register(_, 1)` — LGT 78종 중) | PASS → FAIL **0** |
| `1eaa92092bee` | FAIL(9·13번째 키 Java 예외) → **PASS** 27/27 (#434 도 고친 것) |
| `acc4215b7ec0` | FAIL(1번째 키 Java 예외) → **PASS** 27/27 — **#434 의 표에 없다** |
| `fe76e641bb3d` | FAIL 2번째 키 → FAIL 20·22번째 키 · `Unknown LGT WIPIC SVC id 603` |

★`acc4215b7ec0` 가 #434 착지 뒤에도 그대로인지는 이 회차가 재지 않았다. 다음 회차가 «24줄 여유» 와 «296» 중 어느 쪽이 화면을 맞게 그리는지 정할 때의 입력으로 남긴다.

### 4. 커밋한 계측 — 할당 실패 때 힙 요약

`Allocator::alloc` 이 `AllocationFailure` 면 `tracing::error!` 로 한 줄을 남긴다. 「guest heap allocation of N bytes failed; list used U/T in B blocks, largest free run F; buckets 4:u/c … 512:u/c」. list 헤더가 남은 구간보다 크면 `CORRUPT header … at …` 으로 적는다. §1 의 두 판정이 이 한 줄로 갈렸다. 읽기만 하고, 실패 때만 돈다.

### 5. 짝 재측 — destroy 처방의 범위

전 = `origin/main` `c387adef` · 후 = 이 브랜치(LGT 296 처방 제외) · release · 60초 · 전·후 동시 2프로세스.
- 범위: 429종을 12초씩 돌려 `MC_grpDestroyOffScreenFrameBuffer` 를 부른 타이틀은 **5종**이다(KTF 3 · LGT 2). 12초 안에 부르지 않는 타이틀은 이 목록에 없다 — 한계.
- 시작 시 `host-load-guard --status --recovered` rc=0.

| sha12 | 전 | 후 |
|---|---|---|
| `0606d43702e7` LGT | PASS 27/27 | PASS 27/27 |
| `7da00ecd4804` KTF | PASS 27/27 | PASS 27/27 |
| `7e2247bdf565` KTF | FAIL · `render: magenta color-key not applied` | 같음 |
| `b7699c10dfd1` LGT | PASS 27/27 | PASS 27/27 |
| `d9384b388ea5` KTF | PASS 27/27 | PASS 27/27 |

- `7e2247bdf565` 의 60초 render FAIL 은 전 빌드에서도 5판 중 4판이 같다. 이 처방과 무관한 기존 판정이다. 30초 census 프로브 A 는 PASS 였다.
- `7da00ecd4804` 는 create 0 · destroy 1 이다. 오프스크린으로 만들지 않은 핸들을 지운다. 이제 그 핸들의 `buf` 도 반납하는데, 짝 재측은 PASS 3/3(r1·r2·r4)이다. 그 핸들이 무엇이었는지는 재지 않았다.
- 296 처방을 포함한 첫 두 묶음(r1 은 시작 시 guard `holding` · r2 는 guard rc=0)에서도 이 5종은 같았다.

### 6. 지원 현황 · 병합 뒤 재확인

`playability-census.mjs run --jobs 2`(후 빌드) · 두 대상만 담은 스크래치 corpus.
- 시작 시 guard 는 **`holding`(rc=1 · 다른 레인 부하)** 이었다.
- 장시간 축은 «오류 없이 버텼나» 다. 부하는 진행을 늦출 뿐이므로, §2 의 **횟수 기준** 재측으로 보강했다.

| sha12 | A(30초) | B | L(600초) | 이 PR 의 행 |
|---|---|---|---|---|
| `7e2247bdf565` | PASS 27/27 | PASS | 854/900 키 · deadline · 예외 0 | **limited → playable** · longplay no → ok · 알려진 문제 삭제 |
| `fe76e641bb3d` | FAIL · 20번째 키 · SVC 603 | PASS | 돌지 않음 | 그대로(#434 소관) |

병합(`43ccfe63`) 뒤 재확인(guard rc=0): `7e2247bdf565` 600초 create 233,442 · 할당 실패 0 · panic 0(§2 표). `fe76e641bb3d` 60초 FAIL · 20번째 키 · SVC 603 — #434 단독과 같다.

### 7. 되돌리면 red

`destroy_offscreen_framebuffer` 에서 `context.free(record.buf)` 를 빼면 `destroyed_offscreen_framebuffer_returns_its_pixels` 가 red 다(1 failed). 이 시험은 같은 크기를 다시 만들면 픽셀 주소가 같은지 본다.

### 8. 게이트

- `cargo fmt --all -- --check` rc=0.
- `cargo clippy --all -- -D warnings` rc=0 · beta rc=0 · wasm32 rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0(병합 뒤 `43ccfe63` · 51 묶음 · 634 통과 · 실패 0).
- 이 worktree 의 target 을 썼고 build-slot 을 거쳤다.
- 러너 블록(후 release): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me`(`--timeout 5`) PASS.
- `keydraw_ktf`·`keydraw_lgt` `--inject --expect-last-frame` 은 기본 `--max-ticks` 에서 UNMEASURED(`stop: max-ticks`)였다 — **전 빌드도 같다**(0391·0392·0393 과 같은 기존 현상). `--max-ticks 2000000000` 으로 전·후 모두 PASS 27/27 · `last_frame_content true` · rc=0.
- 유입(`node scripts/corpus-name-inflow.mjs`): BOUNDED 337쌍 · SUFFIX-ATTACHED 15쌍. 전부 `compat.json` 의 기존 `title`·`fileTitle` 줄이다 — 이 PR 이 그 파일을 건드려 주제가 됐다(#434 의 0402 와 같은 수). 이 회차가 더한 줄(코드 · 이 문서 · worklog · player-updates · `compat.json` 의 `status`/`longplay`/`knownIssues_ko`)의 게임명은 0 이다. 타이틀은 sha12 로만 적었다.
