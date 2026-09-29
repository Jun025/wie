## [2026-09-29] 2차 전수 검수 — main `313ddcd8` 재측 · 남은 군집 7개 수정 · 지원 현황 갱신 (wie-census-wave2-recensus-and-fix-remaining-clusters)

**무엇을**: 현재 main(`313ddcd8`)으로 429종을 여섯 축으로 다시 쟀다. 남은 첫 벽 군집 중 7개를 고쳤다. 고친 빌드(`3c34efee`)로 다시 재서 `docs/player-data/compat.json` 을 만들었다. 전수 도구는 첫 실행 뒤 스스로 끝나는 타이틀을 한 번 다시 켠다(`--relaunch 1`).
**왜**: 운영자 지시(2026-09-29) — 더 많은 게임을 검수·정상화하고 지원 현황을 갱신한다.
**사용자 영향**: 지원 현황이 playable **307 → 349** · limited 75 → 50 · not-yet **47 → 30** 으로 바뀐다(1dd9f81c → 3c34efee). 이 PR 의 수정으로 5종이 나아졌고(이용자 소식 3파일), 첫 실행 안내 뒤 꺼지던 8종이 «다시 켜면 플레이 가능»으로 잡힌다.

증적: `~/orchestrator/reports/evidence/wie-census-wave2-recensus-and-fix-remaining-clusters/`(군집표 둘 · 전이 · 짝 재측 · 변이). sha12 만 있고 게임 제목·바이트는 0 이다.

### 1. 측정
- 도구: `scripts/playability-census.mjs`(0321 · 0348 그대로) · launchd 스크래치 잡 `ProcessType=Interactive` · 프로브 `--jobs 12` · 장시간 `--jobs 32`.
- load1: 재측 1회차 280~700 · 2회차 250~750. 조용한 시간은 없었다.
- 1회차(`313ddcd8` · LTO 릴리스): 429종 전부. 굶은 프로브 0.
- 2회차(`3c34efee` · LTO 릴리스 · 이 PR 의 코드 커밋): 프로브 429종 전부.
  - 장시간은 1회차에서 «마감까지 실행»(281종)이면 그 결과를 썼다. 오류·스스로 끝남·마감 FAIL 68종과 새 후보만 다시 쟀다(88 + 2종).
  - 프로브 A 가 FAIL 인 42종은 예산을 60초로 늘려 1회 더 쟀다(티켓 규칙). 첫 결과는 증적 옆 `census-3c34efee-first-A/` 에 두었다.
- 속도(헤드리스)는 두 회차 중 큰 값이다. 둘 다 부하가 더하기만 한 하한이다(0321 «한계»). 브라우저 판정은 0348 의 2회 측정(`results.jsonl`)을 그대로 넣었다.

### 2. 전/후
| 핀 | playable | limited | not-yet |
|---|---|---|---|
| `1dd9f81c`(0348) | 307 | 75 | 47 |
| `313ddcd8`(현재 main · 이 회차 1회차) | 335 | 63 | 31 |
| `3c34efee`(이 PR · compat.json) | **349** | **50** | **30** |

| 축 | 1dd9f81c 결손 | 313ddcd8 결손 | 3c34efee 결손 |
|---|---|---|---|
| boot | fail 34 | fail 24 | fail 22 |
| render | uniform 9 · none 38 | uniform 4 · none 27 | uniform 4 · none 26 |
| input | none 49 | none 38 | none 24 |
| longplay | error 37 | error 31 | error 30 |
| sound | silent 98 | silent 82 | silent 75 |
| speed | slow 21 | slow 21 | slow 21 |

`313ddcd8 → 3c34efee` 상태 전이(전건: 증적 `transitions.txt`):

| 전이 | 수 | 무엇 |
|---|---|---|
| limited → playable | 19 | 이 PR 의 수정 3(`640428a9cf9e` `b22a7fcfb406` `14a62a8521a0`) · 다시 켜기 8 · 장시간 재측에서 전 회차 오류가 다시 나지 않음 5 · 조작 축 흔들림 3 |
| not-yet → playable | 1 | `6af589a88cf9` — Interface4 레코드 DB |
| not-yet → limited | 1 | `7da00ecd4804` — Interface4 + free(NULL) + 다시 켜기. 장시간에서 주소 접근 오류가 남는다 |
| playable → limited | 6 | 아래 표 — 퇴행 0 |
| limited → not-yet | 1 | `eefc947d8337` — 아래 표 — 퇴행 아님 |

