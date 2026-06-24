# Cloudflare 수동 설정 가이드 (사용자 작업)

이 문서는 **사용자가 직접** Cloudflare 대시보드에서 해야 하는 작업을 화면 단계까지
정리한 것입니다. 코드/스키마/마이그레이션/CI는 이미 리포에 들어 있으므로, 아래 값을
산출해서 알려주시면 다음 런에서 설정 파일(`wrangler.toml` 등)에 반영합니다.

> 보안 원칙: API 토큰·세션 서명키 등 **시크릿은 사용자만 입력**합니다. 코드/커밋/CI
> 어디에도 토큰을 넣지 않습니다(참조만). 게임 파일·게임 보유목록은 서버로 가지 않으며,
> 서버에 저장되는 것은 계정 정보와 세이브 데이터뿐입니다.

---

## 0. 한눈에 보기 — 주고받을 값

| 사용자가 산출하는 값 | 어디서 | 누구에게/어디에 |
| --- | --- | --- |
| Pages 프로젝트명 (예: `wie-web`) | Pages 생성 시 | `wrangler.toml`의 `name`과 일치해야 함 → 알려주세요 |
| D1 `database_id` | `wrangler d1 create` 또는 대시보드 | `wrangler.toml`의 `database_id`에 반영 → 알려주세요 |
| `SESSION_SECRET` (랜덤 32바이트) | 본인 생성 | Pages → Settings → 환경변수(암호화 secret). **값은 공유 금지** |
| `RESEND_API_KEY` (이메일 인증/재설정용, 선택) | resend.com → API Keys | Pages → Settings → 환경변수(암호화 secret). 미설정 시 메일 기능만 비활성 |
| `EMAIL_FROM` (예: `WIE <noreply@yourdomain>`) | 본인의 인증된 발신주소 | Pages → Settings → 환경변수(일반 변수 가능) |
| `CLOUDFLARE_API_TOKEN` | My Profile → API Tokens | GitHub repo Secrets (CI 배포용, 선택) |
| `CLOUDFLARE_ACCOUNT_ID` | 대시보드 우측 | GitHub repo Secrets (CI 배포용, 선택) |

---

## 1. Pages 프로젝트 생성 + Git 연결

배포 방식은 두 가지입니다. **권장은 (A) GitHub Actions 직접 업로드**입니다 — Pages 빌드
이미지에는 Rust가 없어서 wasm을 거기서 빌드할 수 없기 때문입니다. 이 리포의 CI
(`.github/workflows/web.yml`)가 wasm을 미리 빌드해 정적 `dist/`만 배포합니다.

### (A) 권장: GitHub Actions → Pages 직접 업로드(Direct Upload)
1. Cloudflare 대시보드 → **Workers & Pages** → **Create** → **Pages** →
   **"Direct Upload"** (또는 비어 있는 프로젝트로 시작).
2. 프로젝트 이름을 `wie-web`으로 지정 (지금 `wrangler.toml`의 `name = "wie-web"`과 일치).
   다른 이름을 원하면 그 이름을 알려주세요 → `wrangler.toml`/CI에 반영하겠습니다.
3. 첫 업로드는 비어 있어도 됩니다. 이후 `main` push 시 CI가 `pages deploy dist`로 업로드합니다.

### (B) 대안: "Connect to Git" (Pages가 빌드)
> Pages 빌드 이미지에 Rust가 없으므로, 이 방식을 쓰려면 wasm을 리포에 커밋하거나
> 커스텀 빌드 이미지가 필요합니다. 기본 구성에서는 (A)를 권장합니다.
1. **Workers & Pages → Pages → Connect to Git**.
2. GitHub 계정 인증 → **Jun025/wie** 저장소 선택 (아래 2번 참고).
3. **Production branch**: `main`. **Preview deployments**: 활성화(브랜치/PR 프리뷰 켜기).
4. Build 설정:
   - **Framework preset**: None
   - **Build command**: `cd web && npm ci && npm run build`
     (이 명령은 `scripts/build-wasm.sh`(cargo+wasm-bindgen+wasm-opt) → `tsc -b` → `vite build` 수행)
   - **Build output directory**: `web/dist`
   - (이 경우 빌드 이미지에 Rust/wasm-bindgen이 있어야 하므로 비권장 — 권장은 (A))

