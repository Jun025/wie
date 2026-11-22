## [2026-11-22] 게임 내용 지문 `contentKey` — 묶음 껍데기가 달라도 같은 게임으로 알아보는 키 · 지원 목록 칸 · 부팅 전 판별 함수 `identify()` (wie-featurephone-content-fingerprint-for-user-supplied-files)

**무엇을** — 지원 상태 · 조정 표 · 차단 규칙 변경은 없다.
- 지문 정의: 엔진이 그 파일에서 골라 실행하는 **본체 jar 의 MD5** 에 담긴 방식을 붙인 값. zip 꾸러미는 `zip:<32hex>`, jar 하나만 올린 파일은 `jar:<32hex>`.
- 계산 한 곳: 새 크레이트 `wie_identify` 의 `identify(filename, data)` 가 통신사 · LGT 컴파일 모델 · 지문을 답한다. 에뮬레이터를 만들지 않는다.
  jar 를 고르는 규칙은 통신사 크레이트의 `archive_jar()` 가 돌려주고, 그 안의 고르기 함수는 `from_archive` 가 쓰는 것과 같은 함수다(KTF · SKT `jar_filename` · LGT `archive_jar_entry`).
- 공개 칸: `docs/player-data/compat.json` 429행 전부에 `contentKey`. `scripts/content-key.mjs write` 가 네이티브 바이너리 `wie_identify` 로 코퍼스 파일에서 써 넣는다.
- 엔진 API: wasm 내보내기 `identify(filename, data)` → `{ platform, model, contentKey }` 또는 `undefined`. 받은 바이트는 읽기만 한다.
- 검사: `player-data.mjs` 가 형식 · 전 행 보유 · 장부의 jar 키와의 일치를 본다(게임 파일 없이 CI 에서). 파일과의 전수 대조는 로컬 `content-key.mjs check`.

**왜** — 이용자는 게임 파일을 각자 구해 온다(운영자 전제 2026-11-16). 셸은 꾸러미 전체 sha256 하나로만 지원 목록을 찾으므로, 안쪽이 같고 zip 만 다시 묶인 파일은 「목록에 없는 게임」으로 보였고 aot-java 판은 허용 목록에 없어 부팅이 막혔다. 엔진의 한 판 전용 조정 표는 본체 jar 의 MD5 로 걸리므로 엔진과 셸이 게임을 알아보는 단위가 달랐다. 이 회차는 그 단위를 엔진 쪽으로 맞춘다.

**사용자 영향** — 지금 화면에서 달라지는 것은 없다. 셸은 모르는 칸을 그대로 통과시키고 새 함수를 부르지 않는다. 셸이 이 지문으로 파일을 찾으면(otterpebble 쪽 티켓 `otterpebble-featurephone-recognize-same-game-by-content-fingerprint`) 다시 묶인 파일도 같은 게임으로 보인다.

### 1. 지문에 무엇을 넣을지 — 세 후보를 코퍼스 429행에서 쟀다

스크래치 스크립트(엔진의 꾸러미 읽기 규칙을 흉내 낸 독립 계산 · 출력은 sha12 와 수뿐)로 세 후보를 매겼다. 본체 jar 후보의 수는 뒤에 엔진이 낸 값과 같았다(§2).

| 후보 | 서로 다른 지문 | 지문이 같은 묶음 | 묶음에 든 행 | 묶음 안 `status` 가 갈림 | 묶음 안 `runtimeAdjustments` 가 갈림 |
|---|---|---|---|---|---|
| 본체 jar 만 | 408 | 21 | 42 | 0 | 0 |
| 설명 파일 + 본체 jar | 420 | 9 | 18 | 0 | 0 |
| 꾸러미의 전 파일(이름 + 내용) | 421 | 8 | 16 | 0 | 0 |

