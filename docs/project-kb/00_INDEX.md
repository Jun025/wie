# wie — Claude Project 기본지식 인덱스 (00_INDEX)

> **최종 갱신** 2026-07-09 (구 2026-07-02 Project 지식의 깊은 자산[§7·가드레일·용어집·모드 A/B]을 `10_deep-assets.md` 로 승계 통합) · **갱신 주체** Claude Code (wie 레포 세션)
> **이 폴더의 정체** `docs/project-kb/` = **wie 엔진 레포의 외부 Claude Project(선택적 조언자)에 주입하는 기획-계층 요약본**.
> 원본(SoT)이 아니다. 원본은 이 레포의 `AGENTS.md` · `CLAUDE.md` · `README.md` · `docs/` · `.github/workflows/` 다.
> otterpebble admin `/kb` 가 이 폴더를 GitHub API 로 온디맨드 조회해 다운로드만 제공한다(복제 없음).

---

## ⭐ 골든 룰

1. **SoT 는 레포다.** 이 KB 는 요약·포인터만 담는다. 규칙·코드 원문을 복제하지 않는다. 충돌 시 **레포가 이긴다.**
2. **수정 방향은 한 방향.** `레포 변경 → Claude Code(wie 세션)가 KB 재생성 → 사용자가 Claude Project 에 재업로드`. KB 가 레포를 거꾸로 덮어쓰지 않는다.
3. **KB 원문은 wie 레포 소유.** 생성·갱신은 wie 레포 세션의 Claude Code 만 한다. admin 은 조회·다운로드 제공만.
4. **이 KB 는 wie 엔진의 기획/사업 계층 전용.** 상세 구현·디버깅은 Claude Code(wie 세션)의 영역이다.
5. wie 는 연방의 **엔진 레포(#3)** 다 — otterpebble·qts·RustJava 를 이 레포 세션에서 수정하지 않는다(경계 SoT: otterpebble `.claude/rules/repo-boundaries.md`).

---

## KB 파일 목록 (추가 시 이 표에 등록)

| 파일 | 내용 | 갱신 빈도 |
|---|---|---|
| `00_INDEX.md` | 본 인덱스 · 골든룰 | 파일이 늘 때 |
| `01_brief.md` | wie 가 무엇인가 · 연방 내 역할 · featurephone 공급 관계 · 기획 맥락 | 방향 바뀔 때 |
| `02_status.md` | **현재 상태** — 자동발행 파이프라인 · 엔진 웹 계약 · CI · 두 트랙(타이틀 회수·§7) · 잔여 | 매 작업 세션 |
| `10_deep-assets.md` | 고유 기술자산 요약 — §7 벽(LGT AOT-Java) · 가드레일 · 용어집 (원문: docs/lgt_abi.md 등) | §7 국면 바뀔 때 |
| `90_project-setup.md` | 이 레포용 외부 Project 세팅 권장안(이름·설명·지침 — admin /kb 렌더) · 모드 A/B 규약 | 역할 모델 바뀔 때 |

> 신규 KB 파일 명명 규칙: `NN_주제.md`(두 자리 접두). 새 파일을 만들면 위 표에 한 줄 추가 — admin /kb 는 폴더를 자동 스캔하므로 커밋 즉시 반영된다.

---

## 동기화 방법 (사용자용)

1. otterpebble admin `/kb` 에서 **wie 레포 선택** → [전체 순차 다운로드].
2. wie 용 Claude Project 지식에서 기존 KB 전부 삭제 → 방금 받은 파일 전체 업로드. (매번 전체 교체 — 변경 추적 없음)
