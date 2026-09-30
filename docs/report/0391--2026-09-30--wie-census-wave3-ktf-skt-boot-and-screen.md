## [2026-09-30] 3차 군집 — KTF 널 읽기 정책 · Clet 화면 프레임버퍼 표시 · DB slot 8 · SKT 인증 · 재배치 client.bin 판정 (wie-census-wave3-ktf-skt-boot-and-screen)

**무엇을**: 2차 전수(`docs/report/0380` §4)가 남긴 군집을 조사했다. 코드 수정 셋:
⑴ KTF 게스트 명령이 0x1000 아래를 **읽으면** 0 을 돌려준다(쓰기·점프·호스트 읽기는 그대로 오류).
⑵ KTF Clet 모드에서 화면 프레임버퍼에 새로 그린 것이 있으면 화면에 올린다.
⑶ KTF DB slot 8(헤더 이름 `MC_dbSortRecords`)을 0 을 돌려주는 무동작으로 바꿨다.
나머지 군집은 원인을 판정해 아래에 적었다.
**왜**: 운영자 지시(2026-09-29·30) — 더 많은 게임을 검수·정상화하고 지원 현황을 갱신한다.
**사용자 영향**: 지원 현황이 playable **349 → 355** · limited 50 → 49 · not-yet **30 → 25** 로 바뀐다(이 PR 이 원인인 8행만 갱신 · §9). 새로 켜지는 KTF 게임 5종(그중 2종은 화면만 검던 Clet 게임), 플레이 중 꺼지던 2종이 끝까지 간다. SKT 인증 4종·재배치 3종은 판정만 하고 그대로다.

증적: `~/orchestrator/reports/evidence/wie-census-wave3-ktf-skt-boot-and-screen/`. sha12 만 쓴다. 게임 제목·바이트는 0 이다.

### 1. 전/후
측정: `scripts/playability-census.mjs`, LTO 릴리스, KTF 266종 전부 프로브(A·B) 재측(`a184f768` = 이 PR 의 코드 커밋). 장시간은 3c34efee 결과를 재사용한 226종을 빼고 다시 쟀다 — 조건은 «프로브 결과 불변 · 대상 아님 · 전 회차 장시간이 오류 아님»이다. 다시 잰 49종 중 장시간 대상은 12종이다. load1 115~216.

| sha12 | 전(3c34efee) | 후(a184f768) | 무엇 때문 | 다음 벽 |
|---|---|---|---|---|
| `e085e193211d` | not-yet · 부팅 slot 8 `Unimplemented` | **playable** | §4 | 소리 없음 |
| `59263295de74` | not-yet · 부팅 slot 8 | **limited** · 부팅·그림·조작 ok | §4 | 키 4 — 저장 파일 값 `0x1d14250` 을 주소로 읽기 |
| `4166acd8fc62` | not-yet · 한 색 화면 | **playable** · 512색 | §3 | — |
| `ab3d0020d7bc` | not-yet · 그림 0 | **playable** | §3 | — |
| `2b1ed0c8d061` | limited · 장시간 11키 주소 0 | **playable** · 854키 | §2 | — |
| `4decaeed58b1` | limited · 장시간 94키 주소 0 | **playable** · 854키 | §2 | — |
| `568c339a8c07` | not-yet · 부팅 주소 72 | **playable** | §2(추정 — 폴트 지점을 따로 찍지 않았다) | — |
| `bc94ba53677b` | limited · 조작 none | limited · 조작 ok | §2 | 키 6 주소 72(쓰기 또는 점프) |
| `7da00ecd4804` | limited · 11키 주소 0 | limited(변화 없음) | §2 | 프로브 14키에 주소 **8** — 0 을 읽은 뒤 `0+8` 로 쓰거나 뛴다(§2 위험 ①의 실례) |
| `5a59f62d1f1a` | limited | limited(변화 없음) | — | 61키 «주소 0» — 읽기가 아니다(쓰기·점프) |
| `8b899f410f5d` `b1ec149b354c` `01f05f8231f4` | not-yet | 그대로 | — | §7 |
| SKT 4종 · 재배치 3종 · DRM 2종 · kfc 1종 | not-yet/limited | 그대로 | §5 · §6 | — |

프로브 전이(KTF 266 · 부팅/그림/조작): 오름 7 · 내림 1(증적 `probe-transitions.txt`). 상태 전이(KTF): limited→playable 5 · not-yet→playable 4 · not-yet→limited 1 · playable→limited 2.