본체 jar 가 같은 21묶음의 안쪽:
- 8묶음은 꾸러미 안 파일이 전부 같고 zip 껍데기만 다르다.
- 1묶음(`704839f1a6c8`/`8bfd08fe4370`)은 설명 파일과 jar 가 같고 부속 파일만 다르다(사설 폴더의 5개가 내용이 다르고 3개가 한쪽에만 있다 · 그 밖 1개).
- 12묶음은 설명 파일이 다르다. 다른 칸은 전부 설치 칸 식별자 계열이다: `AID` · `Ad` · `MIcon` · `SIcon` · `Sc` 가 12묶음 모두, `MSG` · `from` 이 2묶음, `SLvl2` 가 1묶음. `MClass` · `PID` · `DisplaySize` · `Name` 이 다른 묶음은 0. 그 12묶음 중 9묶음은 부속 파일까지 같다.

고른 것은 **본체 jar 만**이다.
- 넣으면 못 알아본다: 전 파일 지문은 21묶음 중 8묶음만 남긴다. 이용자의 파일은 남의 폰에서 뜬 덤프라 저장 파일이 다른 것이 보통이고, 설치 칸 식별자가 다른 재등록본도 12묶음이나 있다.
- 빼서 잃는 것을 쟀다: 21묶음 전부 `status` 가 같고 `runtimeAdjustments` 도 같다. 축 값이 다른 묶음은 5개인데(`progress` 4 · `speed` 1) 그중 3개는 **꾸러미 내용이 완전히 같은** 묶음이다 — 축 차이는 측정의 흔들림이지 내용 차이의 증거가 아니다.
- 엔진의 단위와 같다: 장부(`docs/contracts/runtime-adjustments.tsv`)에서 `scope` 가 `zip` 인 40행의 `key` 가 바로 이 MD5 다. 그래서 그 40행(꾸러미 36개)은 게임 파일 없이도 CI 에서 대조된다.
- MD5 인 이유: 엔진 표의 키가 MD5 라 값이 그대로 맞물린다. 목록의 게임과 같은 지문을 만들려면 둘째 원상(second preimage)이 필요하고 MD5 는 그것은 아직 버틴다. 표가 긴 해시로 옮겨 가면 같이 옮긴다(`wie_identify/src/lib.rs` 의 `ponytail:` 주석).

남는 한계(공개 데이터 계약 §1 에 적었다): 지문이 같아도 ①AID 로 거는 `local-server` 는 설명 파일의 AID 가 다르면 걸리지 않는다(지금 그 6꾸러미 중 다른 행과 지문을 나눠 갖는 것은 0) ②부속 파일이 다르면 시작 상태가 다를 수 있다 ③맨 jar 는 길이 다르다(§5).

### 2. 수 — 엔진이 매긴 지문(`node scripts/player-data.mjs` 가 센다)

```
player-data OK — 429 titles · status {"playable":426,"not-yet":2,"limited":1} · platform {"KTF":269,"LGT":78,"SKT":82} · 182 updates · runtimeAdjustments on 49 rows (ledger 54 rows)
contentKey — 408 distinct on 429 rows · 21 shared by 42 rows · 0 shared with differing status
```

| | 행 | 서로 다른 지문 | 둘 이상이 나눠 갖는 지문 |
|---|---|---|---|
| KTF | 269 | 249 | 20 |
| LGT | 78 | 78 | 0 |
| SKT | 82 | 81 | 1 |
| 합 | 429 | 408 | 21(42행 · 묶음마다 2행) |

`null`(본체 jar 를 못 고름) 0행 · `jar:` 0행. 지문이 같은데 `status` 가 다른 묶음 **0** · 통신사가 다른 묶음 0 · `runtimeAdjustments` 가 다른 묶음 0.

