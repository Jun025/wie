## [2026-09-26] 놈ZERO 빈 화면은 이미 #301 이 닫았다 — `0x130` 과 무관 · 코드 변경 0 (wie-lgt-nomzero-blank-frames-after-im-wall)

**무엇을** — 채택 제안 `2026-09-24-lgt-wipic-im-group-supported-modes#p0`(놈ZERO «only blank/uniform frames» 의 원인 가르기)의 대전제를 `origin/main` 에서 재측했다. 코드는 바꾸지 않았다.

**왜** — 이 제안은 `0252`(2026-09-24) 직후의 관측이다. 그 뒤 #301(`4d5d639b` · 2026-09-25 · `0266`)이 `MC_GRP_GET_FRAME_BUFFER_BPP` 를 LGT 접근자로 옮겼다. 그 PR 은 `놈ZERO` 를 포함해 흰 단색이던 4건이 FAIL → PASS 가 됐다고 적었다.

**사용자 영향** — 없다(이미 `main` 에 있다). 놈ZERO 는 이용안내 화면을 그리고 키 입력을 받는다.

## 재측 — `origin/main` `5434ab0a` · release `wie_validate` · load1 55~95

| 실행 | result | paints | distinct | nondominant | `MC_imHandleInput` 호출 |
|---|---|---|---|---|---|
| `--timeout 60` #1 | **PASS** | 174 | 3 | 4.1% | 1 |
| `--timeout 60` #2 | **PASS** | 156 | 3 | 4.1% | 1 |
| `--inject` (27/27 키) | **PASS** rc=0 | 54 | 20 | 33.2% | 1 |

- 제안의 수치(FAIL · paints 178/205)는 **재현되지 않는다**. 원인 축은 `0266` §1~§3 의 BPP 인자 오독이었다. 그 PR 의 짝지은 대조에서 ⒝ 는 1색 ×2, C 는 3색 ×2 였다.
- ★**`0x130` 은 빈 화면과 관계가 없다**: 세 실행 모두 `MC_imHandleInput(0x9d, 0x1f6)` 이 **정확히 1회** 부팅 중에 호출된다. 키를 27개 넣어도 호출은 늘지 않는다. no-op 인 채로 화면·입력 모두 PASS 다.
  ⇒ 계약 1(«IME 조합 필요 / 다른 축» 판정)의 답은 **다른 축(BPP) · 이미 수리됨**이다.
- 호출부 7곳 전수 판독은 하지 않았다. 판정은 실행 추적 한 번으로 이미 닫혔다(런타임 호출 1회 · 키 무관). 나머지 6곳은 이 타이틀의 실행 경로에서 도달하지 않는다.

## 처분

- 같은 처방: #301(merged)이다. 티켓 규약(「같은 처방 PR 이 있으면 회신에 적고 끝낸다」)과 AGENTS.md(「형제 PR 이 이미 푼 제안은 알아챈 회차가 닫는다」)에 따라 worklog 로 카드를 닫는다.
- 후속 제안 0.
