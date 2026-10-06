## [2026-10-07] 7da00ecd4804 «주소 8» = 저장 레코드 목록 순서가 프로세스마다 섞였다 — WIPI-C 레코드 목록을 오름차순으로 (wie-ktf-golf-course-loader-addr08-handle-table-zero)

**무엇을**: `MC_dbListRecords`(KTF)·`MC_dbListRecordInfo`(LGT)가 레코드 id 를 오름차순으로 넘긴다. 종전에는 호스트 저장소의 순서 그대로였다.
덤으로 #496 검수 이월(minor 1) — `Display` paint 의 중첩 붙들기 보존(`if !held`)을 지키는 시험 1개.

**왜**: 운영자가 Tower 에서 채택한 제안 `2026-10-06-census-wave8-engine-walls#p0`(0463 §5 · §9). 0463 은 «핸들 표 칸이 0» 까지 쟀고 왜 비는지는 못 쟀다.

**사용자 영향**: 사이트(브라우저)에서는 **바뀌는 것이 없다**. 브라우저 호스트의 저장소는 `BTreeMap` 이라 이미 오름차순이었다. 바뀌는 곳은 데스크톱 `wie_cli`(`read_dir` 순서)와 census 의 `wie_validate`(`HashMap` · 시드 무작위)다. 즉 census 가 이 타이틀을 «장시간 FAIL» 로 적은 것은 측정 호스트의 결함이었다. compat 은 고치지 않았다(다음 census 가 잰다 · §5).

### 1. 원인 사슬(스크래치 계측 — 커밋 0)

계측: `handle_wipic_svc` 에서 SVC 마다 전역 몇 개를 읽어 바뀔 때 1줄 · 메모리 오류 자리에서 레지스터 1줄 · 명령마다 1바이트 쓰기 감시. 패치는 회차 끝에 되돌렸다.

1. 멈춤 자리 `pc 0x13a544` `str r3,[r2]` · r2 = 8. 세 번 잡았다. **칸은 0번이 아니다** — `r4` = 0x48 · 0x60 · 0x88(칸 18 · 24 · 34). 0463 의 «표[0]» 는 고친다.
   - 그 루프(`0x13a52c`~`0x13a562`)는 종류 k = 0..N 마다 `h = T[k]`(`T = *(sl+0x1bc)`) · `*h + 8` 의 물체 좌표에서 원점을 뺀다. 개수는 `C[k]`(`*(sl+0xcf0)`).
2. `T[k]` 와 `C[k]` 는 같은 자리에서 함께 쓴다(`0x13a6c6`~`0x13a724`: `C[k]` = 파일의 16비트 · `T[k] = calloc(C[k]*12)`). 그런데 그 자리에 가는 것은 코스 파일의 레코드 id 가 **«원하는 id 목록»**(`*(sl+0x5f0)` · 개수 바이트 `**(sl+0x1b8)`)에 있을 때뿐이다(`0x13a27c`~`0x13a2bc`).
   - 같은 `0x10.fid` 를 읽은 통과 실행은 `T[k]` 할당 22개, 실패 실행은 18개다. 빠진 4개(`0x228 0x2e8 0x4f8 0x180`)의 id 가 목록에 없었다.
3. 목록은 매 코스 로드 직전 `game.cfg` 파서(`0x1395bc`)가 «코스 번호 구역»에서 채운다. 로더는 그 목록을 파일 순서로 다시 쓰고 일부를 `0x400`·`0x500` 으로 덮는다. 실패 실행에서는 파서가 **채우지 않았다** ⇒ 직전 로드가 덮어 쓴 목록으로 읽었다.
4. 파서 인자(코스 번호)를 찍었다. 통과 = `0x10`·`0x20`·`0x30`·`0x50` · 실패 = **`0x00` · `0xff`** — `game.cfg` 에 그런 구역이 없다.
5. 코스 바이트(`**(sl+0x210)` = `0x16fb40`)의 쓰기 감시: 쓰는 자리는 `0x1383ea` 하나다. 그 함수(`0x138260`)는
   - DB 를 열고(slot 0) · `MC_dbListRecords(fd, buf, 12)`(slot 7)로 id 3개를 받아 · `ids[슬롯]` 을 `SelectRecord`(slot 4)로 읽어 · 레코드 바이트 0xa 를 코스 바이트에 쓴다.
   - 즉 `ids[i]` 를 **i 번째 저장 슬롯**으로 쓴다. 실행마다 `SelectRecord` 의 id 가 2·3·5·6 으로 달랐다.
6. `wie_validate` 의 `MemDatabase::get_record_ids` 는 `HashMap` 키 순서를 돌려준다. Rust `HashMap` 은 프로세스마다 시드가 달라 **같은 키·같은 입력에서도 순서가 바뀐다**. 빈 슬롯이 앞에 오면 코스 바이트가 0/0xff 가 되고, 위 4 → 3 → 2 → 1 로 이어진다.
   - 그래서 «키 시점 의존»처럼 보였다. 실제로는 키와 무관한 프로세스 단위 동전 던지기다(§3 의 키 이동 짝에서도 같은 비율).

### 2. 계급 판정

