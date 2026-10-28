## [2026-10-28] KTF `974e0df9ab1e`·`f770b15f8876` 설치 인증 잠금 해제 — 이 2판만, client.bin 해시 키 패치 3건 · mode 4 jar 리소스 안 심음 (wie-ktf-innotia2-license-lock-normalize)

**무엇을**: 0457 이 «구현하지 않는다»로 둔 이노티아연대기2 두 판을 켰다. 바꾼 것은 둘이다.
- 두 판이 공유하는 `client.bin1149832`(MD5 `a09d998a…`)의 설치 인증 세 벽을 통과 분기로 돌리는 바이트 패치 3건을 `wie-core-arm/data/binary_patches.toml` 의 해시 키 항목으로 넣었다. 번호는 읽지도 쓰지도 않는다.
- KTF 가 그 표를 설치하도록 `install_binary_patches` 를 일반 항목 + 해시 키 항목을 **겹쳐 설치**하게 고쳤다(SVC 디스패처는 한 번만 등록). LGT 는 종전대로 해시 키만 쓴다.
- 잠금 다음에 나온 엔진 결함을 고쳤다: KTF 에서 `MC_dbOpenDataBase` 모드 4(생성)가 같은 이름의 jar 리소스를 DB 내용으로 심던 것을 멈췄다. 이 결함이 지도 타일 테이블을 검은 화면으로 만들었다.

`HAND_WALL` 에서 이 2판을 뺐다. `compat.json` 2행을 고치고 이용자 소식 1건을 더했다.

