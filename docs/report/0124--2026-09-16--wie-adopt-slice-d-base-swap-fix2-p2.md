## [2026-09-16] 브라우저 부팅 축 — 별 회차는 필요 없고, resize·font 는 «어느 쪽으로도» 검증되지 않는다 (wie-adopt-slice-d-base-swap-fix2-p2)

### 무엇을

채택 제안 `2026-09-16-slice-d-base-swap-fix2#p2`(원문 =
`docs/worklog/2026-09-16-slice-d-base-swap-fix2.json` `proposals[2]`)를 집행했다.
그 제안은 **판단을 요구한다** — 자기 `tradeoff` 가 그렇게 적었다:
「CI 는 이미 `engine-contract` 의 `engine` 필터 안에서 그것을 돌리므로 «이 PR 이 착지하면 자동으로 답이
나온다» — 그래서 별 회차가 아니라 이 PR 의 CI 를 읽는 것으로 족할 수 있다. ★**어느 쪽인지가 이 제안의
판단 대상이다**」.

**판단 둘. 코드 0줄.**
1. ★**별 회차는 필요 없다** — 부팅 축은 CI 가 **이미 답했다**(측정으로 확인).
2. ★★**그러나 제안이 약속한 것은 «어느 쪽으로도» 얻어지지 않는다** — 어떤 시나리오도
   `Screen::resize` 와 `Platform::font()` 에 **닿지 않기** 때문이고, 그 공백은 **호스트가 아니라 픽스처**에 있다.

산출물은 그 판단과, 그것을 **시나리오 목록 바로 옆**(`scripts/contract-roundtrip.mjs` 머리)에 박은 주석이다.

### ⑴ 부팅 축 — CI 가 이미 답했다

★**「검사가 success」를 「검사가 돌았다」로 읽지 않았다** — AGENTS.md 가 경고하는 그 형태
(`Reporting success without rebuilding`)를 배제하려고 **스텝 단위**로 조회했다.

