## [2026-09-30] 3차 꼬리 — 단발 벽 14종 중 6종 playable · 8종은 다음 벽 기록 (wie-census-wave3-singleton-tail-boot-and-longplay)

**무엇을**: 전수 `3c34efee` 의 «수 1» 첫 벽 18종 중 형제 wave3 티켓과 겹치는 4종을 빼고 14종을 봤다. 6종이 not-yet/limited → **playable** 이 됐다. 나머지 8종은 다음 벽과 못 고친 이유를 적었다.

**왜**: 운영자 지시(2026-09-30) 「미지원·부분지원 게임을 전수 검토해 모두 플레이 가능하게」. wave3 세 티켓은 수 2 이상 군집만 가져갔다.

**사용자 영향**: 지원 현황 not-yet 30 → 26 · limited 50 → 48 · playable 349 → 355(`docs/player-data/compat.json` 6행).

### 1. 겹쳐서 뺀 것(stand down)
| sha12 | 벽 | 주인 |
|---|---|---|
| `362c57e2b2b7` `568c339a8c07` | 낮은 주소 Invalid memory access | wave3 ktf-skt «낮은 주소 6종» 정책(313ddcd8 목록에 둘 다 있다) |
| `87b04639cdfe` | LGT WIPIC SVC 1100 | wave3 lgt «SVC 8종» |
| `0262a4fe3389` | class_instance unwrap | wave3 lgt «class_instance 5종» |
| `7da00ecd4804` | 이번 재측의 벽이 주소 0 읽기(`07_DOWN` · getNextEvent) | 같은 계급 — ktf-skt 가 도입 중인 «0 페이지 읽기 = 0» 정책 뒤에 재측 |

### 2. 전/후(6축 · 전 = 전수 `3c34efee` · 후 = 이 브랜치 `a5490d26` 로 같은 도구 `playability-census.mjs run` 21종)
| sha12 | 전 | 후 | 고친 것 |
|---|---|---|---|
| `b2da04c55cd4` LGT | not-yet · `/ by zero` @startApp | **playable** ok·ok·ok·ok·ok·n/a | `HandsetProperty.getSystemProperty("PHONENUMBER")` 가 `""` — 게임이 `hash % 번호.length()` |
| `a23f3c9fc2cb` LGT | not-yet · SIGABRT(호스트 스택 넘침) | **playable** ok·ok·ok·ok·silent·ok | `MC_knlGetSystemProperty("PHONENUMBER")` 가 `""` — 게임이 `memcpy(buf, 번호, strlen(번호) - 4)` → 4GB 복사 → 예외 경로 재귀 |
| `96dc32e781d3` KTF | not-yet · NPE @startApp | **playable** ok·ok·ok·ok·ok·ok | `DataBase.openDataBase(name, n, false)` 가 없는 DB 를 만들어 줬다 → `DataBaseException` |
| `33801c1ba14f` KTF | not-yet · `MC_grpEncodeImage` | **playable** ok·ok·ok·ok·ok·n/a | `MC_grpEncodeImage(fb, 0, 0, 240, 320, &len)` 구현 |
| `517ed32c92d6` LGT | limited · bucket.rs index OOB panic | **playable** ok·ok·ok·ok·silent·n/a | 32바이트 칸을 `size 8` 로 반납 — 버킷을 주소로 고른다 |
| `61ed69520fd3` LGT | limited · `DataOutputStream vtable index 12` | **playable** ok·ok·ok·ok·ok·n/a | ABI 12 `write([BII)V` · 13 `flush()V` |
| `71d1d8235bd1` SKT | not-yet | 같음 | — (4절) |
| `f07cbc782828` KTF | not-yet | 같음 | — |
| `3151fdc167b6` KTF | not-yet | 같음 | — |
| `5267badf20b3` KTF | not-yet | 같음(벽 문구에 lr 추가) | — |
| `7e2247bdf565` KTF | limited · AllocationFailure 795_OK | limited · 같은 벽 259_OK | — |
| `f981d228b757` KTF | limited · `MC_dbGetRecordSize` 244_OK | limited · 같은 벽 139_OK | — |
| `fe76e641bb3d` LGT | limited · JavaException unwrap | limited · 같은 벽 02_OK | — |

