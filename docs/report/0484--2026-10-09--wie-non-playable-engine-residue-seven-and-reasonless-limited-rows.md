## [2026-10-09] 비-playable 엔진 잔여 6종 · 이유 없는 행 0 (wie-non-playable-engine-residue-seven-and-reasonless-limited-rows)

**무엇을**: 주인 티켓이 없던 비-playable 6종(not-yet 2 · limited 4)을 현 main 으로 다시 재고, 엔진으로 풀리는 것을 고쳤다. `knownIssues_ko` 가 빈 비-playable 행을 0 으로 만들고, 다시 생기지 않게 검사에 넣었다.
**왜**: 운영자 지시(미지원·부분지원 게임을 모두 플레이 가능하게 · 안내 최신화). 비-playable 26종 중 정책상 안 푸는 15종을 뺀 엔진 후보 가운데 이 6종은 09-30 뒤로 아무도 다시 보지 않았다. limited 3행은 이유 문장이 비어 «화면만»이라고만 보였다.
**사용자 영향**: 시작하자마자 멈추던 KTF 게임 2종이 실행된다(하나는 이야기 모드까지, 하나는 로고·메뉴·이야기 장면이 제자리에 그려진다). 오래 하면 꺼지던 LGT 게임 1종이 600초를 버틴다. 이유 없이 «부분 지원»이던 3종 중 2종은 이미 main 에서 고쳐져 «지원»으로 바뀌고, 1종은 이유 문장이 붙는다.

증적(스크래치 · 커밋 안 함): 짝 프로브 `pair.out`·`h2.out` · 진단 스캔 `scan.out` · head census out `hc`(진도 P2 는 판정에 쓸 `287af341dac8` 까지 받고 멈췄다 — 나머지는 P 가 ok 라 P2 가 판정을 못 바꾼다) · 장시간 짝 `ll.out`. 타이틀은 sha12 로만 적는다.

### 1. 전/후 — 6종 + 이유 없는 행

축 순서 boot·render·input·longplay·sound·speed·progress(o=ok · n=no · u=unknown · s=stuck · -=키 없음). «전» = `origin/main` `74c9d915` compat 행, «후» = 이 브랜치 head census(§5).

| sha12 | 전 | 후 | 계급 · 처방 |
|---|---|---|---|
| `362c57e2b2b7` KTF | not-yet `nnuuuu-` | **playable** `ooooooo` | 엔진 ⒝ — §2 |
| `f07cbc782828` KTF | not-yet `nnuuuu-` | **playable** `ooooouo` | 엔진 ⒝ — §3 |
| `4fdbd64c9fbd` LGT | limited `ooonou-` | **playable** `ooooooo` | 엔진 ⒝ — §4 |
| `287af341dac8` LGT | limited `oonuno-` | **playable** `oooooos` | 측정 — 엔진 결함 아님(§6) |
| `4b8c8f5ff7d6` KTF | limited `oonuno-` | **playable** `ooooooo` | 재측만 — 0456(무료 메모리 2MiB)이 이미 고쳤다 |
| `aa3fcba4598b` KTF | limited `oonuoo-` | limited(같음) | **못 고침** — §7 |
| `8b899f410f5d` KTF(이유 없음) | limited `ooouou-` | **playable** `ooooooo` | 재측만 — 장시간을 안 잰 행이었다 |
| `63332c51d514` LGT(이유 없음) | limited `ooouou-` | **playable** `ooooooo` | 재측만 — 같음 |
| `83fc429f9cbe` KTF(이유 없음) | limited `ooouou-` | limited(같음) + 문장 | 주인 티켓 소관 — 축은 안 바꾸고 «오래 플레이 검사를 아직 못 마쳤다» 문장만 |

비-playable **26 → 19** · `knownIssues_ko` 가 빈 비-playable 행 **3 → 0**.

### 2. `362c57e2b2b7` — 널 클립과 무시되던 문맥 클립

