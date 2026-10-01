## [2026-10-01] KTF «Data 인스톨에 실패» 두 종 — DB slot 7 은 목록이 아니라 rename · 그 뒤 벽 셋(slot 4 lseek · '/' 이름 · slot 8 mkdir) (wie-ktf-data-install-fail-db-list-records-adopt-p0)

**무엇을**: KTF Database 표의 네 슬롯과 이름 해석을 호출부 디스어셈블로 확정하고 고쳤다. `1cf2e6076079`·`5028b8a5d19f` 가 설치를 끝내고 게임(팀 선택 화면)까지 간다.
**왜**: `docs/report/0393` §8 — 두 종이 매 부팅 3초 안에 «Data 인스톨에 실패했습니다» 뒤 종료하는데 지원 목록은 `playable` 이었다(채택 제안 `2026-09-30-progression-axis-census#p0`).
**사용자 영향**: 두 종이 처음 켤 때 설치를 끝내고 시작된다. 크기 탐침(`lseek(h,0,END)`)이 언제나 0 이던 KTF 11종은 이제 실제 크기를 받는다 — 그 가운데 `503d5d2f196b` 는 동봉 세이브 2개(각 32바이트)를 처음으로 읽는다.

증적(디스어셈블 창·전사·스크린샷)은 저장소 밖(`~/scratch/wie-slot4/` · `~/scratch/wie-progress-1001/`)에만 있다. 이 문서는 sha12 와 DB 이름만 적는다. 게임 이름·바이트는 0 이다.

### 1. 첫 벽 — slot 7 은 `(원본 이름, 새 이름, 1)` rename 이다

재현(`wie_validate` release · `origin/main 1becaae2`): `stat("lo.dsk")`·`stat("lo.z_")` 둘 다 없음 → `open("lo.z_", 2)` → `stream_write` 17회(100KB×16 + 17,625B = 1,655,025B · jar 의 `lo.z00`~`lo.z17` 을 이어 붙인다) → `close` → **slot 7** → 「Data 인스톨에 실패」. 로그는 `MC_dbListRecords(0x400fffd8, 0x115fa4, 1)` 로 보였다.

★인자를 읽었다: `r0` 가 가리키는 바이트는 `"lo.z_\0"`, `r1` 은 `"lo.dsk\0"` 이다 — 핸들이 아니라 **이름 둘**이다. 호출부(`client.bin` `0x10108c` · 표 오프셋 `+0x1c` = slot 7):

```
0x101084 ldr r1, [r6, #0x1c]   ; "lo.dsk"
0x101088 mov r0, r8            ; "lo.z_"
0x10108a movs r2, #1
0x10108c bl  <slot 7>
0x101092 cmp r0, #0 ; blt → 오류 대화상자 · 아니면 다음 설치 단계(0x1016ac)
```

다음 부팅은 `stat("lo.dsk")` 로 설치 여부를 본다(§4 의 2회차 부팅에서 실측: `-> 0 (size=1758425)` 이고 설치를 건너뛴다). ⇒ slot 7 = **rename**, `>= 0` 성공. 핸들이 오면 종전대로 `list_record` 다 — `load_handle` 의 매직 검사로 가른다(`list_record_or_rename_ktf`).
코퍼스 범위: `game_lab/working/ktf` 190파일(170종) 스윕에서 slot 7 을 부르는 것은 `5028b8a5d19f` 하나뿐이다(`1cf2e6076079` 는 `broken/`). 다른 제목의 동작은 바뀌지 않는다.

### 2. 둘째 벽 — 같은 DB 를 `"lo.dsk"` 와 `"/lo.dsk"` 로 부른다

rename 뒤 그 제목은 `open("/lo.dsk", 1)` 을 한다. 저장소 키가 문자 그대로라 `-12` → 「잘못된 리소스 파일입니다」.
처방은 **정규화가 아니라 해석**이다(`resolve_db_name`): 주어진 이름의 DB 가 있으면 그것, 없고 앞의 `/` 를 뗀 이름이 있으면 그것, 아니면 주어진 그대로. ★이미 `/` 붙은 키로 저장된 세이브가 있다면 그대로 이긴다 — 정규화로 바꾸면 그런 세이브가 고아가 된다. 이름을 읽는 9곳 모두가 이 함수를 지난다.

### 3. 셋째 벽 — slot 4 는 `lseek` 인데 «언제나 처음부터 · 언제나 0» 이었다

