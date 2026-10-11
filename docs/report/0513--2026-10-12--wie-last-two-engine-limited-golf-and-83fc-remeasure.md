## [2026-10-12] «엔진 탓 limited» 마지막 2종 재측 — 둘 다 playable (wie-last-two-engine-limited-golf-and-83fc-remeasure)

**무엇을**: KTF `7da00ecd4804` · `83fc429f9cbe` 를 현 `origin/main`(`e33440b8`)으로 다시 쟀고, compat 두 행을 `limited` → `playable` 로 바꿨다. 엔진 코드 변경 0.
**왜**: 두 행은 그 사이 착지한 엔진 수정(0467 레코드 목록 오름차순 · 0476/0477 재배치 이미지 진입) 뒤로 다시 재지 않은 채 남아 있었다. 비-playable 20 중 정책 잠금 18 을 뺀 «엔진 2» 가 이 둘이다.
**사용자 영향**: 두 게임이 «부분 지원» 에서 «지원» 으로 바뀌고, «멈추거나 꺼질 수 있어요» · «오래 플레이 검사를 아직 못 마쳤어요» 문장이 빠진다.

증적(스크래치 · 커밋 안 함): census out `L1`(probe + L + P + P2) · `L2`(probe + L). release `wie_validate` @ `e33440b8`. 측정은 `scripts/census-drive.sh`(phase 마다 `build-slot run --long` 임대 1건 · census 호스트 잠금) · 시작 시 `host-load-guard --status --recovered` rc=0 · load1 9~11.

### 1. 전/후

축 순서 boot·render·input·longplay·sound·speed·progress(o=ok · n=no · u=unknown · a=n/a · -=키 없음).

| sha12 | 전(`origin/main` compat) | L1 | L2 | 후(compat) |
|---|---|---|---|---|
| `7da00ecd4804` | limited `ooonoo-` | `ooooooo` | `oooooo-` | **playable** `ooooooo` |
| `83fc429f9cbe` | limited `ooouou-` | `ooooooo` | `oooooa-` | **playable** `ooooooo` |

- L(600초 · 키 루프): 두 회 모두 키 854/900 · `stop deadline` · FAIL 0. `7da00ecd4804` 그림 45,597/45,546 · `83fc429f9cbe` 217/218.
- 진도 600초: P = 둘 다 `ok`(p_stall 50 · 10) · P2 = 둘 다 PASS(`deadline`) · 축 판정 `ok`. `stuck` 이 아니니 P2 가 판정을 바꿀 길은 없었다.
- speed: `83fc429f9cbe` 비율 L1 0.900(ok) · L2 0.896(n/a) — 문턱 위아래로 갈린다. compat 은 P·P2 까지 든 L1 의 report 를 그대로 썼다. «느리다(`slow`)» 판정은 어느 쪽에도 없다.
- `7da00ecd4804` 은 0467 이 고친 «주소 8»(150초 base 9/18 FAIL)의 장시간 판정이다. 600초 × 4회(L 2 · P 2) 모두 끝까지 갔다.

### 2. `83fc429f9cbe` 의 L 에서 본 것 — 엔진 벽이 아님

L2 의 키별 화면 880장 중 키 69 뒤로는 두 장만 번갈아 나온다. 화면은 마을의 «무슨 일이세요? 1.회복 2.부활 3.기부 4.나가기» 메뉴이고, 고정 키 루프가 `NUM2`(부활 → «부활시킬 대상이 없습니다») 와 `OK`(메뉴로 돌아감)를 되풀이한다. 입력에 따라 화면이 바뀌고 예외 0 이다. 진도 정책(P)은 이 메뉴를 빠져나가 `ok` 다. 즉 L 의 정체는 측정 키 루프의 한계다.

### 3. 퇴행

- 엔진 코드를 바꾸지 않았다. compat 은 이 두 행만 바뀐다 — 라이브 등재분·가드(`49ade89578c5` `ddd885583b15`)를 포함한 다른 행은 손대지 않았다.
- `node scripts/check-compat-revert.mjs`: OK(아래 PR 본문에 «착지 기준 바뀐 행» 인용).
- `node scripts/player-data.mjs` 통과.

### 4. 게이트

- 4종(fmt · clippy stable · clippy wasm32 · `RUST_MIN_STACK=4194304 cargo test --all`) — PR 본문에 결과.