«다시 켜기 8»: `1352b27a7898` `1cf2e6076079` `2520654be6de` `40b9537968de` `44b6356d13f8` `5028b8a5d19f` `acf6863fc84c` `b44b5fbe29e6`. 1회차에서 0~1키 만에 «clean exit» 였고, 2회차는 한 번 다시 켜진 뒤 키를 받는다.

**playable → limited 6종 · limited → not-yet 1종 — 짝 재측**(같은 시각 · `313ddcd8` 빌드 vs `3c34efee` 빌드 · 둘 다 `--relaunch 1` · 증적 `pair/`):
| 타이틀 | 무엇 | 판정 |
|---|---|---|
| `0cc4ef7ede37` `5dc5d7091c01` `8f7758fa43b6` | 1회차 장시간이 10·11·1키에 «clean exit» 로 끝나 ok 였다. 다시 켜기로 이어 가자 장시간 18·198·82키에 새 벽(`Option::unwrap` · LGT SVC 2000 · LGT SVC 809) | **드러남** — 전에는 벽까지 가지 않았다. 후속 군집 표에 넣었다 |
| `2a57e33133b5` | 프로브 15키에 `InvalidMemoryAccess(0)` panic | 짝 2회: 전 빌드도 1/2 같은 panic ⇒ 간헐 · 퇴행 아님 |
| `38277d63b0ba` `d9afc4db742c` | 조작 축 none(그림 2~3장) | 짝 2회: 전·후 모두 PASS · 그림 수 같음 ⇒ 조작 축 흔들림 |
| `eefc947d8337` | 그림 0장으로 스스로 끝남 | 짝 2회: 전·후 모두 2~3키에 끝나고 그림 0~2장 ⇒ 흔들림. 아래 SKT 3종과 같은 종료 대화상자 |

추가 짝 재측: `174237758542` `ab64a56b2b44` 는 2회차 첫 프로브에서 오류였다. 전·후 빌드 2회씩 모두 PASS 다. 60초 재측에서도 PASS 라 compat 은 playable 이다. `6103e87874c6` 은 전 빌드 2/5 · 후 빌드 2/5 가 같은 `Option::unwrap` 로 실패한다. 간헐이고 퇴행이 아니다.

