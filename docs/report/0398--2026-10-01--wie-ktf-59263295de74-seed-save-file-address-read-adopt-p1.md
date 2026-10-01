## [2026-10-01] `59263295de74` — 패키지 저장 파일이 원 소유자 단말의 힙 포인터를 담고 있다 · 그 jar 에서만 마운트하지 않는다 (wie-ktf-59263295de74-seed-save-file-address-read-adopt-p1)

**무엇을**: KTF 로더에 «단말에 묶인 저장 파일» 목록(jar MD5 키 · `DEVICE_BOUND_SAVES`)을 두고, 목록에 든 jar 의 `res/save.sav`·`res/savem.sav` 를 마운트하지 않는다. 목록은 이 타이틀 1건이다. `compat.json` 의 이 행을 `limited · longplay no` → `playable · longplay ok` 로 바꿨다.
**왜**: 0391 §4 가 남긴 벽 — 키 4 에서 패키지 저장 파일을 읽은 뒤 920 바이트를 `0x1d14250` 으로 읽다 폴트.
**사용자 영향**: 이 게임이 처음 설치한 단말처럼 새 게임으로 시작하고, 끝까지 간다. 지원 현황 playable 367 → 368 · limited 42 → 41. 다른 타이틀은 바뀌지 않는다(§3).

증적: `~/orchestrator/reports/evidence/wie-ktf-59263295de74-seed-save-file-address-read-adopt-p1/`. sha12 만 쓴다.

### 1. 재현 · 호출부

`wie_validate`(release · origin/main `1becaae2`) 프로브 A(`--inject --pacing 8 --timeout 30`) 2/2 FAIL · 키 4 · `Invalid memory access; address: 30491216`(= `0x1d14250`).
`RUST_LOG=wie_core_arm::function=trace,wie_wipi_c::api::database=debug` 로 스트림 읽기마다 LR 을 찍었다(증적 `stream-read-lr.txt`):

| LR | `db.stream_read(handle, 목적지, 길이)` |
|---|---|
| `0x1190e7` | `0x14a990`, 3 |
| `0x1190f7` | `0x14a993`, 9 |
| `0x11910b` | `0x1470a0`, 472 |
| `0x119123` | `0x1451c0`, 192 |
| `0x119133` | **`0x1d14250`**, 920 |

호출부 디스어셈블(`scripts/ktf-image-sweep.py window … 0x119134` · 증적 `window-0x119134.txt`):

```
0x11910a  ldr r3,[pc,#..]   ; sl+0x24c
0x119112  ldr r4,[r3]       ; r4 = 0x1451c0
0x11911c  movs r2,#0xc0     ; 192
0x11911e  bl  stream_read   ; (fd, r4, 192)
0x119124  movs r2,#0xe6     ; (0x119128 lsls r2,#2 → 920)
0x119126  ldr r1,[r4,#0x30] ; ★방금 읽은 192 바이트 안의 +0x30
0x11912e  bl  stream_read   ; (fd, *(r4+0x30), 920)
```

⇒ **목적지 주소는 저장 데이터에서 온 값이다(확정).** 파일 위치는 3+9+472 = 484, +0x30 = **0x214** 이고 `save.sav` 의 그 자리가 `50 42 d1 01` = `0x01d14250`(LE)이다(증적 `save-sav-0x200.txt`).

### 2. 이 저장 파일은 원 소유자 단말의 것이다

| 근거 | 실측 |
|---|---|
| 게임이 만든 파일이다 | jar(568,416 B)에 `save.sav` 가 없다. 패키지의 `P/`(단말의 앱 전용 저장 공간)에만 있다 |
| 설치 뒤 단말에서 쓰였다 | zip 시각: jar·`__adf__` 13:25 · `P/res/save.sav`·`savem.sav` 13:26(같은 날) — jar 빌드 시각(01-22)이 아니다 |
| 그 단말의 힙 주소를 담고 있다 | `0x01d14250` 은 구조체를 통째로 쓴 포인터 필드다. 게임은 읽어 들인 그 값을 그대로 쓰기 목적지로 쓴다(§1). 그것을 쓴 단말에서는 같은 힙 배치라 맞았을 것이고, 다른 단말·에뮬레이터에서는 아무 곳이다 |
| 새 설치는 이 파일 없이 정상이다 | 두 파일을 뺀 zip: 게임이 3+9 바이트 새 저장을 **스스로 쓰고**(`stream_write` 3·9) 키 입력을 계속 받는다 |

`savem.sav`(863 B)는 30초·600초 실행 어디서도 열리지 않았지만 같은 단말에서 같은 시각에 쓰였으므로 함께 뺐다 — 새 설치 단말에는 둘 다 없다.

