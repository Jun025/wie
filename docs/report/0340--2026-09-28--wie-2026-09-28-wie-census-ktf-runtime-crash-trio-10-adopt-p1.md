## [2026-09-28] KTF DB 가 패키지 P/ 파일을 본다 — 그리고 MC_dbExists 의 극성이 반대였다(0=있음) (wie-2026-09-28-wie-census-ktf-runtime-crash-trio-10-adopt-p1)

**무엇을**
- `read_packaged_database`(wie-wipi-c `database.rs`)가 컨텍스트 리소스(LGT jar) 다음으로 **zip `P/` 파일**(파일시스템 가상층)을 본다 — 새 접근자 `FilesystemOverlay::virtual_file`(게스트가 파일 API 로 쓴 것이 아니라 «패키지가 실은 바이트»만).
- KTF slot 5 `stat_by_name_ktf` · slot 16 `exists_database_ktf` 가 저장소 다음으로 그것을 본다. `open_database` 의 읽기 모드도 `P/` 로 시드된다.
- ★KTF slot 16 `MC_dbExists` 의 반환을 **0 = 있음 · -12(M_E_NOENT) = 없음**으로 뒤집었다(종전 1/0).
- 시험 3 — `ktf_exists_is_zero_for_saved_or_packaged_and_noent_otherwise` · `ktf_packaged_p_file_is_stat_and_read_as_a_seeded_database` · `ktf_create_mode_over_packaged_p_file_starts_empty`.

**왜**
- 티켓의 진단(«`MC_dbExists` 가 `P/` 의 세 파일을 모두 0 으로 받는다»)은 **절반만** 맞았다. `P/` 를 보게만 고치자 155972cac664 는 크래시 대신 «안정적인 구동을 위해 다시 실행해 주세요» 를 **매 실행** 띄우고 스스로 종료했다(`--relaunch 2` 로 3회 연속 같은 화면).
- 게임 자신의 래퍼를 디스어셈블했다(`0x148960`, SL `0x1767a0`): `return MC_dbExists(name, 1) == 0` — **0 이 «있음»**이다. 래퍼가 1 을 내면 게임은 적재 경로(`stat` → `Config.dat` 을 읽기 모드 open)로 간다.
  ⇒ 종전 «전 0» 은 게임에게 «셋 다 있다»였고, 그래서 `Config.dat` 을 열다(`P/` 를 못 봐 -12) 죽었다. `P/` 만 보이게 한 1 은 «없다»가 되어 매번 첫 실행 경로로 갔다.
- 극성은 한 타이틀로 정하지 않았다. KTF 266종(`working`+`broken`, 중복 제거) 을 부팅시켜 slot 16 호출 직후 Thumb 바이트를 떠 디코드했다: **19종**이 부팅에 도달하고, **18종**이 호출 직후 `cmp r0, #0` 로 결과를 검사하며(19번째는 분기 뒤라 디코드하지 않음), 그중 **15종**이 `ret == 0` 으로 바로 접는다(`bne → movs r0,#1` 12 · `beq → movs r0,#0` 3). slot 5(`stat`)는 이미 0=성공이고, LGT `exists_database` 도 0/-12 다.
- 이 극성은 상류가 준 것이 아니다. 상류 `58d252f9`(2023-12-29) 는 `unk16` 스텁을 **무조건 1** 로 바꿨을 뿐이고, 1=있음/0=없음 은 이 포크 `8fff2f0b`(2026-05-11)의 선택이다 — 근거 기록 없음.

