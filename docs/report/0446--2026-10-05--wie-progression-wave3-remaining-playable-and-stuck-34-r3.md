## [2026-10-05] 진도 3차 (3회차) — main 빌드로 41종 측정 · 고친 2종 compat · KTF 블릿이 픽셀 연산을 무시하던 벽(마젠타 2종) (wie-progression-wave3-remaining-playable-and-stuck-34-r3)

2회차(`docs/report/0445` · PR #478)의 이어하기다. 남은 몫은 셋이었다. ⑴`chunk.ac`…`ai` 190종과 `chunk.ab` 의 미측정 4 · n/a 6 진도 측정 ⑵#475 가 고친 2종(`e68b1c8aef85` · `320a5360a0f3`)의 compat 행을 main 빌드로 다시 재기 ⑶엔진 벽 수정.
**부분 완료다.** ⑴은 `ab` 남은 몫 + `ac` 30종까지 쟀고 `ad`…`ai` 159종은 못 쟀다(§1). ⑵는 끝냈다. ⑶은 1군집(2종)을 고쳤다(§3).

### 1. 측정 — 측정 엔진 = 현 `origin/main` 빌드 `1bc1317c`

조건: `wie_validate` release 를 `1bc1317c`(#478 머지 직후 main)에서 빌드 · 정책 v2 600초 + 재기동 120초 · P 뒤 `p2list` 가 고른 P2 짝 · 스윕 «전체»를 `build-slot run --long` 임대 1개로 감쌌다(안쪽 실행은 맨 명령) · census 호스트 잠금 · 착수 전 `host-load-guard --status --recovered` rc=0. 후보 판정(boot·render·longplay)용 A·B·L 프로브는 이전 out 에서 가져왔다(이 회차는 그 축을 다시 재지 않았다).

| 단계 | 시각 | jobs | load1 (시작 → 끝) | idle (시작 → 끝) |
|---|---|---|---|---|
| P `r0`(11) | 16:52–17:40 | 3 | 7.5 → 6.3 | 23% → 18% |
| P2 `r0`(5) | 17:40–18:04 | 3 | 6.3 → 8.9 | 25% → 55% |
| P `ac`(30) | 18:04–20:04 | 3 | 8.6 → 5.2 | 68% → 61% |
| P2 `ac`(11) | 20:04–20:43 | 3 | 5.2 → 9.1 | 49% → 19% |

- 타이틀별 `load1`(census 열): `r0` 최소 9 · 중앙 12 · 최대 139 / `ac` 최소 9 · 중앙 14 · 최대 170.
- `r0` = 고친 2종 + `chunk.ab` 미측정 4 + n/a 6. n/a 6 은 2회차가 옛 빌드(`9b4eb62e`)로 P 만 잰 것이라 P 부터 다시 쟀다(빌드가 섞인 짝을 만들지 않으려고).
- 20:43 에 묶음 경계(`ac` P2 끝)에서 드라이버를 멈췄다. `ad`·`ae` 첫 실행 6개가 막 시작한 참이라 결과 파일이 없는 그 6개의 부분 디렉터리를 지웠다(미측정으로 남는다).

| 묶음 | 모집단 | 잰 수 | ok | stuck | error | n/a | 미측정 사유 |
|---|---|---|---|---|---|---|---|
| `r0`(고친 2 + `ab` 남은 10) | 12 | 11 | 6 | 5 | 0 | 0 | `6eb93824daf8` — longplay error 라 후보가 아니다(2회차와 같다) |
| `ac` | 30 | 30 | 20 | 8 | 2 | 0 | — |
| `ad`…`ai` | 159 | 0 | — | — | — | — | 회차 시간(`ag` 의 `320a5360a0f3` 은 `r0` 에서 쟀다) |
| **합** | 201 | 41 | 26 | 13 | 2 | 0 | |

- stuck 은 P2 짝이 같을 때만 stuck 이다(계약). `ac` 에서 P 가 stuck·error 이던 11종 중 1종(`2cbd63e4427a`, P stall 200)은 P2 가 움직여 ok 다. error 2종은 P2 에서 같은 시각(ms 까지)에 같은 오류로 죽었다.
- 고친 2종(할 일 2): `e68b1c8aef85` **ok**(stall 50 · 종전 `stuck`) · `320a5360a0f3` **ok**(stall 60 · 종전 키 없음). #475 의 고침이 main 빌드에서도 진도로 이어진다.

### 2. 막힘 계급(이 회차의 stuck·error 15)

| 계급 | 수 | sha12 | 근거(마지막 새 화면 · 600초 화면을 직접 봤다) |
|---|---|---|---|
| ⒜ 정책 한계 | 13 | `33801c1ba14f` `859aa864797b` `8d2828ab8a3f` `c181cae84146` `db8ef04a6504` `0c67145b11df` `6d63b4025bef` `ab64a56b2b44` `174237758542` `0cc4ef7ede37` `155586ece7f8` `8c6821fab969` `49f2734f17d8` | 능력치 분배창 · 요리 주문 · 조합창 · «#버튼을 눌러» 튜토리얼 · 메인 메뉴 정보 항목 · 이름 입력 · 펫 방 · 가게 주문 미니게임 2 · 사격장 · 능력치창 · 게임 문의(저작권) 화면 · 도시 건설 튜토리얼 글. 전부 화면이 정상으로 그려지고 정책 키가 그 화면을 못 넘는다 |
| ⒞ 통신 — 남김 | 1 | `30c7bd6fb01b` | 메뉴가 «게임빌 매니아에 접속하시겠습니까? 통화료가 부과됩니다»를 묻고, 정책이 «예»를 고르면 KTF Net 표 34번 슬롯이 `(r0 = "<서버>:<포트>" 문자열, r1 = 0xff)` 로 불려 `Unimplemented` 로 죽는다(P·P2 같은 40.8초). 슬롯 34 의 반환 규약을 모르므로 «안 죽게»만 만들지 않았다(§5) |
| ⒝ 엔진 벽 — 남김 | 1 | `96dc32e781d3` | `com/mc/b.paint` 안에서 `Invalid memory access`(112.8초 · P·P2 같다). 2회차 퇴행 확인 8종 중 하나다 |

### 3. ⒝ 수정 — KTF 블릿이 컨텍스트의 픽셀 연산을 무시했다

**증상.** 2회차가 «렌더 벽 — 남김»으로 둔 KTF 2종(`46b2238f87a6` · `7e2247bdf565`)이 `wie_validate` 렌더 검사 FAIL «magenta color-key not applied»(화면의 17% · 62%)로 끝났다. 진도가 아니라 그 FAIL 때문에 progress 가 `stuck` 이었다.

**원인.** 두 타이틀 다 `MC_grpSetContext(ctx, PIXELOP, proc)` 으로 게임 자신의 픽셀 연산 함수를 컨텍스트에 걸고, 그 컨텍스트로 `MC_grpDrawImage` 를 부른다(`46b2` 는 글자 조각 그림 600번 · 디버그 로그). KTF 의 `MC_grpDrawImage`/`MC_grpCopyFrameBuffer` 는 컨텍스트를 아예 읽지 않았다. 덤프한 이미지(`wie-ktf-dump`)에서 그 proc 을 읽었다:

- `46b2` 의 proc: `r0 == 전역의 키(마젠타 픽셀) ? r1 : 전역의 글자색`. 글자 조각의 마젠타 바탕은 버리고 나머지를 글자색으로 칠한다 ⇒ 무시하면 글자가 마젠타 네모 위에 그려진다(화면으로 확인).
- `7e22` 의 proc: `r0 == 0x2484 ? r1 : r0`.
- ⇒ 인자 순서는 **`proc(src, dst)`** 다(WIPI `MC_GrpPixelOpProc` 순서 · 셋째 인자 = `param1`). LGT 의 같은 장치는 `proc(dst, src)` 이다(`docs/report/0399`) — 두 통신사의 순서가 다르다.

**고친 것.** LGT 의 픽셀 연산 블릿(`blit_with_pixel_op`)을 공용 `wie-wipi-c` 로 옮기고 인자 순서를 `PixelOpArgs` 로 받는다. KTF 표의 DrawImage/CopyFrameBuffer 는 컨텍스트의 `pixel_op_func_ptr`·`param1` 이 있으면 그 블릿을, 없으면 종전 블릿을 쓴다. LGT 는 종전 그대로다(자기 오프셋 +28 에서 proc 을 읽어 `DstSrc` 로 넘긴다 · LGT 시험 `wipic_blits_run_the_native_pixel_op` 그대로 통과).
공용 `draw_image`·`copy_frame_buffer` 자체에 넣지 않은 이유: LGT 는 공용 레코드의 +32(공용 `pixel_op_func_ptr`)를 자기 필드로 쓰므로 거기서 읽으면 LGT 가 엉뚱한 주소를 함수로 부른다.

| sha12 | 전(main `1bc1317c`) | 후(이 PR) | 되돌리면 red |
|---|---|---|---|
| `46b2238f87a6` | 같은 키 재생 200초: FAIL «magenta … 17%» · 기본 주입 20초 샷 31장 중 17장이 마젠타 >1% | FAIL 없음(deadline 까지) · 마젠타 샷 0/31 · 도움말 글자가 깨끗하다 | `test_wipic_blits_run_the_context_pixel_op` FAILED(«the key keeps dst, the rest takes src») |
| `7e2247bdf565` | 같은 키 재생 400초: FAIL «magenta … 62%» | FAIL 없음(deadline 까지) · 미션 메뉴가 그려진다 | 같은 시험 |

- 600초 정책 판정(이 PR 빌드): §3-1.
- **퇴행 확인 — 픽셀 연산을 거는 KTF 를 전수로 찾았다.** KTF 266종을 main 빌드로 15초씩 돌려 `MC_grpSetContext(…, PixelopIdx, …)` 로그가 나오는 타이틀을 셌다(로그는 저장하지 않고 grep 만): **5종**(`23919eb33365` `46b2238f87a6` `503d5d2f196b` `7e2247bdf565` `9d52de42e7f9`). 5종을 옛/새 빌드 기본 주입 20초로 짝 측정했다: result·content·입력 단계가 같고 paints 차 ±8 이다. 화면이 다른 샷은 `23919` 5장(같은 키 단계에서 한쪽이 로딩 중 — 다음 단계부터 같은 메뉴) · `503d` 12장(눈으로 같은 로고) · `9d52` 0장이다. 15초 안에 거는 타이틀만 잡히는 표본이다(그 뒤에 거는 타이틀은 못 셌다).

### 3-1. 고친 2종 600초 정책 판정

같은 정책(v2 600초 + 재기동 120초 · P 와 P2 둘 다) · 이 PR 빌드 · 두 번째 `--long` 임대(측정 드라이버를 멈춘 뒤 · 픽셀 연산 전수·짝과 같은 임대) · 21:00–21:25 · jobs 2 · load1 10.5 → 6.7 · idle 45% → 68%.

| sha12 | P (stall · 새 화면 수) | P2 | 공개 값 | 이전 값 |
|---|---|---|---|---|
| `7e2247bdf565` | 0 · 52 | 30 | **ok** | stuck(렌더 FAIL) |
| `46b2238f87a6` | 420 · 6 | 420 | stuck | stuck(렌더 FAIL) |

- `46b2` 는 이제 렌더 벽이 아니라 ⒜다 — 600초 화면이 옵션 메뉴(소리·진동·속도 설정)이고 글자가 깨끗하다. 정책 키가 그 메뉴를 못 빠져나온다. 값은 같지만 막힌 이유가 바뀌었다.
- 이 두 행은 이 PR 빌드로 쟀다. 고침과 데이터가 같은 PR 로 함께 착지한다(2회차 §4-1 과 같은 처리).

### 4. compat 갱신

- 현 `origin/main`(`1bc1317c`)의 `compat.json` 위에 바꿀 행만 얹었다(2회차 F1 교훈 — 옛 판 위에서 다시 쓰지 않는다). 방법: census `report` 가 만든 표에서 이 회차가 잰 행의 `progress` 만 옮긴다(`ok → ok` · `stuck`·`error → stuck` · `n/a` 는 손대지 않음).
- `axes.progress` 41행: `(없음) → ok` 25 · `stuck → ok` 1(`e68b1c8aef85`) · `(없음) → stuck` 15(error 2 포함).
- 고친 2종(`46b2238f87a6` · `7e2247bdf565`): §3-1.
- `knownIssues_ko`·`status` 는 바꾸지 않았다. `30c7bd6fb01b`(⒞)에 손으로 안내 문장을 넣으면 다음 재생성이 지운다(census 가 그 축을 모른다) — §5.

**행 단위 비교**(`origin/main` `1bc1317c` ↔ head · 키 `sha256+platform` · axes 는 하위 키 단위):

```
rows 429 -> 429 · TOP(schema·generatedAt·enginePin) 차이 0 · 사라진/새 행 0
changed by field set {"axes.progress":42}
  axes.progress:(none)->ok     25
  axes.progress:(none)->stuck  15
  axes.progress:stuck->ok       2   (e68b1c8aef85 · 7e2247bdf565)
changed 42 · 이 회차가 잰 행 밖 0
```

바뀐 행 = 이 회차 progress 41(main 빌드) + 고친 2종 중 값이 바뀐 1(`7e2247bdf565`)뿐이다(`46b2238f87a6` 는 stuck → stuck 이라 바뀐 행이 아니다). 비의도 0.

### 5. 다음 회차

- 이어하기: `cd ~/scratch/w7prog/r3 && ~/orchestrator-live/bin/build-slot run --long -- bash drive.sh`. 같은 `--out` 이라 잰 것은 건너뛰고 `ad` 부터 돈다. ★그 전에 그때의 main 을 다시 빌드해 `drive.sh` 의 `BIN`·`--pin` 을 바꿔라(이 PR 이 착지하면 KTF 블릿이 바뀐다).
- `30c7bd6fb01b`: KTF Net 슬롯 34 `(host:port 문자열, 0xff)` 의 정체와 실패 반환 규약. 호출부를 이미지에서 읽어야 한다(sl 상대 주소라 문자열 상수 검색으로는 안 잡혔다).
- `96dc32e781d3`: paint 안 `Invalid memory access`. KTF 힙 고갈 축(`wie-ktf-guest-gc-roots-port-from-lgt`)과 겹치는지부터 확인한다.
- ⒜ 13종은 레시피 후보다(비커밋 `game_lab/recipes-progress/`).
