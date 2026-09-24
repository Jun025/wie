## [2026-09-24] java/lang/Object index 5 — wait() 를 게임 코드로 배제, notify 유지 (wie-2026-09-23-object-notify-slot-and-monitor-ownership-adopt-p1)

**무엇을**: 배틀몬스터 `binary.mod` 의 lr=0x2308 주변을 디스어셈블해 index 5 의 잔여 후보 셋(notify·notifyAll·wait()) 중 wait() 를 배제했다. 값은 그대로(`notify()V = 5`)이고, `wie-lgt/data/lgt_java_abi.toml` 의 근거 주석을 «도출»에서 «관측»으로 올렸다. 검수 F1(「AAPCS 면 r2:r3」)도 같은 자리에서 고쳤다: 이 런타임은 인자 워드를 정렬 없이 순서대로 싣는다(`wie-lgt/src/runtime/java/jvm_support/method.rs`) ⇒ wait(J) 의 long 은 r1:r2 이고, r1 이 힙 포인터라 배제 결론은 그대로다.

**도구**: capstone 이 없어도 된다. LGT 이미지는 ELF(`.text` VMA 0x1000 · ARM 모드)라서 맥 기본 `objdump --triple=armv5te` 로 바로 읽힌다. `scripts/ktf-image-sweep.py` 는 손대지 않았다(Thumb·KTF 전용 · KTF 경로 무회귀).

**관측**(디스어셈블 원문 발췌):
```
2200: mov r12, sp ; push {r4, r11, r12, lr, pc}        <- 함수 시작
22bc..22f8: this.fields[tbl[0x12a]-계열] ← r1, r2, r4  <- 필드 셋 기록
22f4: ldr r3, [r0]          ; vtable
22fc: ldr r12, [r3, #0x18]  ; word 6 = index 5
2300: mov lr, pc ; 2304: bx r12   -> lr = 0x2308
2308..2330: 호출 두 번 뒤 return — 뒤로 가는 분기 없음

1f10 (형제 함수):
2014: ldr r3, [r6] ; 2018: ldr r12, [r3, #0x28]   ; index 9 — 이미지 전체에서 1회뿐
2020: bx r12
2030..2044: r6 = this.field[tbl 0x015006f4 + 0x12a] ; cmp r6, #0
2048: bne 0x2000                                   ; 뒤로 가는 분기
```
판정: index 5 는 **직선**(필드를 세팅하고 신호) · index 9 는 **같은 필드(0x12a)를 도는 루프**. 이 이미지는 한 필드를 두고 대기 쪽과 신호 쪽을 둘 다 갖고 있다. wait() 는 final 이라 자리가 하나뿐인데 그 자리가 9 에서 관측됐다 ⇒ 5 가 될 수 없다. notify 와 notifyAll 은 코드 모양이 같아 선언 순서만이 둘을 가른다(잔여).

**왜**: 값을 틀리게 두면 배틀몬스터가 «안 기다리고 지나가는» 상태로 돌 수 있었다. 이번 관측은 그런 일이 없다는 것을 게임 코드 수준에서 확인했다.

**사용자 영향**: 없음(값 무변경). 배틀몬스터 두 파일 모두 PASS · paints 3 · content true · stop deadline · java 예외 0 · `vtable index` 벽 0.

**잠금**: `abi_rows_cover_the_indexes_titles_actually_dispatch_on`(`wie-lgt/src/runtime/java/jvm_support.rs:1091`)이 `("java/lang/Object", 5, "notify", "()V")` 를 잠근다.
