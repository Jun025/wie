## [2026-09-08] 공급망 대장 A 를 실제와 맞췄다 — 그리고 `chacha20` 은 «암호 결함»이 아니었다 (wie-supply-chain-ledger-a-is-stale-two-resolved-one-missing)

**무엇을**: `docs/project-kb/02_status.md` 의 대장 **A** 를 재측값과 맞췄다 — 해소된 2건을 **C 로 이동**
(구 ID 병기 · 행 삭제 0), 새 경고 `chacha20` 을 **A-4** 로 등재하고 **도달성 판정 ⓓ** 를 새로 썼다.
★**의존 무접촉**: `Cargo.lock`·`Cargo.toml` **0줄** · `cargo update` **0회** · 억제 추가 **0**.

**왜**: 운영자 채택 제안 `2026-09-06-dependency-exposure-verdict-place#p0`.

## F1 — 재현(제안 수 인용 아님)

**⑴ 도구 = `cargo audit`**(`rust-audit.yaml:80` 이 무-플래그로 부른다 · `deny.toml`·`audit.toml`·
`.cargo/audit.toml` **전건 부재**). 실측 `cargo-audit 0.22.2` · advisory-db `8a1eb4f9`(1,242건) ·
`Cargo.lock` **436 패키지** · **rc=0**:

```
Crate: ttf-parser  Version: 0.25.1  Warning: unmaintained  ID: RUSTSEC-2026-0192
Crate: chacha20    Version: 0.10.0  Warning: yanked
warning: 2 allowed warnings found
```

**⑵ 대장 A 의 행**(`docs/project-kb/02_status.md`): `:53` **A-1** ttf-parser RUSTSEC-2026-0192 ·
`:54` **A-2** event-listener RUSTSEC-2026-0221 · `:55` **A-3** spin(yanked). = **3행**.

**⑶ 양방향 집합 차 — 계수로 끝내지 않았다**:

| 방향 | 원소 |
|---|---|
| 경고에만 있다(표에 없음) | ★**`chacha20` 0.10.0 yanked** |
| 표에만 있다(경고 없음) | ★**A-2 `event-listener`** · ★**A-3 `spin`** |
| 양쪽 다 | A-1 `ttf-parser` RUSTSEC-2026-0192 |

⇒ 교집합 **1**뿐이다. ★**수(2↔3)만 봐도 어긋나지만, 구성은 «셋 중 둘»이 다르다.**

**⑷ 「이미 해결」 확인 — `Cargo.lock` + 커밋**: `event-listener` **5.4.2**(권고 `patched = [">= 5.4.2"]`
경계와 정확히 일치) · `spin` **0.12.2**(비-yanked 패치본). 두 값을 만든 커밋은 **하나**다 —
**`22df9531` 2026-08-08 “chore(deps): bump event-listener 5.4.2 and spin 0.12.2” (PR #55 ·
`wie-supply-chain-cargo-updates-a2-a3`)**. 직전 `00657c34`(2026-07-31)에서는 `5.4.1`/`0.12.0` 이었다.

★★**그래서 이것은 «누가 표를 안 고쳤나»의 문제다** — 그 커밋은 **`STATE.md` 는 갱신했고**(`:822` 에
「2026-08-08: 공급망 대장 A-2·A-3 착지」가 있다) ★**대장 표만 안 고쳤다.** 대장 A 등재(2026-08-03)
**닷새 뒤**에 낡았고 **한 달 넘게** 그대로였다. 같은 파일의 로드맵 6번도 그 둘을 **「발권 대기 — 미착수」**
로 계속 말하고 있었다(같이 정정했다).
★**`chacha20` 은 반대 방향이다**: `0.10.0` 자체는 **2026-03-21 `0bb828e6`(bump rodio)** 부터 있었고,
**나중에 yanked** 돼 경고가 «저절로» 생겼다 — lockfile 변경 없이 표가 낡는 경로다.

★**기계 축은 낡지 않았다**: `.github/rust-audit-expected-warnings.json` + `scripts/check-audit-warnings.mjs`
는 2026-09-06 부터 실제와 **일치**하고 재측 시점 **rc=0**. ⇒ ★**틀어진 것은 «산문 표» 하나뿐이었다.**

## F2 — 표를 맞췄다 (행을 지우지 않았다)

| | 전 | 후 |
|---|---:|---:|
| A 행 | 3 (A-1·A-2·A-3) | **2** (A-1·**A-4**) |
| C 행 | 1 (C-1) | **3** (C-1·**C-2**·**C-3**) |
| 합계 | 4 | **5** — ★**삭제 0 · 신규 1** |

- A-2·A-3 → **C-2·C-3** 으로 이동하며 **「구 A-2」·「구 A-3」 병기** + **해결 시각·근거 커밋·재확인 버전** 기재.
  ★**번호는 재사용하지 않았다** — 회차 기록·로드맵이 그 ID 로 이 표를 가리킨다.
- 근거 블록 **ⓐ·ⓑ 는 물리적으로 옮기지 않고 제목에 리다이렉트**를 달았다(「구 A-2 · 해소분 → C-2」).
  ★**문단 55줄을 옮기면 diff 만 커지고 정보가 늘지 않는다**(제안 스스로 「대부분이 문단 이동」이라 적었다).