| 지문이 같은 행(sha12) | 통신사 | 제목 | status | runtimeAdjustments |
|---|---|---|---|---|
| `070daa5b552c` · `0e6cd188729e` | KTF | 거상 | playable · playable | – |
| `182fa44210dc` · `91c02913e1ea` | KTF | 영웅전설4 주홍물방울 | playable · playable | – |
| `1cf2e6076079` · `5028b8a5d19f` | KTF | 리얼싸커2009 | playable · playable | – |
| `1d18b7cc06d1` · `a5651a180b7a` | KTF | 투스워즈 | playable · playable | – |
| `1fe6c1c897d7` · `d9384b388ea5` | KTF | 크로노스 윙 | playable · playable | – |
| `2f84b8cc870d` · `9e2307e4fa88` | SKT | 당골왕 / 당신은골프왕 | playable · playable | – |
| `3be2a19f5d3f` · `c01e041fa782` | KTF | 리얼비트 | playable · playable | – |
| `3d38a46becd9` · `44a4228cca51` | KTF | 추억의달고나 | playable · playable | – |
| `4120f27288ab` · `739c7657c1f2` | KTF | 던전앤파이터 격투가 / 던파격투가편 | playable · playable | – |
| `41466fc7f709` · `52f1f32e3f72` | KTF | 마이프린세스2 | playable · playable | – |
| `4892a1abc0f8` · `cdd3eb5f9142` | KTF | KBO프로야구2009 | playable · playable | – |
| `4a4d2ac046f7` · `f981d228b757` | KTF | 물가에돌튕기기3 | playable · playable | – |
| `4b1c1947950e` · `eda7ecb85ae3` | KTF | 마린블루스 | playable · playable | – |
| `5bfe10a82849` · `ab64a56b2b44` | KTF | 붕어빵타이쿤3 | playable · playable | – |
| `603dd685d386` · `d5a86e344611` | KTF | 영어뇌습격 | playable · playable | – |
| `704839f1a6c8` · `8bfd08fe4370` | KTF | 지크 | playable · playable | owner-file-skip |
| `8b01d4ae6af4` · `d60c34ffc9f6` | KTF | 던전앤파이터 귀검사 / 던파귀검사2006 | playable · playable | – |
| `93d5b6b8ceb5` · `94a34abdea21` | KTF | 이노티아1 | playable · playable | – |
| `974e0df9ab1e` · `f770b15f8876` | KTF | 이노티아연대기2 | playable · playable | lock-lift |
| `c94d64777926` · `e9fac881e602` | KTF | 드래곤나이트EX | playable · playable | – |
| `d4188f8ef8c4` · `ed6ad7318ac9` | KTF | 던파거너편 | playable · playable | – |

부수 발견: 3묶음(`2f84b8cc870d`/`9e2307e4fa88` · `4120f27288ab`/`739c7657c1f2` · `8b01d4ae6af4`/`d60c34ffc9f6`)은 꾸러미 안 파일이 전부 같은데 지원 목록의 `game` 이 서로 다르다. 이름은 이 회차에서 고치지 않았다(범위 밖 · worklog 제안 1건).

### 3. 같은 계산인가 — 커밋된 값 · 네이티브 · wasm 을 429행에서 대조

`node scripts/content-key.mjs check`(코퍼스의 원본 파일을 읽기만 한다):

```
content-key check: 429 rows · 429 on disk
  committed contentKey == native: 429/429
  native == wasm identify() (contentKey · platform · model): 429/429
  row platform/model == identify(): 429/429
```

셋째 줄은 `identify()` 가 답한 통신사 · 컴파일 모델이 지원 목록 행의 `platform` · `model`(전수 조사가 부팅해서 적은 값)과 같다는 뜻이다.

CI 에 실린 것(게임 파일 없이): `wie_identify` 단위 시험 4개(고정물로 «엔진이 고른 jar 의 MD5» · 껍데기와 부속 파일을 바꿔도 같음 · jar 한 바이트로 달라짐 · 맨 jar 접두) · `check-engine-contract.mjs`(내보내기와 인자 수) · `contract-roundtrip.mjs` 시나리오 I(브라우저에서 부팅 뒤 `platform_kind()` · `lgt_compile_model()` 과 같은 값 · 맨 jar 는 Node 가 계산한 MD5 와 대조 · 호출 뒤 바이트 불변) · `player-data.mjs`(장부 jar 키 40행).

### 4. 개악 대조

**⑴ 계산 한 줄을 바꾼다** — `wie_identify/src/lib.rs` 의 `md5::compute(jar)` 를 `md5::compute(&jar[1..])` 로(첫 바이트를 빼고 해시). 네이티브 바이너리와 wasm 을 둘 다 다시 빌드했다.

