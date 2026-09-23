## [2026-09-24] 도출 ABI 행 5개 호출부 재판정 — 반증 0 (wie-2026-09-23-aot-java-dataoutputstream-and-gregoriancalendar-anchors-adopt-p3)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 의 도출 행 5개를 호출부 관측으로 재판정했다. 계측은 일회용(5행 제거 + missing-entry 스텁에서 lr·r0..r3·호출부/호출자 코드 덤프 후 0 반환)이고 최종 diff 에는 toml 주석만 남는다.

| 행 | 근거 | 관측(LGT 100 `--inject`) | 판정 |
|---|---|---|---|
| Runtime 11 freeMemory()J | 도출 | 당신은골프왕 2회: 인자 레지스터 미세팅 · 결과 ÷1024 후 배열 저장 | 정합(freeMemory\|totalMemory 로 좁힘) · 미결 |
| Runtime 12 totalMemory()J | 도출 | 배틀몬스터 2회: 무인자 · 결과는 첫 명령에서 r0 를 덮는 함수로 | 미결(판별 불가) |
| Runtime 13 gc()V | 도출 | 배틀몬스터·학교가는길·체스마스터 22회: 무인자 · 호출부+래퍼 호출자 10곳 모두 r0 미소비 | 정합(void 모양) · 미결 |
| Thread 13 isAlive()Z | 도출(setPriority=14 앵커) | 디스패치 0회 | 미결(미관측) |
| String 19 startsWith(S)Z | 도출 | 훼밀리마트타이쿤 1회: r1 만 객체 · 결과 필드 저장 | 정합(15 16 19 20 중 · 18 배제) · 미결 |

부수 관측: 일지매영웅전기 split 루프가 String 21=indexOf(I)I · 27=substring(I) 를 규칙 위치 그대로 디스패치(각 24회).

**왜**: 표는 틀려도 파싱 오류를 내지 않는다. 게스트 vtable_count 축은 이 5행에 말하지 못한다 — Runtime 서브클래스 없음, String 은 final, Thread 크기(18)는 12↔13 순서를 가르지 못한다.

**사용자 영향**: 없음(행 변경 0). 게이트: fmt·clippy(stable/wasm/beta) green · `cargo test --all` 421 passed 0 failed · runner 6/6 PASS.
