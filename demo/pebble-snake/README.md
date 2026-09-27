# Pebble Snake — 「바로 해보기」 데모 게임

파일이 없는 첫 방문자가 featurephone 셸에서 **바로 눌러 볼 수 있게** 만든 작은 MIDP 게임이다.
수달(뱀)이 물 위의 조약돌을 모은다. 타이틀 화면이 곧 조작 안내이고, 거기 적힌 키는 전부 게임에서 쓰인다.

| 키 | 동작 |
|---|---|
| 방향키 / 2 4 6 8 | 이동 (타이틀·게임 끝 화면에서는 시작) |
| 확인 / 5 | 멈춤 · 계속 (타이틀에서는 시작) |
| 0 | 소리 켜기 · 끄기 |

소리: 시작 · 조약돌 · 방향 전환 · 멈춤 · 게임 끝 — 5개 짧은 효과음. 엔진이 재생하는 형식은 SMAF 뿐이라
효과음은 **바이너리 파일 없이** `Sound.java` 가 음표 목록을 SMAF 로 조립한다.

## 저작권

우리 저작물이다 — **MIT**(`LICENSE`). 그림·소리·글꼴 자산 0: 화면은 전부 `Graphics` 호출로 그리고,
글자는 호스트 글꼴(neodgm)로 그려진다. jar 안에 `LICENSE.txt` 가 함께 들어간다.
외부 코드·자산을 섞지 마라 — 섞는 순간 이 절이 거짓이 된다.

## 빌드

```
demo/pebble-snake/build.sh      # → demo/pebble-snake/out/pebble-snake.jar (+ sha256 출력)
```

- JDK 9~19 필요(`--release 7` 을 내는 마지막 판이 19). `JAVA_HOME` 이 없으면 Homebrew `openjdk@17` 을 쓴다.
- **preverify 없음**: CLDC preverifier 가 붙이는 StackMap 은 KVM 용이고, wie 의 JVM 은 검증 단계 없이 해석한다.
  그래서 이 jar 는 wie(와 데스크톱 MIDP 에뮬레이터)용이지 실제 CLDC 단말용이 아니다.
- `stubs/` 는 이 게임이 부르는 MIDP 표면만 선언한 **컴파일 전용** 스텁이다(jar 에 안 들어간다) —
  그 밖의 API 를 부르면 컴파일이 실패하므로, 엔진에 없는 메서드를 모르고 쓰는 일을 막는다.
- 재현 가능: `jar --date` 가 타임스탬프를 고정한다. 같은 JDK 면 같은 바이트다.
  실측(`openjdk 17.0.20.1` · 2회 빌드 동일): sha256 `5ed67caa7a8b473cacc0a139fcff2df1a51d17f83903c6581d1c2009d7d05c68` · **7688 B**.
- jar 는 커밋하지 않는다(`*.jar` git-ignore · leak audit 이 추적 jar 를 거부). 셸 번들은 이 스크립트로 만든다.

## 확인

```
cargo run -q --release -p wie_cli --bin wie_validate -- --inject --boot-secs 1.5 --max-ticks 100000000000 \
  --keys "OK DOWN NUM4 UP RIGHT NUM5 NUM6 NUM0 NUM2 NUM6" demo/pebble-snake/out/pebble-snake.jar
```

`result PASS` · `audio.plays` > 0 · `midi_events` > 0 이면 화면·입력·소리가 다 닿은 것이다.
