## [2026-09-25] 간호사타이쿤2 no-frame 벽의 이름 = `DataInputStream` index 22 빈 칸 — `readBoolean()Z` 등재, 다음 벽은 게스트 NPE (wie-2026-09-24-jlet-get-current-jlet-alias-adopt-p0)

채택 제안 `2026-09-24-jlet-get-current-jlet-alias#p0` 의 회차다.
★**판정**: no-frame 은 «무언가를 기다리는 정지»가 아니었다. 부팅 스레드(thread 1)가 세이브 레코드를 읽다가 `java/io/DataInputStream vtable index 22` 빈 칸에서 `Unimplemented` 로 죽는다. 그 뒤로 그릴 주체가 없다.
한 칸짜리였으므로 수리했다. 다음 벽은 적어 두고 멈췄다.

### ⒜ 현 main 에서도 paints 0 인가 — 맞다. 다만 벽의 문면이 다르다

`origin/main` `852143b7` · debug `wie_validate --timeout 120`(loadavg 100~160):
`FAIL · stop deadline · ticks 332 · paints 0` · `java_exceptions.count 3`.
세 번째 예외가 벽이다: `net/wie/WieError Unimplemented: java/io/DataInputStream vtable index 22`.
직전 회차(0241)가 본 «마지막 경고 = DataBase·RecordStore·AnnunciatorComponent 스텁»은 WARN 만 본 것이다. 그 뒤의 이 예외는 INFO(`jvm::jvm`)라 보이지 않았다.
※`--timeout 15` 로는 이 벽까지 못 간다. 부팅 틱 하나가 1.2MB 리소스를 1KB 씩 읽는 동안 끝난다(`ticks 2`).

### ⒝ 스텁 3종이 분기를 막는가 — 아니다

`AnnunciatorComponent`·`closeRecordStore` 는 반환값이 없다(`V`). `openDataBase` 는 실제로 열고 레코드를 돌려준다(게임이 첫 회 `Record not found` → 기록 → 재오픈 → `getRecord(1)` 성공).
정지 지점은 그 **뒤**다. 스텁이 막은 게 아니다.

### 1단계 — 멈춘 자리

임시 계측(커밋 안 함)을 missing-entry 핸들러에 넣어 호출부를 떴다. `lr=0x27914`, ARM:

```
0x27900 ldr r3, [r5]          ; 스트림의 디스패치 표
0x27904 mov r0, r5            ; this = 스트림 — 인자 준비는 이것뿐
0x27908 ldr ip, [r3, #0x5c]   ; index 22
0x2790c mov lr, pc ; bx ip
0x27928 str r0, [r2, r3, lsl #2]   ; 1워드 반환값을 객체 필드에 저장
0x27934 ldr ip, [r3, #0x74]        ; 같은 스트림에 index 28 (readInt)
```

### 2단계 — 수리: `DataInputStream 22 = readBoolean()Z`

`lgt_java_abi.toml` 에 1행을 넣었다. 파일이 이미 적어 둔 파생(측정 앵커 `readByte 23` 의 한 칸 앞)과 같은 값이다. **새 값을 발명하지 않았다.** 호출부 근거는 둘이다.
- **인자가 없다.** 인자를 받는 이웃 19·20(`readFully`)·21(`skipBytes`)은 배제된다.
- **읽기가 게스트 자신의 쓰기와 거울이다.** 행을 넣은 뒤 게임은 레코드를 끝까지 읽는다. rustjava 의 `writeBoolean` 은 안에서 `writeByte` 를 부른다. 그 `writeByte` 를 접으면 쓰기 819회와 읽기 819회가 **자리마다 타입이 같다**(`bool, int×303, bool×128, int×37, …`). 읽은 바이트는 201×1 + 618×4 = **2673** 이고, 게임이 `setRecord` 로 저장한 길이와 **정확히 같다**.

★**한계**: `writeBoolean` 은 0/1 만 쓴다. 그래서 값으로는 `readBoolean` 과 `readUnsignedByte` 를 가를 수 없다. 둘을 가르는 것은 슬롯 순서뿐이다.

### before / after · 양방향 변이

| | result · stop | ticks | paints | 예외(순서) |
|---|---|---|---|---|
| before (main) | FAIL · deadline | 332 | 0 | InvalidRecordID · DataBaseRecord · **`Unimplemented: DataInputStream vtable index 22`** |
| after #1 | FAIL · deadline | 710 | 0 | InvalidRecordID · DataBaseRecord · **NullPointerException · `Invalid memory access; address: 1886342224`** |
| after #2 | FAIL · deadline | 570 | 0 | 위와 같음 |
| 변이(행 제거) | FAIL · deadline | 897 | 0 | before 와 같음 — `Unimplemented: … index 22` 재현 |

ticks 는 부하에 따라 흔들린다. 스레드가 죽은 뒤 도는 틱이라 진전의 척도가 아니다. 판정은 예외 문면으로 한다.
단위 시험 `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 `(DataInputStream, 22, readBoolean, ()Z)` 를 더했다. 원본은 ok. 행을 제거하면 `vtable index 22 is empty` 로 FAILED 가 난다.

### 다음 벽 (범위 밖 — 제안으로 남긴다)

로드 직후 게스트가 import `0x64`/34(`RaiseNullPointerException`)를 `lr=0x297f0` 에서 부른다. 그 경로는 저장해 둔 예외를 다시 던지는 모양이다(`r0 = [fp-0x34] = 0`).
약 1초 뒤 `WieError Invalid memory access; address: 0x706f6e50` 이 난다. 이 주소는 ASCII 바이트다. 0229(게스트가 자기 SVC 스텁 표를 `athrow`)와 같은 계급으로 보이지만 **확인하지 않았다**.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 의 대상은 이 브랜치가 바꾼 5파일이다(게이트③ 이 동봉한 `docs/worklog-coverage-remeasures.json` 포함 — 그 파일에서 잡힌 이름은 0).
결과는 **BOUNDED 27쌍 · SUFFIX-ATTACHED 7쌍 · PREFIX 0** 이다(이 절 자신이 적은 이름 포함).
- BOUNDED 중 이 회차가 새로 쓴 이름은 대상 타이틀 `간호사타이쿤2` 하나다. 이 이름은 0241 이 이미 main 에 적은 것이다. 나머지 쌍은 `lgt_java_abi.toml` 에 원래 있던 주석 속 타이틀 이름이다(배틀몬스터·서든어택포켓 등). 도구가 파일 전체를 세기 때문에 잡혔다.
- SUFFIX-ATTACHED 는 손으로 갈랐다. 전부 stem `간호사타이쿤`·`서든어택` 뒤에 `2`·`포켓` 이 붙은 형태다. 즉 «더 긴 다른 제목»이고, 그 제목들은 위 BOUNDED 에 이미 있다. 새 이름은 0이다.

<!-- corpus-name-inflow v1 subjects=5 tree=00a574226220042d B=57/27 P=0/0 S=15/7 -->
