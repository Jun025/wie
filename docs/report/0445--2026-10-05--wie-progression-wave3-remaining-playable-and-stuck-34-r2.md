## [2026-10-05] 진도 3차 (2회차) — 250종 중 57종 측정 · KTF 예외가 호출자 프레임의 catch 를 못 찾던 벽 · 서버 인증서 벽 1종 (wie-progression-wave3-remaining-playable-and-stuck-34-r2)

1회차(`docs/report/0442` · PR #475)의 이어하기다. 남은 몫은 넷이었다. ⑴안 잰 playable 250 + 1 진도 측정 ⑵고친 2종 compat 갱신 ⑶레시피 미완 7 · ⒞ 1 확정 ⑷엔진 벽 수정.
**부분 완료다.** ⑴은 9묶음 중 1묶음(31종)을 끝까지 쟀다. 둘째 묶음은 P 26종까지 재고 회차 시간에 끊었다(§1). ⑵는 #475 가 아직 열려 있어 미뤘다(§2).

### 1. 측정 — `chunk.aa` 31종(250 의 첫 묶음 + `9789fec50f39`) · `chunk.ab` 26종

조건: 핀 `9b4eb62e`(1회차와 같은 사본) · 정책 v2 600초 + 재기동 120초 · 스윕 «전체»를 `build-slot run --long` 임대 1개로 감쌌다(안쪽 실행은 맨 명령) · census 호스트 잠금.

★**측정 엔진이 둘이다**(반려 F2 · §9). ok 41행 = `9b4eb62e`(#470 시점 main · #476·#477·이 PR 의 엔진 변경 이전). stuck 8행 = 이 PR head 빌드 `33b38ac2`(#475 -fix 를 merge 한 뒤 · main `d4a0f330` 포함)로 다시 쟀다 — 8행 값이 그대로다. 고친 2종 = 이 PR 빌드(§4-1).

| 단계 | 시각 | jobs | load1 (시작 → 끝) | idle (시작 → 끝) |
|---|---|---|---|---|
| P `chunk.aa` | 07:51–09:51 | 3 | 32.9 → 9.1 | 0% → 42% |
| P2 짝(11종) | 09:51–10:29 | 3 | 8.9 → 13.1 | 32% → 36% |
| P `chunk.ab`(끊음) | 10:45–13:22 | 2 | 8.8 → 9.8 | 11% → 40% |

- 타이틀별 `load1`(census 열): `aa` 최소 8 · 중앙 15 · 최대 164 / `ab` 최소 9 · 중앙 13 · 최대 156. 60 을 넘은 타이틀은 `aa` 3종이 전부 ok, `ab` 4종 중 3종이 ok 다. 나머지 1종(`c181cae84146`, load1 139)은 P stuck 이고 짝을 못 쟀다 — 부하 속 stuck 이라 짝이 필요한 바로 그 경우다.
- 06:29–07:51 은 측정이 없다. 호스트 부하 가드가 `recovering` 에서 풀리지 않았고 long 슬롯도 다른 스윕이 쥐고 있었다. 드라이버를 «15분 기다려도 안 풀리면 `--jobs 2`»로 바꿔 다시 띄웠다(CLAUDE.md «측정 스윕» ⒞).

| 묶음 | 모집단 | 잰 수 | ok | stuck | error | n/a | 미측정 사유 |
|---|---|---|---|---|---|---|---|
| `chunk.aa` | 31 | 31 | 23 | 5 | 3 | 0 | — |
| `chunk.ab` | 30 | 26 | 18 | 0 | 2 | 6 | 6 = P stuck 인데 P2 짝을 못 쟀다(끊음). 미측정 4 = 3종(`e9fac881e602` `eda7ecb85ae3` `f981d228b757`)은 끊음, 1종(`6eb93824daf8`)은 이 핀에서 longplay error 라 후보가 아니다 |
| `chunk.ac`…`ai` | 190 | 0 | — | — | — | — | 회차 시간 |
| **합** | 251 | 57 | 41 | 5 | 5 | 6 | |

- stuck 은 P2 짝이 같을 때만 stuck 이다(계약). P 가 stuck·error 이던 11종 중 3종(`5891a0c5d595` `de00506611a5` `74bfbf3dd0ab`)은 P2 가 움직여 ok 다. 나머지 8종은 P2 도 같았다(stuck 5 · error 3).

### 2. 고친 2종(`e68b1c8aef85` · `320a5360a0f3`) compat 갱신 — 미뤘다

티켓 경계 그대로다. #475 가 아직 OPEN 이다(MERGEABLE). 미착지 브랜치 빌드로 공개 데이터를 갱신하지 않았다. #475 머지 뒤 `~/scratch/w7prog/drive2.sh` 의 F·F2 단계를 main 빌드로 돌린다.

### 3. 막힘 계급

| 계급 | 수 | sha12 | 근거 |
|---|---|---|---|
| ⒝ 엔진 벽 — **고침** | 2 | `0e6cd188729e` `070daa5b552c` | §4 |
| ⒝ 엔진 벽 — 남김 | 1 | `dbd078113b97` | 시작 직후 키에서 `Network.connect()` 가 -1 → NPE → `jump native address is null (lr 0x1345bd)`. 같은 키 순서에서 매번 재현된다. 연결 실패를 게임이 받는 모양(반환값·예외)을 실기 기준으로 모른다 |
| ⒞ 통신 대기 | 1 | `3185174d2121` | 정책 키가 타이틀에서 NUM5 로 «랭킹 등록 · 통화료가 부과됩니다 · 1.예 2.아니오»를 열고 NUM1(예)을 고른다. 그 뒤 보라·초록 대기 화면이 590초 고정이다. OK 로 들어가면 이야기·지도·대화가 정상으로 흐른다(1초 간격 프레임으로 확인) |
| ⒜ 레시피 필요(메뉴·정보창) | 3 | `44c292be4e4c` `cb7c7f87f9e6` `135d1291501f` | 지도 창 · 재료 주문표 · 일시정지에 머문다 |
| ⒜ 정책 한계(이름 입력) | 1 | `33f3e7669599` | «STEP 1. 별명» 글자 입력 화면. 정책 키에는 글자 입력이 없다 |
| ⒝ 렌더 벽 — 남김(`ab`) | 2 | `46b2238f87a6` `7e2247bdf565` | 진도가 아니라 `wie_validate` 의 렌더 검사 FAIL «magenta color-key not applied»(화면의 17% · 62%). KTF 마젠타 투명색이 안 빠진 그림 |

### 4. ⒝ 수정 — KTF 예외가 호출자 프레임의 catch 를 못 찾았다

**증상.** 두 KTF 타이틀이 같은 정책 키 지점(`esc03_01_CLR`, 311초)에서 매번 `Invalid memory access` 로 죽었다. 위치는 `h.paint` 안이다.
디버그 로그로 보면 그 지점까지 같은 `java_throw` 가 235번 났고, 234번은 같은 프레임에서 잡혔다. 마지막 한 번은 현재 프레임의 catch 표에 맞는 행이 없었다. 예외가 paint 밖으로 나갔고(뒤이어 호스트의 `Graphics::reset`), 다음 게스트 실행에서 죽었다.

**원인 둘.**
- ⑴ `JavaMethod::handle_exception` 은 «현재» 핸들러 레코드의 표만 봤다. KTF 핸들러 레코드는 `ptr_old_handler` 로 사슬을 이룬다(`wipi_types::ktf::java::JavaExceptionHandler`). 호출자 프레임의 catch 를 찾으려면 사슬을 걸어야 한다. `docs/report/0343` 이 이 공백을 «범위 밖»으로 적어 두었다.
- ⑵ 아무 프레임도 못 잡은 예외는 호스트 오류로 JVM 호출 밖까지 나간다. 이때 건너뛴 프레임들은 자기 레코드를 해제하지 못한다. 그래서 스레드의 «현재 핸들러»가 이미 죽은 스택을 가리킨 채 남았다.

**⑴만 고치면 안 된다 — 실측.** 사슬 걷기만 넣은 빌드는 두 타이틀 다 20초 안에 멈췄다(CPU 97%, 무한 루프). 죽은 레코드가 서로를 가리켜 사슬이 고리가 됐다.

**고친 것.**
- `handle_exception` 이 사슬을 걷는다. 잡는 레코드를 «현재»로 되돌린다. 걸음 수 상한은 4096 이다(상한을 넘는 사슬은 이미 깨진 것이다).
- JVM→게스트 호출(`impl Method for JavaMethod::run`)마다 사슬을 비우고 시작해서, 나갈 때 호출자의 것을 되돌린다. 되돌리는 대상은 들어올 때의 스레드 컨텍스트다(await 사이에 스레드가 바뀌어도 맞는 곳에 쓴다).
- 이 경계 덕분에 걷기가 JVM 호출 너머로 넘어가지 않는다. 넘어가면 원래 `"Java exception unwind crossed into JVM caller"` 오류였다.

| sha12 | 전 | 후 | 되돌리면 red |
|---|---|---|---|
| `0e6cd188729e` | 311초 FAIL `Invalid memory access` · 600초 판정 error | 420초 재현 키로 FAIL 없음 · 상태정보·단축번호 화면까지 진행(재현 판정 ok · stall 100) | `test_throw_walks_to_the_caller_frames_catch` FAILED |
| `070daa5b552c` | 같은 지점 FAIL · 600초 판정 error | 420초 재현 키로 FAIL 없음 | 같은 시험 |

- 600초 정책 판정(고친 빌드): §4-1.
- 퇴행 확인. 예외가 잦은 KTF 8종을 60초 기본 주입으로 옛/새 빌드에서 짝으로 쟀다(`e9fac881e602`(0343 가드) `aa3fcba4598b` `44a4228cca51` `3d38a46becd9` `ea35907b22a4` `9789fec50f39` `96dc32e781d3` `52f1f32e3f72`).
  - result · content · frozen · throw 수가 같다. paints 차는 ±3 이다.
  - `52f1f32e3f72` 은 한 번 «clean exit» 가 났다. 재측 4회 중 옛 빌드에서도 1회 같은 결과가 나와 퇴행이 아니다(새 2/2 PASS · 옛 1/2 PASS).

### 4-1. 고친 2종 600초 정책 판정

census 와 같은 키·탈출 규칙으로 쟀다(고친 빌드 · 재기동 꼬리 없음 · 판정은 census 곡선 그대로).

| sha12 | 시각 | load1 · idle | 판정 | result | 새 화면 시각(초) |
|---|---|---|---|---|---|
| `0e6cd188729e` | 11:29 | 12.9 · 12% | **ok**(stall 30) | PASS | 10 30 120 130 320 570 |
| `070daa5b552c` | 11:39 | 9.3 · 34% | **ok**(stall 30) | PASS | 10 30 120 130 320 570 |

- 두 파일은 같은 게임의 다른 판이다(새 화면 시각이 같다).
- 이 판정은 이 PR 의 고친 빌드로 쟀다. 고침과 데이터가 같은 PR 로 함께 착지하므로 `compat.json` 에 `ok` 로 실었다(§6).

### 5. 레시피 미완 7종 · ⒞ 1종

레시피는 키 이름만 적은 비커밋 파일이다(`game_lab/recipes-progress/<sha12>.keys`). 판정은 census 와 같은 진도 곡선(새 화면 16×16 지문 · 마지막 1/3)으로 600초를 쟀다.

| sha12 | 결과 | 무엇이 막았나 / 레시피 |
|---|---|---|
| `2d66945008c1` | **ok**(stall 20) | 상점 판매창에 머물렀다 → 걷기에 CLR 을 섞었다 |
| `f5bd7a91a107` | **ok**(stall 170) | 돈이 떨어지면 상점 «현금 부족»에서 돈다 → 상점에 안 들어가고 목장에서 시간을 보낸다 |
| `c6cadf75c454` | **ok**(stall 180) | 마을 한 블록을 맴돌았다 → 오른쪽·아래로 쏠린 걷기(3초 누름) |
| `ea35907b22a4` | **ok**(stall 60) | 메인 메뉴 커서가 «이어하기»(빈 슬롯)에 있다 → UP 으로 «새로시작». 전투에서 OK 가 레벨 부족 기술을 고른다 → 방향·NUM5·CLR |
| `c3057f46c59b` | 게임 진행 확인 · 판정 stuck(210) | «무한모드»로 미션 5개를 깼다(화면으로 확인). 시간 초과 뒤 메뉴에서 판정 창이 끝났다 |
| `0266ca417880` | 미완 | 1회차에 마을까지 갔다. 이번 2판은 상태창에 머물렀다. 셋째 판은 load1 135 에서 메뉴 커서가 밀려 «게임종료»를 8번 골랐다 |
| `55c453b46254` | 미완 · ⒝ 후보 | 게임·캐릭터 선택에서 **0.15초 탭은 무시되고 0.6초 누름만 받는다**. 캐릭터 확정 키는 OK·NUM5·소프트키·숫자·# · * 를 0.6초씩 눌러도 못 찾았다 |

**`b9bfcaf42722` ⒞ 확정.** 첫 실행에 «인증서가 없습니다 … 서버에 접속한 뒤 인증서를 받아야 합니다»를 묻는다. «예»마다 `MC_netConnect` 가 1번 불리고(stub), 바로 «인증오류(오류번호 4000)»가 뜬 뒤 같은 질문으로 돈다(P stderr 접속 11회 = 답 11회). «아니오»는 이용안내 → 타이틀로 돌아간다.
- 전수 도구의 통신망 벽 술어는 프로브 A 접속 200회 이상이다. 이 타이틀은 2회라 그 술어로는 안 잡힌다.
- 그래서 `playability-census.mjs` 에 `NET_HAND`(sha12 → 그 벽의 문장)를 두었다. 같은 처리(`limited` + 안내 한 줄)를 받는다. 다음 재생성에서도 남는다.
- `compat.json` 그 행: `playable → limited` · `knownIssues_ko` 1줄.

### 6. compat 갱신

- `axes.progress`: `chunk.aa` 31행 중 고친 2종을 뺀 29행. 계약 어휘대로 `error → stuck` 이다. `(없음) → ok` 22 · `(없음) → stuck` 6 · `stuck → ok` 1(`9789fec50f39`). ok 23 은 `9b4eb62e` 측정, stuck 6 은 §9 에서 이 PR head 빌드로 다시 쟀다.
- `chunk.ab` 20행: `(없음) → ok` 18 · `(없음) → stuck` 2(렌더 FAIL · 계약상 «실행 FAIL 도 stuck»). P2 짝이 없는 6종은 `n/a` 라 싣지 않았다. stuck 2 도 §9 에서 다시 쟀다(같은 렌더 FAIL · 17% · 62%).
- 고친 2종(`0e6cd188729e` `070daa5b552c`): `ok`(§4-1). 이 PR 이 고친 빌드로 쟀다.
- `b9bfcaf42722`: §5.
- 6차 판(`enginePin` `4ac38566`) 위의 행 단위 수정이다. 1회차와 같은 방식이다. `player-data.mjs` OK.
- ★§2(할 일 2)와 어긋나지 않는다: 미룬 것은 #475 가 고친 2종의 행이고, 이 절의 progress 행은 그 2종을 건드리지 않는다. 다만 ok 41 은 옛 엔진(`9b4eb62e`) 값이다 — 그 뒤 엔진 변경(#476·#477·이 PR)이 그 타이틀을 퇴행시켰다면 이 값은 낡은 `ok` 다. 이 회차는 그 41종을 다시 재지 않았다(§9).

### 7. 다음 회차

- `~/scratch/w7prog/drive3.sh` 를 다시 돌린다. `chunk.ab`(남은 3)…`ai` 다. 같은 `--out` 이라 잰 것은 건너뛴다. `build-slot run --long -- bash drive3.sh` 하나로 감싼다.
- `chunk.ab` 의 P2 짝 6종은 드라이버가 `ab` 의 P 를 끝낸 뒤 스스로 뽑는다(`p2list.mjs`).
- #475 머지 뒤 `drive2.sh` F·F2(main 빌드) → `compat.json` 두 행.
- `dbd078113b97`: `Network.connect()` 실패를 게임이 받는 모양.
- `3185174d2121`: 랭킹 등록 «예» 뒤 대기 화면이 끝나지 않는다. 연결 실패가 게임에 알려지지 않는 모양이다(`dbd078113b97` 과 같은 통신 축).
- `55c453b46254`: 짧은 탭 유실. 아래 제안.

### 8. 게임 파일명 유입

- `corpus-name-inflow`(origin/main 대비 · #475 커밋 포함): BOUNDED 342 + SUFFIX-ATTACHED 15.
- 342 는 `compat.json` 의 기존 제목, #475 가 고친 `wie-lgt` 주석의 기존 언급, `jvm_support.rs` 의 기존 줄(main 에 이미 있다)이다.
- 이 PR 이 더한 줄만(`875f3a71..HEAD`, `compat.json` 제외) 코퍼스 파일 이름과 대조하면 0 이다.


### 9. 반려 승계(`-r2-fix`) — compat 되돌림 · 측정 엔진 · 경계 시험

**F1 — compat 되돌림.** 이 브랜치는 #475 의 옛 pin `875f3a71` 위에 있어 #473·#476 이 고친 23행을 옛 값으로 되돌리고 있었다. #475 -fix head `55b01999`(main `d4a0f330` 포함)를 merge 했다. `compat.json` 충돌은 «`55b01999` 판 + 이 PR 이 `875f3a71 → ddbd8e84` 에서 바꾼 필드만» 으로 풀었다(progress 51행 · `b9bfcaf42722` 의 status·knownIssues_ko). `origin/main` 대비 `sha256+platform` 행 단위 비교:

| 항목 | 수 |
|---|---|
| 바뀐 행 | 66 |
| = 이 PR(progress 51 + `b9bfcaf42722` 1) | 52 |
| + #475 -fix(progress 14) | 14 |
| 겹침 · 설명 안 되는 행 · 빠진 행 | 0 · 0 · 0 |
| progress 밖 필드가 바뀐 행 | 1(`b9bfcaf42722` · 의도) |
| top 필드(`schema`·`generatedAt`·`enginePin`) 차이 | 0 |

검수가 겹침 후보로 든 `5028b8a5d19f`·`1793f87924d4`·`d4188f8ef8c4` 는 #475 -fix 행이고 `e085e193211d` 는 이 PR 행이다. 둘이 같은 행을 건드리지 않는다. `e085e193211d` 는 progress 만 이 PR 값이고 `sound`·`knownIssues_ko` 는 main 값이다.

**F2 — ⒜ 를 골랐다.** stuck 8행을 이 PR head 빌드(`33b38ac2` · release `wie_validate`)로 다시 쟀다. 같은 정책(v2 600초 + 재기동 120초 · P 후 `p2list.mjs` 가 고른 P2 짝) · `build-slot run --long` 임대 1개 · `--jobs 3` · 시작 전 `host-load-guard --recovered` rc=0 · 14:32–15:44 · load1 7.6(시작) · 10.5(14:57). 후보 판정용 A·B·L 프로브는 옛 out 에서 가져왔다(그 축은 이 회차가 바꾸지 않는다).

| sha12 | P (stall · 화면 수) | P2 | 공개 값 | 이전 값 |
|---|---|---|---|---|
| `135d1291501f` | 530 · 7 | 같음 | stuck | stuck |
| `3185174d2121` | 590 · 1 | 같음 | stuck | stuck |
| `33f3e7669599` | 300 · 6 | 같음 | stuck | stuck |
| `44c292be4e4c` | 260 · 6 | 같음 | stuck | stuck |
| `cb7c7f87f9e6` | 400 · 10 | 같음 | stuck | stuck |
| `dbd078113b97` | 590 · 1 | 같음 | stuck | stuck |
| `46b2238f87a6` | 렌더 FAIL 17% | 같음 | stuck | stuck |
| `7e2247bdf565` | 렌더 FAIL 62% | 짝 없음(stall 70 · `p2list` 기준 밖 · 1회차도 같다) | stuck | stuck |

값이 하나도 바뀌지 않아 `compat.json` 에 더 고칠 것이 없다. ok 41행은 다시 재지 않았다(§6 마지막 줄).

**m1 — 경계 시험.** `test_jvm_call_runs_in_its_own_exception_scope`: 시험 클래스의 정적 host 메서드를 `<JavaMethod as jvm::Method>::run` 으로 부른다. 몸체는 그 순간의 handler 를 돌려준다. 호출자 chain 을 `0x1234` 로 두고 부르면 몸체는 `0` 을 보고, 돌아온 뒤 `0x1234` 가 복원된다. `method.rs` 의 `enter/leave_exception_scope` 두 줄을 지우면 **FAILED**(`the body ran under the caller's chain`)다. 되돌리면 PASS 다.

**연번.** `0443` 은 #476(`wie-compat-apply-guest-thread-death-rejudge-and-five-walls`)이 먼저 착지했다. 이 파일을 `0445` 로 옮기고 이 PR 이 더한 참조 5곳을 고쳤다.

**게이트.** fmt OK · clippy `--all` / wasm / `+beta` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **701 passed / 0 failed**. 첫 두 번은 rustc ICE(`failed to open LTO bitcode file … pre-lto.bc`)로 빌드가 죽었다. 증분 캐시 파일이 빌드 도중 사라진 것이라 `CARGO_INCREMENTAL=0` 으로 다시 돌렸다. `player-data.mjs` OK.

<!-- corpus-name-inflow v1 subjects=14 tree=a8af1df7a1d9aa90 B=735/342 P=0/0 S=35/15 -->