- ★**대장 자기 불변식 복원**: 「경고 수 == A 행 수」 → **2 == 2**.
- ★**형식 무접촉** — 새 절·새 열·새 표기법 **0**. C 의 **판정 유효 범위** 칸은 기존 규약 그대로 채웠다.

## F3 — `chacha20` 도달성 판정 (M 축)

★**성격부터**: advisory-db 에 `chacha20` 권고는 **없다**. `yanked` 는 crates.io 인덱스 상태다(ⓑ 와 같은 형태).
`0.10.0`·`0.10.1` 둘 다 yanked · 비-yanked 최신 **`0.10.2`(2026-08-27)**.

★**사유는 추정하지 않고 원문을 떴다**(ⓑ 의 방법 그대로 · `.crate` 2본 diff): `0.10.2` CHANGELOG =
**“Use of SSE4.1 intrinsic in SSE2 backend of RNG and legacy (64-bit counter) variants”**(#580).
소스에서 자리를 찍었다 — `src/backends/sse2.rs:114 rng_inner` 는 `#[target_feature(enable = "sse2")]` 인데
**`:133` 이 `_mm_extract_epi32`(SSE4.1 `PEXTRD`)를 «무조건» 부른다**(RNG 경로엔 카운터 폭 가드도 없다).
⇒ SSE2 만 있고 SSE4.1 이 없는 CPU 에서 **SIGILL** — ★**암호 결함이 아니라 이식성 결함이다.**

**⑴ 호출 경로는 «있다»**(명령 출력):

```
$ cargo tree -i chacha20
chacha20 v0.10.0 └── rand v0.10.1 ├── rand_distr v0.6.0 └── rodio v0.22.2 └── wie_cli v0.0.1
$ cargo tree -p wie_web --target wasm32-unknown-unknown -i chacha20
error: package ID specification `chacha20` did not match any packages   ← 배포 WASM 에는 «없다»
```

**⑵ 취약 «기능»은 우리 API 에 «닿지 않는다» — 독립 3축**(모두 명령 출력으로):

| 축 | 근거 |
|---|---|
| **호출** | 워크스페이스 전 소스: `StdRng` **0** · `ThreadRng` **0** · `rand::rng()` **0** · `make_rng` **0** · `.dither(` **0** · `Dither` **0** · `noise::` **0** · `chacha20`/`ChaCha` 직접 참조 **0**. 상류 `rodio 0.22.2` 의 `rand` 사용처는 `dither.rs`(기본 `SmallRng` = Xoshiro, **ChaCha 아님**)·`noise.rs` 뿐이고 **둘 다 옵트인 콤비네이터**다. 우리 rodio 표면은 `main.rs:26` 의 `DeviceSinkBuilder`·`Player`·`SamplesBuffer`·`SampleTypeConverter` 넷 |
| **아키텍처** | SSE2/AVX2 백엔드는 `lib.rs:167~199` 상 **x86/x86_64 에서만 컴파일**된다(그 밖은 neon·soft) |
| **런타임** | 디스패치 avx512 → **avx2** → **sse2** → soft(`rng.rs:70~79`) ⇒ **AVX2 가 있으면 도달 자체가 없다** |

**⑶ 전제를 «되돌릴 수 있게» 적었다**(표의 판정 유효 범위 칸 + ⓓ):
⑴**`rodio@0.22.2` 기준** — dither/noise 가 옵트인이라는 것은 그 판본의 소스 실측이다. **rodio 를 올리면 축1 을 다시 재라.**
⑵우리가 `.dither(...)`·`noise::*`·`StdRng`·`ThreadRng`·`rand::rng()` 중 하나라도 쓰기 시작하면 **무효**.
⑶축2·3 은 상류 구조라 `chacha20` 판본이 바뀌지 않는 한 유효.
⑷★**축2·3 이 「우리 맥이 aarch64 라 안전」을 뜻하지 않는다** — 배포 네이티브 바이너리는 x86_64 에서도 돈다.
그 축들이 좁히는 것은 «AVX2 없는 x86» 뿐이고, **그 경우에도 축1 이 지배한다**.

★**미완 없음** — ⑴⑵⑶ 을 전부 했다.

## 다른 문서의 «복제 표» — 세어서 적기만

| 곳 | 결과 |
|---|---|
| `STATE.md` | ★**복제 표 0**. A-n 토큰 **2건**은 회차 서술이고(`:347` 낡음을 «관측»만 한 기록 · `:822` A-2·A-3 착지 기록), `chacha20` **5건**도 전부 회차 서술 |
| `REPORT.md` | **0**(고정 안내) |
| `docs/report/**` | 그 권고를 언급하는 파일 **3건** — 회차 기록이라 **정정 대상 아님** |
| `.github/rust-audit-expected-warnings.json` | 기계 기대목록 **1건** — ★**이미 실제와 일치**(손대지 않았다) |

**사용자 영향**: 없다(문서 · 의존 무접촉). 값은 ★**표를 믿고 인용하는 사람이 더는 틀리지 않는다**는 것과,
`chacha20` 판정이 남아 **다음 회차가 처음부터 다시 파지 않는다**는 것이다.
