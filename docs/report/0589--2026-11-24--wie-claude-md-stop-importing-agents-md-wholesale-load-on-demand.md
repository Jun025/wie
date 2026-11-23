## [2026-11-24] `CLAUDE.md` 가 `AGENTS.md` 를 통째로 싣지 않는다 — 첫 행동 한 줄 · 제목 목록 · 절 색인으로 바꿨다 (wie-claude-md-stop-importing-agents-md-wholesale-load-on-demand)

**무엇을** — `CLAUDE.md` 첫 줄의 `@AGENTS.md`(통째 import)를 없애고 그 자리에 셋을 뒀다. ⑴ 세션 첫 행동: `AGENTS.md` 를 맨 위부터 «## Definition of Done» 제목 앞까지 읽는다. ⑵ Constraints 표 12행과 «Held by you» 5개의 제목 목록. ⑶ 「어떤 일을 하기 전에 어느 절을 읽는가」 를 적은 색인. 바뀐 파일은 `CLAUDE.md` 하나다(`git diff --numstat origin/main...HEAD` = `23 4 CLAUDE.md` · 6,843 → 10,665 B). `AGENTS.md` · 워크플로 · 엔진 · 웹은 고치지 않았다.

**왜** — 그 import 한 줄이 `AGENTS.md` 전문(64,875 B · 821줄)을 모든 세션의 기동 컨텍스트에 실었다. 조사만 하는 회차도, 문서 한 줄을 고치는 회차도 `wie_validate` 의 FAIL 판독 절차와 사건 대장까지 싣고 시작했다. 작성 회차의 기동 프로브로 52,369 → 29,835 tok(−22,534 · 전/후 1회씩)이다.

**사용자 영향** — 이용자에게 보이는 변화는 없다. 엔진 · 웹 · 공개 데이터를 건드리지 않았고 엔진 릴리스와 핀 범프도 서지 않는다. 달라지는 것은 이 저장소에서 일하는 세션이다. 약 22.5k 토큰 가볍게 시작하는 대신, 규칙 본문이 저절로 실리지 않으므로 색인이 가리킨 절을 읽지 않은 회차는 그 절의 규칙을 모르는 채 일하게 된다.

### 1. 무엇이 남고 무엇이 색인으로 갔나

