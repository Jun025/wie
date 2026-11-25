## [2026-11-25] ARM 해석기 — Thumb 명령 해독을 256칸 표 한 번 찾기로 (`arm32_cpu` 0.1.2 를 `patches/` 로 들임) (wie-arm-interpreter-thumb-decode-single-table-lookup-vendor-arm32-cpu-into-patches)

**무엇을**: 외부 크레이트 `arm32_cpu` 0.1.2 의 소스를 `patches/arm32_cpu` 로 들이고(`[patch.crates-io]` + 워크스페이스 멤버),
Thumb 해독(`Instruction::decode`)을 21줄 표 순차 대조에서 상위 바이트로 256칸 표를 한 번 찾는 것으로 바꿨다. 옛 해독은
시험 안에 남겨 가능한 Thumb 워드 65,536개 전부를 새 해독과 대조한다. 패치가 빠진 빌드는 컴파일되지 않는다(§3).

**왜**: 운영자 채택 제안 `2026-11-22-arm-interpreter-throughput-step1#p0`(0575 §5 ⒝). 해독이 실행 루프의 가장 큰 한 덩이였다.

**사용자 영향**: 속도뿐이다. KTF · LGT 전 판이 같은 해석기를 타므로 화면이 복잡한 장면이 빨라진다 — 네이티브 성 대화
(LGT `3cc7a9b4cb15`)에서 프레임당 해석 루프 22.37 → 16.99 ms(−24.1%). 그리는 내용 · 판정 · 지원 목록은 그대로다.
**고지 판단**: 한다 — 속도 변화는 종전에도 `kind: "speed"` 로 알렸다(#592 · #605). `docs/player-updates/2026-11-25-heavy-scenes-run-faster.json`.

### 1. 착수 전 다시 잰 것 (할 일 0 · upstream 본 병합 뒤의 main `933bf58da`)

| | 0575 때(`76cc27c57`) | 지금 main(`933bf58da`) |
|---|---|---|
| `wie-core-arm` 의 요구 버전 | `^0.1.0` | `^0.1.2`(병합이 올렸다) |
| `Cargo.lock` | 0.1.2 | 0.1.2 |
| `[profile.web.package.arm32_cpu]` · `[profile.dev.package.arm32_cpu]` | 없음 | 있다(`opt-level` 3 · 1) — 경로 패치에도 그대로 걸린다(패키지 이름이 키다) |
| 프레임당 실행 루프(성 대화 · 3회) | 20.68 · 20.81 · 21.50 → 21.00 ms | 23.25 · 21.69 · 22.17 → **22.37 ms** |
| 해독 몫(루프 표본 대비) → 프레임당 | 24.8~26.3% → 5.35 ms | 22.6~23.8% → **5.14~5.28 ms** |

- **upstream 새 릴리스는 같은 변경을 싣지 않았다.** crates.io 의 `arm32_cpu` 0.1.3(2026-11-24 · `d97ffa9b`)은 Thumb-2 `B.W` 를
  더했고 해독 함수는 0.1.2 와 한 글자도 다르지 않다(두 타르볼의 `fn decode` 를 `diff` — 같음). 그래서 «버전만 올리기»는 길이 아니다.
  0.1.3 으로 옮기는 것은 종전 `Undefined` 이던 워드가 분기가 되는 동작 변경이라 이 회차에 섞지 않았다.
- 해독은 여전히 루프의 가장 큰 한 덩이였다 — 전제는 섰다.

### 2. 바꾼 것

- `patches/arm32_cpu/` — 발행된 타르볼(잠금이 가리키던 그것 · `dlunch/arm32_cpu@7488118a`) 그대로에 변경 넷. 소스 2,312 → 2,313줄.
  1. **해독 표**(`src/thumb.rs` · 약 40줄): `Nop`(`0xbf00` 한 워드) 말고는 21줄의 마스크가 전부 상위 8비트 안이다. 상위 바이트마다
     «`INST_MATCH_ORDER` 에서 처음 맞는 줄»을 컴파일 때 `const` 표로 만들고, `decode` 는 그 칸을 읽은 뒤 `0xbf00` 만 따로 본다.
     표는 옛 표와 옛 패턴에서 **계산**하므로 손으로 적은 256칸이 없다.
  2. 선택 기능 `serde-serialize` · `advanced_disasm` 과 그 `cfg` 블록을 뺐다 — 아무도 켜지 않는데, 멤버가 되면 그 의존성
     (`capstone` 0.6 의 C 라이브러리 · `serde-big-array`)이 `Cargo.lock` 에 들어와 `cargo audit` 과 `coverage.yml --all-features` 에 걸린다.
  3. `src/lib.rs` 머리의 `#![allow(...)]` 한 줄 — 타르볼 자신의 지적 7종(rustc `deprecated` 1 + clippy 6). 레지스트리 크레이트는 린트되지 않지만 멤버는 `-D warnings` 로 린트된다.
  4. `pub const WIE_PATCH: () = ();` — 아래 가드.
  - 타르볼의 시험(`tests/` 3파일 + 단위 30건)과 그 입력(`tests/data/*.bin` 23개 · 12~60바이트 · 같은 폴더의 `.S` 를 어셈블한
    upstream 의 시험 프로그램이다 — 게임 바이트가 아니다)도 같이 들였다. 종전에는 돌지 않던 upstream 시험 55건이 이제 `cargo test --all` 에서 돈다.
- `Cargo.toml` — `members` 에 한 줄 · `[patch.crates-io]` 절 신설(#605 가 지운 뒤 main 에 없던 절이다). `Cargo.lock` 은
  `arm32_cpu` 항목에서 `source` · `checksum` 두 줄이 빠진다.
- **관례를 적은 곳**(걸리는 것 ①): `patches/arm32_cpu/README.md`(출처 버전과 커밋 · 변경 · 타르볼과 견주는 명령 · 가드 · «버릴
  조건») · `docs/architecture.md` 의 디렉터리 표에 `patches/` 행 · `Cargo.toml` 의 절 머리 주석. 버릴 조건 = «순차 대조 없이
  Thumb 를 해독하는 `arm32_cpu` 릴리스가 나오면 디렉터리 · `members` 줄 · `[patch.crates-io]` 줄 · `WIE_PATCH` 줄을 지운다».
- `scripts/arm-interp-profile.mjs` — 해독 묶음의 이름을 소스에서 읽는다(`const DECODE_TABLE` 이 있으면 «256칸 표 찾기»).
  경계 판정은 그대로다(`selftest` ok).
- CI 경로 필터는 손대지 않았다 — `publish-artifact.yml` · `engine-contract.yml` 의 `**/*.rs` · `**/Cargo.toml` 이 `patches/` 를 이미 덮는다.

### 3. «패치가 실제로 쓰이는가» 가드 (걸리는 것 ②)

`wie-core-arm/src/engine/arm32_cpu.rs` 가 `const _: () = arm32_cpu::WIE_PATCH;` 로 그 표식을 이름으로 부른다. 표식은 들인
사본에만 있으므로, 레지스트리 판으로 돌아간 빌드는 **컴파일 오류**다 — 네이티브 · wasm · 시험 · 릴리스 빌드 어디서든.

티켓이 예로 든 «`Cargo.lock` 에 `source` 줄이 없음을 시험으로 검사» 대신 이것을 고른 까닭은 실측이다. 미적용 형상
(`wie-core-arm` 의 요구를 `^0.1.3` 으로 올림)을 만들어 보니:

```
    Updating crates.io index
     Locking 1 package to highest compatible version
      Adding arm32_cpu v0.1.3
error[E0425]: cannot find value `WIE_PATCH` in crate `arm32_cpu`
  --> wie-core-arm/src/engine/arm32_cpu.rs:17:26
```

- cargo 는 **경고를 내지 않았다**(`patch ... was not used` 가 없다). 들인 디렉터리가 워크스페이스 멤버라 그 패키지는 그래프에 남기 때문이다.
- 그때 `Cargo.lock` 에는 `arm32_cpu` 항목이 **둘**이다 — `source` 없는 0.1.2(멤버)와 `source` 있는 0.1.3(실제로 링크되는 것).
  «`source` 줄 없는 항목이 있다»를 보는 시험은 이 형상에서 초록이다.
- 시험은 시험이 도는 곳에서만 막는다. 발행 빌드(`build-wasm.sh`)는 시험을 돌리지 않는다.

### 4. 같음의 증명 · 개악 대조

- `thumb::test::decode_table_matches_linear_match_for_every_word` — `0..=0xffff` 전부에서 `decode` = 옛 `decode_linear`. 0.00 s.
  해독이 같으면 그 뒤 실행은 같은 코드이므로 판 단위 전수를 대신한다(0575 §6).
- **표 한 칸을 틀리게**(`table[0xd0] = Instruction::Branch`): red — `assertion left == right failed: 0xd000 · left: Branch · right: CondBranch`. 되돌림.
- **패치 미적용**: §3 의 컴파일 오류. 되돌림(`Cargo.lock` 포함).

### 5. 전/후 — 0575 의 자(네이티브 성 대화 · 녹화 창 안의 루프 표본 ÷ 프레임 수)

`scripts/arm-interp-profile.sh` · LGT `3cc7a9b4cb15` · 대본 `docs/keys/lgt-3cc7a9b4cb15-intro-castle.keys` · 시작 뒤 110초 · 창 8초 ·
두 빌드 release + 줄 표. 전 = main `933bf58da` · 후 = 이 브랜치 `973534619`. 2~3회째는 한 임대 안에서 후 · 전 · 후 · 전 · 후 로 번갈아
떴고(12:18~12:29 · 2분 남짓 간격), 전 1회째만 2시간 앞(10:18)이다 — long 슬롯이 그 사이 다른 작업에 잡혀 있었다.
여섯 실행의 110~130초 캡처는 기준 캡처(전 2회째 120초)와 픽셀의 0~2.2% 만 다르다(같은 화면 · 눈이 아니라 픽셀 대조로 확인했다).

| | 전 `933bf58da` | 후 `973534619` |
|---|---|---|
| 실행 3회의 load1 | 22.5 · 11.6 · 10.0 | 13.4 · 8.8 · 7.7 |
| 표본(전체 · 실행 루프 아래) | 5,716 · 3,139 / 8,497 · 3,406 / 8,421 · 3,481 | 7,670 · 2,694 / 8,578 · 2,632 / 8,387 · 2,574 |
| 창 길이 · 그 안의 프레임 수 | 8.90 s · 135 / 8.60 s · 157 / 8.62 s · 157 | 8.56 s · 154 / 8.65 s · 158 / 8.43 s · 153 |
| **프레임당 실행 루프 시간** | 23.25 · 21.69 · 22.17 → **평균 22.37 ms** | 17.49 · 16.66 · 16.82 → **평균 16.99 ms** |
| 명령당(÷ 프레임당 193.2만 명령 · 0575 §3) | 11.58 ns | 8.79 ns |

- **−5.38 ms/프레임 · −24.1%.** 전 3회와 후 3회를 어떻게 짝지어도 −19.4~−28.3% 이고, 바로 이웃한 네 짝(후1·전2 / 전2·후2 /
  후2·전3 / 전3·후3)은 −19.4 · −23.2 · −24.9 · −24.1% 다. 0575 의 어림(−4.4~5.5 ms · −21~26%) 안이다.
- **해독 묶음은 표에서 사라졌다**(전 22.6~23.8% → 후 0.0%). 다만 그 0.0 은 «해독이 공짜가 됐다»가 아니다 — 표 찾기가
  `execute_thumb` 안으로 인라인돼 표본이 이웃 줄에 찍힌다. 그 이웃이 녹화마다 다르다: 후 1회째는 «명령별 실행»(`thumb.rs:162` ·
  17.2%)에, 2 · 3회째는 «선행부»(`thumb.rs:141` · 17.6%)에 찍혔다(0575 §2 가 적은 미끄러짐 그대로다). 그래서 묶어서 읽는다:

| 프레임당 ms(3회) | 전 | 후 |
|---|---|---|
| Thumb 해독 + 선행부 + 명령별 실행 | 8.97 · 8.44 · 8.29 → 8.57 | 4.77 · 4.51 · 4.66 → **4.65**(−3.92) |
| 그 밖(레지스터 · `Cpu::step` · run 루프 · shim · 플래그 …) | 13.80 | 12.34(−1.46) |

- 줄어든 5.38 ms 가운데 3.92 ms 가 해독이 든 묶음이고 나머지 1.46 ms 는 손대지 않은 이웃 코드에서 빠졌다 — #604 때와 같은
  방향이다(표본 몫보다 실제 감소가 크다 · 0575 §2). 왜 이웃이 같이 빨라졌는지는 가르지 않았다.
- 루프가 프로세스 표본에서 차지하는 몫은 40.1~54.9% → 30.7~35.1% 다. 이 수는 러너가 타이머를 기다리는 몫과 부하로 움직이므로
  판단에 쓰지 않았다(스크립트 머리말).

### 6. 회귀 그물

측정 중이다(long 슬롯 대기) — 이 PR 의 다음 커밋이 채운다.

### 7. perf-watch compare

측정 중이다(long 슬롯 대기) — 이 PR 의 다음 커밋이 채운다.

### 8. 게이트

이 브랜치의 마지막 코드 형상에서(2026-11-25 12:33~12:35 · 앞선 두 번도 같은 값):

| 게이트 | rc |
|---|---|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --all -- -D warnings` | 0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | 0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | 0 — `test result: ok` 72줄 · FAILED 0 · `decode_table_matches_linear_match_for_every_word ... ok` |
| `cargo +beta clippy --all -- -D warnings` | 0 — PR 의 beta 다리는 빈 통과라 이 로컬 값이 beta 의 유일한 그물이다 |

- 러너 블록: `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS · `keydraw_ktf.zip` PASS rc 0(paints 79) ·
  `keydraw_lgt.zip` PASS rc 0(paints 55) · `text_j2me.jar` PASS.
- wasm: `npm run build:wasm` → `node scripts/check-engine-contract.mjs`(143 pass · 0 violation) → `node scripts/contract-roundtrip.mjs`
  (OK — boot round-trip matches the pinned featurephone contract) · `npm run audit` PASSED. export 표면은 그대로다(계약 파일 변경 0).
  산출물 크기 `wie_web_bg.wasm` 18,377,412 → 18,376,974 바이트(−438 · perf-watch 가 두 rev 에서 따로 지은 것).
- `node scripts/check-worklog-json.mjs` OK · `node scripts/check-docs-report-serial.mjs` OK(0598 · 0599 는 열린 PR #630 · #631 이 먼저 잡아 0600 이다).

### 9. 한계

- 기기 1대(M1 Max) · 장면 1개(네이티브 성 대화). 0575 §1 의 브라우저 저녁 마을 쌍(합성 세이브)은 다시 뜨지 않았다 — 0575 §5 의
  화면 효과(초당 16.0 → 17.9~18.2 화면)는 여전히 모형 값이다. 브라우저 축은 §7 의 고정 표본이 전부다.
- 전 1회째는 나머지 다섯 실행보다 2시간 앞서 떴다(load1 22.5). 그 실행을 빼도 전 21.69 · 22.17 → 21.93 ms, 후 16.99 ms 로 −22.5% 다.
- 프레임당 명령 수(193.2만)는 0575 가 센 값을 그대로 썼다. 이 회차에 다시 세지 않았다.
- smoke 는 «뜨고 그린다»까지만 본다. 해독의 같음은 §4 의 전수 대조가 증명이고 smoke 는 그 위의 실기 확인이다.
- ARM(비-Thumb) 해독은 그대로 순차 대조다(티켓 경계 · 성 대화에서 Thumb 99.69%).
- upstream 에 같은 변경을 내는 일은 하지 않았다(티켓 경계 · 저장소 밖 행위).