어느 방식이든 **Functions는 `functions/` 디렉터리에서 자동 인식**되고, 정적 자산은
`web/dist`에서 서빙됩니다.

---

## 2. GitHub 연동 인증 (Connect to Git를 쓸 때)
1. Connect to Git 클릭 시 Cloudflare가 GitHub App 설치를 요청합니다.
2. **Only select repositories → `Jun025/wie`** 만 선택(최소 권한).
3. 설치 후 Cloudflare로 돌아오면 저장소 목록에 `Jun025/wie`가 보입니다.

> 참고: 이 작업 브랜치(fork)의 origin/upstream 구분 — 코드 push는 `Jun025/wie`(fork)로만
> 갑니다. upstream(`dlunch/wie`)에는 절대 push하지 않습니다.

---

## 3. D1 데이터베이스 생성 + 바인딩 + 마이그레이션

### 3-1. 생성 (둘 중 하나)
- **CLI**: `npx wrangler d1 create wie-db`
  - 출력의 `database_id = "xxxxxxxx-...."` 값을 복사 → **알려주세요**.
- **대시보드**: Workers & Pages → **D1** → Create → 이름 `wie-db` → 생성 후 상세에서 ID 확인.

### 3-2. `wrangler.toml`에 반영
```toml
[[d1_databases]]
binding = "DB"
database_name = "wie-db"
database_id = "여기에-사용자의-database_id"   # 현재는 placeholder
```
→ `database_id`를 알려주시면 다음 런에서 채웁니다(또는 직접 교체).

### 3-3. Pages 프로젝트에 D1 바인딩 추가 (대시보드)
- Pages 프로젝트 → **Settings → Functions → D1 database bindings** →
  **Variable name = `DB`**, **D1 database = `wie-db`** 선택 → Save.
- Production/Preview 양쪽 환경에 모두 추가.

### 3-4. 마이그레이션 적용
- **원격(프로덕션)**: `npx wrangler d1 migrations apply wie-db --remote`
- **로컬 테스트**: `npx wrangler d1 migrations apply wie-db --local`
- 마이그레이션 파일은 `migrations/0001_init.sql` (users / sessions / saves / inquiries /
  rate_limits). 게임 파일/보유목록 컬럼은 없습니다.

---

## 4. Secret 등록 — 세션 서명키

세션 토큰 서명에 쓰는 `SESSION_SECRET`이 **반드시** 필요합니다(없으면 로그인/세션이 503).

1. 값 생성(로컬에서): `openssl rand -hex 32`
2. Pages 프로젝트 → **Settings → Environment variables** →
   **Add variable** → Name `SESSION_SECRET`, Value=생성한 값, **Type: Secret(암호화)** →
   Production/Preview 양쪽에 등록.
3. 로컬 개발은 리포 루트 `.dev.vars`(gitignore됨)에 `SESSION_SECRET="..."`로 둡니다.
   예시는 `.dev.vars.example` 참고.

> 값은 **사용자만** 입력합니다. 저에게 값을 알려줄 필요는 없습니다(이름만 알면 됩니다).

### (선택) CI 직접 배포용 secret — GitHub
GitHub Actions가 `pages deploy`까지 하려면:
1. Cloudflare **My Profile → API Tokens → Create Token** →
   템플릿 "Edit Cloudflare Workers" 또는 Pages 편집 권한 토큰 생성.
2. GitHub `Jun025/wie` → **Settings → Secrets and variables → Actions** →
   `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` 등록.
   - 미등록 시 CI는 빌드/아티팩트까지만 하고 배포 단계는 자동 skip됩니다.

