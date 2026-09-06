## [2026-08-20] wrangler `account_id` 를 otterpebble 계정으로 고정 (wie-wrangler-account-id-pin)
- **무엇을**: `wrangler.toml` 최상단에 `account_id = "17024dfe5a8ff38798c35942d116026b"` 를 사유 주석과 함께 박았다. **1파일 +6/-0** — 코드·워크플로·의존성 무변경.
- **왜**: 이 맥에는 Cloudflare 계정이 둘(otterpebble·dodu) 있고, `wrangler` 는 계정이 명시되지 않으면 **자격증명이 가리키는 아무 계정**으로 붙는다. `wie-db`(D1)·R2 버킷은 otterpebble 계정 소유이므로 잘못된 계정으로 붙으면 조용히 «빈 프로젝트에 배포»가 된다. 게이트② 검수가 `wrangler d1 info wie-db` 를 이 자격증명으로 돌려 **핀한 계정이 실제로 `wie-db` 를 소유함**을 양성 대조로 확인했다.
- **★권고 — 핀이 env 를 덮는다**: `wrangler.toml` 의 `account_id` 는 `CLOUDFLARE_ACCOUNT_ID` 환경변수보다 **우선한다**. ⇒ 이후 CI 에서 «다른 계정으로 배포»하려 해도 **`CLOUDFLARE_ACCOUNT_ID` 시크릿 교체만으로는 계정이 바뀌지 않는다** — 이 파일의 핀을 함께 고쳐야 한다(고정의 목적 자체가 그것이므로 의도된 동작이다).
- **사용자 영향**: 없음(에뮬레이터 동작·배포 대상 무변경). 바뀐 것은 «어느 계정인지»가 자격증명이 아니라 **파일에 적혀 있다**는 것뿐이다.

