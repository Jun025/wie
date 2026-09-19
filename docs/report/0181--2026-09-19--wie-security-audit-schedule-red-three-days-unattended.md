## [2026-09-19] `Security audit` 3일 red 은 «바깥이 바뀐 것»이 아니었다 — base swap 이 우리가 이미 올려 둔 판본을 되돌렸다 (wie-security-audit-schedule-red-three-days-unattended)

### ⒜ 무엇이 red 였나 — 로그 그대로

`gh run view 35420153215 --repo Jun025/wie --log-failed` 의 `cargo audit` 스텝(발췌, 원문 그대로):

```
    Scanning Cargo.lock for vulnerabilities (557 crate dependencies)
Crate:     rtrb
Version:   0.3.4
Title:     Double free / use-after-free in `ReadChunk::commit` when an element's `Drop` panics
Date:      2026-08-04
error: 2 vulnerabilities found!
warning: 3 allowed warnings found
ID:        RUSTSEC-2026-0274
Solution:  Upgrade to ^0.3.5 OR >=0.4.0

Crate:     rustls
Version:   0.23.44
Title:     TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries
Date:      2026-09-14
ID:        RUSTSEC-2026-0285
Severity:  5.3 (medium)
Solution:  Upgrade to >=0.23.45
```
실패 step = **`cargo audit`** · 종료 `##[error]Process completed with exit code 1.`

### ⒝ green→red 경계 — ★**코드가 아니라 `Cargo.lock` 이 통째로 갈렸다**

`d70b93f8`(09-16 green) ↔ `75ca3451`(09-17 red) 사이 `Cargo.lock` **+2,193 / −795줄**.
그 구간에서 lockfile 을 만진 우리 커밋은 둘이고, 판본을 옮긴 것은 ★**`36df9c31`
「[wie-p3-slice-d-merge-upstream-main-as-base-fix] base swap — upstream/main 을 base 로」** 다.
그 커밋의 **부모 ↔ 자신**을 직접 떴다:

| 대상 | swap 직전(`36df9c31^`) | swap 직후(`36df9c31`) | 뜻 |
|---|---|---|---|
| `rtrb` | **0.3.5** | **0.3.4** | ★대장 **C-1**(RUSTSEC-2026-0274)이 해소한 판본이 **되돌아갔다** = 취약점 부활 |
| `event-listener` | **5.4.2** | **5.4.1** | ★대장 **C-2**(RUSTSEC-2026-0221)이 해소한 판본이 **되돌아갔다** = unsound 경고 부활 |
| `rustls` | **없음** | **0.23.44** | swap 이 들여온 `sentry` → `ureq` → `rustls`. 그 판본이 RUSTSEC-2026-0285(2026-09-14) |
| `spin` | 0.12.2 | 0.12.3 | **전진** — C-3 은 무사 |
| `chacha20` | 0.10.0 | 0.10.1 | 둘 다 yanked — A-4 판정 불변, **설치 버전 표기만** 낡음 |

⇒ ★**한 문장: 브리프의 「코드가 아니라 바깥이 바뀌었다」 가설은 «반증됐다». 바뀐 것은 우리 `Cargo.lock` 이다.**
`rustls` 한 줄만 «바깥»(새 자문 + 새 의존)이고, `rtrb` 는 **우리가 고쳐 둔 것을 우리가 되돌린 것**이다.
`push` CI 가 green 이었던 이유도 이것으로 설명된다 — `rust.yml`·`coverage`·`Web`·`Engine contract` 중
**`cargo audit` 을 도는 것이 하나도 없다**(이 워크플로는 schedule·dispatch 전용이다).

### ⒞ 처방 — 갈래와 고른 이유

| 갈래 | 판정 |
|---|---|
| ⒜**의존성 올림** | ★**채택**. `rtrb`·`event-listener` 는 «새 판단»이 아니라 **대장 C-1·C-2 가 이미 낸 판단의 복원**이고, `rustls` 는 권고가 `>=0.23.45` 를 명시한다. `--precise` 3건 = `Cargo.lock` **+6/−7줄**, `Cargo.toml` 무접촉 ⇒ 상류 동기 부담 최소(경계 ⒡) |
| ⒝**권고 예외 등재** | ★**기각**. 억제는 해결이 아니고, 이 저장소는 `--ignore`·`audit.toml`·`deny.toml` **0건**을 Constraint 5 로 지킨다. 무엇보다 **패치본이 실재**해 예외를 쓸 이유가 없다 |
| ⒞**액션/툴체인 핀 이동** | ★**해당 없음**. 액션 버전은 움직이지 않았다 — 원인이 lockfile 임이 위 표로 확정됐다 |
| ⒟**지금은 못 고친다** | ★**해당 없음**. 세 대상 전부 crates.io 에 패치본이 있다 |