- **`CLAUDE.md` 본문에 그대로(상시)**: 「완주 = 열린 PR 이지 머지가 아니다」 · 커밋 전 게이트 요약(`-D warnings` · wasm 타깃 · `RUST_MIN_STACK=4194304`) · `build-slot` 규율 · 게임 파일 무수정 원칙. 종전에도 `CLAUDE.md` 에 리터럴로 있던 줄이고 손대지 않았다.
- **제목만**: Constraints 12행 + «Held by you» 5개. 본문 · 근거 · 「Locked by」 칸은 옮기지 않았다. 판단은 제목이 아니라 `AGENTS.md` 의 그 절로 한다고 적어 뒀다.
- **색인**: 커밋 · PR 전과 엔진 코드 → «### The four gates» · `web/` `functions/` `scripts/` `migrations/` → «### Web-surface commands» · PR 을 내기 전(모든 회차) → «### Landing paperwork» · 브랜치와 PR → «### Git Workflow» · `-merge` 티켓 → «### For the `-merge` task only» 와 아래 2절의 세 줄 · `AGENTS.md` 에 fenced `sh` 를 더할 때 → «### Documented-command liveness» · `Cargo.toml` · required check · 버전 핀 → «## Incident ledger (사건 대장)» · 코드 스타일과 크레이트 지도 → «## Reference».
- 색인은 줄번호가 아니라 제목으로 가리킨다. 색인의 «» 제목 17개는 `AGENTS.md` 의 `##` ~ `####` 제목 17개와 하나씩 대응한다. 이 기록을 쓰는 시점의 `origin/main`(`de0969794` · #617 착지 뒤)에서도 17개가 각각 1건으로 맞는다.

명령을 `CLAUDE.md` 로 옮겨 적지 않은 까닭은 검사되지 않는 사본이 생기기 때문이다. four gates 의 명령 구간은 `wie_cli/tests/dod_ci_parity.rs` 가 `AGENTS.md` 와 `rust.yml` 을 대조하는데, `CLAUDE.md` 에 적은 사본은 그 대조 밖이라 조용히 낡는다.

### 2. 반려 1회 — 게이트③ 의무가 색인에서 닿지 않았다

첫 판(`e5a831c8`)의 `-merge` 줄은 «### For the `-merge` task only» 하나만 가리켰다. 그런데 착지시키는 회차의 의무는 그 절 밖에도 있다. 착지 뒤 운영 별칭 검증과 배포 red 의 소유는 «### Web-surface commands» 에, 주간 schedule red 의 소유는 «### The four gates» 말미와 «### Documented-command liveness» 에 있다. `-merge` 티켓만 받은 세션은 그 셋에 닿지 못했다(검수의 규율 유실 표 14행 가운데 「닿지 않음」 1행).

보완 커밋(`fa7f39a3`)이 `-merge` 줄 아래에 그 세 곳을 가리키는 세 줄을 달았다(「닿지 않음」 0행). 같은 커밋에서 방아쇠 셋을 더했다: 「PR 을 내기 전(모든 회차 — 회차 기록은 회차마다의 의무다)」 · 「`Cargo.toml` 을 만진다(주석만이어도 착지하면 릴리스가 선다)」 · 「회신에 게임 파일명 유입 수를 적는다(`corpus-name-inflow.mjs` 를 돌린다)」. 파일은 9,522 → 10,665 B 가 됐다.

### 3. 잰 것과 못 잰 것

- **기동 프로브**: 전 52,369 · 첫 판 29,263 · 보완 뒤 29,835 tok. 각 1회라 분산은 모른다. 목표 「30k 이하」 까지 여유가 165 tok 이라 `CLAUDE.md` 에 몇 줄만 더 붙어도 넘는다.
- **실제 회차의 절감은 프로브보다 작다.** 첫 행동으로 읽는 머리 42줄과 그 회차가 읽는 절이 다시 들어온다. 작성 회차의 추정으로 조사 · 머지 회차가 16 ~ 21k, four gates 와 Landing paperwork 를 둘 다 읽는 PR 회차가 약 7k 다. 측정이 아니라 추정이다.
- **못 잰 것**: 색인을 보고도 절을 읽지 않는 회차가 얼마나 나오는가. 이 PR 은 그것을 막지 못한다. 착지 뒤 게이트 누락이나 회차 기록 누락이 늘면 그것이 이 판단이 틀렸다는 증거다.
- **색인으로 돈 회차 1건**: 이 PR 을 착지시키는 `-merge` 회차의 세션은 PR 브랜치가 체크아웃된 트리에서 떠서 새 `CLAUDE.md` 를 실었다. 첫 행동(머리 42줄)을 읽었고, 색인을 따라 Landing paperwork · Git Workflow · For the `-merge` task only 와 Web-surface commands 의 유입 표식 항을 읽어 이 기록을 썼다. 한 건이고, 색인이 무엇을 하라는지 아는 티켓을 받은 회차라 일반화할 근거는 아니다.

### 4. 범위 밖에서 본 것

- `.github/workflows/engine-contract.yml:93` 의 주석 「AGENTS.md is loaded by every wie session」 은 이 착지 뒤 사실이 아니다. 주석이라 동작에는 영향이 없고, 워크플로는 범위 밖이라 고치지 않았다.
- 색인의 운영 별칭 줄은 별칭 주소와 스크립트 이름을 `AGENTS.md` 에서 옮겨 적은 사본이다. 주소가 바뀌면 이 줄이 낡는데 잡는 검사가 없다(doc-liveness 는 `AGENTS.md` 의 fenced 블록만 본다).
- 색인의 주간 schedule 줄은 「«### The four gates» 말미」 라고 위치로 가리킨다. 그 절 끝에 문단이 더 붙으면 어긋난다. `schedule` 로 찾으면 닿는다.

### 5. 확인

- `git diff --quiet origin/main -- AGENTS.md` rc=0(이 PR 이 `AGENTS.md` 를 고치지 않았다).
- #617(upstream 병합 · 321파일)이 검수 뒤 `main` 에 착지했다. 그 착지분은 `CLAUDE.md` · `AGENTS.md` 를 만지지 않았고 `git merge-tree --write-tree HEAD origin/main` rc=0 이다.
- 네 게이트와 `cargo +beta clippy --all -- -D warnings` 는 이 기록을 더한 트리(PR 브랜치 · #617 을 품기 전 base)에서 다시 돌려 다섯 다 rc=0 이다(`test result: ok` 58줄 · 실패 0). #617 과 합친 트리의 값은 PR 의 CI 가 잰다 — 그 값은 `-merge` 회신에 적는다.
- 게임 이름 유입(`corpus-name-inflow` · 대상 2파일): BOUNDED 0회 0쌍 · SUFFIX-ATTACHED 0회 0쌍.

<!-- corpus-name-inflow v1 subjects=2 tree=6ebf0787e7090789 B=0/0 P=0/0 S=0/0 -->