`/lo.dsk` 를 연 뒤 `slot4(h, 0, 1)` → `0`, `slot4(h, -982, 0)` → `-22`. 그 제목의 libc shim 을 읽었다:

- POSIX 연산 표(`0x10a554` · 연산 `0x1001`~`0x1014`): `0x100a` 가 slot 4 로 가며 whence 를 **1→1 · 2→2 · 그 밖→0** 으로 넘기고, slot 4 의 반환값을 그대로 `lseek` 의 결과로 돌려준다(`0x10a630`).
- `ftell`(`0x102238`) = `lseek(fd, 0, 1)`. `fseek(SEEK_CUR)`(`0x102450`) = `ftell()+off` 를 whence 0 으로.

⇒ slot 4 = `lseek(h, off, whence{0 처음 · 1 현재 · 2 끝})` → **새 위치**. 종전 구현은 whence 를 무시하고 `off` 로 옮긴 뒤 0 을 돌려줬다: `ftell` 이 늘 0 이라 `ftell()-982` 가 음수가 됐다.
코퍼스(190파일 스윕): slot 4 를 부르는 KTF 16종. 모양은 `(0, 0)` 13종 · `(0, 2)` 11종 · `(+n, 1)` 3종 · `(+n, 0)` 1종. `(0,2)` 뒤 `(0,0)` 은 읽기 전 크기 탐침이고, 종전 구현은 그 11종 모두에게 **빈 파일**이라고 답하고 있었다.
끝을 넘는 위치는 허용한다(다중 슬롯 세이브가 고정 오프셋에 쓴다 — `stream_write` 가 틈을 0 으로 채운다). 음수 결과는 `-22` 로 거절하고 위치를 옮기지 않는다. `whence=1` 의 «현재»는 두 커서 중 큰 쪽이다(탐색 때 함께 옮겨지고 각자 자기 연산에서만 전진한다).

### 4. 넷째 벽 — slot 8 은 «디렉터리 보장»(0175 의 H4)

설치·로드 뒤 `slot8("/shared", 1)` → `stat("/shared")` 실패 → `MC_knlExit(0)`. 둘 다 shim 이 부른다: 연산 `0x1010` → slot 8 `(path, 1)` · 바로 다음 `0x1011` → slot 9 `(path, 1)` · `0x100d`(stat) → slot 5. mkdir 뒤 stat 으로 확인하고 실패하면 끝내는 모양이다.
스크래치 실험으로 «마지막 벽인가»를 먼저 쟀다: `stat("/shared")` 만 성공시키자 27/27 키 · 336 paints · 팀 선택 화면 · 세이브 기록(`/recordStore/rms_game_sav.sav` 등).
⇒ slot 8 은 그 이름으로 **빈 DB 를 만든다**(이미 있으면 손대지 않는다) — slot 5·16 이 보는 것이 그것이다. 반환은 종전대로 0(0175 의 세 호출부는 결과를 버린다). 0175 의 H1~H3 은 이 관측으로 죽지는 않지만, H4 가 처음으로 «결과가 쓰이는» 호출부에서 맞았다. 대가: 0175 의 두 이미지(`"res"`·`"ga"`)도 같은 이름의 빈 DB 를 갖게 된다 — 두 제목(0391 의 `59263295de74`·`e085e193211d`, `broken/`)을 전/후로 쟀다: 각각 FAIL·4키(0391 의 알려진 벽 — 키 4 의 메모리 읽기) / UNMEASURED·`max-ticks` 로 판정이 같다.

### 5. 부산물 — slot 15 = 열린 DB 의 크기

§3 을 고치자 `503d5d2f196b` 가 **FAIL**(`Unimplemented: 15: MC_dbUnk15`)로 바뀌었다 — 크기 탐침이 0 이 아니게 되자 처음 닿은 호출이다. 호출부(`0x194dc8`~`0x194e14`):
`size = lseek(h,0,2); if (size > 0) size = slot15(h); lseek(h,0,0)` — 결과가 크기 변수를 덮는다. ⇒ slot 15 = 열린 DB 의 바이트 수(`file_size_ktf`). 고친 뒤 그 제목은 동봉 세이브 `./Hero3OptionSave`·`./Hero3SlotSave_0`(각 32바이트)를 읽고 27/27 PASS(2회). 코퍼스에서 slot 15 를 부르는 것은 이 제목뿐이다.

### 6. 짝 재측

