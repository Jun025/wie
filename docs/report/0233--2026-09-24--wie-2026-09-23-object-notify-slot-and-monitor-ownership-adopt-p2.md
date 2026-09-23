## [2026-09-24] 배틀몬스터의 «paints 3» 은 «이용안내 · 아무키나 누르세요!!» 에서 입력을 기다리는 것이었다 — 키 1개로 타이틀, 2개로 메뉴 (wie-2026-09-23-object-notify-slot-and-monitor-ownership-adopt-p2)

**무엇을**: 코드 변경 없음. 측정만 했다. 채택 제안 `2026-09-23-object-notify-slot-and-monitor-ownership#p2`
(「paints 3 에서 멈춘다 — 입력에 반응하는가」)의 전제를 재고, 입력을 주지 않은 실행과 키 1개 실행을 **같은 예산**으로 짝지어 쟀다.

**왜**: #261(`24a920c4`) 착지 후 배틀몬스터는 `PASS · content true` 인데 `paints 3` 이었다. 제안은 이것을
「굶은 게 아니라 그릴 것이 더 없어서」로 읽었고, 입력이 닿는지 모른다고 했다.

### ⒜ 전제 재측 — `main`(`7e40b2f4`) release `wie_validate`, 문서 명령 그대로
```
RUST_LOG=warn wie_validate "game_lab/broken/lgt/<배틀몬스터 두 파일>"
```
| 파일 | result | stop | ticks | paints | ms | ERROR |
|---|---|---|---|---|---|---|
| `배틀몬스터.zip` | PASS | **max-ticks** | 50,000,000 | 3 | 20,250 | 0 |
| `lgt 배틀몬스터.zip` | PASS | **deadline** | 42,063,352 | 3 | 20,064 | 0 |

★`max-ticks` 는 «일찍 끝났다»가 아니다. 두 파일 다 **약 20 s**(기본 `--timeout 20`)에 섰고, 한쪽은 tick 상한,
다른 쪽은 시간 상한에 먼저 닿았을 뿐이다. `ticks` 는 게스트가 막혀 있는 동안의 executor 공회전을 센다(AGENTS.md).
상한을 풀고 `--timeout 90` 으로 늘려도 `main`·#263 head 모두 **paints 3 · ERROR 0 · stop deadline** 이었다.

### ⒝ 3장 이후 게스트는 무엇을 하나 — ★입력 대기
기준 실행(무주입)의 5 s 간격 스크린샷 12장(0~55 s)은 **전부 바이트 동일**(md5 `411c7503`)이다. 그 화면은:

> `< 이용안내 >` 이 게임은 게임 이용 시 별도의 비용이 발생하지 않는 단독형 게임입니다. … **아무키나 누르세요!!**

⇒ 미구현·예외·타이머에서 멈춘 것이 **아니다.** 화면이 스스로 «키를 누르라»고 말하고 있고, 로그에 ERROR 가 0 이다.
§7 per-frame ABI 벽(`docs/lgt_abi.md` cp53~58)과는 관계없다.

### 주입 1회 — 무주입 기준과 짝지은 측정
`wie_validate` 에는 «키 N개만» 을 줄 수단이 없어서, **스크래치 전용 계측**(커밋하지 않음 · 13줄)을 붙인 빌드로 쟀다:
`WIE_INJECT_N` = 주입 스크립트를 앞 N개로 자른다 · `WIE_KEEP_TIMEOUT` = 데드라인을 주입 스케줄이 아니라 `--timeout` 으로 둔다 ·
`WIE_SHOT_EVERY` = 고정 시각 스크린샷. 패치 원문: `~/orchestrator/reports/evidence/wie-2026-09-23-object-notify-slot-and-monitor-ownership-adopt-p2/scratch-instrumentation.patch`.

```
WIE_INJECT_N={0|1} WIE_KEEP_TIMEOUT=1 WIE_SHOT_EVERY=5 RUST_LOG=warn \
  wie_validate --inject --boot-secs 20 --timeout 60 --max-ticks 100000000000 --shotdir <dir> "<배틀몬스터.zip>"
```
**예산 차이 = 0.** 두 실행 모두 데드라인 60.0 s(`ms` 60,1xx~60,4xx), tick 상한 무제한이다. 다른 것은 t = 20.3 s 에 `OK` 한 번을 누르느냐 하나뿐이다.
네 실행을 **동시에 기동**했다(load1 70~85).

