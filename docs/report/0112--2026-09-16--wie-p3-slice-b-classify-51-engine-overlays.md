## [2026-09-16] 조각 B — ② 엔진 오버레이 51건을 「폐기 / 재적용 / upstream 발신」으로 분류 (wie-p3-slice-b-classify-51-engine-overlays)

### 무엇을

`docs/upstream-realign-p3-slice-b-triage.md` 를 신설해 upstream 머지 예행의 **미해결 51행 전건**을 세
갈래로 분류했다 — **폐기 21 · 재적용 24 · 발신 6 · 미분류 0**. 제품 코드 **0줄**.
계획 문서 `docs/upstream-realign-p3-slices.md` 의 `UD 13` 분해를 **2곳 전건** 정정했다(전수 계수로 닫음).

측정 트리: `origin/main c77998ad` ↔ `upstream/main 44fbf265` ↔ `merge-base fa641a8a`(2026-09-16 07:2x KST).
머지 예행은 격리 워크트리에서 돌고 `--abort` + `worktree remove --force` 로 제거했다.

### 왜

- **수를 다시 쟀다.** 미해결 총계 **74**(UU 35 · AU 19 · UD 17 · AA 3)는 계획과 같다. 그러나 갈래 분해는
  **② 50 + ③f 1** 이다 — 계획 §1 은 `③f = wie_cli/` 를 별 갈래로 선언하는데 §3 의 갈래별 표에는 그 칸이
  없어 그 1건이 ② 로 접혀 51 이 됐다. **총계는 양쪽이 같고 틀린 것은 분해뿐**이라, 51행 전건(② 50 + ③f 1)을
  분류해 어느 읽기에서도 미분류가 0 이 되게 했다.
- **심볼 계수만으로 닫지 않았다 — 양방향으로 틀린다.** `wie-jvm-support/src/runtime.rs` 는 `inUp=1 notUp=0`
  으로 폐기처럼 보이는데 실제 델타는 `hardening::harden` 호출이고 upstream 에 `harden` 은 0건이다. 반대로
  `wie-ktf/.../wipi_c/context.rs` 는 `added=0` 으로 볼 것이 없어 보이는데 행 전체가 폐기다. 그래서 `added=0`
  17행과 `inUp>0` 전건을 diff 와 upstream 파일을 **열어서** 따로 봤고, 폐기 21행은 전건
  `git show upstream/main:<path>` 로 해당 심볼의 실재를 확인했다.
- **폐기 7건은 「upstream 이 가졌다」가 아니라 「우리 의존이 이미 가졌다」이다.**
  `wie_wipi_java/src/classes/java/**`(4 클래스 + 모듈 배선 3)는 `rustjava@5b84dd1
  java_runtime/src/loader.rs:26/36/80/96` 이 같은 FQCN 을 등록하고, `wie_jvm_support/src/lib.rs` 가
  클래스패스를 `RT_RUSTJAR : WIE_RUSTJAR : <jar>` 순으로 세우므로 **우리 사본은 오늘 이미 그림자에 있다.**
  원 커밋 `5603a7f9` 는 2026-07-02(구 포크 핀)이고 핀이 `5b84dd1` 로 간 것은 2026-09-04(`1762a32c`) —
  **파일 주석의 「Not provided by the bundled java_runtime」가 그 사이에 낡았다.**
- **verdict §6-P4 의 「선 안쪽 10종」이 오늘 값으로 6종이 된다.** 위 4종이 폐기로 빠지므로 보내 봐야 중복이다.
- **verdict §6-P4 의 「`canvas.rs` +149줄은 전부 단위테스트 9개이고 구현이 아니다」는 두 곳이 틀렸다.**
  `+149` 는 오늘도 맞다. 그러나 ⑴시험 9는 우리 파일의 **총** 수이고 base 가 이미 1개를 가져 **더한 것은 8개**,
  ⑵**구현 hunk 가 둘 있다** — `Rgb332Pixel` u8 오버플로 교정(★upstream 에 이미 있다 ⇒ 폐기 몫)과
  `decode_image` 의 `data.len() >= 4` 경계 가드(★upstream 0건 ⇒ 재적용 몫).

### 사용자 영향

지금은 **없다** — 분류표이고 코드는 0줄이다. 다음 회차에 나타난다: 조각 C 가 폐기 21행을 지우면 머지 미해결이
**74 → 53 이 기대값**(★등식이 아니라 상한이다 — `UU` 5행은 파일이 남고 델타만 사라지므로 `53 ≤ N < 74`)이고,
그만큼 조각 D 의 머지가 사람이 읽을 수 있는 크기에 가까워진다.

### 한계 — 숨기지 않는다

- **분류는 «동작»을 보증하지 않는다.** 폐기 #9~#16 은 Java 가시 시그니처 일치까지만 봤고 본문 의미는
  비교하지 않았다. 그래서 이 표는 **폐기 후보**이고 집행은 조각 C 가 4게이트 + 5픽스처 + 「우리 안의 호출자
  0건」 grep 을 지고 한다.
- **upstream 의 `rustjava-runtime ^0.1.1` 내용은 못 쟀다** — crates.io 판이고 이 머신 레지스트리에 없다
  (`~/.cargo/registry/src/*/java_runtime-*` 0건). 단 §3-A 의 폐기 판정은 **우리 핀 `5b84dd1` 실측만으로 성립**한다.
- **폐기 #5~#8 이 걸린 원 커밋의 「5 KTF 타이틀」은 재현 불가** — 292 코퍼스가 이 머신에 없다.
- **`skt` 는 이 표에 한 행도 없다**(② 미해결에 `wie_skt/` 0건). 사실이지만, 커밋된 skt 픽스처가 0 이라
  조각 D 에서 skt 는 신호 0 이라는 계획 §5 의 대가는 그대로다.