```
== cargo test -p wie_identify
tests::skt_archives_and_files_no_loader_takes --- FAILED
tests::a_bare_jar_never_shares_a_key_with_its_archive --- FAILED
tests::the_key_is_the_md5_of_the_jar_the_loader_runs --- FAILED
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

== node scripts/content-key.mjs check --wasm <개악 빌드>          (네이티브 · wasm 둘 다 개악)
content-key check: 429 rows · 429 on disk
  committed contentKey == native: 0/429 (6e5a1f88f0c0 a7d1f26af3e1 bc94ba53677b e538cbb4e687 3185174d2121 5a0fa312a793 de00506611a5 f07cbc782828 070daa5b552c 0e6cd188729e …)
  native == wasm identify() (contentKey · platform · model): 429/429
  row platform/model == identify(): 429/429
rc=1

== node scripts/content-key.mjs check                              (네이티브만 개악 · wasm 은 원래 빌드)
  committed contentKey == native: 0/429 (…)
  native == wasm identify() (contentKey · platform · model): 0/429 (…)
rc=1
```

둘 다 개악했을 때 둘째 줄이 429/429 인 것이 «계산이 한 벌»이라는 뜻이다 — 한 줄을 바꾸면 두 입구가 같이 움직이고, 커밋된 값과의 대조(첫 줄)가 그것을 잡는다. 원복 뒤 다시 돌려 세 줄 모두 429/429 · rc=0.

**⑵ 지원 목록의 값을 손으로 바꾼다**(빌드 없음 · 끝 글자 하나)

| 바꾼 행 | `node scripts/player-data.mjs`(CI) | `node scripts/content-key.mjs check`(로컬) |
|---|---|---|
| 장부에 오른 꾸러미 `59263295de74` | red — `ledger line 87 (device-bound-saves 59263295de74): the engine table keys this package by jar 697ccf638df0, compat.json contentKey says "zip:697c…d580"` | red |
| 장부에 없는 꾸러미 `070daa5b552c` | **통과**(묶음 수만 `409 distinct · 20 shared` 로 바뀐다) | red — `committed contentKey == native: 428/429 (070daa5b552c)` · rc=1 |
| 한 행에서 칸을 지움(`e538cbb4e687`) | red — `compat: 1 rows have no contentKey (e538cbb4e687)` | – |

둘째 행이 §7 의 한계다: 장부에 없는 행의 값은 게임 파일이 있어야 사실과 대조된다.

### 5. 맨 `.jar` 로 올렸을 때

지원 목록 429행의 파일은 전부 zip 이다. 각 행의 본체 jar 를 스크래치(repo 와 코퍼스 밖)에 꺼내 jar 하나만 올린 것처럼 `wie_identify` 에 물었다(사본은 측정 뒤 지웠다).

| | 수 |
|---|---|
| 꺼낸 jar | 429 |
| 지문이 `jar:` 로 시작 | 429 |
| `:` 뒤 32자가 그 행의 지문과 같음 | 429 |
| 지문 문자열 전체가 그 행과 같음 | **0** |
| 답한 통신사가 그 행과 같음 | 425 |
| 답한 컴파일 모델이 그 행과 같음 | 429 |

- 즉 같은 게임을 jar 만 올리면 지문은 `jar:<같은 32자>` 다. 지원 목록의 어느 행과도 같은 문자열이 되지 않는다(의도). 소비자가 접두를 떼고 비교하면 «목록의 본체를 꾸러미 없이 올린 것»은 알 수 있다.
- 통신사가 달라지는 4개: SKT 2(`1367261bc3ee` · `e3276ce8557c` — jar 머리에 SKT 표식이 없어 맨 jar 로는 J2ME 로더가 받는다) · KTF 2(`60bd6cbc5936` · `dab2d537f3ef` — 안쪽 jar 가 깨진 `not-yet` 2행).
- 그때 걸리지 않는 조정(현행 범위 · 장부 `scope` 열): `zip` 40행(꾸러미 36개)은 `from_archive` 에서만 불리므로 **전부 걸리지 않는다.** `any` 8행(꾸러미 8개 · 바이너리 · 클래스 MD5 키)은 걸린다. `aid` 6행(꾸러미 6개)은 파일 이름(확장자 제외)이 AID 와 같을 때만 걸린다.
- 부팅 자체도 길이 다르다: 맨 jar 길에서 호스트는 설명 파일이 없으므로 진입 클래스를 넘기지 않는다(`main_class_name = None`).
  커밋된 고정물에서 jar 만 꺼내 `wie_validate --timeout 5` 로 돌리면: `helloworld_ktf` 의 jar → `FAIL`(`Main class not found` · KTF 와 SKT 로더는 진입 클래스가 없으면 시작하지 못한다 — `wie-ktf/src/emulator.rs` `start` · `wie-skt/src/emulator.rs` `do_start`) · `helloworld_lgt` 의 jar → `PASS`(LGT 는 `binary.mod` 에서 바로 뜬다). zip 으로는 둘 다 `PASS` 다.
  그래서 지원 목록 행의 상태는 맨 jar 에 옮겨 가지 않는다.