- 첫 벽: `startApp` 안 `MC_grpSetContext(ctx, CLIP_IDX, NULL)` 이 네 단어를 주소 0 에서 읽어 부팅이 끝났다(`Invalid memory access; address: 0` · PC 가 호스트 함수 영역 `0x710076aa`). 널 클립은 «클립 없음»으로 읽어 `MC_grpInitContext` 상태로 되돌린다.
- 둘째 벽(첫 벽을 넘자 보였다): 로고가 30픽셀 창을 옮기며 드러나는 애니메이션인데 화면이 겹친 사본으로 찼다. 게임은 `[79,143,109,172]` 를 걸고 62폭 그림을 x 47 에, `[108,143,138,172]` 를 걸고 같은 그림을 x 76 에 그린다. WIPI-C 그리기(`wie-wipi-c` graphics)가 문맥의 클립을 **읽지 않았다**(그리는 사각형만 클립으로 썼다).
- 처방(`context_clip`): `MC_grpSetContext(CLIP)` 이 마스크 비트 0(`1 << CLIP_IDX`)을 켜고, 그 비트가 있으면 채우기·호·그림·영역 복사·프레임버퍼 복사·사각형·선·다각형이 문맥 클립(x1,y1,x2,y2 포함 · LGT 레코드의 `clip_x1`…`clip_y2` 와 같은 꼴)과 겹친 곳에만 그린다. 널이면 비트를 끈다. 클립을 건 적 없는 문맥은 종전과 같다.
- **글자는 일부러 뺐다**: `4decaeed58b1` 이 줄마다·버튼마다 클립을 걸고 글자를 쓰는데, 이 층의 글자 상자가 단말보다 몇 픽셀 아래라(종전에도 버튼 아래 테두리에 걸쳤다) 클립을 걸면 줄마다 아래 절반이 잘렸다. 첫 짝 재측에서 잡아 `MC_grpDrawString` 은 종전처럼 문맥 클립 밖에 둔다(주석).
- 픽셀 연산 블릿(`blit_with_pixel_op`)도 클립하지 않는다(`ponytail:` — 클립과 키 색을 함께 쓰는 타이틀을 못 봤다).
- 후: 로고·메뉴(새로하기·이어하기)·이야기 장면이 제자리에 그려진다. head census 6축 + progress 전부 ok.
- 시험: `a_null_clip_resets_the_clip` · `drawing_stays_inside_the_context_clip`.

### 3. `f07cbc782828` — `com/ktf/kfc` 폼 툴킷 3종

- 0329 가 «툴킷이라 `GForm(IIII)` 만 넣으면 벽이 `GTextField` 로 옮겨 갈 뿐»이라 남긴 그 사슬을 끝까지 따라갔다. 실행 추적(`get_java_method`)이 정한 모양만 넣었다:
  - `GForm` — 부모 `GMenubarForm`(`GTextField.<init>` 이 `GMenubarForm` 을 받는데 게임이 `GForm` 을 넘긴다), `<init>(IIII)V`(startApp 에서 124, 214, 65, 17).
  - `GTextField` — `TextComponent` 자식, `<init>(GMenubarForm,String,I)V`(글은 `setString` 으로), `getGTextListener()`(필드마다 하나 — 같은 객체를 두 번 돌려준다).
  - `GTextListener` — 이름과 달리 인터페이스가 아니다. 게임이 돌려받은 객체에 `setIMEModes([I)V` 를 부른다.
- 후: 첫 화면 → 메인 메뉴 → 게임시작 → 스토리모드 → 이야기 대화까지 간다(손 키 재현). head census 5축 ok · speed `n/a`(0.829 · load1 69 — 헤드리스 하한) · progress ok.
- 0329 의 «온라인 게임» 판정: 도움말에 «정보 이용료(패킷당)» 쪽이 있고 이미지가 `Socket`·`URL` 을 참조하지만, 스토리모드는 접속 없이 진행했다(이 회차 범위). 온라인 메뉴는 열어 보지 않았다.
- 시험: `g_form_is_a_menubar_form` · `g_text_field_keeps_its_text_and_one_listener`.

### 4. `4fdbd64c9fbd` — LGT Clet 은 수거가 멈춰 있었다