**라이브 LGT 5종**(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`): 두 회차 모두 playable · 여섯 축 ok. **퇴행 가드**(`49ade89578c5` `ddd885583b15`): 두 회차 모두 playable · 속도 slow(T1 · 별 티켓 `wie-census-wave2-timer-wake-off-the-frame-grid`).

### 3. 고친 군집
| # | 군집 | 무엇 | 걸린 타이틀 | 전 → 후 |
|---|---|---|---|---|
| 1 | SKT `javax/microedition/io/Connector` 없음 | `open` 3종이 `IOException("no network")` 를 던진다. 호출부(키 처리 안의 소켓 연결)가 `IOException` 을 잡는다 | `640428a9cf9e` | limited(25키 NoClassDefFound) → **playable** |
| 2 | KTF `InputMethodHandler` 메서드 없음 | `setInputMethodListener`(필드에 저장) · `notifyKeyInput`(false = 키를 게임 처리기로 돌려준다). 게임 상수 풀에서 쓰는 메서드 4개 중 없던 2개다 | `b22a7fcfb406` | limited(12키 · 다음 15키 `notifyKeyInput`) → **playable** |
| 3 | KTF `MC_netSocket` 미구현 | -1(`M_E_ERROR`). `docs/wipi-c-abi-error-codes.md` 에 5번째 자리로 올렸다(`socket_close` 와 같은 이유) | `7d007391e4a1` | 이 회차 두 핀 모두 playable(0348 의 장시간 오류는 키 순서에 따라 난다). 치명 오류 경로를 없앴다 |
| 4 | KTF 미식별 표 Interface4 | 헤더의 레코드 DB(`MC_dbOpenDataBase(name, rsize, create, mode)` 순서)로 이었다. 근거는 실측이다 — 슬롯 0 은 세 타이틀에서 `(이름, 크기, create, 1)` 이다. 0 이 돌려준 핸들을 다음 호출의 r0 로 넘기면 슬롯 3 `(fd, buf, 8)`·7 `(fd, buf, 12)`·10 `(fd)` 가 헤더의 insert·list·getNumberOfRecords 모양 그대로다. 8·9·11 이후는 이름 없는 스텁 그대로 둔다 | `6af589a88cf9` `7da00ecd4804` `4166acd8fc62` | not-yet → **playable** · not-yet → limited · not-yet 유지(다음 벽 아래) |
| 5 | KTF `free(NULL)` 이 주소 4 를 읽음 | LGT `free_indirect` 와 같은 null 가드. 첫 실행 종료 경로가 `MC_grpDestroyOffScreenFrameBuffer(0)` 를 부른다 | `7da00ecd4804` | 첫 실행 종료가 오류 → «clean exit» |
| 6 | `sprintf` `%i` 미지원 | C 의 `%d` 와 같게 처리한다. 리소스 이름 `ch0%i.mbac` 이 그대로 나가 `GetResource(-12)` 였다 | `4166acd8fc62` | 그 경로 통과(타이틀은 아래 다음 벽) |
| 7 | SKT `TextComponentHandler.getTextComponentHandler` 없음 | 공유 핸들러 1개 · `getInputMode` 0 · `keyPressed/Released/Repeated` false. 텍스트 필드 생성자가 무조건 받아 가고 키를 넘긴다 | `14a62a8521a0` | limited(장시간 210키) → **playable**(장시간 854키 완주) |
| 8 | 전수 도구 | 프로브·장시간에 `--relaunch 1`. «다시 실행해 주세요» 안내 뒤 스스로 끝나는 타이틀을 플레이어처럼 한 번 다시 켠다 | `7da00ecd4804` + 위 8종 | 위 «다시 켜기 8» |

«되돌리면 red»: 변이 11종(1·2×2·3·4 라우팅·4 create·4 insert·5·6·7×2) 전부 red(증적 `mut.log`). 전수 도구의 `--relaunch` 는 인자 한 줄이라 시험이 없다.

### 4. 후속 군집(못 고친 것 · 3c34efee 기준 · 총괄 발권용)
| 걸린 수 | 첫 벽 | 추정 계급 | 크기 | 타이틀(sha12) |
|---|---|---|---|---|
| 75 | 소리 없음(엔진 쪽 재생 0) | 30초 안에 소리를 안 내는 타이틀 포함 — 원인 여럿 | L | 증적 `clusters-3c34efee.md` |
| 24 | 조작 축 none(키 프레임이 무입력과 같음) | 절반은 1회 측정 흔들림(위 짝 재측) · 나머지는 입력 대기 화면 | M | 증적 |
| 8 | LGT `Unknown LGT WIPIC SVC id` — 번호가 제각각(103 · 239 · 240 · 809 · 1100 · 2000 …) | LGT WIPI-C 표 미식별 칸. 칸마다 호출부 역공학이 필요하다 | M(칸마다 S) | `1cd151222bde` `619d98bc8f64` `8f7758fa43b6` `b7699c10dfd1` `3ff5948e235e` `5dc5d7091c01` `863b8ab6a21d` `87b04639cdfe` |
| 6 | KTF 재배치 표 client.bin `JavaException` | 0340 이 «구현하지 않는다»고 판정한 3종 + 같은 모양 3종 | L | `1d5831e42a8a` `60bd6cbc5936` `83fc429f9cbe` `b907b0faf483` `bfa8ec352451` `dab2d537f3ef` |
| 6 | `Invalid memory access` — `EventQueue.getNextEvent` · `CardCanvas.keyPressed` | 게임의 null 역참조(주소 0~0x16) · 실기에서는 낮은 주소 읽기가 통했을 수 있다 | M(정책 결정 필요) | `0093012b8c36` `2a8a3dcd07eb` `2b1ed0c8d061` `4decaeed58b1` `5a59f62d1f1a` `7da00ecd4804` |
| 5 | `class_instance.rs` `Option::unwrap` panic(KTF · SKT) | 간헐(`6103e87874c6` 은 전·후 빌드 2/5) — 해제된 인스턴스 참조 추정 | M | `0cc4ef7ede37` `249e655147a1` `85e94babc247` `c7f543c73b91` `0262a4fe3389` |
| 4 | SKT 시작 때 4줄 대화상자 → `Thread.sleep(2000)` → `System.exit(-1)` | 설치 검사(`MIDlet-Key` 류) 추정 | S~M(조사) | `0ed66634d3dc` `a42f77f44955` `c33090c12755` `eefc947d8337` |
| 3 | 한 색 화면 | `4166acd8fc62`: 화면 버퍼에 그리고 `MC_grpRepaint` 만 부른다(`FlushLcd` 없음). clet 모드는 MIDP paint 를 끄므로 앱 paint 가 불리지 않는 것으로 보인다(추정) · `b1ec149b354c`: MIDP-on-KTF 에서 `org/kwis/msp/lcdui/Graphics` 조회 실패 · `8b899f410f5d` 미조사 | M | `4166acd8fc62` `8b899f410f5d` `b1ec149b354c` |
| 2 | 그림 0 · 멈춤 | 미조사 | M | `01f05f8231f4` `ab3d0020d7bc` |
| 4 | LGT JVM 지원 층 panic(클래스 이름 읽기 실패 · `JavaException` unwrap) | 간헐 포함(`2a57e33133b5` 는 전 빌드 1/2) — panic 을 Java 예외로 바꾸는 것이 먼저 | M | `2a57e33133b5` `a16f08d025eb` `1eaa92092bee` `fe76e641bb3d` |
| 2 | 마젠타 투명색 미적용 | 렌더 | S | `5814101b8010` `6b515884dbc1` |
| 2 | KTF `MC_dbSortRecords` 인자 모양 미상 | 0175 부록의 역공학 이어 가기 | M | `59263295de74` `e085e193211d` |
| 1씩 | SKT `m/V3` 3D 라이브러리 없음 · KTF `com/ktf/kfc/GForm` 없음 · KTF 시작 NPE(jar 리소스 null) · LGT `/ by zero` · LGT `DataOutputStream` vtable 12 · KTF `MC_dbGetRecordSize` · KTF `MC_grpEncodeImage` · KTF `MC_knlReserved` · LGT 할당기 bucket 범위 밖 panic · 호스트 스택 넘침 · `AllocationFailure` panic | 각각 | S~L | `71d1d8235bd1` `f07cbc782828` `96dc32e781d3` `b2da04c55cd4` `61ed69520fd3` `f981d228b757` `33801c1ba14f` `3151fdc167b6` `517ed32c92d6` `a23f3c9fc2cb` `7e2247bdf565` |

### 5. 데이터
- `docs/player-data/compat.json` — pin `3c34efee`(이 PR 의 코드 커밋 · GitHub 에 있다). 그 뒤 커밋은 시험·문서뿐이라 엔진 동작이 같다.
- 이용자 소식 3파일(`docs/player-updates/2026-09-29-census-wave2-*.json` · 5종). `4166acd8fc62` 는 상태가 그대로라 소식을 내지 않았다. `7d007391e4a1` 은 이미 playable 이었다.
- 셸 반영: 계약 §3(릴리스 → `repository_dispatch` `publicData`). 수신부 otterpebble #1160 은 머지됐다.
- 측정 뒤 main 에 #406(LGT paint 교착 해소)·#410(전수 `--jobs` 상한)·#411 이 착지했고 이 브랜치에 병합했다. compat.json 은 그 앞 `3c34efee` 로 잰 값이다. #406 이 나아지게 한 타이틀은 다음 전수에서 반영된다.

### 6. 한계
- 장시간 281종은 1회차(`313ddcd8`) 결과를 그대로 썼다. 이 PR 이 바꾼 경로(위 1~7)를 그 타이틀들이 10분 동안 오류 없이 지났다는 뜻이지, 새 빌드로 다시 잰 것은 아니다. `%i` 처럼 오류 없이 동작만 바뀌는 수정은 이 방식으로는 보이지 않는다.
- 조작 축은 여전히 1회 측정이다(0321 한계 그대로). 전이 중 3종은 흔들림으로 판정했다.
- Interface4 의 8·9·11 이후는 이름을 붙이지 않았다. 8(Sort)·9(GetAccessMode)는 헤더 순서상 이름이 있지만, 그 칸에 온 타이틀이 없어 레지스터로 확인하지 못했다.

### 7. 게이트
- 로컬: fmt · clippy `-D warnings`(stable · beta · wasm32) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 599/0 rc=0 · `npm run build:wasm` rc=0.
- `player-data.mjs` OK(429 · 349/50/30) · `--selftest` 19 · worklog OK · 연번 OK(0380 · 0378 은 #410 이 먼저 잡았다).
- 유입: 331쌍(BOUNDED) + 판단 필요 15쌍(SUFFIX-ATTACHED). compat.json 밖은 1쌍이고 `wie-ktf/src/runtime/wipi_c/context.rs` 에 main 부터 있던 주석이다. compat.json 은 계약상 제목 목록이고 main 대비 새 `"title"` 줄은 0 이다.

<!-- corpus-name-inflow v1 subjects=20 tree=1961640c3f05cc04 B=719/331 P=1/1 S=35/15 -->
