## [2026-09-27] 훼밀리마트타이쿤 StringBuffer 22 — 이미 main 에 있다 · 벽은 DataOutputStream 14 로 옮겨 갔다 (wie-2026-09-24-baos-tobytearray-and-string-indexof-slots-adopt-p2)

**무엇을**: 코드 변경 없음. 판정과 측정만 남긴다.

**왜**: 2026-09-24 에 채택된 두 제안(`2026-09-24-baos-tobytearray-and-string-indexof-slots#p2` ≡
`2026-09-24-validator-reports-runtime-java-exceptions#p1`)은 훼밀리마트타이쿤(LGT)이
`Unimplemented: java/lang/StringBuffer vtable index 22` 에서 멈춘다고 했다. 그 행 —
`{ name = "append", descriptor = "(C)Ljava/lang/StringBuffer;", index = 22 }` — 은 이미
`a42e6867`(2026-09-25, `wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0`)이 배틀몬스터 호출부로
넣었다. 그 호출부는 `ldrh r1, [r3, #4]` 로 16비트 배열 원소를 넘기고 관측값이 r1 = 0xc624 라서
append(Z)(0/1) 가 아니라 append(C) 다 — 이 티켓의 ⒝(r1 분포) 질문의 답이 이미 그 주석에 있다.

**측정** (`origin/main` `7eb58422`, release `wie_validate --timeout 20`, 훼밀리마트타이쿤.zip):

| 트리 | 벽 | ticks | paints |
|---|---|---|---|
| main 그대로 (2회) | `Unimplemented: java/io/DataOutputStream vtable index 14` | 16 | 0 |
| main − StringBuffer 22 행 (2회) | `Unimplemented: java/lang/StringBuffer vtable index 22` | 11 · 10 | 0 |
| PR #328 의 `lgt_java_abi.toml` (2회, head `59718034`) | 벽 없음 · `stop: max-ticks` · 검은 화면 | 50,000,000 | 1 |

변이 검사는 양방향으로 선다: 22 행을 빼면 원래 벽이 그대로 돌아오고, 넣으면 다음 칸으로 간다.

**사용자 영향**: 이 회차로 바뀌는 것은 없다. 훼밀리마트타이쿤의 다음 벽 `DataOutputStream 14` 는
형제 PR #328(`OutputStream 14 = close()V`)이 여는 바로 그 칸이고, 그 파일로 돌리면 벽 없이 검은 화면
(1 paint)까지 간다. 그 다음 진단은 #328 착지 뒤의 일이다.
