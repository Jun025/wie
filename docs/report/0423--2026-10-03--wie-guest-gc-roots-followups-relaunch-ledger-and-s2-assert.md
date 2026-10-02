## [2026-10-03] 게스트 GC 루트 후속 — 재기동 장부 판정 · 한계 주석 · S2 단언 순서화 (wie-guest-gc-roots-followups-relaunch-ledger-and-s2-assert)

#454(0421) 검수의 비차단 지적 3건.

### 무엇을

- **m1 — 재기동 시 장부 오염: 재현 안 됨 → 무조치(한계 주석만).** `guest_roots.rs` 의 `BLOCKS`·`REGIONS`·`INSTALLED`
  는 코어 id(`Arc` 주소)로 키를 잡고 지우는 경로가 없다. 오염은 «옛 코어가 해제되고 그 주소를 새 코어가 받을 때»만 난다.
  임시 계측(커밋 안 함): `install` 에서 같은 id 의 기존 항목 수를, `ArmCoreInner::drop` 에서 해제를 찍고
  `wie_validate --restart-at 2 --timeout 6 test_data/keydraw_lgt.zip`(같은 프로세스 2회 부팅)을 2회 돌렸다.
  - 1회차 id `0x10733b590` · 2회차 id `0x902c85810` — **다른 id**, 2회차 `installed_before=false` · 같은 id 의 `regions=0`.
  - 장부에 코어 id **2개**가 남는다(옛 코어 블록 298개 잔존) — 옛 코어가 살아 있다는 뜻.
  - `ArmCoreInner::drop` 발화 **0회**(2회 실행 모두). ⇒ 재부팅된 옛 코어는 해제되지 않으므로(참조 순환) 주소가 재사용될 수 없다.
  웹 타이틀 전환도 같은 엔진 경로라 같은 결론. 이 안전은 «코어가 안 풀린다»에 기대므로 `ponytail:` 주석으로 그 전제와
  상향 경로(코어가 풀리게 되면 drop 시 장부 제거)를 적었다.
- **m2** — 게스트 malloc 블록이 스캔 밖이라는 한계를 모듈 주석 `ponytail:` 1줄로.
- **m3** — `contract-roundtrip.mjs` S2 의 `bodyAt < nodeAt`(ms 타임스탬프 엄격 비교)를 로그 위치 순서(`sf-body` 가 `node` 보다 앞)로.
  하네스가 모듈 해석을 body 도착에 걸어 두므로 노드가 같은 ms 안에 붙는다 — #454 첫 CI(run `37064078004` attempt 1)가
  `body 20192 ms · node 20192 ms` 동률로 RED. main 의 최근 `engine-contract` 40회는 40/40 success(동률이 났어도 그 회차들은 통과)
  — 실패 이력은 PR 쪽 1회(#454)뿐이다. 로그는 이벤트 순으로 push 되므로 위치 비교는 동률이 없다.

### 사용자 영향

없음(주석 · 시험 단언만).