**쓰기 경로 의미(티켓 요구)**
- 쓰기는 **저장소(DatabaseRepository)에만** 간다. `P/` 의 실린 바이트는 바뀌지 않는다(`virtual_file` 로 시험이 고정).
- `P/` 는 «저장 기록이 없을 때의 시드»다: 저장소에 있으면 저장소가 이긴다(exists·stat·open 모두).
- **mode 4(create)는 `P/` 로 시드하지 않는다** — 패키지가 없을 때처럼 빈 DB 로 시작한다. 종전 `open_database` 는 «packaged 가 있으면 mode 4 도 보존»이었는데(LGT jar 리소스용), `P/` 를 거기 넣으면 `Save0.dat` 처럼 `P/` 에 실린 이름으로 재저장하는 KTF 타이틀이 **잘리지 않고 꼬리가 남는다**. 그래서 LGT 리소스(모든 모드 보존)와 KTF `P/`(읽기 시드만)를 갈랐다.
- KTF 에는 DB 삭제 슬롯이 없다(method_table 실측) — «지웠는데 `P/` 때문에 되살아나는» 경로는 KTF 에 없다. LGT `exists_database` 는 종전부터 packaged 를 «있음»으로 봤다(변화 없음).

**사용자 영향**: 155972cac664 가 첫 확인 버튼을 넘어 **저장된 판(`P/Save0.dat`, LV7)으로 게임 화면**까지 간다. `P/` 에 초기 저장을 싣는 KTF 타이틀과 slot 16 을 부르는 타이틀 여럿이 부팅·조작을 통과한다(아래 표).

### 155972cac664 — 첫 확인 버튼 통과 전/후
| | 전(`da6c7a6a`) | 후 |
|---|---|---|
| `MC_dbExists` FirstRun/Certification/Config | 0 · 0 · 0 (게임에게 «있음») | 0 · 0 · 0 (진짜 있음 — `P/`) |
| `stat(Config.dat)` | -22 | 0 · size 6 |
| `open(Config.dat, 1)` | -12 → 크기 0 할당 역참조 | 시드 6바이트 |
| `--inject --timeout 30` | FAIL(주소 0) 3/3 · 도달 2–4 | **P27** · U18 · U10(max-ticks) — FAIL 0/4 |
| 마지막 화면 | 크래시 | 인게임(LV7 · 던전) |

※`P/` 만 고치고 극성을 두었을 때: `UNMEASURED clean exit` — 재실행 안내 무한 반복(`--relaunch 2` 3회 모두).

### KTF 전수 전/후 (266종 · `--inject --timeout 30` · 같은 작업 안에서 전→후 연달아 · P6 · loadavg 73–160)
| 전 → 후 | 종 |
|---|---|
| PASS → PASS | 122 |
| UNMEASURED → UNMEASURED | 67 |
| FAIL → FAIL | 32 |
| UNMEASURED → PASS | 18 |
| PASS → UNMEASURED | 15 |
| FAIL → PASS | 7 |
| FAIL → UNMEASURED | 3 |
| PASS → FAIL | 1 |
| UNMEASURED → FAIL | 1 |

단발 표는 부하에 흔들린다(max-ticks·clean exit 가 **양방향**으로 뒤집힌다). 그래서 «나빠진» 17종 전부와 «좋아진» 것 중 DB 가 닿는 10종을 **3회씩 전/후 재측**하고 DB 호출을 떠서 판정했다.

**나빠진 쪽 17종 — 회귀 0**
- **14종은 DB 호출이 전/후 모두 0** — 이 변경이 닿지 않는다. 뒤집힘은 전 바이너리에서도 재현된다(예: `3b82763edba8` 전 `Ux10 P27 P27` / 후 `P27 P27 P27` · `f12d97040c33` 전 `Ux12 P27 Ux12` / 후 `P27 Ux12 Ux12`).
- `a20c2044305c` — DB 호출 있음 · 전/후 P27 3/3.
- `2b1ed0c8d061` — 전 `P27 F23 F27` / 후 `F23 F24 P27` — 양쪽 흔들림(0331 이 이미 «부하 의존»으로 적은 그 타이틀).
- `04159045a7ea` — **좋아졌다.** 전 `Ux1 Ux1 Ux1`(+전수 1회 U1) = 1단계에서 스스로 종료 / 후 `P27 P27 Ux10`. `P/nv.dat`·`*.sav` 를 읽기 모드로 열 때 종전에는 -12 였다. 전수 후 1회는 **15단계에서 `Undefined instruction`(PC 0)** — 전은 1단계를 넘은 적이 없으므로 회귀가 아니라 **새로 닿은 다음 벽**이다(worklog 제안).

