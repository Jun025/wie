## [2026-09-25] #161 이 떨군 LGT 수리분 전수 감사 — SVC 행 소실 5 · 의미 변경 3 · 가드 소실 0(이미 복원된 1 제외). 새로 드러난 것은 WIPIC 604/605 (wie-2026-09-24-lgt-hard-fail-14-axes-adopt-p4)

**무엇을**: 기반 교체 직전 트리 `d70b93f8` 과 `origin/main`(`ddc83420`)의 LGT SVC 표·dispatch·조기 반환 가드를 기계로 대조했다.
★**복원 코드 0줄** — 이 회차의 산출물은 목록이다. 복원 후보는 worklog `2026-09-25-lgt-swap-dropped-repairs-audit.json` 의 proposals 에 적었다.

**왜**: 채택 제안 `2026-09-24-lgt-hard-fail-14-axes#p4`. `0230` 은 14건을 눈으로 분류했고 소실 4종을 찾았다. 같은 병이 더 있는지 기계로 물었다.

**사용자 영향**: 지금은 0이다(고친 것이 없다). 바뀐 것은 두 가지다. 「교체가 떨군 것은 이것이 전부다」라고 말할 근거가 생겼고, 그 전부 가운데 아직 아무도 맡지 않은 행 2개(604/605)가 드러났다.

## 1. 전제 반증부터