`gh api repos/Jun025/wie/actions/jobs/104720234942`(PR **#161** head `f6fe7839` · 08:23:12→08:30:32Z):

| 스텝 | 결과 |
|---|---|
| `Detect engine-relevant changes` | success |
| `No engine-relevant changes — skipping contract check` | ★**skipped** |
| `Contract check — static surface` | success |
| ★`Contract check — browser boot round-trip` | ★**success** |

건너뛰기 스텝이 **skipped** 라는 것이 곧 필터가 발화했다는 뜻이고, 라운드트립이 **실제로 돌았다**.
`f6fe7839` 는 `Screen::resize`·`Platform::font` 를 `wie_featurephone` 에 **이식한 바로 그 커밋**이다.

★**일회성이 아니다**: 같은 두 스텝을 **오늘** PR #166 head `5af3df8c` 의 `contract` 잡(`104844473745`)에서
재확인했고 **동일**(skipped / success)하다. ⇒ `.rs` 를 만지는 PR 마다 자동으로 답이 나온다.

⇒ **로컬 Playwright 회차를 따로 돌릴 이유가 없다** — 같은 스크립트·같은 시나리오이고,
로컬은 chromium fetch 비용만 더 든다. ★그래서 이 회차는 그것을 **돌리지 않았다**(§한계에 명시).

### ⑵ 그런데 제안의 `userBenefit` 은 거짓이다 — 둘 다 닿지 않는다

제안: 「이번에 이식한 `Screen::resize`·`Platform::font` 가 **실화면에서 맞는지가 처음으로 검증된다**」.
★**검증되지 않는다.** 두 축을 각각 쟀다.

**`Screen::resize` — 호출부 2곳, 오늘 둘 다 죽어 있다.**
- `wie-ktf/src/emulator.rs:71` — `if let Some((width, height)) = adf.display_size` 일 때만 돈다.
  ★그리고 **`Err` 를 `tracing::warn!` 으로 삼킨다** — 실패해도 부팅이 계속되므로, 설령 닿더라도
  라운드트립은 **초록**이다.
- `wie-lgt/src/runtime/wipi_c/graphics.rs:195` — PR #161 이 **배선을 끊은** 그 27줄(⒝ 결정) 안에 있다.

그리고 **커밋된 두 KTF 픽스처의 `__adf__` 에 `DisplaySize:` 줄이 없다**(`AID`/`PID`/`MClass` 3줄뿐).

★**양방향으로 쟀다 — 「0」이 «못 잰 0»이 아님을 보이려고**:
`wie_validate` 의 `HeadlessScreen::resize` 에 임시 `eprintln!` 을 두고,

| 입력 | `PROBE-RESIZE` |
|---|---|
| 커밋 픽스처 `test_data/keydraw_ktf.zip` | **0건** |
| ★같은 픽스처에 `DisplaySize:176*220` **한 줄만** 추가한 사본 | **1건** (`176x220`) |

⇒ 계측기는 살아 있고 **공백은 실재한다**. 프로브·사본은 전부 제거했다(`git status wie_cli/` 빈 값).

**`Platform::font()` — 프로브 없이 판정된다.**
엔진의 호출부는 **전부 `wie_midp`**(lcdui 텍스트)이고 이 픽스처들 중 문자열을 그리는 것이 없다.
★증거는 공짜로 이미 있다 — **`wie_validate` 의 `font()` 가 `unimplemented!()`** 라 닿으면 패닉한다.
러너 5픽스처 전건이 통과한다(`not implemented` **0건**):

```
draw_j2me.jar        PASS rc=0        helloworld_ktf.zip  PASS rc=0        helloworld_lgt.zip PASS rc=0
keydraw_ktf.zip --inject --expect-last-frame  PASS paints 55 rc=0
keydraw_lgt.zip --inject --expect-last-frame  PASS paints 55 rc=0
```

★**다만 «구성»은 덮인다**: `WebPlatform::new` 가
`Font::try_from_static(include_bytes!(".../assets/neodgm.ttf"))?` 를 **즉시** 평가하므로, 자산이 없거나
파싱이 깨지면 **부팅이 실패하고 Scenario A 가 red** 가 된다. ⇒ ★**적재는 검증되고 사용은 안 된다.**
이 구분을 흐리면 다음 사람이 「font 는 커버된다」로 읽는다.

### 왜 «주석 한 블록»이 이 회차의 자리인가

제안이 요구한 것은 판단이고, 판단의 수명은 **그것을 되밟을 사람이 읽는 자리**에 달렸다.
`scripts/contract-roundtrip.mjs` 머리는 **각 시나리오가 무엇을 단언하는지**의 정본이고,
「resize 시나리오를 넣자」고 생각한 사람이 **처음 여는 파일**이다. 그래서 거기에 넣었다.

★**AGENTS.md 에 같은 사실을 또 적지 않았다** — 그 파일은 이미 「`contract-roundtrip.mjs` 는
`engine-contract.yml` 에서 돈다」를 말하고 있고, 이 repo 의 `CLAUDE.md` 규율이
「**같은 사실을 두 곳에 적으면 한쪽이 낡는다**」다.

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가

- **잃는 것 ⑴ — 커버리지는 1비트도 늘지 않았다.** 이 회차는 공백을 **재고 적었을 뿐**이다.
  ★「측정했다」를 「고쳤다」로 읽으면 안 된다.
- **잃는 것 ⑵ — 주석은 썩는다.** 「호출부 2곳」·「배선이 끊겼다」·「`DisplaySize` 없음」은 **오늘의 사실**이고,
  누가 픽스처에 한 줄을 넣거나 LGT 배선을 되살리면 **아무 게이트도 울지 않은 채** 이 주석이 거짓이 된다.
  ★기계 가드를 두지 않았다 — 그것은 이 제안의 범위 밖이고, 가드 자체가 «공백을 단언하는» 이상한 시험이 된다.
- **잃는 것 ⑶ — 판단을 «하지 않는» 선택지를 닫았다.** 이제 「브라우저를 한 번 띄워 보자」는
  **기각된 것으로** 기록된다. 그 기각이 틀렸다면(예: 라운드트립이 CI 에서 조용히 skip 되기 시작하면)
  되밟아야 하고, 그 조건은 위 스텝 조회 두 줄이다.
- **안 하면**: ★같은 제안이 다시 채택된다 — 그리고 다음 회차는 **chromium 을 받아 라운드트립을 돌린 뒤
  «초록이니 resize·font 가 검증됐다»고 적을 것이다.** 그것이 이 회차가 실제로 막은 것이고,
  이 저장소가 반복해 규탄한 형태(「돌았다」를 「덮였다」로 읽기)다.

### 범위 밖 — 일부러 안 한 것

- **resize 시나리오를 만들지 않았다.** 그것은 「크기를 요구하는 픽스처」가 있어야 하는데, 커밋 픽스처는
  **바이너리**이고 Scenario E/F 가 **현재 기하에 정확 픽셀 수**를 단언한다 ⇒ 기존 것을 고치면 함께 깨진다.
  **별 픽스처 + 별 시나리오**여야 하고, 그것은 effort M 짜리 별 티켓이다(후속 제안 `#p0`).
- `contract-roundtrip.mjs` 를 **실행하지 않았다** — 위 ⑴이 「로컬 실행이 CI 보다 더 주는 것이 없다」를
  보였으므로 chromium fetch 비용을 쓰지 않았다. 부팅 축의 근거는 **CI 잡 스텝의 원격 조회**다.
- `npm run frontend` 도 돌리지 않았다 — 조각 D `-fix2` 의 한계가 그대로 남는다(TS 소비 코드 무접촉).
- 형제 `#p0`(PR #166 으로 착지 대기) · `#p1` · `#p3` **무접촉**.
