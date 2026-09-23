## [2026-09-24] `ByteArrayOutputStream 16 = toByteArray()[B` · `String 21 = indexOf(I)I` — 반환값 사용이 이름을 강제했다 (wie-2026-09-23-aot-java-dataoutputstream-and-gregoriancalendar-anchors-adopt-p0)

### 무엇을

`wie-lgt/data/lgt_java_abi.toml` 에 두 줄(`java/io/ByteArrayOutputStream` 행 신설 · `java/lang/String` 에 21) +
그 두 줄을 `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 고정. 코드 0줄.
채택 제안: `2026-09-23-aot-java-dataoutputstream-and-gregoriancalendar-anchors#p0` · `#p2`.

### 먼저 반증 — 전제는 절반만 섰다

재측정(`RUST_LOG=info wie_validate --timeout 90`, #263 head 위):

| 타이틀 | 벽 |
|---|---|
| 턴 · (LGT)턴 · 서든어택포켓 · lgt 서든어택 포켓 | `java/io/ByteArrayOutputStream vtable index 16` ✓ |
| 일지매영웅전기 | `java/lang/String vtable index 21` ✓ |
| ★훼밀리마트타이쿤 | `java/lang/StringBuffer vtable index 22` — BAOS 16 이 **아니다** |

⇒ 「3타이틀 수렴」은 **2타이틀(4파일)** 이다. 훼밀리마트타이쿤의 벽은 PR #259 보고서 표에 이미 그렇게 적혀 있었다.

### 왜 — 측정 (임시 계측으로 누락 스텁 진입 시 레지스터와 호출부 코드를 읽고 되돌렸다)

**BAOS 16** (턴): `r0=this` 만 세팅, 반환을 받아

```
str r0, [fp, #-0x28]
ldr r2, [fp, #-0x28] ; cmp r2, #0 ; moveq r0, #0 ; beq …   ← null 검사
ldr r3, [r2, #8]                                           ← instance->ptr_fields
ldr r2, [r3]                                               ← 배열 길이 워드 = b.length
```

null 검사 뒤 두 번 역참조해 배열 길이에 닿는다 — `size()I` 의 정수는 포인터로 읽힌다. CLDC 순서
(OutputStream 15칸 뒤 `reset toByteArray size`)도 16 = `toByteArray` 다. 15·17 은 비워 뒀다.

**String 21** (일지매영웅전기): `r1=0x7c('|')`, `r2` 는 잔여 힙 포인터(= 인자 1개). 반환은 `cmp r0,#0 ; blt` 뒤
index 28 `substring(0, i)` 의 끝으로 쓰인다 — '|' 로 자르는 코드. 같은 모양의 `lastIndexOf(I)I` 는 위치(23)가 가른다.

### 변이 검사 (그 뒤 원상태로 되돌려 재빌드·재현)

| 변이 | 결과 |
|---|---|
| BAOS 16 = `size()I` | 턴: **5 tick 만에 host panic** — `LGT object reference 0x9 does not point at a live instance` → `Option::unwrap()` on `None` |
| String 21 = `lastIndexOf(I)I` | 일지매영웅전기: 벽은 같은 27이지만 그 호출 인자가 `r1=0x2` → `0xe` — 자르는 위치가 바뀐다 |
| (서든어택포켓, size 변이) | 다음 벽까지 차이 없음 — 판별은 턴 호출부가 진다 |

### 사용자 영향 — 전/후 (같은 분에 짝지어 실행)

| 파일 | 전 | 후 | ticks 전→후 | paints |
|---|---|---|---|---|
| 턴.zip | BAOS 16 | `DataOutputStream vtable index 14` | 2257→2244 | 0→0 |
| (LGT)턴.zip | BAOS 16 | 〃 | 2282→2254 | 0→0 |
| 서든어택포켓.zip | BAOS 16 | 〃 | 2039→2023 | 0→0 |
| lgt 서든어택 포켓.zip | BAOS 16 | 〃 | 2020→1995 | 0→0 |
| 일지매영웅전기.zip | String 21 | `java/lang/String vtable index 27` | 2241→2288 | 0→0 |
| 훼밀리마트타이쿤.zip | StringBuffer 22 | StringBuffer 22 (불변) | 2240→2213 | 0→0 |

다섯 파일이 벽 하나씩 더 갔다. 아직 아무도 paint 하지 않는다. 다음 벽 셋(OutputStream `close()`=14 ·
String `substring(I)`=27 · StringBuffer 22 = `append(C)` 후보)은 worklog 제안으로 남겼다 — 이 회차 범위 밖의 행이다.