| 빌드 | 주입 | result | input_steps | paints | distinct | last_frame_content | ERROR | 20 s 이후 화면 |
|---|---|---|---|---|---|---|---|---|
| `main` `7e40b2f4` | 0 | UNMEASURED (rc=2 · 무주입이라 정상) | 0 | **3** | 2 | true | 0 | 12장 동일 `411c7503` |
| `main` `7e40b2f4` | **1** | PASS | 1 | **36** | 143 | **false** | **1** | t25 타이틀 → t30~55 **흰 화면 6장 동일** `fec9f48b` |
| #263 head `216edeee` | 0 | UNMEASURED (rc=2) | 0 | **3** | 2 | true | 0 | 12장 동일 `411c7503` |
| #263 head `216edeee` | **1** | PASS | 1 | **165** | 382 | true | **0** | t25~55 **7장 전부 다름**(타이틀 애니메이션 · «PRESS ANY KEY») |

`main` 의 ERROR 1건 문면: `Uncaught exception in thread 1622147107: net.wie.WieError: Unimplemented: java/io/ByteArrayInputStream vtable index 13`
— 형제 PR **#263**(`skip(J)J` = 13 · 게이트③ 대기)이 고치는 바로 그 벽이다. 흰 화면 해시 `fec9f48b` 도 #263 회신·검수의 해시와 같다.

`lgt 배틀몬스터.zip`(#263 head, 같은 프로파일에 `--action-secs 8`): 무주입 paints **3** · 12장 동일 ↔ 키 1개 paints **205** · 뒤쪽 7장 전부 다름 · ERROR 0.

**키 2개면 메뉴**(#263 head · `WIE_INJECT_N=3 --action-secs 8`): 1번째 `OK` → 타이틀, 2번째 `OK` → **메인 메뉴**
(「1 새로하기 · 2 이어하기 · 3 환경설정 · 4 도움말 · 5 게임문의 · 6 랭킹보기 · 7 나가기」 · 커서 표시), paints 190 · ERROR 0.
스크린샷(육안 판정 · 전부 사람이 읽는 화면): 위 evidence 디렉터리의 `pr263_noinput_t055.png`(이용안내) ·
`pr263_1key_t035.0.png`·`t055.0.png`(타이틀) · `main_1key_t055.png`(흰 화면) · `pr263_3key_03_LSOFT.png`(메뉴).

### 판정
- **입력 전달 경로에 결함 없음.** 키가 이용안내 → 타이틀 → 메뉴로 게스트를 움직인다. 코드 변경 사유가 없어서 바꾸지 않았다(Contract 2).
- 제안이 말한 「paints 3 = 그릴 것이 더 없다」는 절반만 맞다. 더 그리지 않는 이유는 **입력을 기다리기 때문**이다.
  무주입 검증기 실행은 배틀몬스터에 대해 구조적으로 3 이상을 볼 수 없다.
- `main` 에서 키 1개 뒤에 멈추는 곳은 입력이 아니라 `ByteArrayInputStream vtable index 13` 이고, 그 벽은 #263 이 진다.
  #263 이 착지하면 그다음 벽은 #263 회신이 적은 **마을 진입 직후 `Invalid memory access; address: 0`** 이다
  (제안 `2026-09-23-bytearrayinputstream-skip-slot#p0` · 이 회차는 좇지 않았다).

**측정 위생**: 스크래치 worktree 두 개(`origin/main` · `origin/pr/263`)에 `CARGO_TARGET_DIR` 을 분리했고, 계측 패치는 두 트리에 같게 넣었다.
작업 트리·다른 세션의 트리는 건드리지 않았다. 게임 바이트는 이 repo·로그·PR 어디에도 들어가지 않았다(스크린샷은 repo 밖 `~/orchestrator/reports/evidence/`).

**사용자 영향**: 없음(문서만). 배틀몬스터 «플레이 가능» 판정선은 #263 착지 후 «마을 이후 벽»이 남은 상태다.
