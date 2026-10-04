## [2026-10-04] 6차 — 한 핀(`4ac38566`)으로 429종 전 축 재측 · compat.json 재생성 · 통신망 규칙을 프로브 A 로 좁힘 (wie-census-wave6-full-recensus-at-main-all-axes-r2)

**무엇을**: `origin/main` `4ac38566`(#446 착지) 한 핀·한 바이너리로 429종을 쟀다. 축은 프로브 A·B · 장시간 L(600초) · speed 재측이다. 진도 v2 는 좁힌 표본으로만 쟀다. 그 결과로 `docs/player-data/compat.json` 을 다시 만들었다(`enginePin` = 그 핀).
- 전수 도구: 통신망 벽(`netWall`)을 **프로브 A 에서 50회 이상**으로 좁혔다(§5).
- 계약 문서: 그 한 줄을 고쳤다.

**왜**: 운영자 지시(2026-09-30·10-02). compat 은 wave2 이후 «걸린 행만» 갱신돼, 그 사이 엔진 수정이 다른 행에 준 효과(특히 speed `no` 21)가 안 잡혔다. 총괄 결정 ⒝(2026-10-03): L 은 끝까지, 진도는 좁힌 표본.

**사용자 영향**: 지원 현황 **381/29/19 → 388/24/17**(playable/limited/not-yet). 내림 11 · 올림 18. 내림은 전부 §3 짝 재측으로 판정했다(엔진 퇴행 확정 1).

### 1. 측정 조건
- 회차 5개(2026-10-02 23:11 → 10-04 10:17). jobs 3 · 전 단계 `build-slot run` · census 호스트 락.
- 단계마다 `host-load-guard --status --recovered` rc=0 을 기다렸다. 예외 1건: §3-3 소리 짝은 guard 가 «holding»(load1 11 · idle ≈0)이던 때 시작했다.
- load1 5~252(다른 레인 · 정점은 10-03 03:12). 굶은 프로브 0. 회차 밖 프로세스 0(매 회차 pid 지정 종료).
- 진척 기록: `~/scratch/w6census/progress.log`(로컬).

### 2. 무엇을 쟀고 무엇을 재사용했나(행·축 단위)
| 축 | 이 핀에서 잰 것 | 재사용 |
|---|---|---|
| boot · render · input | 429/429 | 0 |
| longplay | 후보 395/395 | 0 |
| sound | 429 전 실행 | **9행** — 짝 재측(§3-3)에서 «두 빌드 다 census 키로는 무음». 직전 `ok` 는 타이틀별 레시피로 잰 값이라 그대로 뒀다(`bc94ba53677b` `3d2381657960` `8c71be3ad26d` `a20c2044305c` `cbe4cfe098c6` `a10a1f02b41b` `2dbde9acca99` `1045007289d8` `3de4bcb4f5e2`) |
| speed | 프로브 + 0.9 미만 10종 재측 | **75행** — 이 핀 헤드리스가 `n/a`(부하 속 하한)인데 직전이 `ok`. 헤드리스 비율은 하한이라 직전 `ok` 가 반증되지 않는다(도구 `judge` 의 «두 읽기 중 나은 쪽» 규칙과 같다) |
| progress | 표본 70 + P2 38 → 키 66행 | **67행** — 이 회차가 재지 않은 행은 직전 값을 그대로 둔다 |
| 행 전체 | — | **16행** — 핀 이후 main 의 손 변경 행(§4)은 main 값을 그대로 둔다 |

- 진도 표본(결정 ⒝):
  - ① #446 v2 비-ok 26
  - ② #446 이 시간으로 끊은 8
  - ③ playable 무작위 40 — 모집단 = `progress` 키 없는 playable 281. 추출 = `sha256("wave6-r2:"+sha256)` 오름차순 앞 40.
  - 74 중 70 을 쟀다. 4종은 이 핀에서 input `none` 이라 후보가 아니다.
- 표본 결과(P2 짝 뒤):
  | 무리 | stuck | ok | 그 밖 |
  |---|---|---|---|
  | ① | 19 | 5 | n/a 2 |
  | ② | 5 | 3 | — |
  | ③ | 10 | 27 | n/a 2 · error 1 |
  - ③ 은 playable 중 진도가 막히는 비율의 표본 추정 ≈ **29%**(11/38)다.

### 3. 내림 판정 — 전 = 직전 compat 핀 `bd2337ff` · 후 = 이 핀 · 같은 도구·같은 인자로 연달아 · 동시 ≤3
#### 3-1. input ok → no 9
- 7종: 두 빌드 다 `none`(`0eb19d9bbe7a` `1cdea1985955` `3b5b98afa890` `89c214dbd15d` `8c71be3ad26d` `4b44b31e8107` `bf54c05e58a9`) ⇒ 퇴행 아님. 직전 값이 그 측정 때의 흔들림이었다(측정 정정).
- `44b6356d13f8`: 잠긴 파일(#455 규칙).
- `bc94ba53677b`: 짝 2회 ok ⇒ 흔들림.
#### 3-2. longplay ok → error 4
| sha12 | 전 | 후 | 판정 |
|---|---|---|---|
| `8d8c24b7c198` | 600초 생존 2/2 | FAIL 3/3(`Java exception` · 키 631~658) | ★**엔진 퇴행 확정**(`bd2337ff..4ac38566`) |
| `4fdbd64c9fbd` | 같은 panic | 같은 panic | 기존 결함 |
| `55aadf368b8e` | input none(L 미도달) | null 네이티브 점프 | 후 빌드가 더 들어가 다음 벽 |
| `4a4d2ac046f7` | 생존 | 짝 생존 · 전수 1회 FAIL(`MC_dbGetRecordSize` 미구현) | 키 시점 의존 흔들림 |
#### 3-3. sound ok → no(손 변경 행 제외 9)
두 빌드 다 census 키로 무음이 8종, 두 빌드 다 재생이 `bc94ba53677b` 1종이다 ⇒ 퇴행 0. §2 의 재사용 9행이 이것이다.

### 4. 핀 이후 main 손 변경 16행 — 덮지 않았다(main 값 유지)
출처 커밋: `aaf22218`(#449) · `8d5c3681` · `c3b00a4b` · `04efa400` · `3c688d3f` · `75125caf` · `a3b5988e`(#455).
| sha12 | main 손 변경 | 이 핀 census 와의 차이 |
|---|---|---|
| `2fc792485d91` `3185174d2121` `5e53e490c6f1` `6a885f89343c` `bfa8ec352451` `de00506611a5` | 소리 끔 동봉 설정 제외(#449 · 핀 뒤 엔진) | census sound `no` — 그 수정이 핀에 없다 |
| `1b3b4868d46e` `7218e8720f8c` `8bfd08fe4370` | KTF 소리(#453 · 핀 뒤) | `7218` · `8bfd` census sound `no` |
| `a16f08d025eb` | not-yet → playable(#453 인터페이스 표) | census longplay `no` · sound `no` |
| `44b6356d13f8` | 잠긴 파일 표기(#455) | census input `no` |
| `a540945188ca` `b475b6399684` `78bd51675574` `735a579d82ac` `38277d63b0ba` | 진도 손 측정(진도 3차 등) | census 진도 `stuck` 또는 미측정 |

### 5. 통신망 규칙 — 프로브 A · 50회
wave5 규칙(A 또는 L 에서 20회 이상)을 이 핀에 대면 **playable 5종이 거짓으로 걸렸다**(`93d5b6b8ceb5` `94a34abdea21` `2d5cada03004` `2a8a3dcd07eb` `3ff5948e235e`).
- L(600초 키 반복)에서 66·66·29·28회를 되풀이한다 — 게임 안 메뉴가 재접속한다.
- `3ff5948e235e` 는 A 에서 21회지만 시작 메뉴가 살아 있다.
- 진짜 벽 `f44271803135` 는 A 100회다.
⇒ A 만 세고 문턱을 50 으로 했다. 429종에서 1종(`f44271803135`)만 걸리고, wave5 판정과 같다. selftest 52/52 · 문턱을 20 으로 되돌리면 FAIL.

### 6. 남은 벽 군집표(이 핀 · 총괄 발권용)
| 수 | 첫 벽 | 추정 계급 | 크기 | sha12(대표) |
|---|---|---|---|---|
| 1 | longplay — Java 예외(키 631~658) | ★**엔진 퇴행**(두 핀 사이 이분 탐색) | S~M | `8d8c24b7c198` |
| 57 | sound silent(census 키 기준) | 레시피 미적용 · 원 주인 설정 · AOT 조건 섞임 | 타이틀별 S | `clusters.md` 참조 |
| 34 | 진도 stuck(정책 v2 600초) | ⒜ 정책 한계 / ⒝ 엔진 벽 섞임 — 표본만 잼 | 타이틀별 | ① 19 · ② 5 · ③ 10 |
| 19 | input none | 측정 흔들림(§3-1) · 잠금 · 키 이전 렌더 벽 | 조사 S | `0eb19d9bbe7a` … |
| 5 | boot — `unwrap JavaException` @ KTF jvm_support | 엔진(DRM 잠금 2 포함 — 정책 대상) | M | `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` |
| 2 | longplay — 주소 0/8 메모리 접근 | 엔진 | M | `5a59f62d1f1a` `7da00ecd4804` |
| 1씩 | `MC_dbGetRecordSize` 미구현 · null 네이티브 점프 · KTF 커널 확장 미식별 · `NoClassDefFoundError` · 검은 화면 | 엔진 | S~M | `4a4d2ac046f7` `55aadf368b8e` `5267badf20b3` `3151fdc167b6` `71d1d8235bd1` `01f05f8231f4` `8b899f410f5d` |

정책 판단이 필요한 것: DRM·구매 단말 잠금 7(안내 완료 · 해제 안 함) · 통신망 1(안내).

### 7. 갱신 방법
1. `node scripts/playability-census.mjs report --out <out> --pin 4ac38566adfe64b3d2255835de44847fdc6b6ab7`(main 판 도구 + §5).
2. `fromCensus`(공개 어휘).
3. 재사용 규칙(§2) · 손 변경 16행 유지(§4).
4. `node scripts/player-data.mjs` OK(429 · 388/24/17).
- 이용자 소식은 «엔진 변경으로 상태가 실제로 바뀐» 2종(not-yet → playable · `b1ec149b354c` `87b04639cdfe`)만 냈다.
- limited → playable 16 은 원인을 행마다 귀속하지 못했다. 측정 갱신으로 두고 소식은 내지 않았다(wave3 선례).

### 8. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 330쌍 + SUFFIX-ATTACHED 15쌍. 전부 `compat.json` 의 기존 제목 값이다(재생성 전후로 제목 문자열 차이 0). 그 밖 파일 0. 타이틀은 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=6 tree=aa7f1ea452cd8e31 B=718/330 P=0/0 S=35/15 -->