※**무시 등재를 쓰지 않았으므로 「재검토 조건」은 해당 없다.** 대신 ★**A-4 `chacha20` 의 기존 처분(별건 발권)은
그대로 두었다** — 그것은 권고가 아니라 레지스트리 yank 이고, 대장이 이미 «별건»으로 판정해 둔 자리다(경계 ⒠: repo 규칙이 이긴다).

### ⒟ ★**첫 red 뒤에 «둘째 red» 가 숨어 있었다** — 이것이 이 회차의 둘째 산출물

`cargo audit` 이 exit 1 로 죽어 **그 다음 step 이 아예 돌지 않았다.** 취약점을 고치자 그 step 이 red 를 냈다:

```
❌ 기대 목록에 있는데 cargo audit 이 «더는 내지 않는» 경고 1건 — 목록이 낡았다:
    - chacha20@0.10.0 yanked
❌ 대장 A 가 낡았다 …
    + 경고에만 있다(§A 에 행이 없다): event-listener@5.4.1 RUSTSEC-2026-0221
    + 경고에만 있다(§A 에 행이 없다): chacha20@0.10.1 yanked
    - §A 에만 있다(경고가 없다 = 해소됐다): A-4 chacha20@0.10.0 yanked
```
⇒ ★**취약점 red 가 목록 red 를 «가리고» 있었다.** 그래서 이 회차는 셋을 함께 닫는다:
`event-listener` 는 **5.4.2 로 올려 경고 자체를 없앴고**(권고 `patched = [">= 5.4.2"]` · 대장 C-2 가 이미 낸 결론),
`chacha20` 은 **올리지 않고 «설치 버전 표기»만** 0.10.0 → 0.10.1 로 고쳤다(A-4 의 별건 처분 존중).

### ⒠ 개악 양방향 — 5축 전건

| 개악 | 결과 |
|---|---|
| M1 `rtrb` → 0.3.4 | `cargo audit` **rc=1** ★red |
| M2 `rustls` → 0.23.44 | `cargo audit` **rc=1** ★red |
| M3 `event-listener` → 5.4.1 | `cargo audit` rc=0(경고는 비게이팅) · `check-audit-warnings` **rc=1** ★red |
| M4 기대 목록만 `chacha20` 0.10.1→0.10.0 | `check-audit-warnings` **rc=1** ★red(GONE) |
| M5 대장 §A 만 `A-4` 0.10.1→0.10.0 | `check-audit-warnings` **rc=1** ★red(집합 불일치) |
| **원형** | `cargo audit` **rc=0** · `check-audit-warnings` **rc=0** |

★M4·M5 를 갈라 잰 이유: 두 검사는 **같은 경고 집합**을 읽지만 **다른 파일**을 본다 — 한쪽만 고치면 다른 쪽이 red 로 남는다.

### ⒡ 남긴 것 · 안 한 것

- ★**`chacha20` 0.10.1 → 0.10.2 는 하지 않았다** — 대장 A-4 가 「별건 발권 대상」으로 이미 처분한 자리다(로드맵 6번).
- ★**`rustls` 의 «도달성 판정»은 하지 않았다** — 버전축으로 닫았다. 잰 것은 범위 하나뿐이고 대장 **C-4** 에 그렇게 적었다:
  `cargo tree -p wie_featurephone --target wasm32-unknown-unknown -i rustls` → **`did not match any packages`**
  (대조군 `wie-backend` 는 정상 출력 ⇒ «빈 답»이 아니라 «부재»다) ⇒ **배포 WASM 아티팩트에는 없고**, 도달면은 네이티브 호스트뿐이다.
- ★**워크플로를 끄거나 `continue-on-error` 를 더하지 않았다** — 그것은 신호를 지우는 것이다(브리프 Non-goal).
- ★**재발 방지는 대장에 적었다**: 「base 를 갈아끼우는 회차는 §C 행을 **전건** 재확인하라」.
  §C 의 「판정 유효 범위」 칸은 «의존성이 올라갈 때»를 겨눴는데 실제로 문 것은 ★**«lockfile 이 통째로 교체될 때»** 였다.
  ★기계 축(base swap 회차에 C 행 대조를 강제)은 **이 회차에서 만들지 않았다** — 새 검사기는 브리프의 Non-goal 이고,
  표본이 **1건**이라 지금 굳히면 오탐이 굳는다. 제안으로 남긴다(worklog).