### 3. 왜 «씨앗 저장 파일을 쓰지 않는다» 규칙이 아니라 목록인가 · 짝 재측

KTF 266종 중 `P/` 항목을 가진 아카이브가 **66종**이다. 규칙으로 바꾸면 그 66종의 시작 상태가 전부 바뀐다. 그래서 `wie-lgt` 의 `ORPHAN_ENTRIES` 와 같은 모양(jar MD5 키 · 목록)을 택했다. KTF 266종의 jar MD5 를 전부 계산해 목록 키와 일치하는 것이 **이 타이틀 1종뿐**임을 확인했다.

짝 재측: 같은 시각 · 전 = origin/main `1becaae2` · 후 = 이 PR `5ec33182` · release · 프로브 A(`--max-ticks 100000000000`) · 2회씩 · jobs 3. 시작 시 `host-load-guard --status --recovered` rc=0(증적 `hl-start.txt`·`t0`·`t1` · load1 32~34). 대상 + 라이브 가드 `49ade89578c5` + `P/` 저장을 가진 KTF working 6종.

| sha12 | `P/` 항목 | 전 ×2 | 후 ×2 |
|---|---|---|---|
| `59263295de74` | 2 | **FAIL · 4/27 키 · 주소 `0x1d14250`** ×2 | **PASS · 27/27** ×2 · 예외 0 |
| `49ade89578c5` | 0 | PASS 27/27 ×2 | PASS 27/27 ×2 |
| `04159045a7ea` | 5 | PASS 27/27 · 337색 ×2 | PASS 27/27 · 335~337색 ×2 |
| `0e72b6bc12bb` | 64 | PASS 27/27 · 그림 9 · 예외 2 ×2 | 같다 ×2 |
| `0f9e1026724d` | 150 | PASS 27/27 · 125색 ×2 | 같다 ×2 |
| `155972cac664` | 6 | PASS 27/27 · 512색 ×2 | 같다 ×2 |
| `23919eb33365` | 2 | PASS 27/27 · 14/8색 | PASS 27/27 · 14/8색 |
| `306dcdb03842` | 1 | PASS 27/27 · 69색 ×2 | 같다 ×2 |

다른 7종은 판정·키 수·예외 수·색 수가 같다(`paints` 수는 부하로 흔들린다 — 같은 빌드 두 회차 사이 차이와 같은 크기).

### 4. 장시간 — census 3회(`playability-census.mjs run --jobs 1` · 후 빌드)

| 회차 | host-load-guard | A | L(600초) | census `report` |
|---|---|---|---|---|
| 1 | rc=0 | PASS 27/27 | 854/900 키 · deadline · 예외 0 | playable · longplay ok |
| 2 | rc=0 | 13/27 · `max-ticks`(UNMEASURED) | 854/900 키 · deadline · 예외 0 | playable · longplay ok |
| 3 | **rc=1**(시작 load 77) — 판정에 쓰지 않는다 | 20/27 · `max-ticks` | 854/900 키 · deadline · 예외 0 | playable · longplay ok |

L 의 `UNMEASURED · rc=2` 는 600초 예산 끝이지 오류가 아니다(0395 와 같은 판독). 회차 2·3 의 A 는 부하로 기본 `--max-ticks` 에 닿은 것이고 FAIL 이 아니다. `compat.json` 은 `status`·`longplay`·`knownIssues_ko` 만 옮겼다 — census 는 `speed: ok` 도 냈지만 speed 축은 이 회차가 따로 재지 않아 `unknown` 그대로 둔다.

### 5. 되돌리면 red
목록 대조를 `!=` 로 뒤집으면 `device_bound_saves_dropped_for_listed_jar_only` red(1 failed).

### 6. 게이트
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable · beta) · wasm32 clippy · 전부 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` **627/0** rc=0. 스크래치 target · build-slot 경유.
- 러너 블록(후 빌드): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me`(`--timeout 5`) PASS. `keydraw_ktf`·`keydraw_lgt` `--inject --expect-last-frame --max-ticks 2000000000` PASS 27/27 · `last_frame_content true`.
- `node scripts/player-data.mjs` OK(429 · 368/41/20).
- 유입(`node scripts/corpus-name-inflow.mjs`): BOUNDED 330쌍 · SUFFIX-ATTACHED 15쌍 — 전부 `compat.json` 파일 단위 스캔이 잡은 기존 `title`·`fileTitle` 값이다(main 과 같은 수 · 0395 와 같다). 이 회차가 더한 줄(코드 · 이 문서 · worklog · player-updates · `compat.json` 의 `status`/`longplay`/`knownIssues_ko`)의 게임명은 0.

<!-- corpus-name-inflow v1 subjects=7 tree=770ae5412ba1ef7f B=718/330 P=0/0 S=35/15 -->