- 6차가 «기존 결함»으로 둔 `unwrap JavaException` panic(600초 · `243_UP`) 직전 stderr: `guest heap allocation of 0xc bytes failed … buckets … 16:524288/524288` — 16바이트 버킷만 가득, 리스트 영역은 최대 빈 구간 `0x747c254`.
- 임시 계측(커밋 안 함 · 65,536 할당마다 버킷 사용량): 16B 가 할당 65,536번마다 **약 +50,750** 씩 직선으로 늘었다 — 거의 아무것도 돌아오지 않는다. 같은 실행에서 수거(`guest roots … instances`) 로그는 처음 10초에 18번, 그 뒤 0번.
- 원인: LGT/KTF 의 수거는 MIDP `Display` paint 가 1초마다 부른다(`GC_INTERVAL_MS`). Clet 은 `MC_knlSetTimer` 콜백에서 `MC_grpFlushLcd` 로 그리므로 타이틀 카드가 지나면 그 paint 가 더 오지 않는다 ⇒ 그 뒤 만든 Java 객체가 하나도 수거되지 않는다.
- 처방(`wie-lgt` `LgtWIPICContext::set_timer`): 타이머 콜백이 끝날 때 `Display` 와 같은 일정(1초 · 직전 비용의 20배)으로 수거한다. 시계는 `ArmCore::id` 마다 하나다.
  - ★첫 판은 시계를 컨텍스트에 두었다가 짝 측정에서 잡았다 — LGT 는 SVC 마다 컨텍스트를 새로 만든다(`handle_wipic_svc`) ⇒ 매 틱 수거(`236c7da689f6` 110초에 754회 · gc 1,064ms). 고친 뒤 100·102회 · 143·102ms(벽시계의 약 0.1%).
- 확인(임시 계측 · 같은 키 120초): 16B 사용량이 1,344~9,261 에서 오르내리고 쌓이지 않는다. head census 600초 longplay ok · progress ok.
- 시험: `timer_collection_waits_a_second_and_for_its_cost`(일정과 «코어마다 하나»). ★이 시험은 **호출 자리**를 잠그지 않는다 — `set_timer` 에서 호출을 빼면 green 이다. 그 자리를 잠그는 것은 위 측정뿐이다(타이머를 쓰는 LGT 픽스처가 없다).
- KTF Clet 은 같은 모양인지 재지 않았다(KTF 는 Clet 모드에서도 `Display` paint 가 돈다 — 0391 §3).

### 5. 측정 조건 · 퇴행

- 엔진: base = `origin/main` `74c9d915` · head = 이 브랜치 release `wie_validate`. 도구 = 저장소 `scripts/playability-census.mjs`(이 브랜치 판 — `RESTART_FIRST` 가 base 측정에도 같게 들어간다).
- 자원: 측정 실행마다 `build-slot run --long` 임대 1개 · 그 안 2~3 병렬. `nohup` 0 · 끝난 뒤 자기 `wie_validate` 0. 호스트가 자주 포화였다(`host-load-guard` rc=1 → 폭을 2로 줄였다) · load1 28~92.
- census 잠금: 다른 레인의 진도 묶음이 약 3시간 쥐고 있었다. 짝 프로브는 census 대신 같은 인자(프로브 A: 27키 30초 · `--pacing 8` · `--relaunch 1`)의 2폭 러너로 쟀고, head census 는 잠금이 풀린 뒤 돌렸다.
- **진단 스캔**(임시 빌드 · 커밋 안 함): KTF·LGT 344종을 20초 돌려 «문맥 클립이 실제로 좁혔다»·«타이머 수거가 돌았다»를 찍었다 ⇒ 클립 **45종** · 타이머 수거 **59종**(LGT · playable 53 · limited 6). 이 둘이 변경이 닿는 전부다.
- **짝 프로브 98종**(위 104 의 합집합 + 라이브 LGT 5 `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684` + 가드 `49ade89578c5` `ddd885583b15` + 대상): result·stop·content 가 바뀐 것은 **2종**(`362c57e2b2b7` `f07cbc782828` FAIL → PASS)뿐. 글자를 뺀 판으로 클립 45종을 다시 재도 같다(44 같음 + `362c57e2b2b7`).
  - 프레임이 크게 다른 클립 타이틀 10종을 눈으로 봤다: 키 타이밍에 따른 다른 경로(메뉴 진입 시점 등)였고 그림 깨짐은 없었다. `04159045a7ea` 의 «OK:확인» 글자 유무 차이는 base 끼리도 갈렸다(두 base 실행의 프레임 열이 각각 head 와 같다) — 깜빡임이다.
- **장시간 짝**(LGT 수거 대상 playable 6종 · 600초 · `LONG_KEYS` 루프): `8f7758fa43b6` `1cd151222bde` `af7d82e5e239` `580a66c32fff` `c9b287e3edcf` `236c7da689f6` — base·head 모두 FAIL 0 · 할당 실패 0 · 854/900 단계. (이 짝은 «매 틱 수거» 판으로 쟀다 — 지금보다 수거가 잦은, 더 센 쪽이다.)
- head census 가드: `49ade89578c5` `ddd885583b15` 6축 ok — 퇴행 0.
- 러너 줄(AGENTS · 머지 뒤 head release): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 첫 실행이 `UNMEASURED · stop max-ticks`(load1 52)였다 — 지시대로 `--max-ticks` 를 올리자 둘 다 PASS · rc=0 · 27/27 · paints 79 / 55(유휴 범위 안).

