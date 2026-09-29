# otterpebble 아케이드 — 「바로 해보기」 데모 게임

featurephone 셸의 「바로 해보기」용으로 새로 만든 작은 MIDP 게임이다. 지금은 **한 개**다(2026-09-29 운영자 지시로
나머지 데모를 걷어 내고 몽글셋을 이 게임으로 키웠다). 틀(`src/Arcade.java`) · 효과음 조립기(`src/Sound.java`) ·
컴파일 스텁(`stubs/`) · 빌드 조리법(`build-game.sh`)이 여기 있고, 게임은 `demo/<게임>/` 에 `src/` · `art/` · `LICENSE` ·
`MANIFEST.MF` · `build.sh` · `playthrough.keys` 를 둔다.

| 게임 | 디렉터리 | 한 판 | 조작 |
|---|---|---|---|
| 동물줄맞춤 — 같은 동물 셋을 한 줄로 | `dongmul-julmatchum` | 시간 막대가 다 줄 때까지(쉬움 90초 · 보통 60초 — 화면에는 숫자 없이 막대만) | 2 4 6 8 · 방향키 고르기 · 5 집기 → 방향키 바꾸기 · CLR 그만하기 |

틀이 주는 것: 게임 첫 화면은 **전용 홈**(시작 · 설정 · 도움말 · 최고 점수 · 기록 초기화, 2 8 고르기 · 5 선택).
설정은 소리 · 진동 · 난이도(쉬움/보통)이고 최고 점수(난이도별)와 함께 `RecordStore`(게임 이름) 한 레코드에 저장한다.
**CLR = 뒤로**: 설정 · 도움말 · 최고 점수에서는 바로 홈, 게임 중에는 「그만할까요? 5 예 · CLR 아니오」(그동안 시간이
멈춘다), 홈에서는 아무것도 안 한다(셸 밖으로 나가는 것은 셸 몫). wie 에서 CLR 은 `keyPressed(8)` 로 들어온다(실측).

## 저작권

우리 저작물이다 — **MIT**(게임 디렉터리 `LICENSE`, jar 안에 `LICENSE.txt`).
**빌린 것은 장르의 규칙뿐**이고, 이름 · 그림 · 소리는 전부 새로 만들었다. 동물 얼굴은 `art/Faces.java` 가 빌드 때
Java2D 도형으로 그려 PNG 로 넣고, 나머지 화면은 `Graphics` 호출로 그린다. 글자는 호스트 글꼴(neodgm), 효과음은
`Sound.java` 가 음표 목록을 SMAF 로 조립한다 — 외부 코드 · 그림 · 글꼴 · 소리 파일 0.
얼굴은 일부러 기존 캐릭터 묶음(애니팡 · 주주클럽 · 카카오프렌즈 · 라인프렌즈 · 산리오 · 포켓몬 · 동물의 숲)과
다르게 그렸다: 흰자 없는 작은 점 눈 · 볼터치 없음 · 표정은 짧은 선 하나 · otterpebble 톤의 탁한 파스텔.
섞는 순간 이 절이 거짓이 된다.

## 디자인

otterpebble `DESIGN.md` 의 밝은 무채색 토큰(바탕 `#F4F5F7` · 카드 `#FFFFFF` · 글자 `#191F28`/`#616B78`) 위에
동물 여섯 색만 얹는다. 호스트 글꼴은 크기가 하나뿐이라(엔진의 `Font` 가 크기를 무시한다) 위계는 굵기(1px 겹쳐 그리기)와
색으로 낸다. 색만으로 가르지 않게 동물마다 귀(둥근 귀 · 작은 옆귀 · 뾰족 귀 · 긴 귀 · 귀 없음+얼굴 가면 · 눈 혹)가 다르다.

화질: 엔진 화면은 240x320 고정이고(셸 `SCREEN_W/H` · 게임이 바꿀 수 없다) `Graphics` 도형에는 안티앨리어싱이 없다.
그래서 곡선이 많은 얼굴은 빌드 때 부드러운 가장자리(알파)로 그린 PNG 를 쓰고, 엔진 `drawImage` 가 알파를 섞는다.

## 빌드

```
demo/<게임>/build.sh      # → demo/<게임>/out/<게임>.jar (+ sha256)
```

조리법은 `build-game.sh` 하나다(JDK 11~19 · `--release 7` · preverify 없음 · `stubs/` 로 엔진에 없는 API 는 컴파일
단계에서 실패 · `art/*.java` 가 PNG 를 그림 · `jar --date` 고정 → 같은 JDK 면 같은 바이트). 이유는 그 파일 머리주석에 있다.
jar 는 커밋하지 않는다(`*.jar` git-ignore · leak audit 이 추적 jar 를 거부).

## 확인

```
cargo run -q --release -p wie_cli --bin wie_validate -- --inject --boot-secs 1.5 --max-ticks 100000000000 \
  --keys demo/<게임>/playthrough.keys --shotdir <dir> demo/<게임>/out/<게임>.jar
```

`playthrough.keys` 는 홈 → 설정 → 도움말 → 시작 → CLR 그만하기 → 다시 시작 → 시간 끝 → 홈까지 한 번에 가는 키 목록이다
(판이 무작위라 눈 감고 누른다). `result PASS` · `java_exceptions.count 0` · `audio.plays > 0` 과 `--shotdir` 의 장면을 본다.
