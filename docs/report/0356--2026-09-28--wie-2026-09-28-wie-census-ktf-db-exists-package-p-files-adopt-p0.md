## [2026-09-28] KTF net 표가 30칸에서 끝나 있었다 — 04159045a7ea 의 «PC 0» 은 널 콜백이 아니라 표 끝을 넘은 호출 (wie-2026-09-28-wie-census-ktf-db-exists-package-p-files-adopt-p0)

**무엇을**
- KTF net 메서드 표(`get_net_method_table`)를 **64칸**으로 늘렸다(다른 미식별 KTF 표와 같은 크기). 30–63 은 좌표를 싣는 미식별 스텁(`gen_unnamed_table_stub`), **33 은 `net::socket_close`**(네트워크 없음 → `M_E_ERROR`).
- `Arm32CpuEngine` 의 `Undefined instruction` 오류가 **고장 난 자리의 `pc`·상태(thumb/arm)·명령어 워드·`lr`** 를 싣는다.
- 시험 2 — `net_table_covers_the_ktf_extension_slots` · `undefined_instruction_names_the_faulting_pc_and_lr`.

**왜 — «PC 0» 은 덤프 위치가 만든 허상이었다**
- 원 보고(0342)의 레지스터 덤프(전부 0 · PC 0 · `[SP] = 0xdeadbeef`)는 **고장 지점이 아니다.** `KtfEmulator::tick_for` 가 오류가 **다 풀려 올라온 뒤** 덤프하는데, `run_function` 은 오류 경로에서도 호출자 문맥을 복원한다. 그래서 끝에 남는 것은 **스레드의 초기 문맥**(`ThreadState::new`: 스택 꼭대기 · 나머지 0 · CPSR 0x10)이고, `0xdeadbeef` 는 할당기 `CANARY_VALUE` — SP 가 스택 할당의 끝에 있다는 뜻일 뿐이다.
- 같은 전수(#374, `/tmp/dbx-census`)의 KTF FAIL 덤프 **44건 중 44건**이 똑같은 초기 문맥이다. «널 콜백 호출로 보임» 은 이 덤프에서 나온 추론이고 근거가 없었다.
- 고장 지점을 오류 메시지에 싣자 **매번 같은 값**이 나왔다: `Undefined instruction at pc=0x4904eea4 (arm) insn=0x71004791 lr=0x10936f`.
  - 호출부(`0x109368`)는 `(**(sl+0x21c)) + 0x84` 를 읽어 `bx r3` 베니어(`0x162850`)로 부른다. 그 값 `0x4904eea0` 이 **짝수**라 ARM 상태로 들어갔다.
  - `sl+0x21c` 가 가리키는 표는 **net 인터페이스**다 — 게임의 용법이 우리 표 순서와 맞는다: +0 `MC_netConnect(cb, param)` · +4 `MC_netClose` · +8 `MC_netSocket(2, 1)` · +0x10 `MC_netSocketWrite`. 임시 계측(커밋 안 함)으로 확인: 표의 30칸 스텁 `0x710058a1…0x71005a71` 바로 뒤에 `WIPICInterface` 가 할당돼 있고, +0x84(= **33번**)는 그 구조체의 `misc_interface` 포인터 — 함수가 아니라 **표 주소**를 코드로 실행했다.
- 게임은 net 표를 **33번까지** 쓴다. 30–33 은 모두 slot 2 가 돌려준 fd 를 받는다: 30 `(fd, …, 콜백)` · 31 `(fd, buf, len)` · 32 `(fd, buf, len, 3000)` · **33 `(fd)` 단독 — 오류 정리 경로**. 그래서 33 을 소켓 닫기로 읽었다. 우리 `MC_netConnect` 는 항상 실패 콜백을 주므로(네트워크 없음) 게임은 정리 경로로 가고, 거기서 33 을 부른다. 실측 인자는 `MC_netSocketClose(-1)` — 게임 자신의 «소켓 없음» 값이다.
- 30–32·34–63 은 **이름을 붙이지 않았다** — 닿는 타이틀이 생기면 좌표 메시지(`selector 11 (struct slot Net), function N` + r0–r3)가 그 다음 사람의 출발점이다. 종전처럼 표 끝을 넘어 이웃 할당으로 뛰지는 않는다.

**부하 의존인가 — 아니다(티켓 요구 ⑴)**
- 수정 전 바이너리(첫 8회는 `origin/main` 그대로 · 이후는 오류 메시지에 좌표만 더한 판 — 동작 동일), release, `--inject --timeout 30`:

| 조건 | loadavg(1분) | FAIL | 나머지 |
|---|---|---|---|
| 순차 8회 | 371–399 | 0/8 | PASS 7 · UNMEASURED(clean exit) 1 |
| 순차 12회 | 165–188 | **7/12** | PASS 4 · UNMEASURED 1 |
| P6 병렬 12회 | 170–182 | 1/12 | PASS 6 · UNMEASURED 5 |
| 순차 14회(계측판) | 첫 회 ≈170 · 이후 290–375 | 1/14(첫 회) | PASS 5 · UNMEASURED 8 |

- FAIL 은 **전부 같은 `pc`/`lr`** 이고 키 단계(15·22·25)만 다르다. ⇒ 경쟁 상태가 아니라 **결정적 결함**이고, 부하는 «키 스크립트가 네트워크를 부르는 화면에 닿느냐»만 바꾼다. 부하가 높을수록 덜 닿는다.
- 조용한 시간대는 이 회차에 없었다(시작 시 load 215 · idle 0%). 가장 낮은 창(≈165)에서 가장 잘 재현됐다.

**수정 후** — 같은 타이틀 23회: **FAIL 0**(PASS 21 · UNMEASURED clean exit 2, 10단계). 로그를 받은 11회 중 **5회가 정확히 그 경로**(`MC_netConnect` → 실패 콜백 → `MC_netSocketClose(-1)`)를 지났고 5/5 PASS 27. load 128–186.

**사용자 영향**: 이 게임에서 네트워크 기능으로 들어가면 에뮬레이터가 죽던 것이, 연결 실패로 처리되고 게임이 계속된다.

### 퇴행
- 4게이트 + beta clippy 전부 rc=0. 엔진 러너 블록 전건 PASS(`keydraw_*` `--inject --expect-last-frame` paints 55 · rc=0 · `text_j2me --timeout 5` PASS).
- KTF 표본 30종(#374 전수 목록 9개 걸러 1개, P3, 수정 후 1회): **net 표를 부른 타이틀 0** · FAIL 5 → 5(같은 타이틀) · 새 FAIL 0. 뒤집힘 PASS→UNMEASURED 3 은 전부 `max-ticks`(부하 백스톱), UNMEASURED→PASS 2 — 모두 net 호출 0 이라 이 변경이 닿지 않는다.
- SVC 스텁 소비: `get_wipic_interfaces` 1회당 +34(이 타이틀 2319/4096).

### 변이(되돌리면 red)
- net 표 확장을 `30..30` 으로 되돌린다 → `net_table_covers_the_ktf_extension_slots` red(30 ≠ 64). 실기 대응: 수정 전 판 순차 7/12 FAIL.
- `Undefined instruction` 메시지에서 좌표를 뺀다 → `undefined_instruction_names_the_faulting_pc_and_lr` red — ★실행하지 않았다(정확 문자열 단언이라 자명).

### 남긴 것
- `tick_for` 의 레지스터 덤프는 여전히 초기 문맥을 보여 준다(44/44). 이번 회차는 `Undefined instruction` 만 고장 좌표를 싣게 했고, `InvalidMemoryAccess` 는 주소만 싣는다(`pc` 없음). 덤프 자체를 고장 시점으로 옮기는 것은 이번 범위 밖이다.
- 다른 짧은 표(util 7 · misc 5 · media 27 · uic 45 · unk3 0 · unk12)도 같은 모양의 위험이 있지만 **닿는 타이틀을 관측하지 않았다** — 넓히지 않았다(스텁 공간 비용 대비 근거 없음).

<!-- corpus-name-inflow v1 subjects=4 tree=a566087743341340 B=0/0 P=0/0 S=0/0 -->