- **엔진(호스트 계약) 결함** — `Database::get_record_ids` 는 순서를 약속하지 않는데 WIPI-C 두 함수가 그 순서를 게스트에 그대로 넘겼다. RMS `enumerateRecords` 는 이미 `sort_unstable` 했다.
- 게임 정상 동작이 아니다(레코드 id 오름차순 = 생성 순 = 슬롯 순을 전제한 코드다). 측정 흔들림도 아니다(흔들림의 원천이 엔진 쪽 `HashMap` 순서다).
- 고친 자리: `wie-wipi-c/src/api/database.rs` `list_record` · `list_record_info` 두 곳에서 정렬. 호스트 구현 5개(`wie_validate` · 데스크톱 · `wie_featurephone` · `wie-web` · `test-utils`)를 따로 고치지 않았다 — 게스트로 순서가 나가는 곳이 이 둘뿐이다(`rename` 경로는 순서와 무관).
- 쓰기를 삼키지 않았다.

### 3. 재현 설계와 재현율

- 인자: census L 과 같다(`--inject --keys <LONG_KEYS 반복> --keep-timeout --max-ticks 100000000000 --relaunch 1`). release 빌드 · 동시 ≤3 · 스윕 전체를 `build-slot run --long` 한 임대 · 배치마다 `host-load-guard --status --recovered`.
- **짝(150초 · base `33720b84` ↔ head 를 한 배치 안에 섞어서)**:

| 키 | base | head |
|---|---|---|
| census 장시간 루프 그대로 | **FAIL 7/12**(키 19·37 · 전부 주소 8) | **0/11** |
| 앞에 `OK:0.3` 을 넣어 시점 이동 | **FAIL 2/6**(키 20·38) | **0/6** |
| 합 | **9/18** | **0/17** |

  - head 1건은 실행 중 결과 파일을 지워 빼고 셌다.
- 600초(census L 길이) 짝 3+3: base 0/3 · head 0/3(키 854 · 셋 다 끝까지). base 의 150초 비율(약 1/2)이면 0/3 은 1/8 확률이다 — 이 짝은 «head 가 장시간도 산다»만 보탠다.
- 진단 빌드(동작은 base 와 같고 로그만 더함) 실행 42회 중 FAIL 14(전부 주소 8 · `pc 0x13a544`).
- 600초 L 에서 0463 이 본 «2/5» 와 이번 150초 «9/18» 은 같은 동전이다 — 실패는 코스를 두 번째로 다시 여는 키 19~38 사이에서만 났다.

### 4. 시험(되돌리면 red)

- `record_listings_are_in_ascending_id_order_test`: 레코드 16개를 넣고 두 목록이 오름차순인지 본다. `TestPlatform` 도 시드 무작위 `HashMap` 이라 정렬 없이 우연히 통과할 확률은 1/16!.
  - `list_record` 정렬 제거 → FAILED `MC_dbListRecords: [12, 4, 8, 16, 1, 11, …]`
  - `list_record_info` 정렬만 제거 → FAILED `MC_dbListRecordInfo: [4, 10, 5, 6, 13, …]`
- `a_paint_inside_a_held_key_handler_leaves_the_hold_in_place`(#496 이월): 붙들기를 쥔 채 `handlePaintEvent` → 끝난 뒤에도 쥐고 있다. `if !held` 를 지우고 항상 풀게 하면 → FAILED `the key handler's hold outlives its paint`.

### 5. KTF·LGT 표본 퇴행

- 모집단: `game_lab` 의 KTF·LGT 파일 381개(고유 332종)를 head 로 20초씩 돌려 `MC_dbListRecords(`·`MC_dbListRecordInfo(` 로그 줄을 셌다(로그는 세기만 하고 저장하지 않았다). **29종**이 20초 안에 1회 이상 부른다. 두 함수를 부르지 않는 타이틀은 이 변경으로 달라질 길이 없다(20초 뒤에 처음 부르는 타이틀은 이 스캔이 못 본다).
  - `5a59f62d1f1a` 는 20초 안에 부르지 않는다.
- 29종 × 프로브 A(census 인자 그대로: 30초 · `--pacing 8` · `--relaunch 1`) × base/head × 2회(02:56→03:16 · 동시 ≤3).
  - 판정(result/stop/입력 단계) **58쌍 전부 같다** — PASS/k27 28종 · `01f05f8231f4` 는 양쪽 FAIL «no frame rendered»(0463 §4 구매 휴대폰 잠금 그대로).
  - 그림 수 차는 ±3% 이내다(`53d8ce5a8ef0` 382/376 ↔ 369/375 가 가장 크다).
  - **퇴행 0.** 30초 프로브는 저장 슬롯을 다시 읽는 장면까지 못 가는 타이틀이 많다 — 순서가 실제로 갈리는 곳은 §3 의 장시간 루프다.

### 6. 게이트

- fmt rc=0 · clippy `-D warnings` stable/wasm32/beta rc=0(beta 는 이 회차와 무관한 `unused dependency` 경고 9줄 — cargo 경고라 거부되지 않는다).
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · **726/0**(`CARGO_INCREMENTAL=0`).
- 러너 픽스처: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS · keydraw_ktf PASS(그림 79) · keydraw_lgt PASS(그림 55) · rc=0.

### 7. 남은 것

- `5a59f62d1f1a`(같은 회사)의 장시간 «주소 0»(0463 에서 0/5 재현)은 보지 않았다. 20초 안에 목록을 부르지 않으니 같은 원인이라는 근거는 없다.
- census `wie_validate` 의 `MemDatabase` 는 그대로 `HashMap` 이다. 게스트로 나가는 순서는 이제 엔진이 정하므로 고칠 이유가 없다.
