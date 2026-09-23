## [2026-09-24] `game_lab/broken/lgt` sha256 중복 9쌍 — 태그 붙은 쪽을 `game_lab/_dup/` 로 옮겨 두 번 세지 않게 했다 (wie-2026-09-21-aot-java-render-rebaseline-adopt-p4)

**무엇을** — `game_lab/broken/lgt` 의 바이트 동일(sha256) 쌍에서 통신사 태그가 붙은 이름(`lgt …`·`LGT …`·`(LGT)…`·`… LGT`)
9개를 `game_lab/_dup/broken/lgt/` 로 **옮겼다**(삭제 0). 태그 없는 이름이 남는다. `game_lab/` 은 git-ignored 라
이 PR 의 diff 는 이 문서와 worklog 뿐이다 — 옮김 자체는 로컬 코퍼스에서 일어났다.

**왜** — 제안(`2026-09-21-aot-java-render-rebaseline#p4`)이 6쌍으로 지적했다. 착수 시 전수 재측:
**9쌍**(제안 6 + `당구마스터2010`·`붉은보석`·`크로이센` 3쌍 추가). 같은 바이너리를 두 번 돌려 세는 수가 부푼다.

| 남긴 쪽 (`broken/lgt/`) | 옮긴 쪽 (`_dup/broken/lgt/`) | sha256 앞 12 |
|---|---|---|
| 턴.zip | (LGT)턴.zip | a16f08d025eb |
| 월드장기체스.zip | LGT 월드장기체스.zip | 70d709c40e10 |
| 레전드오브마스터.zip | lgt 레전드 오브 마스터.zip | 735a579d82ac |
| 배틀몬스터.zip | lgt 배틀몬스터.zip | a30bbe008b5e |
| 서든어택포켓.zip | lgt 서든어택 포켓.zip | 517ed32c92d6 |
| 당신은골프왕.zip | 당신은골프왕 LGT.zip | b2da04c55cd4 |
| 당구마스터2010.zip | lgt 당구마스터.zip | 2dbde9acca99 |
| 붉은보석.zip | lgt 붉은보석.zip | 2520654be6de |
| 크로이센.zip | LGT 크로이센.zip | a23f3c9fc2cb |

**어느 쪽을 남기나** — 태그 없는 이름. 형제 P0(`wie-battlemonster-*`)의 재현 명령이
`game_lab/broken/lgt/배틀몬스터.zip`(태그 없음)을 전체 경로로 쓰고, report 0165·worklog 2건이 `lgt/크로이센.zip`
을 쓴다. 반대쪽 이름을 경로로 쓰는 살아 있는 명령은 0이다(아래).

**왜 `broken/lgt.dup/` 이 아니라 `game_lab/_dup/` 인가** — 두 집계 스크립트가 `broken/` 을 **재귀로** 걷는다:
`game-lab-recensus.sh:183` 은 `find "$CORPUS" -type f`, `game-lab-census-map.mjs` 의 `walk()` 는 하위 디렉터리로
내려간다. `broken/` 안에 두면 그대로 두 번 센다. 스크립트에 sha 접기를 넣는 쪽은 코드 diff 가 생기므로 옮김이 작다.
`corpus-name-inflow.mjs` 는 `vendor_sdk` 만 빼고 `game_lab/` 전체를 걷으므로 옮긴 이름도 계속 유입 검사 모집단에 있다.

**전/후** (sha256 전수 · 로컬 코퍼스)

| 축 | 전 | 후 |
|---|---|---|
| `broken/lgt` 파일 | 46 | 37 |
| `broken/lgt` 고유 sha | 37 | 37 |
| `broken/**` 파일 | 187 | 178 |
| `broken/**` 고유 sha | 170 | 170 |
| `game-lab-recensus.sh --dry-run` 대상 | 187 | 178 (lgt 37) |
| `census-map.tsv` 행 (재생성) | 188 | 179 (lgt 46 → 37 · 빠진 9행 = 옮긴 9개) |

**참조 깨짐** — `smoke_gate_baseline.tsv` 의 `broken/` 참조 **0**(베이스라인은 `working/` 만 식별한다 — `lgt/` 행 52는
전부 working). 추적 파일 + orchestrator `tasks/wie-*`·`queue/wie*` 안의 `broken/lgt/<이름>.zip` 인용 9건 중 없는 파일을
가리키는 것 2: `wie-battlemonster-…-resume-clip.md` 의 «★중복 — `lgt 배틀몬스터.zip` 은 sha 동일» 서술 1(명령 아님) +
`<타이틀>.zip` 자리표시자 1. 실행 명령으로 깨지는 인용 **0**. 원문은 NFC 정규화 후 셌다 — 디스크 이름은 NFD 라
정규화 없이 세면 0이 나온다(이 회차가 처음 그렇게 0을 받았다).

**사용자 영향** — 없음(엔진·웹 무변경). 코퍼스 기반 수(스윕·센서스)가 lgt 에서 9건 덜 부푼다.

**남긴 것** — `broken/ktf` 에도 같은 모양의 sha256 중복 **8쌍**이 있다(범위 밖 — worklog 제안으로 남겼다).
`broken/**` 전체로는 여전히 178 파일 / 170 고유.