| 무엇 | 결과 |
|---|---|
| 두 대상 제목 · `--inject` 27키 | 전: `UNMEASURED · clean exit · 1/27키 · 19 paints`(오류 화면) → 후: **PASS · 27/27 · 327~330 paints** |
| 두 대상 · `--relaunch 2` | 2회차 부팅이 설치를 건너뛰고(`stat("lo.dsk") -> 0`) 같은 지점까지 간다 |
| 진도 축(`--only progress` 600초 · P·P2) | 두 종 모두 P·P2 PASS · 624/624키 · 4,145~4,231 paints. 마지막 새 화면 뒤 정체 P 550·230초 / P2 320·230초(load1 181~187) ⇒ census 판정 **`stuck`(짝 확정)**. 마지막 화면은 게임 안 메뉴(팀 업그레이드 · 「ATT +3」 반영 = 입력은 먹는다) — 0393 의 ⒜ 정책 부족 형태이지 엔진 벽이 아니다 |
| DB 를 쓰는 KTF 39종 · 같은 시각 전/후 쌍 | 39쌍 중 **38 동일** · 바뀐 1 = `5028b8a5d19f`(UNMEASURED·1키 → PASS·27키). FAIL 전 0 · 후 0 · PASS 29 → 30. `503d5d2f196b`(§5)는 전·후 PASS |
| 엔진 runner 블록(AGENTS.md) | 6줄 모두 PASS |

★스윕 노이즈: 170종 전체 전/후 스윕(3 jobs · load1 13~47)에서 (판정·키 수·stop)이 바뀐 것은 DB 를 건드리지 않는 131종 중 77종(판정 자체는 6종) · DB 39종 중 12종이다. 앞의 131종은 이 변경이 닿을 수 없으므로 그만큼이 부하 노이즈다(대부분 50M `max-ticks` 백스톱에서 멈추고 키 수만 다르다). 그래서 DB 39종만 같은 분에 전/후를 붙여 다시 쟀다. `db8ef04a6504`(전 PASS → 후 clean exit)는 4회 재측에서 전·후 모두 PASS/UNMEASURED 를 오갔다 — 노이즈.

### 7. compat

`compat.json` 은 **바꾸지 않았다** — 실측 판정이 기존 행과 같다: `playable` · `progress: stuck`(§6). 달라진 것은 근거다. 종전 `playable` 은 30초 프로브가 오류 대화상자를 «그려진 화면»으로 센 거짓 양성이었고(0393 §8), 이제는 설치를 끝내고 게임 안에서 27/27·624/624 키를 받는다. `stuck` 은 «설치 실패로 멈춤»에서 «진도 정책 키가 메뉴를 맴돎»으로 바뀌었다. sound 축은 범위 밖이라 그대로다.

### 8. 손대지 않은 것

- slot 9(shim 의 `0x1011` · `(path, 1)`)는 이 코퍼스에서 한 번도 불리지 않아 stub 그대로다.
- `KTF_DATABASE_STORAGE_LIMIT`(1MB)는 이 제목의 1.7MB 설치보다 작다. slot 12 를 부르지 않아 영향이 없었고, 바꾸지 않았다.

**게임 파일명 유입**: BOUNDED 0회/0쌍 · SUFFIX-ATTACHED 0회/0쌍(`node scripts/corpus-name-inflow.mjs` · 이 브랜치의 5파일). 게임명은 sha12 로만 적었다.

## 반려 승계 F1 (wie-ktf-data-install-fail-db-list-records-adopt-p0-fix)

slot 7 rename 이 해석 후 «원본 = 대상»이면(`/x`→`x` 에서 `/x` 부재 · `x` 존재, 또는 `x`→`x`) 복사·삭제 없이 0 을 돌려준다(POSIX `rename(x,x)`). 종전은 `delete(dst)` 가 원본을 먼저 지워 rc=0 인 채 세이브가 사라졌다. 테스트 `ktf_slot7_rename_onto_itself_keeps_the_data` 가 두 형태를 잠그고, 가드를 빼면 red 다. 대상 2종 `--inject` 재측: `5028b8a5d19f` PASS 27/27 · 282 paints, `1cf2e6076079` PASS 27/27 · 302 (load1 34).

minor 처분: m1 그대로(측정된 호출 형태 `ftell` 직후 `fseek` 에 맞고 주석이 가정을 말한다) · m2 그대로(shim 이 0/1/2 밖을 0 으로 접는다) · m3 그대로(같은 DB 의 두 이름이라는 설계의 귀결 · 일관적).

<!-- corpus-name-inflow v1 subjects=5 tree=05f8b0adb2dfaaba B=0/0 P=0/0 S=0/0 -->
