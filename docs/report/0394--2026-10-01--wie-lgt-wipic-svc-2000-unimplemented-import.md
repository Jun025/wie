## [2026-10-01] LGT WIPI-C SVC 2000 — 이미 #419 가 채웠다 · a23f3c9fc2cb 호출부 확인 · 장시간 짝 재측 (wie-lgt-wipic-svc-2000-unimplemented-import)

**무엇을**
- 엔진 동작 변경 0. `wie-lgt/src/runtime/svc_ids.rs` 의 `SocketAlt = 0x7d0` 주석만 고쳤다 — 「그 칸에 닿는 8종은 모두 602·2000 둘 다 import 한다」가 이 타이틀로 **반증**돼서다.
- a23f3c9fc2cb 의 2000 호출부 근거 · 장시간(600초) 짝 재측 3회를 여기 남긴다.

**왜**: 전 회차(`wie-census-wave3-singleton-tail-boot-and-longplay-fix`, PR #420)의 장시간 1/3 에서 `tick error during '22_NUM5': Unknown LGT WIPIC SVC id 2000` 이 났고 「2000 행 없음」으로 발권됐다.

**사용자 영향**: 없음(이 PR 단독으로는). 이 타이틀의 부팅 수정은 #420 이 지고 있고 아직 열려 있다.

### 1. 원인 — 측정 빌드가 #419 이전이었다

- 그 오류 문구에 `(r0=… lr=…)` 꼬리가 **없다** — 꼬리는 #419(`449e470e`, 2026-09-30 18:13 착지)가 붙였다. 측정 빌드 `6c65e586`(#420 head)은 `449e470e` 를 조상으로 갖지 **않는다**(`git merge-base --is-ancestor` 실패).
- `origin/main` 에는 `0x7d0 => Self::SocketAlt` → 공유 `net::socket`(-1 «네트워크 없음»)이 있고 단위 시험 `wipic_svc_ids_named_from_call_sites_answer` 가 2000 을 단언한다.
- ⇒ 「미구현」은 #420 브랜치 기준의 사실이었고 main 기준으로는 이미 닫혀 있었다. #420 이 main 에 합쳐지면 자동으로 해소된다(아래 §3 은 그 합친 트리로 쟀다).

### 2. 근거 — a23f3c9fc2cb 의 2000 호출부

`binary.mod`(ELF · `.text` 0x1000 · `.data` 0x1400000) import 썽크는 #419 가 적은 모양(`push {lr}; bl <resolver>; .word 0x1fb; .word <id>`) 그대로 52개이고, id 2000 썽크는 `.data+0x254`(va `0x1400254`)다. 리터럴로 그 주소를 싣는 곳은 한 곳:

```
0x3ebba ldr r4, =0x15081c4   ; 상태 바이트
0x3ebbe ldrb r3, [r4] ; cmp r3, #2 ; bhi out
0x3ebc4 ldr r3, =0x1400254   ; -> import 2000
0x3ebc6 movs r0, #2 ; movs r1, #1 ; bl <thunk-call>      ; 2000(2, 1)
0x3ebce str r0, [=0x1503ba0] ; cmp r0, #0 ; blt 0x3ebe8
        (성공: 상태 3 · import 125 로 시각 기록)
0x3ebe8 r0+14==0? r0+7==0? r0+99<0? → out
0x3ebf8 adds r0, #1 ; bne …  ; -1 → 경과 시간 비교 → 상태 코드 0x55/0x59 (연결 실패 화면)
```

- `(2, 1)` = AF_INET·SOCK_STREAM, `< 0` 으로 가름 — #419 가 3ff5948e235e·863b8ab6a21d 에서 읽은 모양과 같다.
- ★이 타이틀은 2000 을 import 하지만 **602 는 import 하지 않는다**(import 집합: 600·601·603–606·901·904·2000 …). 그래서 `SocketAlt` 주석의 「8종 모두 둘 다」를 고쳤다.
- -1 은 게임 자신의 «소켓 실패» 분기로 간다(-14·-7·-99 어디에도 안 걸리고 `-1+1 == 0`). 그 경로에서 다른 import 를 부르지 않는다(125 는 이미 표에 있다).
- 이 타이틀 import 중 표에 없는 것: 293 · 603 · 1300. 603(KTF net 3 `MC_netSocketConnect` 자리)은 소켓이 -1 이면 이 호출부에서 닿지 않는다. 셋 다 이 회차 범위 밖(다른 SVC 변경 0).

### 3. 장시간 짝 재측 — 조용한 호스트

- 전 = `6c65e586`(#420 head, #419 없음) · 후 = `origin/main`(`22f4c044`) + `6c65e586` 로컬 병합(push 안 함). 둘 다 release `wie_validate`, 자기 target.
- `build-slot run -- node scripts/playability-census.mjs run --bin <전|후> --out … --jobs 1 <1종>` · 회차마다 `host-load-guard --status --recovered` rc=0(03:28 load 9.91 · 03:49 7.12 · 04:11 9.20). census 잠금이 호스트당 1런이라 전·후는 교대로 돌았다.

| 회차 | 빌드 | 장시간(L) | 판정(report) | load1 |
|---|---|---|---|---|
| 1 | 전 | 600초 · 854/900키 · 오류 0 | playable · longplay ok | 7.3 |
| 1 | 후 | 600초 · 854/900키 · 오류 0 | playable · longplay ok | 8.1 |
| 2 | 전 | 600초 · 854/900키 · 오류 0 | — | 8.0 |
| 2 | 후 | 600초 · 854/900키 · 오류 0 | playable · longplay ok | 8.7 |
| 3 | 전 | 600초 · 854/900키 · 오류 0 | — | 6.5 |
| 3 | 후 | 600초 · 854/900키 · 오류 0 | playable · longplay ok | 8.5 |

- 후 3/3 오류 0. ★**전도 3/3 오류 0** — 조용한 호스트에서는 이 짝이 두 빌드를 가르지 못한다. 전 회차의 오류는 `--jobs 3` · load5 48 도중 22번째 키에서 났다(키 타이밍으로 상태 바이트 ≤2 구간에 닿는 경로). 게임 층 red 는 이번에 재현하지 못했다고 적는다.
- 가르는 red 는 단위 시험이다(§4).
- `docs/player-data/compat.json` 은 **고치지 않았다**. main 의 이 행은 `not-yet`(부팅 실패)이고, 부팅은 #420 몫이다 — main 단독으로 `playable` 을 적으면 거짓이 된다. #420 착지 뒤 compat 재생성 때 위 판정이 들어간다.

### 4. 되돌리면 red

`0x7d0 => Self::SocketAlt` 행 삭제 → `cargo test -p wie-lgt wipic_svc_ids_named` FAILED:
`Unknown LGT WIPIC SVC id 2000 (r0=0x2 r1=0x1 r2=0x0 r3=0x0 lr=0x7f000000)`. 복원 후 통과.

### 5. 게이트

스크래치 target · `build-slot` 경유.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0(51 묶음 · 실패 0)

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 4회/4쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 4회는 전부 `svc_ids.rs` 의 **기존 줄**이다 — 이 회차가 더한 줄(`git diff origin/main -- wie-lgt` 의 `+` 줄)의 한글은 0자이고, 타이틀은 sha12 로만 적었다.



<!-- corpus-name-inflow v1 subjects=2 tree=0794011efc23e73f B=4/4 P=0/0 S=0/0 -->