**내림·흔들림 짝 재측**(같은 시각 · 전 = origin/main `67c3f5d8` 빌드 · 후 = `a184f768` · 증적 `pair/`):
| 타이틀 | 무엇 | 짝 | 판정 |
|---|---|---|---|
| `a540945188ca` | 조작 ok → none | 전 1/2 none · 후 1/2 none | 조작 축 흔들림 |
| `ab64a56b2b44` | 장시간 ok → error(프로브 키 23 «Fatal error: Invalid memory access») | 전 0/6 · 후 1/6 실패 | 3c34efee 첫 프로브가 같은 서명(`R1: 0x484339f0` · 키 24)으로 실패한 기록이 있다(`census-3c34efee-first-A/`) ⇒ 기존 간헐 벽 · 퇴행 아님 |
| `f2280c6699a0` | 소리 ok → silent | 후 2/2 재생 3회 | 흔들림 |

오름 중 흔들림(`229291e20b13` 조작 · `249e655147a1` `7e2247bdf565` 간헐 panic 장시간 · `0eb19d9bbe7a` 소리)은 이 PR 이 원인이라는 근거가 없어 compat.json 에 넣지 않았다(§9).

**라이브 LGT 5종 · 가드**(`49ade89578c5`(KTF) `ddd885583b15`(LGT)): 이 PR 은 LGT 를 바꾸지 않는다 — 널 읽기는 페이지 0 이 비어 있을 때만(LGT 는 `.text` 가 0x1000 에 있어 매핑돼 있다), 화면 합성기는 KTF 에만 등록된다(`has_screen_compositor` 거짓이면 `Display` 동작이 종전과 같다). 그래서 LGT 는 재측하지 않았다. `49ade89578c5` 는 이번 KTF 전수에 들어 있고 전·후 모두 playable(후: 여섯 축 모두 ok)이다.

### 2. 낮은 주소 읽기 — 정책

군집 6종의 첫 벽은 «주소 0~0x16 접근 오류»였다. 진단용 임시 빌드(커밋 0)로 오류 순간의 게스트 PC·레지스터를 찍었다.

| 타이틀 | 오류 지점 | 원인 |
|---|---|---|
| `2b1ed0c8d061` | `ldr r0,[r3]` · r3=0 · 함수 = `*(obj->buf) + obj->off + 8` | 게임이 효과음 `s/serv_smash.mmf` 를 찾는다. 패키지의 파일은 `s/serve_smash.mmf` 다(게임 쪽 오타). 소리 객체가 빈 채로 남고(버퍼 0 · 크기 0), 첫 서브에서 그 객체의 데이터 주소를 계산하다 0 을 읽는다. 게임이 스스로 `fail to open file:[-12]` 를 찍는다 |
| `7da00ecd4804` | `ldr r3,[r0]` · r0=0 | 빈 이름 `""` 으로 `MC_knlGetResourceID` → -12 → 돌아온 0 을 역참조한다. 빈 이름이 왜 생기는지는 찾지 못했다 |
| `4decaeed58b1` `2a8a3dcd07eb` | 재현 안 됨 | 240초 2회 모두 오류 없음(간헐) |
| `0093012b8c36`(LGT) | 이번 실행은 `Double free` | 다른 벽. LGT 는 `.text` 를 0x1000 에 올려 페이지 0 이 이미 매핑돼 있다 ⇒ LGT 의 «주소 16» 은 읽기가 아니라 **널+16 으로의 점프**(`pc < 0x1000`)다 |
| `5a59f62d1f1a` | 진단 실행 못 함(장시간 키 파일이 없던 타이틀) | 이번 전수 장시간 61키 «주소 0» — 읽기가 아니다(쓰기 또는 `pc` 0) |

| 질문 | 답 |
|---|---|
| 실기에서 0 번지 읽기는 어떻게 됐나 | **직접 자료는 없다**(SDK·단말 문서 0건). 간접 근거는 출하 바이너리다. `2b1ed0c8d061` 의 오타는 출하본에 그대로 있고, 첫 서브마다 지나는 경로다. 실기에서 그 읽기가 폴트했다면 매 경기 첫 서브에 꺼졌을 것이다. ⇒ **실기에서 0 번지 읽기는 폴트하지 않았다.** 읽힌 값은 모른다(ARM 예외 벡터가 있었을 수 있다) |
| 택한 정책 | 게스트 명령의 **읽기**가 매핑 안 된 **0x1000 미만**이면 0. 쓰기·점프(`pc < 0x1000`, 종전 그대로)·호스트 API 의 읽기(`read_range`)는 여전히 오류다. 페이지 0 이 매핑된 이미지(LGT)는 영향이 없다 |
| 왜 0 인가 | 실측 두 타이틀에서 0 이 안전한 값이다 — 길이 0 과 함께 넘겨지는 주소, 개수 0 인 모델. 실기 값(벡터 코드 바이트)을 흉내 내면 그 값을 길이·개수로 읽는 게임이 엉뚱한 길로 간다 |
| 위험 ① 진단 손실 | 에뮬레이터 버그로 생긴 널 읽기도 조용히 0 으로 이어진다. 완화: 프로세스당 1회 `warn` 로그(`guest read through a null pointer`) |
| 위험 ② 실기와 다른 값 | 읽은 값으로 분기하는 게임은 실기와 다른 길로 갈 수 있다. 쓰기는 막으므로 조용한 메모리 오염은 없다 |
| 위험 ③ 범위 | 0x1000 이상, 널−8 같은 음수 주소(`0093012b8c36` 의 `0xfffffff8`)는 여전히 오류다. 넓히지 않았다 |