- 이 경로의 동작은 바꾸지 않았다(범위 밖). 지문과 통신사 판별만 답한다.

### 6. 판단한 것

- **모든 행이 칸을 갖게 했다**(`null` 은 «본체 jar 없음»). 소비자에게는 선택 칸이지만, 커밋된 목록에서 칸이 빠진 행은 «잊은 것»과 «없는 것»을 가를 수 없어서다. `player-data.mjs import` 는 지문을 만들지 않으므로 가져온 뒤 `content-key.mjs write` 를 돌려야 검사가 통과한다(계약 §4).
- **맨 jar 는 접두를 달리했다.** 본체가 같아도 설명 파일 없이 뜨고 zip 전용 조정이 안 걸리므로, 꾸러미 행과 같은 문자열이 되면 소비자가 상태를 그대로 물려줄 수 있다. `:` 뒤 32자가 같으면 «목록의 본체를 꾸러미 없이 올린 것»이라는 것은 알 수 있다.
- **새 크레이트를 뒀다.** 통신사 판별 순서와 jar 고르기는 통신사 크레이트 위에서만 한 번에 쓸 수 있는데, 그 위에 있는 것은 호스트 둘뿐이고 `wie_featurephone` 은 wasm 밖에서 빈 라이브러리여야 한다(Constraint 7). 네이티브 도구와 wasm 이 같은 함수를 부르게 하는 가장 작은 자리가 이것이었다.
- **부팅 경로의 판별 순서는 건드리지 않았다.** `identify` 는 호스트 로더와 같은 술어를 같은 순서로 부른다. 순서 자체는 여전히 호스트마다 한 벌씩 있다(종전부터 3벌) — 어긋나면 §3 의 셋째 줄과 시나리오 I 이 본다.
- `wie_web.js` 는 40,773 B 로 상한 42,000 B 의 97.1% 다(이 회차 +914 B). 상한은 올리지 않았다. 다음 내보내기는 상한 조정이 필요할 수 있다.

### 7. 한계

- 코퍼스 안의 «본체가 같은 묶음»은 21개뿐이고 전부 `playable` 이다. 남의 덤프에서 부속 파일 차이로 상태가 갈리는 빈도는 이 자료로 잴 수 없다.
- 전수 대조(§3)는 로컬 전용이다. 코퍼스가 없는 CI 는 장부에 오른 36꾸러미의 지문만 사실과 대조한다 — 나머지 행의 값을 손으로 바꾸면 형식만 맞으면 CI 는 통과한다(로컬 `check` 가 잡는다).
- wasm 쪽 대조는 Node 에서 글루를 불러 했다. 브라우저에서 본 것은 고정물 3개(시나리오 I)다.