- ⒜ **기계로 뽑을 수 있다.** 두 트리 모두 SVC 표가 손으로 쓴 `0xNN => Self::X` 행이고, dispatch 는 `WIPICSvcId::X => callee` / `StdlibSvcId::X as u32 => …call(&callee` 모양이다. ⇒ `git show` 만으로 {id → 변형 이름, 호출 대상}을 양쪽에서 뽑아 id 로 맞붙일 수 있다(§4 스크립트).
- ⒝ **스크립트 자기 검증**: 같은 스크립트를 `d70b93f8 ↔ 37734e74`(#161 자신)에 돌리면 `GUARD FEWER wie_lgt/src/runtime/wipi_c/context.rs::free 1->0` 을 **낸다**. `d70b93f8 ↔ origin/main` 에서는 **내지 않는다**. #265(`cd3b17c2`)가 이미 복원했기 때문이다. ⇒ 가드 축은 알려진 소실을 실제로 잡는다.
- ⒞ **티켓 전제 하나는 절반만 맞다.** 「교체 전 stdlib `0x32`(native new)도 현 표에 없다」에서 **표에 없다는 것은 참이다**. 하지만 호출은 표까지 오지 않는다. 현 `init.rs::get_import_function` 이 `(import_table 1, index 0x32)` 를 stdlib 표보다 **먼저** 가로채 `JavaSystemSvcId::PendingException` 으로 보낸다. ⇒ 소실이 아니라 **업스트림 Java 런타임의 재해석**이다(§2 6행). 스크립트의 원출력은 이 행을 `LOST` 로 낸다. 표에서 판정을 고쳐 적었다.

## 2. 전수 표 (Contract 1)

「도입」은 교체 전 그 행을 넣은 우리 커밋이다. 「영향 타이틀」은 그 커밋 메시지·`0230`·이번 실행(§3)에서 가져왔다.

| # | SVC | 교체 전 의미 (도입) | 현 상태 (`origin/main`) | 공용 대응 | 영향 타이틀 | 권고 |
|---|---|---|---|---|---|---|
| 1 | stdlib `0x415` | `memmove` (`071213a2`) | **행 없음** → `Unknown lgt stdlib import: 0x415` | 있음: `wie-core-arm::stdlib::memmove` | 데몬헌터 · (LGT)알바타이쿤2(`0230`). 도입 커밋은 놈ZERO · 게임빌2010프로야구도 적었다. 그 둘은 지금 `0x410` 축(#7)에 먼저 걸린다 | **복원**. ★형제 `…-adopt-p0` 에 **이미 발권됨** |
| 2 | WIPIC `0xcf`(207) | `MC_grpGetContext` (`453b48bd`) | **행 없음** → `Unknown LGT WIPIC SVC id 207` | 있음: `wie_wipi_c::api::graphics::get_context` | 바이오크로니클 | **복원**. ★형제 `…-adopt-p0` 에 **이미 발권됨**. 조각 D §D 문면도 같이 고친다 |
| 3 | WIPIC `0x19c`(412) | `MC_dbListDataBase`. LGT 로컬 스텁 → 0(「DB 없음」) (`ba42ba59`) | **행 없음** → `Unknown LGT WIPIC SVC id 412` | **의미가 다르다**. 공용 `list_databases` 는 KTF 의미(남은 저장 바이트)다 | 리듬페스티발 ×2 · 하이브리드(`0230`, 이번 실행에서도 412 재현) · 제노니아1 · 하이브리드2(도입 커밋) | **LGT 로컬 스텁으로 복원**. ★형제 `…-adopt-p0` 에 **이미 발권됨** |
| 4 | WIPIC `0x25c`(604) | `MC_netSocketWrite` → -1 (`02ad8b5c`) | **행 없음** | **없음**. 공용 `net` 은 `connect`·`close`·`socket_close` 뿐이다 | 테라-영원의혼돈(도입 커밋. 로컬 코퍼스에는 없다) | ★**새로 드러났다. 복원 권고**(worklog p0) |
| 5 | WIPIC `0x25d`(605) | `MC_netSocketRead` → -1 (`02ad8b5c`) | **행 없음** | 없음 | 같음 | ★**새로 드러났다. 복원 권고**(4와 한 묶음) |
| 6 | stdlib `0x32` | native `new`. 게스트 객체를 할당한다 (`7f9ab2f7`) | stdlib 표에는 없다. **`(1, 0x32)` → `JavaSystemSvcId::PendingException` 으로 재배선** | 업스트림 Java 런타임이 자기 의미로 쓴다 | 교체 전 Java 경로(배틀몬스터 세대) | **보류**. 옛 의미를 되돌리면 새 Java 런타임과 충돌한다 |
| 7 | stdlib `0x410` | `unk5`: no-op, r0 을 건드리지 않는다 | **`strstr`**(업스트림 판정) | — | 이번 실행에서 7건이 이 서명이다(§3) | **보류**. 진짜 원인은 WIPIC `0x12d`(unk4)가 0 을 돌려주는 것이고, PR #288(형제 `…-adopt-p3`)이 그것을 맡는다 |
| 8 | WIPIC `0x581`(1409) | `misc_unk9` → **Err**(모듈·인덱스·인자를 이름으로 대는 fail-loud). 문구는 시험으로 잠겨 있었다 (`bccf11f1`·`6f9dbae7`) | `unk16` → **warn + Ok(0)** | — | 영웅서기5 LGT(upstream #1260. 로컬 코퍼스에는 없다) | **보류**. 조용한 0 이 이득인지는 관측해야 가린다(worklog p2) |
| 9 | 가드: `LgtWIPICContext::free(NULL)` | no-op (`0942b0fb`) | ★**복원됨**(#265 `cd3b17c2`) | — | 메탈슬러그 서바이벌 ×2 | **완료**. 단 교체 전과 **똑같이 시험이 없다** ⇒ 시험 제안(worklog p1) |

**대조했으나 소실이 아닌 것**(공용 `wie-wipi-c` 로 옮겨진 우리 수리분. 함수 본문을 직접 읽어 확인했다):
`fd4a1ff1` 프레임버퍼 정보 접근자 null 가드(`read_framebuffer_or_null`) · `28ec8cbd` alloc/calloc OOM → NULL(`alloc_or_null`) ·
`9d45ad20` EUC-KR 디코딩 · `1da44f6c` `get_resource` 버퍼 부족 `-18` · `6845e440` `MC_dbGetNumberOfRecords`: **전부 생존**.
가드 축이 낸 공용 2건도 소실이 아니다. `graphics.rs::get_rgb_pixels` 의 가드는 `primitives::get_rgb_pixels` 의 `rgb_row_bytes` 로 **옮겨졌고**,
`kernel.rs::sprintf` 의 `%s` NULL 가드(`ptr == 0`)는 `kernel/sprintf.rs` 에 **그대로 있다**. 행동 차이는 하나뿐이다: `get_rgb_pixels` 의 행 주소 오버플로가
교체 전에는 `warn + Ok(())` 였고 지금은 `Err(AllocationFailure)` 다. 닿으려면 `ipl × dy` 가 32비트를 넘어야 한다 ⇒ **보류**.

**WIPIC 행 대조의 나머지**: `0x581`(#8) 말고는 `CHANGED` 가 **0**이다. graphics 행은 교체 전에도 공용 `wie_wipi_c::api::graphics` 로 갔고 지금도 그렇다. 스크립트가 경로 별칭(`graphics::` · `shared_graphics::` · `wie_wipi_c::api::graphics::`)을 한 이름으로 접는다.
`NEW` 5행(`0x6f` GetProgramName · `0xd8`/`0xd9` DrawArc/FillArc · stdlib `0x3f7`/`0x403`/`0x404` sprintf/rand/srand)은 업스트림이 더한 것이라 이 감사의 대상이 아니다.

## 3. `broken/lgt` 에서 같은 원인 후보 (Acceptance ⑵)

**방법**: 37건 전부를 `wie_validate` 로 돌렸다. `--timeout 60`(첫 7건은 100), 병렬 8, load 176~207 이었다.
★바이너리는 04:04 에 빌드된 **debug** `wie_validate` 다. 그때 트리는 `1b4944de` 였고, `git diff 1b4944de ddc83420 -- '*.rs' '*.toml'` 이 **비어 있다** ⇒ `origin/main` 과 같은 코드다.
release 빌드는 LTO 링크가 이 부하에서 40분 넘게 CPU 1분을 못 받아서 접었다.

| 서명 | 건수 | 타이틀 | 교체 소실과의 관계 |
|---|---|---|---|
| `Unknown LGT WIPIC SVC id 412` | **1** | 하이브리드 | ★**#3 그대로**(형제 p0) |
| `stub unk4(0x0,…)` → `strstr(0x0,…)` → `Invalid memory access; address: 0` | **7** | (LGT)냐옹타이쿤2 · (LGT)제노니아1 · 게임빌2010프로야구 · 뮤직팩토리 · 정통맞고2009 · 제노니아2 · 놈ZERO | ★**#7**. `RUST_LOG=debug` 추적에서 7/7 이 이 순서다. `0230` 의 E 축 5건보다 넓다(냐옹타이쿤2 · 뮤직팩토리 · 정통맞고2009 가 새로 잡혔다). PR #288 이 맡는다 |
| `Unknown LGT WIPIC SVC id 900` | 2 | 2008베이징올림픽 · KBO프로야구2009 | 소실 **아님**. `0x384` 는 `d70b93f8` 표에도 없다 |
| `Unknown lgt stdlib import: 0x426` | 1 | 리얼사커매니저2009 | 소실 **아님**. 교체 전 표에도 없다 |
| Java ABI(`vtable index`) | 3 | 월드장기체스(String 16) · 학교가는길(ax 29) · 스파이더맨3(DataInputStream 27) | SVC 축 밖이다(`lgt_java_abi.toml` 소관) |
| panic `attempt to add with overflow` | 1 | 2010밴쿠버올림픽 | 교체 전 보고서(2026-06-30)에도 같은 문면이다 ⇒ 소실 아님 |
| 호스트 스택 오버플로(debug) | 1 | 크로이센 | 못 쟀다(debug 바이너리의 스택) |
| deadline(hang/blank) | 15 | 나머지 | 이 부하에서는 판정할 수 없다 |
| PASS | 6 | SD한국전쟁 · 배틀몬스터 · 메이플스토리2007 · 붕어빵타이쿤3 · 체스마스터 · 현영맞고2006 | — |

⇒ **교체 소실이 직접 원인인 후보는 `broken/lgt` 에서 8건**이다(412 1건, `0x410` 축 7건). 0x415 · 207 · 604/605 서명은 0건이다.
**0x415 가 0건인 이유**: 도입 커밋이 0x415 로 넘어졌다고 적은 놈ZERO · 게임빌2010프로야구가 지금은 그보다 **먼저** `0x410` 축에서 죽는다.
#288 이 착지하면 이 둘은 0x415 에 걸릴 수 있다. 형제 p0 의 확인 표본에 넣을 만하다.
★교체 전 보고서(`game_lab/reports/*.json`, 대부분 2026-06-25~07-01)로는 이 질문에 답할 수 **없다**. 그 시점에는 소실 행이 아직 있었으므로 그 서명이 나올 수 없다.

## 4. 재현 명령 (Acceptance ⑴)

읽기 전용이다(`git show`·`git ls-tree` 만 쓴다). 아래 python 블록을 `audit.py` 로 저장하고 repo 루트에서 돌린다:

```
python3 audit.py d70b93f8 origin/main    # 이 표의 원출력
python3 audit.py d70b93f8 37734e74       # 자기 검증: context.rs::free 가드 소실을 낸다
```

```python
# usage: python3 audit.py <old-rev> <new-rev>   (run inside the wie repo; read-only, git show only)
import re, subprocess, sys
OLD, NEW = sys.argv[1:3]
def show(rev, p):
    r = subprocess.run(['git', 'show', f'{rev}:{p}'], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ''
def ls(rev, p):
    return [f for f in subprocess.run(['git', 'ls-tree', '-r', '--name-only', rev, p], capture_output=True, text=True).stdout.split() if f.endswith('.rs') and '/tests/' not in f]
def bodies(src):
    out = {}
    for m in re.finditer(r'\bfn\s+(\w+)\s*(<[^>]*>)?\s*\(', src):
        i, semi = src.find('{', m.end()), src.find(';', m.end())
        if i < 0 or 0 <= semi < i: continue
        d, j = 0, i
        while j < len(src):
            d += {'{': 1, '}': -1}.get(src[j], 0)
            if d == 0: break
            j += 1
        out.setdefault(m.group(1), []).append(src[m.start():j + 1])
    return out
# axis 1: SVC id -> (variant, normalized callee), WIPIC + stdlib
SHARED = re.compile(r'^(?:wie_wipi_c::api::|shared_)?(graphics|database|kernel|media|misc|net)::')
def svc(rev, d):
    ids = show(rev, f'{d}/src/runtime/svc_ids.rs'); out = {}
    blk = ids.split('impl TryFrom<SvcId> for WIPICSvcId')[1].split('impl From')[0]
    arms = dict(re.findall(r'WIPICSvcId::(\w+) => ([\w:]+)', show(rev, f'{d}/src/runtime/wipi_c.rs')))
    for k, v in re.findall(r'(0x[0-9a-f]+|\d+) => Self::(\w+)', blk):
        out[('wipic', int(k, 0))] = (v, SHARED.sub(r'shared::\1::', arms.get(v, '?')))
    enum = ids.split('pub enum StdlibSvcId')[1].split('}')[0]
    sarms = dict(re.findall(r'StdlibSvcId::(\w+) as u32 =>(?: EmulatedFunction::call\(&([\w:]+))?', show(rev, f'{d}/src/runtime/stdlib.rs')))
    for v, k in re.findall(r'(\w+) = (0x[0-9a-f]+|\d+)', enum):
        out[('stdlib', int(k, 0))] = (v, sarms.get(v) or 'inline')
    return out
a, b = svc(OLD, 'wie_lgt'), svc(NEW, 'wie-lgt')
for k in sorted(set(a) | set(b)):
    x, y = a.get(k), b.get(k)
    tag = 'LOST' if y is None else 'NEW' if x is None else 'CHANGED' if x[1] != y[1] else None
    if tag: print(f'SVC {tag:7} {k[0]:6} {k[1]:#06x} ({k[1]})  {x} -> {y}')
# axis 2: early-return guards per function (lost function, or fewer guards in the same-named function)
G = re.compile(r'\bif\b[^{]*(==\s*0(x0+)?\b|\.is_null\(\)|<\s*0\b|checked_sub|\.is_none\(\))')
for po, pn in [('wie_lgt/src/runtime', 'wie-lgt/src/runtime'), ('wie_wipi_c/src', 'wie-wipi-c/src')]:
    def coll(rev, p): return {(f[len(p):], n, i): body for f in ls(rev, p) for n, bs in bodies(show(rev, f)).items() for i, body in enumerate(bs)}
    A, Bn = coll(OLD, po), coll(NEW, pn)
    names = {k[1] for k in Bn}
    for k in sorted(A):
        ga = len(G.findall(A[k]))
        if not ga: continue
        if k not in Bn: print(f'GUARD {"MOVED?" if k[1] in names else "GONE":6} {po}{k[0]}::{k[1]} guards={ga}')
        elif ga > len(G.findall(Bn[k])): print(f'GUARD FEWER  {po}{k[0]}::{k[1]} {ga}->{len(G.findall(Bn[k]))}')
```

원출력(`d70b93f8 origin/main`, 2026-09-25):

```
SVC LOST    stdlib 0x0032 (50)  ('Unk0x32', 'inline') -> None
SVC NEW     stdlib 0x03f7 (1015)  None -> ('Sprintf', 'sprintf')
SVC NEW     stdlib 0x0403 (1027)  None -> ('Rand', 'rand')
SVC NEW     stdlib 0x0404 (1028)  None -> ('Srand', 'srand')
SVC CHANGED stdlib 0x0410 (1040)  ('Unk5', 'unk5') -> ('Strstr', 'strstr')
SVC LOST    stdlib 0x0415 (1045)  ('Memmove', 'stdlib::memmove') -> None
SVC NEW     wipic  0x006f (111)  None -> ('GetProgramName', 'shared::kernel::get_program_name')
SVC LOST    wipic  0x00cf (207)  ('GetContext', 'shared::graphics::get_context') -> None
SVC NEW     wipic  0x00d8 (216)  None -> ('DrawArc', 'shared::graphics::draw_arc')
SVC NEW     wipic  0x00d9 (217)  None -> ('FillArc', 'shared::graphics::fill_arc')
SVC LOST    wipic  0x019c (412)  ('ListDatabases', 'list_databases') -> None
SVC LOST    wipic  0x025c (604)  ('SocketWrite', 'net_socket_write') -> None
SVC LOST    wipic  0x025d (605)  ('SocketRead', 'net_socket_read') -> None
SVC CHANGED wipic  0x0581 (1409)  ('MiscUnk9', 'misc_unk9') -> ('Unk16', 'unk16')
GUARD GONE   wie_lgt/src/runtime/java/interface.rs::read_cstring guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_class.rs::read_count guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_class.rs::read_cstring guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_jvm.rs::guest_to_value guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_jvm.rs::handle_java_trampoline guards=2
GUARD GONE   wie_lgt/src/runtime/java/native_jvm.rs::install_platform_tables guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_jvm.rs::read_cstr guards=1
GUARD GONE   wie_lgt/src/runtime/java/native_jvm.rs::scan_class_headers guards=2
GUARD FEWER  wie_wipi_c/src/api/graphics.rs::get_rgb_pixels 1->0
GUARD MOVED? wie_wipi_c/src/api/kernel.rs::sprintf guards=2
```

## 한계

- ★**LGT Java 런타임은 대조하지 않았다.** `native_jvm.rs`/`native_class.rs`/`java/interface.rs` 는 `jvm_support/*` 로 구조째 재작성됐다. 그래서 함수 이름 대조가 의미 없다.
  원출력의 `GUARD GONE …/java/*` 8행은 목록만 남긴다. `InitSvcId` 표(16행 → 4행 + `JavaSystemSvcId` 36행)도 같은 이유로 제외했다. 그쪽 소실은 `lgt_java_abi.toml` 회차들이 행 단위로 진다.
- 가드 축은 정규식이다(`if` 안의 `== 0` · `.is_null()` · `< 0` · `checked_sub` · `.is_none()`). `let Some(..) else`, `match` 가드, 모양이 다른 조기 반환은 **못 본다**.
  공용 쪽 null 가드 생존은 그래서 본문을 직접 읽어 확인했다(§2).
- SVC 축은 **표와 dispatch** 를 본다. 행이 살아 있고 호출 대상 이름도 같은데 **본문이 바뀐** 로컬 함수는 따로 셌다. `wie-lgt/src/runtime` 의 같은 이름 함수 중 본문이 다른 것은 `init.rs`·`svc_ids.rs`·dispatch 자신과 `context.rs` 두 곳(`get_resource_size`·`read_resource`: 핀 이동에 따른 호출 모양 + `collect_garbage`)뿐이다. `unk*` 스텁 본문은 전부 같다.
- §3 은 한 번 돌린 결과다. load 가 ~200 이고 debug 바이너리여서 deadline 15건은 판정이 없다. `smoke_gate`·`0230` 처럼 짝지어 측정하지 않았다.

## 검증

- `RUST_MIN_STACK=4194304 cargo test --all` **rc=0**: 46 suites · **433 passed · 0 failed**.
- `cargo clippy --workspace --all-targets` **rc=0** · 경고 12줄은 전부 기존 것이다(`wie-backend` 테스트 11 · `wie-jvm-support` 테스트 1). 이 회차의 diff 는 `docs/` 뿐이다 ⇒ **경고 증가 0**.
- 네 게이트 중 fmt/wasm clippy 는 Rust 무변경이라 결과가 main 과 같다. 수치는 PR CI 가 진다.