**좋아진 쪽 — DB 가 닿는 10종 3회 재측**
| sha12 | 무엇이 닿나 | 전 | 후 |
|---|---|---|---|
| `155972cac664` | P/ 5 · slot 16 | F4 F2 F2 | P27 U18 U10 |
| `4892a1abc0f8` | P/ 17 | Ux1 Ux1 Ux1 | **P27 P27 P27** |
| `cdd3eb5f9142` | P/ 17 | Ux1 Ux1 Ux1 | **P27 P27 P27** |
| `4a4d2ac046f7` | slot 16 | F0 F0 F0 | P27 U20 U15 |
| `f981d228b757` | slot 16 | F0 F0 F0 | U19 U26 U14 |
| `d4188f8ef8c4` | slot 16 | F9 F8 F6 | U20 U21 U17 |
| `d60c34ffc9f6` | slot 16 | F8 F9 F6 | U16 U19 U10 |
| `0c603a84f162` | slot 16 | U17 U7 U7 | U8 U7 U8 |
| `34ab350dc98a` | P/ 2 · DB 호출 0 | F23 P27 P27 | P27 P27 |
| `9e8bc87708dc` | P/ 1 · DB 호출 0 | U13 U14 U12 | U15 F2 U9 |

`U` = UNMEASURED(max-ticks — 이 부하에서 입력이 다 닿기 전에 틱 백스톱) · `Ux` = clean exit · 숫자 = 전달된 입력 단계.
`9e8bc87708dc` 의 후 F2 는 `Unsupported pixel format: 0` panic 이고 그 실행의 DB 호출은 0 — 이 변경과 무관한 기존 결함이다. `34ab350dc98a` 도 DB 호출 0(부하 흔들림).
slot 16 타이틀 4종(`4a4d…` `f981…` `d418…` `d60c…`)은 **FAIL 이 사라졌지만 PASS 가 된 것은 1회** — 나머지는 max-ticks 로 UNMEASURED 다. 판정: «죽지 않게 됐다»까지만 주장한다.

### 퇴행
- 엔진 러너 블록 전건: `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame` PASS paints 55 · rc=0 · `text_j2me --timeout 5` PASS.
- LGT: `exists_database`·`open_database` 는 종전과 같은 값을 낸다(LGT 리소스 조회가 이미 파일시스템으로 떨어진다 — `wie-lgt` context `get_resource_size`). 기존 LGT DB 시험 5건 그대로 통과.

### 변이(되돌리면 red)
- slot 16 을 `1/0` 으로 되돌린다 → `ktf_exists_is_zero_for_saved_or_packaged_and_noent_otherwise` red.
- `read_packaged_database` 의 `P/` 폴백을 끈다 → 위 + `ktf_packaged_p_file_is_stat_and_read_as_a_seeded_database` red.
- mode 4 도 `P/` 로 시드하게 한다 → `ktf_create_mode_over_packaged_p_file_starts_empty` red.

### 한계
- 극성의 «의미»(0 이 «있음»이지 «없음»이 아니다)는 **155972cac664 한 타이틀의 행동**으로 확정했다. 나머지 17종은 «0 과 비교한다»까지만 실측이다 — 그 래퍼 결과를 각 게임이 어느 쪽으로 쓰는지는 따로 확인하지 않았다. 대신 전수·재측에서 그 17종 중 회귀 0 · FAIL→비FAIL 4종이다.
- `-12` 라는 값 자체는 추측이다(게스트는 0 이냐 아니냐만 본다 — 실측한 18곳 전부). LGT 와 open 의 NOENT 에 맞췄다.
- 전수는 `--inject` 20초 창이다 — 저장/재저장이 일어나는 장시간 경로(mode 4 재저장 뒤 exists 등)는 시험으로만 고정했다.

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 0회/0쌍 · SUFFIX-ATTACHED 0회/0쌍.

<!-- corpus-name-inflow v1 subjects=4 tree=7fe5ed444b73ef71 B=0/0 P=0/0 S=0/0 -->
