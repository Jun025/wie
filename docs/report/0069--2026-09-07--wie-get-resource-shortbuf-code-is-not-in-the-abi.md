## [2026-09-07] `get_resource` 의 버퍼-부족 코드를 `-1` → `-18` 로 (wie-get-resource-shortbuf-code-is-not-in-the-abi)

**무엇을**: `wie_wipi_c/src/api/kernel.rs` 의 `get_resource` 가 버퍼 부족에 돌려주던 `-1` 을
`-18`(`M_E_SHORTBUF`)로 바꿨다. 비-주석 변경은 **두 줄**(분기값 1 + 그 값을 잠그는 시험 단언 1)이다.

**왜**: `-1` 은 핀 rev 의 `WIPICError` 어휘 `{1, 0, -9, -12, -18, -22, -25}` 에 **없다**. 그리고
`wipic_sys::kernel::get_resource` 는 `-> WIPICError` 로 타입돼 있고 `from_raw` 가 `transmute` 다 ⇒
게스트에 `-1` 을 건네는 것은 **유효하지 않은 판별자 = UB** 였다. 그 핀에서 그 `transmute` 를 지나는
호스트 함수는 `get_resource` 와 `graphics::create_image` **둘뿐**이고(래퍼 6 = ktf·lgt·simulation ×2),
`create_image` 는 `1` 만 돌려주므로 ★**이 한 자리가 transmute 경로 위의 유일한 어휘 밖 값**이었다.

**사용자 영향**: 게스트가 리소스를 자기 버퍼보다 크게 읽으려 할 때 받는 코드가 바뀐다.
★**실게임 호환성은 이 저장소에서 «확인할 수 없다»** — 상용 코퍼스가 없고(Constraint 9) WIPI 오류코드
규격 문서도 이 저장소·핀 저장소 **양쪽에 0건**이다(`M_E_*` 정의 0 — 이 파일의 그 이름들은 맨 주석이다).
⇒ ★**「안전하다」가 아니라 「측정 불가능한 호환성 위험과 확정된 UB 를 맞바꿨다」**가 정확한 문장이다.
같은 조건에 이미 `-18` 을 쓰던 형제 두 자리(`get_system_property`·`get_program_name`)는 **무접촉**이고,
이로써 세 자리가 일치한다.
