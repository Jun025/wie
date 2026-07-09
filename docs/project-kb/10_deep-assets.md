# wie 고유 기술자산 요약 (10_deep-assets)

> 이 레포가 수십 라운드의 리버스엔지니어링으로 축적한 **고유 자산의 기획-계층 요약**. 원문(SoT)은
> `docs/lgt_abi.md`(§7 전체 트레일 cp1~cp59) · `docs/lgt.md` · `docs/reference/`(ezi_dispatch_reference ·
> lgt_0x64_ordinal_table · AromaWIPI 자료) · `docs/ktf.md`. 여기엔 결론·경계·용어만 담는다.

## §7 벽 — LGT AOT-Java(ez-i) 렌더 (모드 B 대기: 외부 산출물 필요)

**무엇**: LGT 타이틀 102종 중 24종은 Java 를 ARM 으로 AOT 컴파일한 `binary.mod`(ELF) 앱(ez-i/Xceed).
이들이 부팅은 되나 **화면을 못 그리는 단 하나의 벽**. 대표 오라클: 배틀몬스터 타이틀 화면(크림 배경+로고+스프라이트).
나머지 78종은 WIPI-C clet 으로 이미 구동 렌더 경로가 있다.

**어디까지 왔나** (cp1~59, 전부 레포 커밋으로 검증됨):
- 클래스 등록·부팅·2단 vtable·필드 레이아웃·싱글톤·백버퍼까지 ✅. **렌더 경로 자체는 end-to-end 동작
  실증**(cp28: 게이트 강제 시 배경이 그려짐 — 실험, revert).
- **per-frame 구동 모델 확정**(cp55, 최대 성과): ez-i 는 EventQueue 에 `TIMER_EVENT(21)`을 프레임 주기로
  post 하고 게임의 자체 getNextEvent 루프가 스스로 디스패치한다. wie 가 21을 안 쏴서 루프가 영구 블록.
  **구현 가능하며 독점 기술이 아님**(실험으로 루프 159회 구동 확인) — 단독 커밋은 보류(가시 변화 0 + 공유
  경로 게이팅 필요).
- 소비되는 no-op import 중 `0xb/0xd` = 예외 핸들러 push/pop 판명(렌더 무관, cp58에서 제거).

**단 하나 남은 외부 블로커**: `0x64`(Java interface) import 테이블의 **ordinal→native 이름 등록표**.
렌더 필수 import `{0xe,0x10,0x12,0x1f,0x22}`가 각각 어떤 네이티브인지 모른다. **오프라인 획득은 소진 증명됨**:
AromaWIPI SDK 는 ABI 형태만 확인·번호 불일치 입증(`docs/reference/lgt_0x64_ordinal_table.md` — 0x0e 충돌),
dual-form 대조는 KTF 트윈이 native 라 사망, 후보 에뮬레이터들은 다른 VM. → **유일 언락 = 실기(디바이스) 트레이스**
(`get_import_function(0x64, index)→callee` 로깅).

**산출물 도착 시 활성화는 4단계로 즉시**(cp59 스캐폴드 커밋됨, 기본 비활성·회귀 0):
① 확정 ordinal 에 이름 채움 → ② 기존 `wie_wipi_java` org.kwis 임플로 라우팅(새 모델 날조 금지) →
③ LGT-AOT 한정 TIMER(21) 활성화(공유 Redraw(41) 미접촉) → ④ `scripts/lgt_render_probe.sh` + smoke_gate
(ADVANCE=YES·회귀 0 판정, 회귀 시 revert).

## 가드레일 (자율 세션 절대 제약 — 위반 시 회귀·신뢰 손상 전력)

- **회귀 0 최우선**: 미확정 가설로 런타임 동작 변경 금지. 실험/포싱 진단은 허용하되 반드시 revert,
  커밋은 회귀-0 검증분만. 커밋 전 `cargo fmt`+`clippy --workspace` 필수(AGENTS.md).
- **추측·날조 금지**: 0x64 ordinal 추측 바인딩 금지(가짜 핸들은 역참조→크래시). 게이트 직접 write·핸들
  날조 등 포싱을 영구 코드화 금지. PENDING 은 PENDING 으로 기록.
- **미접촉 영역**: 업스트림 dlunch/wie·PR 브랜치(푸시는 origin 만) · 게임 파일/`game_lab/`(절대 커밋 금지,
  BYOF) · 선재 핀(하이브리드 널점프·제노니아1 inject runaway) · 공유 코드 경로(Redraw(41) 매핑·공유
  EventQueue)는 LGT-AOT 한정 게이팅 없이 변경 금지.
- **작업 위생**: 캐시 JSON 불신 — 항상 라이브 실행 검증(cp47 stale-report 교훈). macOS 에 `timeout` 없음
  → kill-watchdog 패턴. false-PASS(강제 flush·더미 draw 로 게이트 속이기) 절대 금지. 보고는 정직하게.
- **종료 조건**: 결과가 "code 0 / 외부 블로커"로 수렴하면 더 짜내지 말고 외부 필요를 명시하고 멈춘다
  (→ 모드 B, `90_project-setup.md`).

## 용어집 (대화에서 그대로 쓰는 축약어)

- **§7**: LGT AOT-Java per-frame 렌더 벽(위). **0x64/0x1fb/0x1**: binary.mod import table id(Java interface/WIPI-C/C stdlib).
- **AOT thunk**: binary.mod 의 16바이트 고정 패턴 — (table,index) 정적 추출 근거. index 전량 확보됨; 이름표만 미확보.
- **dual-form(사망)**: named JAR↔binary.mod 대조안 — KTF 트윈이 native 라 폐기.
- **모드 A / 모드 B**: CC 자율주행 가능 국면 / 외부 산출물·사람 행동 대기 국면(§7이 대표적 모드 B).
- **감독형(vs 무인)**: 공유 코어·스케줄러·포크를 건드리는 작업은 무인 방치 금지 — 단계별 게이트 자기검증 후 회귀 0일 때만 커밋.
- **2-run 교집합**: smoke_gate baseline 등재 기준(두 독립 실행 모두 PASS — 부하성 플레이크 배제).
- **false-PASS**: 게이트를 위조 값으로 속이는 것(절대 금지). **선재 핀**: 손대면 안 되는 알려진 지점.
- **STUB_WITH_IMPL_SCAN**: 스텁으로 배선됐지만 기존 impl 이 있는 메서드를 찾아 라우팅하는 회수법.
- **callSerially 교훈**: additive 메서드 추가도 제어흐름을 바꿔 타 게임 회귀 가능 → 모든 수정은 전 플랫폼 게이트.