### 8. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 바꾼 파일 전체가 대상) ⇒ BOUNDED 410종 · SUFFIX-ATTACHED 16종.
수가 큰 까닭은 `docs/player-data/compat.json`(330종)과 장부(37종)가 바꾼 파일에 들기 때문이다. 두 파일의 제목은 이미 공개된 것이고 이 PR 이 그 파일에 더한 줄은 `contentKey` 429줄과 장부 머리 주석 한 줄뿐이다(제목 0).
이 PR 이 새로 적은 제목은 이 문서 §2 표의 20종과 「지크」 언급이다. 전부 `compat.json` 에 이미 있는 표시 이름 그대로다. `scripts/player-data.mjs` · `wie-lgt/src/emulator.rs` · `wie_featurephone/src/lib.rs` 에서 잡힌 제목은 종전부터 있던 주석과 표이고 이 PR 이 더한 줄에는 없다.
SUFFIX-ATTACHED 16종은 `compat.json` 안의 더 긴 다른 제목과 안내 문장 속 낱말이고, 이 PR 이 더한 줄에서 나온 것은 없다.
게임 바이트 · 꾸러미 안 파일 본문 · 경로 유입 0. 새로 실린 해시는 본체 jar 의 MD5(지문)이고, 그중 장부에 오른 것은 엔진 표에 이미 있던 값이다.

### 9. 게이트② 반려 R1 — jar 가 둘 이상인 꾸러미 (`wie-featurephone-content-fingerprint-for-user-supplied-files-fix`)

검수가 `archive_jar` 에 넣은 개악 둘(KTF → 첫 번째 `.jar` · SKT → 마지막 `.jar`)이 `cargo test` 에서 살아남았다. 시험 고정물이 전부 jar 1개짜리였고, 코퍼스에서 jar 가 둘 이상인 꾸러미는 2행(둘 다 SKT)뿐이다.

- **시험**: `wie_identify` 에 `with_several_jars_the_key_is_the_one_the_loader_runs` — 손으로 짠 합성 꾸러미 다섯 형상(게임 파일 0).
  KTF `AID:B` + `A.jar` + `B.jar` → `B.jar` · KTF AID jar 없음 + jar 1개 → 그 jar · KTF AID jar 없음 + jar 2개 → 키 없음 · SKT `a.jar` + `g.msd` + `g.jar` + `z.jar` → `g.jar` · LGT `binary.mod` 없는 `a.jar` + 있는 `b.jar` → `b.jar`.
- **구조**: KTF · SKT 도 LGT 처럼 묶었다. 종전에는 `from_archive` 와 `archive_jar` 가 같은 이름 함수를 각자 부른 뒤 각자 `files.get` 했다. 이제 고른 항목(이름 + jar)을 돌려주는 함수 하나(KTF `archive_jar_entry` · SKT `archive_entries`)를 양쪽이 읽는다.
  로더가 실행하는 줄은 같은 이름 계산 · 같은 `get` 이다(SKT 의 설명 파일 고르기도 같은 `find`). `from_archive` 가 그 함수를 거치므로 위 시험이 보는 jar 가 곧 로더가 실행하는 jar 다.
- **개악 대조**(`cargo test -p wie_identify -p wie-ktf -p wie-lgt -p wie-skt`): 검수의 둘 — KTF `archive_jar` → 첫 번째 `.jar` rc=101 · SKT `archive_jar` → 마지막 `.jar` rc=101. 묶은 함수 쪽 둘(로더와 지문이 함께 틀어지는 형상) — KTF rc=101 · SKT rc=101. 넷 다 새 시험이 red, 되돌리면 rc=0.
- **동작 0 의 증거**: `content-key.mjs check` — 커밋된 값 == native 429/429 · native == wasm 429/429 · 행의 통신사/모델 == identify 429/429(wasm · native 를 이 커밋에서 다시 지은 뒤). `compat.json` diff 0. 러너 블록 6종 PASS.
- 문서: 계약 §1 `contentKey` 절에 «같은 게임 알아보기용 · 무결성·보안 판정에 쓰지 않는다», §4 갱신 순서에 `write` 뒤 `check` 와 그 한계(장부 밖 393행은 다른 행의 지문으로 바꿔치기해도 CI rc=0).
- 남긴 것: 지문이 같은 묶음의 `status` 가 갈려도 검사기는 rc=0 이다(red 면 참인 데이터를 막는다). 「갈린 묶음은 상태를 물려주지 않는다」는 셸 쪽 요건이다.

<!-- corpus-name-inflow v1 subjects=23 tree=dab29c8f94c6abf0 B=1156/410 P=7/5 S=52/16 -->
