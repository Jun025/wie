## [2026-11-22] 로더가 «지문과 같은 jar» 를 고른다는 것을 시험으로 잠갔다 — KTF · SKT `from_archive` 시험에 미끼 jar 둘 (wie-content-fingerprint-loader-side-jar-choice-has-no-test)

**무엇을**
- `wie-ktf` · `wie-skt` 의 `from_archive_turns_the_owner_sound_on` 고정물에 미끼 jar 를 이름순으로 앞뒤에 하나씩 더했다(`a.jar` · `z.jar` — 본체는 `probe.jar`). 같은 시험이 `archive_jar(&files) == WIRING_PROBE_JAR` 도 단언한다. 합성 바이트뿐이고 제품 코드는 한 줄도 바뀌지 않았다(두 파일 모두 `#[cfg(test)]` 안 · 각 +6 −1).
- 계약 `docs/contracts/featurephone-public-data.md` §4: 문장에 박혀 있던 수(「꾸러미 36개 · 장부 밖 393행」)를 세는 명령으로 바꿨다.

**왜** — #599 재검의 F1 이다. #599 의 새 시험(`wie_identify` 의 `with_several_jars_the_key_is_the_one_the_loader_runs`)은 지문 쪽(`archive_jar`)과 묶은 함수를 본다. 로더(`from_archive`)가 묶은 함수를 버리고 다른 jar 를 고르는 개악은 시험이 전부 통과했다(검수 실측 rc=0 2건). `identify` 는 `from_archive` 를 부르지 않으므로 `content-key.mjs check` 도 그 형상을 못 본다.

**사용자 영향** — 없다. 시험과 문서뿐이다(엔진 산출물 · `compat.json` 무변경).

### 1. 어떻게 잡는가

주인 파일 처리 3종(`probe.sav` 빼기 · `probe.opt` · `probe.xor` 고쳐 쓰기)은 로더가 고른 jar 의 MD5 가 `WIRING_PROBE_JAR` 의 것일 때만 걸린다. 로더가 미끼를 고르면 세 파일이 꾸러미에 든 그대로 남아 기존 assert 가 red 다. `archive_jar` 단언은 같은 고정물에서 지문 쪽이 그 jar 를 고른다는 것을 한 자리에 적는다 — 둘이 합쳐 「로더가 올리는 jar == 지문이 고르는 jar」다.

### 2. 개악 실측 — 로더 쪽만 바꾼 것(지문 쪽 · 묶은 함수 무변경)

명령: `cargo test -p wie-ktf -p wie-skt --lib --no-fail-fast`. 개악은 `from_archive` 의 고르기 한 줄을 `files.iter()[.rev()].find(.jar)` 로 바꾼 것이다.

| 개악 | 이 PR 전(검수 실측) | 이 PR 뒤 |
|---|---|---|
| KTF `from_archive` 만 첫 번째 `.jar` | rc=0 · 생존 | rc=101 · `wie-ktf` `from_archive_turns_the_owner_sound_on` red(`probe.sav` 가 `Some([0])` · 기대 `None`) |
| SKT `from_archive` 만 마지막 `.jar` | rc=0 · 생존 | rc=101 · `wie-skt` `from_archive_turns_the_owner_sound_on` red(같은 자리) |
| KTF `from_archive` 만 마지막 `.jar`(반대 방향) | — | rc=101 · 같은 시험 red |
| SKT `from_archive` 만 첫 번째 `.jar`(반대 방향) | — | rc=101 · 같은 시험 red |

개악 없는 트리: 같은 명령 rc=0(`wie-ktf` 54 · `wie-skt` 9).

### 3. 계약 §4 의 수

`awk -F'\t' '$8=="zip"{print $3}' docs/contracts/runtime-adjustments.tsv | sort -u | wc -l` 이 오늘 36 을 낸다(종전 문장의 수와 같다). 전체 행 수는 `node scripts/player-data.mjs` 의 `titles`(429)다.

### 4. 한계

- LGT 는 손대지 않았다(F1 의 범위가 KTF · SKT 다 — LGT 로더는 #599 의 델타 밖이었다).
- KTF 의 「`<AID>.jar` 없음 + jar 1개」 대체 가지는 이 고정물이 지나지 않는다. 그 가지는 `jar_filename_falls_back_to_the_only_jar` 와 `wie_identify` 의 시험이 본다.