speed `n/a` 는 헤드리스가 «느림»을 판정하지 않는 값이다(부하 load1 115~154).

### 3. 고친 것의 근거
- **전화번호 ⑴ LGT C** (`a23f3c9fc2cb`, `binary.mod` Thumb 0x1aa8~0x1b6c): `memset(g, 0, 12)` · `GetSystemProperty("PHONENUMBER", sp+0x2c, 16)` · `memcpy(g, sp+0x2c, 12)` · … `memset(sp, 0, 8)` · `memcpy(sp, g, strlen(g) - 4)` 다음 `strcmp(sp, "0198080")` 류 13개 비교(개발·시험 번호 목록). 번호가 `""` 면 길이 `0xfffffffc` 복사가 매핑 끝 `0x1520000` 에서 잘못된 메모리 접근이 되고, 게임의 예외 처리가 `String.<init>(II[C)V` 를 찾다 `NoSuchMethodError` 를 되풀이해 호스트 스택이 넘쳤다.
- **전화번호 ⑵ LGT Java** (`b2da04c55cd4`): `getSystemProperty(PHONENUMBER)` → `""` → `String.length()` → `Math.abs(…)` → `ArithmeticException: / by zero`(디버그 로그 순서).
- 값은 SKT `MIN` 과 같은 `01000000000`. 이 값은 위 개발 번호 목록의 어느 앞자리와도 같지 않다(일반 이용자 경로).
  종전 C 쪽 주석 「넣으면 어떤 게임이 인증에 실패」는 타이틀을 대지 않았다 ⇒ **번호 문자열을 가진 116종 전·후 짝 재측**(아래 5절).
- **DB 없음** (`96dc32e781d3`): startApp 에서 `openDataBase(<이름>, 0, false)` → `closeDataBase` → 다시 열고 `getNumberOfRecords` 뒤 NPE(게임 코드 `java_throw`). WIPI 의 `create=false` 는 없는 DB 에 `DataBaseException` 이다. 종전 «항상 만든다»(e828e474) 는 MIDP 의 RecordStoreNotFoundException 을 피하려던 것이라, 그 예외를 `DataBaseException` 으로 바꿔 던진다.
- **EncodeImage** (`33801c1ba14f`): 헤더 `M_Int32 MC_grpEncodeImage(MC_GrpFrameBuffer src, x, y, w, h, M_Int32 *len)`. 반환은 메모리 id — `MC_grpCreateImage(img, bufID, off, len)` 이 받는 그 형태라 되읽을 수 있게 BMP(24bit)로 담는다. 영역은 프레임버퍼로 자른다. 게임이 그 바이트를 직접 해석하는지는 재지 못했다(호출 1곳 · 인자만 측정).
- **할당기** (`517ed32c92d6`): 경고 1줄이 원인을 찍었다 — `bucket free of 0x4904bb20: size 8 names a different bucket than the 32-byte slot it is`. 크기로 버킷을 고르면 다른 버킷 머리 밖(패닉) 또는 남의 비트를 지운다. `free` 는 C 처럼 주소로 버킷·리스트를 가른다. 크기를 잘못 넘긴 **호출자는 특정하지 못했다**(경고로 남긴다).
- **DataOutputStream** (`61ed69520fd3`, 0x1b218~0x1b254): 같은 스트림에 12(r1=배열·r2=0·r3=배열 길이) → 13(r0 만) → 14(r0 만, 기존 close) — `write(b, 0, b.length); flush(); close();`.

