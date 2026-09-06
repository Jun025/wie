## [2026-09-06] 클렛 카드 식별을 «이름»에서 걷어낼 수 있는가 — 두 경로를 실행으로 재고 **기각**했다 (wie-clet-card-identity-by-class-name-design-decision)
- **무엇을**: ★**설계 결정 회차 · 코드 변경 0.** 산출물은 `docs/worklog/2026-09-06-clet-card-identity-design.json` 뿐이다. `card_canvas.rs` **무접촉**.
- **왜**: 운영자 채택 제안 `2026-09-06-lgt-black-screen-name-compare#p1`. ★그 제안이 요구한 것은 코드 변경이 아니라 「설계 + 두 경로(KTF/LGT) 각각의 실측」이고, 이 회차가 그것을 했다.
- **★★⑴ 결론 1줄**: ★**대체 술어가 «없다» — 두 경로가 다른 종류의 객체라 한 벌로 덮이지 않고, 각각을 덮으면 술어 하나가 두 기구로 쪼개지면서 KTF 쪽 이름 의존은 그대로 남는다.**
- **★★⑵ 실측**(임시 프로브 → 픽스처 4종 → ★프로브 되돌림):
  `keydraw_lgt`·`helloworld_lgt` → `classDef=net/wie/CletWrapperCard` · `isInstance(CletWrapperCard)=`★**true**
  `keydraw_ktf` → `classDef=CletCard` · `isInstance(CletWrapperCard)=`★**false** / `helloworld_ktf` → ★**pushCard 자체가 안 불린다**
  ⇒ ★**LGT 의 카드는 «우리 것»**(Rust 프로토 · `CletWrapper::startApp` 이 호스트에서 민다) · ★**KTF 의 카드는 «게스트 것»**(클래스 이름을 **게스트 ARM 메모리의 널종료 문자열**에서 읽는다 — `wie_ktf/…/jvm_support/class_definition.rs`).
- **★⑶ 후보별 판정**: ⒜`isInstance(CletWrapperCard)` = **LGT 만** ⒝`isInstance(Card)` = 둘 다 true 인데 ★**모든 카드가 true** ⇒ 쓰면 일반 MIDP 게스트에 `disablePaint()` 가 불려 **방금 고친 검은 화면을 전 경로에 재현**한다 ⒞**부팅 플래그** = LGT 가능 · ★**KTF 불가**(부팅이 ADF `MClass` → `Main.main` 범용 경로이고 카드는 게스트 ARM 이 민다. `loadable_jar` 가 **모든** KTF 앱에 `client.bin` 을 요구하므로 「네이티브인가」도 아무것도 가르지 못한다 — 두 픽스처 ADF 가 **둘 다** `MClass:Clet` 인데 하나만 카드를 민다).
- **★⑷ 미지를 숨기지 않는다 — 그리고 그것이 판정을 흔들지 않는다**: `CletCard` 가 고정 이름이라는 것은 **픽스처 빌더**(`dlunch/wipi@068312d` 의 `clet_card.rs` — `ptr_name: c"CletCard"` · 부모가 `org/kwis/msp/lcdui/Card` **하나**)에서만 확인됐고 실게임은 코퍼스 부재로 확인 불가(Constraint 9). ★**고정이면 「새 카드가 생겨 놓친다」가 KTF 에서 성립하지 않고, 가변이면 대체할 호스트 신호가 없어 어차피 못 고친다** ⇒ 양쪽 갈래가 같은 결론이다.
- **사용자 영향**: 없음(코드 무변경). 대신 이 축이 **다시 열리지 않는다** — 왜 못 하는지가 실측과 함께 남았다.
- **★남는 제안 1건**(구현하지 않았다): `getClass().getName()` → **`class_definition().name()`**. 실측상 두 경로 모두 «내부 형식»을 그대로 주므로 점/슬래시가 **방어 대상이 아니라 비존재**가 되고 `pushCard` 마다 도는 JVM invoke 2회가 사라진다. ★단 **#p1 이 원한 것을 주지 않는다**(이름 두 개는 그대로) ⇒ 별건.
