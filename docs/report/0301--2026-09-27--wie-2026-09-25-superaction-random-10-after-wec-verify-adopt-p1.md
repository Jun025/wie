## [2026-09-27] 슈퍼액션히어로 Random 10 — #320(setSeed(J)V) 착지로 이미 넘어갔다 · 다음 벽은 LGT import 0x5b (wie-2026-09-25-superaction-random-10-after-wec-verify-adopt-p1)

**무엇을**: 코드 변경 0. #320(`0dd3faab`)이 등재한 `java/util/Random` 10 = `setSeed(J)V` 가 슈퍼액션히어로의 벽도 넘기는지 재고 기록했다.

**왜**: 같은 칸(Random 10)을 두 PR 이 각자 채우면 이중 등재가 된다 — 형제 착지 뒤 먼저 잰다.

**사용자 영향**: 없음 — 슈퍼액션히어로는 여전히 균일색(colors 1). 게임 스레드가 한 칸 더 간다.

### 전/후 — release `wie_validate`, 3회씩 (loadavg 22~46)

| 트리 | 타이틀 | 옵션 | stop | input | paints | colors | 첫 치명 오류 |
|---|---|---|---|---|---|---|---|
| before `4aee5055`(#320 머지의 1부모) | 슈퍼액션히어로 | `--inject` | max-ticks ×3 | 6·6·6/27 | 3·3·2 | 1 | `Unimplemented: java/util/Random vtable index 10` (`i.run()V`) 3/3 |
| after `origin/main` `a741291f` | 슈퍼액션히어로 | `--inject` | max-ticks ×3 | 12·12·11/27 | 2·3·2 | 1 | `Fatal error: Unknown lgt java import: 0x5b` (`i.run()V`) 3/3 |
| before `4aee5055` | 월드장기체스 | `--timeout 20` | error ×3 | — | 0 | 0 | `Unimplemented: java/util/Random vtable index 10` 3/3 (ticks 2) |
| after `a741291f` | 월드장기체스 | `--timeout 20` | error ×3 | — | 0 | 0 | `Unimplemented: java/util/Vector vtable index 31` 3/3 (ticks 140,178 · 207,456 · 191,028) |

- 판정: 계약 ⒜ — 벽이 넘어갔다. 월드장기체스는 #320 회신(0287)이 적은 Vector 31 그대로(무회귀).
- 호출부 인자 워드는 재지 않았다: 칸 10 행을 가진 트리에서 그 오류가 0/3 이고, 그 행이 없는 트리에서 3/3 이라 판정에 불필요하다.
- 0x5b 는 `wie-lgt/src/runtime/java/interface.rs` `get_java_interface_method` 표에 없다(인접 등재: 0x57 MonitorExit · 0x61 StoreReferenceArray).
- 부수 관측: `java_exceptions` 3건 `InvalidRecordIDException: Record not found`(3/3, 전후 동일 — RMS 첫 실행 경로로 보임, 미판정).
- 게임 파일명 유입: 4건(BOUNDED) + 판단 필요 3건(SUFFIX-ATTACHED) — 전부 이미 앞 회차(0264·0287)가 적은 두 타이틀명.

<!-- corpus-name-inflow v1 subjects=2 tree=aa8f3b35e7aa8d07 B=9/4 P=0/0 S=4/3 -->