### 4. 못 고친 것 — 다음 벽(실측)
| sha12 | 다음 벽 | 왜 못 했나 |
|---|---|---|
| `71d1d8235bd1` | `m/V3` `m/A3` `m/XO_World` 27 멤버(`loadMBAC`·`loadMTRA`·`loadBMP`·`setView`·`draw`…) | MBAC/MTRA 3D 모델 해석·렌더러가 필요하다. 그리지 않는 스텁은 «안 죽게만»이라 넣지 않았다 |
| `f07cbc782828` | `com/ktf/kfc` 툴킷(`GForm`·`GMenubarForm`·`GMsgBox`·`GTextField`·`GTextListener`) + `org/kwis/msf/io/Network`·`Socket` | 툴킷 API 문서가 없고 온라인 게임이다(0329 판정 그대로) |
| `3151fdc167b6` | 커널 36 = 이름으로 인터페이스를 찾는 함수(`"MXUserMemInterf"`) — 반환의 slot0 을 곧바로 부른다 | 0 을 주면 곧 null 호출이다. slot0 이후 칸의 뜻을 모른다 |
| `5267badf20b3` | `FileSystem.exists` false → `new Exception` → `jump_2(예외, 0, *(g+8))` 대상 0 · lr `0x1385bd`(함수 0x1385a8) | `g` 가 가리키는 런타임 표가 무엇인지 식별 못 했다 |
| `7e2247bdf565` | 259~795 단계에서 `define_class_rust` 의 AllocationFailure panic | 손님 힙이 찼다. 무엇이 채우는지 재지 않았다 |
| `f981d228b757` | KTF DB 슬롯 11(헤더 이름 GetRecordSize) · 139~244 단계 | KTF DB 표는 헤더 순서와 다르다(5 = stat). 인자·반환을 재지 못했다 |
| `fe76e641bb3d` | 02 단계 `res/logo/lo0.nb`(17KB) 읽기에서 `Failed to instantiate array: Allocation failure` → 예외 생성 중 `class_definition.rs:87` unwrap panic | 몇 초 만에 힙이 찬다. 원인 미측정 |
| `7da00ecd4804` | 주소 0 읽기 | 1절 |

### 5. 퇴행
- **전화번호 116종**(바이너리에 `PHONENUMBER` 문자열 · KTF 55 · LGT 58) 전·후 짝(같은 시각 30초 A 프로브): 다른 것 5 — 좋아진 4(`b2da04c55cd4` `517ed32c92d6` `287af341dac8` `484108f6c8de`) · 나빠 보인 1 `4a4d2ac046f7`(주소 4 읽기) → 짝 3회 재측 **전·후 모두 3/3 PASS** ⇒ 간헐, 퇴행 아님.
  `484108f6c8de` 는 전 «clean exit(22/27)» → 후 끝까지 PASS.
- **DB 83종**(`openDataBase` 문자열 · KTF 76 · LGT 7) 전·후 짝: 다른 것 2 — 둘 다 좋아짐(`96dc32e781d3` `b2da04c55cd4`).
- **라이브 LGT 5종 · 가드 2종**: 후 전수 7종 모두 playable. speed 는 `49ade89578c5` slow → ok, `13d7e3c21856` `b475b6399684` ok → n/a(부하 속 헤드리스는 판정 안 함 · 계약상 unknown) — 속도 코드는 건드리지 않았다.

### 6. 되돌리면 red — 7 변이 전건
Java 번호 · C 번호 · 버킷 크기 선택 · 할당기 크기 분기 · DB 항상 생성 · EncodeImage 빈 반환 · ABI 12 행 — 각각 새 시험이 FAILED, 원상 green.

### 7. compat.json 갱신 방법
`playability-census.mjs run --bin <이 브랜치 wie_validate> --out <밖> <대상 21종 심볼릭 링크 디렉터리>` → `report` → 그 compat.json 의 6행을 `scripts/player-data.mjs` 의 `fromCensus`(import 와 같은 어휘 변환)로 바꿔 `status`·`axes`·`knownIssues_ko` 만 덮었다. 최상위 `enginePin` 은 `3c34efee` 그대로다 — 이 6행만 이 브랜치 코드로 쟀다.

### 게이트
`cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` · `npm run build:wasm` · `check-engine-contract` · `npm run audit` · `player-data` 전부 rc=0.
러너 줄(엔진 변경): `draw_j2me` · `helloworld_ktf/lgt` · `keydraw_ktf/lgt --inject --expect-last-frame` · `text_j2me --timeout 5` 전부 PASS · rc=0(이 브랜치 release `wie_validate`).
