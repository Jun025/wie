## [2026-09-28] new String((char[]) null) 호스트 패닉 → NullPointerException (wie-2026-09-28-ktf-classloader-cluster-8-verdict-adopt-p0)

**무엇을**: `wie-jvm-support/src/hardening.rs` 에 `java/lang/String` arm — 배열을 받는 `<init>` 6종
(`([C)V` `([CII)V` `([B)V` `([BII)V` `([BLjava/lang/String;)V` `([BIILjava/lang/String;)V`)의 배열 인자가 null 이면
게스트에 `NullPointerException` 을 던진다. 시험 2건 확장(`every_guard_is_actually_applied` 에 `String 6` ·
`null_arguments_raise_npe_instead_of_panicking` 에 `String([C)`·`String([B)` null).

**왜**: `docs/report/0330` — d552e095ddcf(KTF) 4회 중 2회가 `jvm-0.1.1 class_instance.rs:108:32` 패닉,
백트레이스 `rustjava_runtime String::init_with_char_array`. 핀 `5b84dd1` 의 여섯 생성자 모두 첫 줄이
`array_length`/`load_array` 라 null 이면 같은 자리에서 호스트가 죽는다 — 본 것은 하나지만 여섯을 함께 막았다.
★핀 밖 가드가 6개 늘었다. 핀을 올릴 때의 재확인은 `every_guard_is_actually_applied` 가 진다(기대 수 6 — 생성자가
사라지거나 이름이 바뀌면 red · AGENTS.md 사건대장 「If you move the pin, re-run that module's tests」).

**사용자 영향**: 빈(null) 문자 배열로 문자열을 만드는 게임이 에뮬레이터째 멈추지 않고 게임 쪽 예외 처리로 넘어간다.

### 검증
- 변이: `String` arm 무력화 → `every_guard_is_actually_applied` FAILED · `null_arguments_…` 가 **`class_instance.rs:108:32` 에서 패닉**(티켓 관측과 같은 자리) → 원복 green.
- d552e095ddcf `wie_validate`(release) 전/후 동시 짝 실행 18쌍(무옵션 6 · `--timeout 20` 8 · `--inject` 4) · load1 28–52:
  전 PASS 17 / FAIL 1 · 후 PASS 17 / FAIL 1 · ★**패닉 전 0/18 · 후 0/18**.
  FAIL 두 건은 같은 다른 벽(`java.lang.Error at aq.paint` · 부팅 중)으로 양쪽 1회씩이다.
- ★**한계 — 게임 A/B 는 이 수정을 증명하지 못했다**: 전 쪽에서도 패닉이 재현되지 않았다(0330 은 load1 167–223 에서 2/4).
  간헐 경로라 부하·타이밍에 달린 것으로 보이고, 수정의 증거는 위 시험·변이다. 재현되면 후 바이너리로 같은 조건을 다시 재라.

### 게임 파일명 유입
`corpus-name-inflow`: BOUNDED 3회/2쌍 · SUFFIX-ATTACHED 0 — 셋 다 `hardening.rs` 의 **기존** 줄(모듈 머리·주석, `origin/main` 에 이미 있음)이다. 이 회차가 더한 줄의 유입은 0.