### (선택) 이메일 인증·비밀번호 재설정 — Resend (사용자 작업)

이메일 인증 가입 + 비밀번호 재설정 메일은 **Resend**(https://resend.com)로 보냅니다.
SMTP 소켓을 못 여는 Workers/Pages Functions에서 HTTPS API 한 번으로 동작하고, **무료
한도**가 넉넉하기 때문입니다.

- **무료 한도(작성 시점):** 월 3,000통 / 일 100통. 본 용도(인증·재설정)에는 충분합니다.
- **발신 도메인 요건:** 임의의 수신자에게 보내려면 Resend에서 **본인 도메인을 인증**해야
  합니다. 샌드박스 발신주소 `onboarding@resend.dev`는 **계정 소유자 본인에게만** 전달되므로
  테스트용입니다.
- **등록할 값(사용자만 입력 — S3):**
  1. Resend → **API Keys → Create** → 키 생성.
  2. Pages → **Settings → Environment variables** →
     - `RESEND_API_KEY` = 생성한 키, **Type: Secret(암호화)**.
     - `EMAIL_FROM` = 인증된 발신주소(예: `WIE <noreply@yourdomain>`). 일반 변수 가능.
     - Production/Preview 양쪽에 등록.
  3. 로컬 개발은 `.dev.vars`에 `RESEND_API_KEY="..."`, `EMAIL_FROM="..."` 추가(gitignore됨).
- **graceful 처리:** 두 값이 없으면 메일 단계만 비활성화됩니다 — 가입/로그인은 그대로
  동작하고(이메일 없이 즉시 active), 인증/재설정 UI는 "이메일 기능 미설정"으로 안내합니다.
  키를 넣지 않아도 빌드/기존 기능은 죽지 않습니다.

> 값은 **사용자만** 입력합니다. 키 자체를 저에게 알려줄 필요는 없습니다(이름만 알면 됩니다).

---

## 5. (사용자가 추후 직접) main 자동배포 연결 시 주의점
- 현재 `main` 자동 프로덕션 배포는 **연결되어 있지 않습니다**. 연결 전까지 `main` 머지는
  배포를 일으키지 않습니다.
- 연결할 때:
  - Production branch를 `main`으로 두고, **마이그레이션을 먼저 `--remote`로 적용**한 뒤
    배포해야 스키마 불일치를 피합니다.
  - `SESSION_SECRET`이 Production 환경에 등록돼 있는지 먼저 확인(없으면 로그인 503).
  - Preview와 Production의 D1 바인딩이 각각 올바른지 확인.

---

## 6. 동작 확인 체크리스트 (로컬)
```bash
npm install                      # 루트: ops 도구(wrangler, playwright)
cp .dev.vars.example .dev.vars   # SESSION_SECRET 채우기 (openssl rand -hex 32)
(cd web && npm ci && npm run build)   # 프론트 빌드 (wasm + tsc + vite → web/dist)
npm run db:migrate:local         # 로컬 D1 마이그레이션
npm run dev                      # wrangler pages dev web/dist → http://localhost:8788
npm run audit                    # 무전송 자가감사
# 실게임 렌더 확인(파일은 리포 밖 경로, git/dist 미포함):
node scripts/verify-browser.mjs /path/to/your/game.jar
```

---

## 7. 알려주실 값 요약 (다음 런 반영용)
- [ ] Pages 프로젝트명 (기본 `wie-web` 유지 여부)
- [ ] D1 `database_id`
- [ ] (CI 배포 원하면) `CLOUDFLARE_API_TOKEN` / `CLOUDFLARE_ACCOUNT_ID`를 **GitHub Secrets에**
      등록했는지 여부 (값은 알려주지 마세요)
- [ ] `SESSION_SECRET`을 Pages 환경변수에 등록했는지 여부 (값은 알려주지 마세요)