### 3. KTF Clet 화면 프레임버퍼

`4166acd8fc62`: 타이머마다 `MC_grpRepaint` → Clet paint 가 **화면 프레임버퍼**(`MC_grpGetScreenFrameBuffer` 가 준 핸들)에 한 장을 다 그린다 → `MC_grpFlushLcd` 는 없다. 실기에서 그 버퍼는 LCD 메모리다(0361). wie 의 Clet 모드는 Java 화면을 올리지 않으므로(`disablePaint`) 그 그림이 한 번도 화면에 가지 않았다. 0361 의 KTF 전수 표가 이 모양(Clet · 화면 프레임버퍼를 가져감 · `FlushLcd` 0)을 **11종**으로 셌다.

수정: 화면 합성기(`ScreenFramebufferSync`)가 «네이티브 화소를 올렸는가»를 돌려주고, Clet 모드의 paint 는 그때만 화면에 올린다. 오프스크린 버퍼를 `FlushLcd` 하는 Clet 타이틀은 화면 프레임버퍼가 바뀌지 않으므로 덮어 그려지지 않는다.

### 4. DB slot 8 (`MC_dbSortRecords` 자리)

0175 가 잰 세 호출 지점 모두 반환값을 버린다(0175 ⑶). 네 가설(확장자 등록 · 패턴 삭제 · 종류별 나열 · 디렉터리 선택/보장) 중 무엇이어도, 새로 설치한 게임이 필요로 하는 부수 효과는 없다. 그래서 **이름은 붙이지 않고** 무동작 + 0 으로 바꿨다. 게스트 메모리는 쓰지 않는다(시험이 잠근다).
- `e085e193211d`: 부팅 실패 → 프로브 PASS.
- `59263295de74`: 부팅·그림 ok. 다음 벽 — 패키지에 든 `res/save.sav` 를 스트림으로 읽다가, 저장 데이터 안의 값(`0x1d14250`)을 주소로 920바이트를 읽는다.

### 5. SKT 시작 대화상자 → `System.exit(-1)` 4종 — **구현하지 않는다**

4종(`0ed66634d3dc` `a42f77f44955` `c33090c12755` `eefc947d8337`) 모두 같은 검사 클래스(`SecureUtil` 또는 난독화 이름 · 같은 salt 문자열)다. 검사 내용:
`MIDlet-Key`(jad/msd 속성) == MD5(`011`/`016`… 통신사 접두 + **전화번호**(`MIN`) + `MIDlet-Jar-URL` 의 `SERVICE_ID` 10자리 + 고정 salt).
키는 **구매자 단말의 전화번호**에 묶여 있다. 실기에서도 구매하지 않은 단말은 정확히 이 대화상자를 띄우고 꺼진다 — 지금 에뮬레이터의 동작이 실기와 같다.
조건을 만족시키는 길은 둘뿐이고 둘 다 택하지 않는다:
- MD5 를 역산해 원 구매자의 전화번호를 찾는 것 — 실존 인물의 개인정보를 복원하는 일이다.
- 검사를 건너뛰게 바이트코드·속성을 바꾸는 것 — 라이선스 검사 우회다.
⇒ 운영자 정책 결정 사안으로 올린다. 코드 변경 0.

### 6. KTF 재배치 client.bin «6종» — 실제로는 3 + 2 + 1