**왜**: 운영자 지시 2026-10-28 「이노티아연대기2 두 판도 게임빌2010프로야구와 같은 조건으로 잠금 해제 티켓을 발권해라」. 이 지시는 **이 2판에만** 2026-10-02 정책(인증 우회 금지)을 푼다(선례 `wie-lgt-gamevil2010-baseball-phone-lock-normalize`·PR #573·0540). 다른 잠금 판(SKT `SecureUtil`·`install.dat`·KTF OMA DRM `dab2d537f3ef`·`60bd6cbc5936`)의 정책은 그대로다.

**사용자 영향**: 두 판이 «제한»에서 «플레이 가능»이 된다. 로고 → com2us 인증 상자 → 타이틀 → 시나리오 모드 → 캐릭터 생성(성소 배경) → 스토리 → 마을 필드 → 전투(임프 추적자, 데미지 표시)까지 들어간다.

### 1. 세 벽과 패치 — 번호를 몰라도 되는 자리 (적재 기준 0x100000)

| 자리 | 함수 | 원래 | 패치 뒤 |
|---|---|---|---|
| `0x12af9c` | `0x12af8c`: `(MC_knlGetAccessLevel() & 0xbc)==0xbc` 면 PHONENUMBER 읽음, 아니면 오류 1001 | 슬롯 13 미구현 → 종료 | 슬롯 13 호출(`bl 0x12b0c0`)을 `movs r0,#0xbc; nop` 으로. `0xbc` 비트는 0457 이 실기 값에 대해 가진 유일한 근거 |
| `0x176a80` | `0x176956` 흐름: `MC_knlGetExecNames` 로 설치 라이선스 기록을 읽어 자기 일관성 검사(2001/2002) 뒤 `strcmp(aid,buf)` | 미구현 커널 호출 → 멈춤 | `r7=licence(buf)` 호출을 `movs r7,#1; b 0x176aac`(strcmp 일치 분기)로. r7 은 그대로 `0x1768f6` 에 저장됨 |
| `0x176ab6` | `0x12ae44`: 동봉 `cert.c2s` 를 복호화해 AID 와 소유 12바이트 대조, 0 이면 오류 3001 | cert 대조 실패 → 3001 | 호출을 `movs r0,#1; nop` 으로. 이 함수는 버퍼를 free 하고 1을 돌려주므로 아무것도 안 남김 |

- 세 패치 모두 `expect`(원래 바이트)를 달았다. 바이너리가 다르면 설치가 바로 실패한다.
- 번호 값은 어느 단계에서도 읽지 않았다. PHONENUMBER 답은 기존 `01000000000` 그대로다. 캡처에 번호 없음(오류 상자 안의 번호는 com2us 고객센터 대표 번호다 — 3001 캡처).

### 2. 겹침 설치 — 왜 install_entries 하나로

KTF 는 종전 `install_binary_patches`(일반 항목 **또는** 해시 키 항목, 택1)를 썼다. 이 `client.bin` 은 일반 memcpy/memset/strcpy/strlen 훅도 맞는다(전부 실측) — 둘 다 걸어야 한다. 두 항목을 각각 `install_entry` 로 설치하면 두 번째가 SVC #0x80 디스패처를 두 번 등록해 부팅에서 «SVC handler already registered for 128» 로 죽는다(실측). `install_entries` 가 여러 항목의 훅을 모아 한 번만 등록한다. LGT 의 `install_title_binary_patches` 도 이 경로를 탄다. 시험 1건 추가(`install_entries_stacks_a_title_entry_on_the_generic_one`).

### 3. 엔진 결함 — mode 4 가 jar 리소스를 심던 것 (검은 바닥의 원인)

잠금을 지나자 캐릭터 생성 뒤 필드가 **바닥만 검게** 나왔다(HUD 는 그려짐, 예외 0, stderr 깨끗). 널폴트 스크래치 빌드로 추적: `0x1602ec` 이미지 그리기가 널 테이블(`table[1]`→주소 0x18)을 읽었다. 그 테이블은 `0x14dde4` 타일 로더가 채우는데, 첫 타일 레코드 읽기가 0을 돌려받아 로더가 테이블을 free 했다.

원인: 설치 단계가 `i_tile.dat`(jar 리소스, 압축 164,919B)를 `MC_dbOpenDataBase(mode 4)` 로 열고, 끝으로 seek 해 **푼** 테이블(194,920B)을 거기 쓴다. 그런데 KTF `open_database` 가 모드 4 를 같은 이름의 jar 리소스로 **씨딩**하고 있어서, 푼 테이블이 압축 데이터 뒤에 붙었고 뒤이은 타일 읽기가 전부 빗나갔다.

- 이 씨딩은 upstream 의 «packaged database» 규칙(b144b4e94)인데, KTF jar 리소스는 DB 가 아니다(게스트가 `MC_knlGetResource` 로 읽는다). `P/` 파일을 모드 4 에서 안 심는 것과 같은 이유로, jar 리소스도 모드 4 에서 안 심어야 한다.
- `open_database_seeded(.., create_keeps_resource)` 로 가른다. KTF(`open_database`) = false(생성은 빈 DB), LGT(`open_database_lgt`) = true(upstream 규칙 유지). 읽기·stat·exists(모드 1·8)는 둘 다 그대로 씨딩한다.
- 시험 1건 추가(`create_mode_over_a_jar_resource_starts_empty_on_ktf_and_keeps_it_on_lgt`): KTF 는 빈 DB(EOF), LGT 는 리소스 유지.

### 4. 측정 (release `wie_validate` · census-drive.sh · load1 10~286)

| 실행 | 결과 |
|---|---|
| head · census `run`(A·B·L 600초·S·speed) → `report` · 두 판 | **playable** · boot/render/input/longplay/sound ok · speed: `974e` ratio 0.880(load1 286) → unknown · `f770` 0.979 → ok |
| head · `--keys` 60키 두 판 | PASS · 키 60/60 · 예외 0 · 전투 화면 도달(캡처) |
| base(`origin/main`) · 두 판 | limited · boot/render/input ok · longplay n/a · sound silent(0457 그대로) |
| 개악 ⑴ 해제 항목 전체 제거 | FAIL · `Unimplemented: 13: MC_knlGetAccessLevel`(슬롯 13 에서 멈춤) |
| 개악 ⑵ 슬롯 13 패치만 | FAIL · `Unimplemented: 2: MC_knlGetExecNames`(라이선스 벽) |
| 개악 ⑶ cert 패치만 제거(슬롯13+라이선스 유지) | «인증오류 … (오류번호:3001)» 상자 → clean exit(캡처) |

범위 대조:
- ⑴ 코퍼스 KTF 전수(고유 패키지 429개 · `client.bin` 보유 267개 · blob 268개)에서 MD5 `a09d998a…` 가 맞는 것은 이 2판뿐이다(실측).
- ⑵ KTF DRM `dab2d537f3ef` 는 전·후 모두 `not-yet` · «암호로 잠긴 파일» 그대로다(census 실측).

KTF 표본 15종(`f44271803135` 드래곤아이즈2 포함) probe A·B 전·후:
- 판정 30쌍이 전부 같다(PASS · deadline · 키 27/0). `f44271803135` 는 전·후 모두 A UNMEASURED·clean exit(이 판의 재시작 요구 · 0511·0541), B PASS.
- status 15종 전부 `limited → limited` · 축 변화 0. **회귀 0.**

### 5. 번호 유출 0

- 패치는 비교 결과만 바꾸므로 번호 값을 읽을 필요가 없었다. 어느 단계에서도 PHONENUMBER 나 cert 소유 바이트를 출력하지 않았다. 캡처·보고·측정 로그에 번호 0.

### 검증

- 4 게이트: fmt rc 0 · clippy `-D warnings` rc 0 · wasm32 rc 0 · `+beta` rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **834 passed / 0 failed**.
- 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS rc 0.
- `npm run audit` PASSED · census selftest 65/65 · `check-compat-revert` OK · `check-docs-report-serial` OK(0546 · 충돌 0).
- 증적(repo 밖): `~/orchestrator/reports/evidence/wie-ktf-innotia2-license-lock-normalize/`(두 판 경로 시트 · 전투 캡처 · 개악 3종 JSON · 3001 캡처).

<!-- corpus-name-inflow v1 subjects=7 tree=15765b2358236288 B=721/332 P=0/0 S=38/18 -->