### 6. `287af341dac8` — 엔진이 아니라 «첫 실행 재시작»

- 이미지에서 읽었다: 저장 로더가 `save0.data` 를 열고(모드 1) 없으면 만들어(모드 4) 30·30·20바이트를 쓴 뒤 화면 상태를 `0x22` 로 둔다. 그 상태 칸이 «안전한 실행을 위해 완전히 종료후 다시 실행해 주세요»를 그린다. 어떤 키도 거기서 나가지 않고(OK·CLR·소프트키·5 각각 실측) 게임이 스스로 끝나지도 않는다 ⇒ `--relaunch` 가 못 잡는다.
- 0453 이 남긴 «`--relaunch 1` 뒤에도 같은 안내 · 원인 미식별»의 답이 이것이다. 엔진 결함이 아니고 실기에서도 같다 — 플레이어는 껐다 켠다.
- 처방: census 가 이 sha 만 프로브·장시간을 `--restart-at 4`(데이터베이스 유지)로 잰다(`RESTART_FIRST` · 진도는 자기 재시작이 따로 있어 제외). 이용자 문장 한 줄(«처음 실행하면 … 껐다가 다시 켜면 시작할 수 있어요»).
- 후: 로고 → 메뉴 → 새로하기 → 캐릭터 생성까지. head census 6축 ok ⇒ playable. progress 는 정책 v2 가 창 끝에서만 재시작하므로 그 600초를 안내 화면에서 보낸다 — P·P2 정체 590·590 ⇒ `stuck`. 배지는 그대로 «막힘»이고, 문장이 재시작을 안내한다.

### 7. 못 고친 것

| sha12 | 실측 | 크기 |
|---|---|---|
| `aa3fcba4598b` KTF | 로고 카드 paint 가 매 프레임 NPE(`java_throw` NPE 문자열 · 첫 칸 흰 채우기 직후). 호출부를 읽으면 `getfield`(카드의 배열 필드) → `aaload` 의 null 검사(`0x134cc8`)다. 게임 스레드는 살아 있고(`Uncaught` 0) 예외 없이 돈다. 그 필드를 채우는 코드가 왜 안 도는지는 못 찾았다 | M — 후속 제안 `2026-10-09-non-playable-engine-residue#p0` |

### 8. 재발 방지 — 이유 없는 비-playable 행

- `scripts/player-data.mjs`: `status` 가 `playable` 이 아닌 행이 `knownIssues_ko` 0개면 거부. selftest 2건(limited · not-yet). 이 브랜치 이전 main 에 걸면 **3건**(`8b899f410f5d` `83fc429f9cbe` `63332c51d514`).
- `scripts/playability-census.mjs` `report`: 같은 행을 만들지 않는다 — 축이 이유를 말하지 못하는 유일한 경우(장시간을 안 잼)에 그 사실을 한 줄로 쓴다(`UNMEASURED_KO` · `knownIssues()` · selftest 2건). 그래서 다음 `import` 가 검사에 막히지 않는다.

### 9. 검증

- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` · `cargo +beta clippy --all -- -D warnings`(rc=0 · beta 1.100 이 손대지 않은 Cargo.toml 9곳에 «unused dependency» 경고를 낸다 — 이 PR 과 무관) · `RUST_MIN_STACK=4194304 cargo test --all`(FAILED 0) 통과.
- `npm run build:wasm` rc=0 · `node scripts/check-engine-contract.mjs` 113 pass · `npm run audit` 통과 · `node scripts/player-data.mjs` · `--selftest` 28 · census `selftest` 64/64.
- 되돌리면 red(각각 되돌려 FAILED 확인 · 원상 green): 널 클립 갈래 제거 → 클립 시험 2 FAILED · `context_clip` 무력화 → `drawing_stays_inside_the_context_clip` FAILED · `GForm` 등록 제거 → kfc 시험 2 FAILED · `getGTextListener` 의 재사용 제거 → `g_text_field_…` FAILED · 수거 일정에서 비용 항 제거 → `timer_collection_…` FAILED · player-data 규칙 제거 → selftest 2건 «NOT rejected» · census 대체 문장 제거 → census selftest 63/64.

