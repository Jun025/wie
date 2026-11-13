## [2026-11-13] beta clippy red 해소 — `scale_image` 의 `dst` 를 슬라이스로 (wie-beta-clippy-red-window-as-mut-slice)

**무엇을**: `src/window.rs` 의 `Scaler::scale_image` 두 줄. `dst.as_mut_slice().align_to_mut::<u8>()` → `dst.align_to_mut::<u8>()`, 그리고 매개변수 `dst: &mut Vec<u32>` → `dst: &mut [u32]`. 호출부(1곳 · `&mut self.scaled_image_buf`)는 그대로다 — 역참조 강제 변환으로 같은 슬라이스가 넘어간다.

**왜**: beta `1.101.0-beta.1`(clippy `0.1.101` · 2026-11-10)의 새 린트 `clippy::unnecessary_as_slice` 가 `origin/main` `2e66f9d68` 에서 그 줄을 물어 `cargo +beta clippy --all -- -D warnings` 가 rc=101 이었다(이 트리의 변경과 무관 — 툴체인 이동). 린트가 권하는 대로 `as_mut_slice()` 만 지우면 이번에는 `clippy::ptr_arg` 가 stable·beta 양쪽에서 문다(`Vec` 이어야 할 이유가 사라져서). 그래서 매개변수까지 슬라이스로 바꿨다. `#[allow]` 는 쓰지 않았다.

**사용자 영향**: 없다. 동작 변경 0 — 같은 메모리를 같은 길이의 슬라이스로 본다. 네이티브 창 호스트만의 코드라 wasm 산출물에는 들어가지 않는다.

### 재현 (`origin/main` `2e66f9d68` · 손대지 않은 트리)

```
$ cargo +beta clippy --all -- -D warnings        # rc=101
error: this `as_mut_slice` is unnecessary because the following method (`align_to_mut`) can be called on `Vec` directly
   --> src/window.rs:217:50
217 |                 let (_, dstarr, _) = unsafe { dst.as_mut_slice().align_to_mut::<u8>() };
    = note: `-D clippy::unnecessary-as-slice` implied by `-D warnings`
error: could not compile `wie` (lib) due to 1 previous error
```

`as_mut_slice()` 만 지운 중간 상태(stable 도 rc=101):

```
error: writing `&mut Vec` instead of `&mut [_]` involves a new object where a slice will do
   --> src/window.rs:211:36
    = note: `-D clippy::ptr-arg` implied by `-D warnings`
```

### 같은 린트의 다른 자리
없다. 수정 뒤 `cargo +beta clippy --all -- -D warnings` 가 워크스페이스 전체에서 rc=0 이다. `as_slice()`/`as_mut_slice()` 호출은 그 밖에 18곳 있으나(`wie_featurephone`·`wie-web`·`wie-core-arm` 등) 전부 `From<&[T]>` 인자·비교·`match` 대상이라 이 린트(뒤따르는 메서드가 `Vec` 에서 바로 불리는 경우)에 해당하지 않는다.

### 게이트 (로컬 · 수정 뒤)

| 게이트 | rc |
|---|---|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --all -- -D warnings` (stable 1.99.0) | 0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` (stable) | 0 |
| `cargo +beta clippy --all -- -D warnings` (1.101.0-beta.1) | 0 |
| `cargo +beta clippy --target wasm32-unknown-unknown -- -D warnings` | 0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | 0 (837 passed · 0 failed · 54 스위트) |

`wie_validate` 러너는 돌리지 않았다 — 고친 곳은 `wie_validate` 가 쓰지 않는 창 호스트(`WindowHandle`)의 스케일러다. GitHub Actions 는 정지 중이라 CI 결과는 없다.

### 남는 것
- beta cargo 가 `cargo::unused_dependencies` 경고 9건을 낸다(`test-utils` 1 · `wie-util` 1 · `wie_cli` 7). cargo 매니페스트 경고라 `-- -D warnings` 에 걸리지 않고 rc 는 0 이다. 이 회차는 손대지 않았다(의존성 정리는 린트 수리가 아니다).
- upstream(`dlunch/wie` `57b1bce76`)은 이 파일을 `src/cli/window.rs` 로 옮겼고 같은 줄을 옛 형태로 갖고 있다. 다음 upstream 동기화에서 이 두 줄은 우리 쪽을 남긴다.