| sha12 | 실측 | 판정 |
|---|---|---|
| `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | 재배치 표로 시작하는 client.bin(0340) | 0340 판정 유지. 로더 앞부분(재배치 2회 · bss · `+0x24` init)은 0340 에서 이미 실증했다. 막는 것은 `MNInterface`(30칸 이상 · 참고 자료 0)다. 0340 의 «다시 열 조건» 셋 중 어느 것도 생기지 않았다 |
| `60bd6cbc5936` `dab2d537f3ef` | jar 자리의 파일이 OMA DRM `odcf` 컨테이너(암호화) | 범위 밖(0330) — 복호 키가 없다 |
| `bfa8ec352451` | 표준 client.bin · `com/ktf/kfc/GMenubarForm` 없음 | 범위 밖(0330) — KTF 단말 UI 라이브러리, 참고 자료 0 |

세 오류가 같은 `JavaException` unwrap 문구를 내서 한 군집으로 묶였다. 이번 회차의 설계 산출은 «첫 2종»이 아니라 **군집 분해**다. 재배치 3종에 필요한 설계는 0340 §2 에 이미 있다.

### 7. 그 밖 — 조사만

| 타이틀 | 관측 | 다음 |
|---|---|---|
| `5814101b8010` `6b515884dbc1`(LGT 마젠타) | `MC_grpDrawImage` 0회. 게임이 `MC_GRP_GET_FRAME_BUFFER_POINTER` 로 받은 포인터에 **스스로** 화소를 쓴다. 키 비교는 게임 코드 안에 있다. 문맥 전경색 `0xf81f`(마젠타) 채우기가 453회 | 이미지 화소 형식(키 값이 게임 기대와 같은 비트인가)부터 잰다 |
| `8b899f410f5d` | 첫 실행 DB 없음 예외를 정상 처리한 뒤 한 장만 그리고 검은 화면 | 미조사 |
| `b1ec149b354c` | `NoClassDefFoundError: org/kwis/msp/lcdui/Graphics` 는 프로브에서만 난다. 키 없이 20초는 예외 없이 검은 화면 | 타이밍 의존 — 미조사 |
| `01f05f8231f4`(LGT) | 그림 0 | 미조사 |

### 8. 되돌리면 red
제품 코드에 변이를 넣고 해당 시험을 돌렸다(증적 `mut.log` · `mut3.py`). 4건 모두 red.
| 변이 | 시험 | 결과 |
|---|---|---|
| slot 8 이 `Ok(1)` | `sort_records_is_a_no_op_that_never_writes_test` | red |
| 널 읽기 갈래를 끔 | `guest_reads_through_null_are_zero_but_writes_and_higher_reads_still_fault` | red |
| `\|\| native_drawn` 제거 | `a_paint_disabled_display_presents_only_when_the_compositor_drew` | red |
| 합성기가 «그렸다»를 안 올림 | `screen_sync_is_two_way` | red |

### 9. 측정 방법 · 지원 현황(compat.json)
- 프로브: `driver.sh`(KTF 두 디렉터리 · `--only probe` · jobs 5). 장시간: 위 재사용 규칙으로 `L.json` 을 옮긴 뒤 `driver-long.sh`(`--only long` · 600초).
- **compat.json 은 8행만 갱신했다**: 이 PR 의 메커니즘(§2~§4)이 원인인 타이틀(위 표의 `e085…` `5926…` `4166…` `ab3d…` `2b1e…` `4dec…` `568c…` `bc94…`). 새 전수 `report` 출력을 `player-data.mjs` 의 `fromCensus` 로 공개 어휘로 바꾼 뒤, 그 행의 `status`·`axes`·`knownIssues_ko` 만 옮겼다(증적 `splice.mjs`). 최상위 `enginePin` 은 `3c34efee` 그대로다 — 나머지 421행이 그 핀으로 잰 값이기 때문이다. 다음 전수가 전체를 한 핀으로 다시 맞춘다.
- `node scripts/player-data.mjs` OK(429 · 355/49/25).

### 10. 게이트
- 로컬(이 브랜치 · 시험 추가 뒤): `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable · beta) · wasm32 clippy rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **605/0** rc=0 · `npm run build:wasm` rc=0.
  - 첫 `cargo test --all` 은 rustc 내부 패닉(`rustc_codegen_ssa/src/back/lto.rs:56`)으로 컴파일 중에 죽었다. 이어서 증분 캐시가 깨졌다(`file-system error deleting outdated file`). `CARGO_INCREMENTAL=0` 으로 다시 돌려 통과했다 — 시험 실패가 아니다.
- 러너 블록(`a184f768` 릴리스): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me`(`--timeout 5`) PASS. `keydraw_ktf`·`keydraw_lgt` 는 기본 `--max-ticks` 에서 `UNMEASURED`(`stop: max-ticks` · load1 100~150) — **전 빌드 `67c3f5d8` 도 같다**. `--max-ticks 2000000000` 으로 둘 다 PASS · 27/27 · `last_frame_content true` · rc=0(전 빌드도 같다).
- 유입(`node scripts/corpus-name-inflow.mjs`): BOUNDED 336쌍 + SUFFIX-ATTACHED 15쌍. `compat.json` 밖의 일치는 이 PR 이 건드린 파일에 main 부터 있던 주석이다 — 이 PR 이 더한 코드 줄의 한글은 0줄이다. `compat.json` 은 계약상 제목 목록이고 main 대비 새 `"title"` 줄은 0 이다.
