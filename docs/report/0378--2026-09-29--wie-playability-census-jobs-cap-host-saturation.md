## [2026-09-29] playability-census — --jobs 기본 ncpu/2 · ncpu 초과는 깎는다 (wie-playability-census-jobs-cap-host-saturation)

**무엇을**: `scripts/playability-census.mjs` 의 `--jobs` 기본값을 `floor(ncpu×0.8)` → `max(1, floor(ncpu/2))` 로,
`ncpu` 초과 요청은 `ncpu` 로 깎고 stderr 경고 1줄(`jobsFor()` · `selftest` 에 5건). AGENTS.md 해당 절에
«`build-slot run --` 로 감싸고 `--jobs` 는 기본값» 1줄.

**왜**: 2026-09-29 17:5x `--jobs 32` 전수 짝 1회가 10코어 호스트를 load1 450 · sys 80%+ · idle 0% 로 포화시켰다
(총괄 실측 · perf-sample 간격 5→9~11분 · HOSTLOAD_UNMEASURABLE).

**전/후 실측** — 같은 바이너리 · `working/ktf` 앞 40파일(38 고유) · `--only probe` · 순차 실행.
★배경 부하가 매우 높았다: 다른 레인의 `--jobs 32` census(longplay 단계)가 계속 돌고 있었다(무접촉).

| | 소요 | load1 시작→끝 | 기록된 probe | UNMEASURED | 굶주림(미기록) | ticks 중앙 | paints 중앙 |
|---|---|---|---|---|---|---|---|
| 전 `--jobs 32` | **122 s** | 782 → 737 | 74/76 | **2** | **1** | 4,241,720 | 93 |
| 후 기본(5) | **489 s** | 699 → 782 | 76/76 | **0** | **0** | 5,233,601 | 129 |

★**「줄여도 느려지지 않는다」는 성립하지 않았다 — 벽시계로 4배 느리다.** probe 는 고정 예산(`--secs 30`)이라
동시성을 줄이면 소요가 선형으로 는다. 얻은 것은 측정 품질(UNMEASURED·굶주림 0 · probe 당 ticks +23% · paints +39%)과
다른 레인의 CPU 다. 빨리 끝내려면 `--jobs` 를 명시하라(상한 ncpu).

**시험**: `node scripts/playability-census.mjs selftest` → 21/21. 개악 2종 red 확인 — 상한 제거 → «above ncpu is capped» FAIL ·
기본값 0.8 복원 → «defaults to half» FAIL (각 20/21 · rc=1).

**사용자 영향**: 없음(로컬 전용 도구 · CI·배포 무접촉).
