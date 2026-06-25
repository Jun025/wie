# Compliance runbook — takedown & repeat-infringer (B안 server vault)

The logged-in server vault (`user_files` + R2) is a **1인 전용 보관함**: strictly
per-owner, no sharing/discovery (S5). This doc is the operator procedure for
rights-holder notices. There is intentionally **no public admin UI and no admin
secret in the codebase** — operator actions are deliberate D1 commands run with a
D1-scoped token (least privilege). The *enforcement* of these states is in code;
only the *trigger* is a human operator step.

## What's enforced in code
- `user_files.disabled = 1` → the file is **inaccessible to its owner too**:
  `/api/files/:id` returns 404, it is excluded from `/api/files` listing, and it
  no longer counts toward the owner's quota. The per-user `UNIQUE(user_id,
  content_hash)` row remains, so the owner **cannot re-upload** the same file to
  dodge the takedown.
- `users.status = 'restricted'` → upload is blocked (`/api/files` POST → 403
  `account_restricted`); the user can still log in to delete their own files.
- `users.status = 'disabled'` → login is refused (`/api/auth/login` → 401).
- `users.strikes` → running count of upheld infringements for the repeat-infringer
  policy.

## Intake
Rights-holder notices arrive via `POST /api/reports` (the "권리 침해 신고" form in
서비스 정보, usable without an account) and land in `file_reports` (status `open`).

```sh
# review open notices
npx wrangler d1 execute wie-db --remote --command \
  "SELECT id, work_title, reporter_contact, target_hint, created_at FROM file_reports WHERE status='open' ORDER BY created_at"
```

## Action: disable a target file (upon a valid notice)
Identify the offending file id (the reporter's hint + the owner's own metadata;
the operator never browses other users' files casually). Then:

```sh
npx wrangler d1 execute wie-db --remote --command \
  "UPDATE user_files SET disabled=1, disabled_reason='DMCA/notice <report-id>', disabled_at=<epoch_ms> WHERE id='<file-id>';"
# increment the owner's strike count
npx wrangler d1 execute wie-db --remote --command \
  "UPDATE users SET strikes = strikes + 1 WHERE id=(SELECT user_id FROM user_files WHERE id='<file-id>');"
# (optional) purge the bytes from R2 once the dispute window has passed
npx wrangler r2 object delete wie-games/<r2_key>
# log the action on the report
npx wrangler d1 execute wie-db --remote --command \
  "UPDATE file_reports SET status='actioned', action_log='<when> / file <file-id> disabled / <handler>', updated_at=<epoch_ms> WHERE id='<report-id>';"
```

## Action: repeat-infringer escalation
Policy (suggested): 1st–2nd strike → notify; 3rd → `restricted` (no new uploads);
4th → `disabled` (account suspended).

```sh
# restrict uploads
npx wrangler d1 execute wie-db --remote --command "UPDATE users SET status='restricted' WHERE id='<user-id>';"
# suspend the account
npx wrangler d1 execute wie-db --remote --command "UPDATE users SET status='disabled' WHERE id='<user-id>';"
```

## Notes
- The bucket is **private** — never enable a public bucket URL/custom domain.
  Bytes are only ever served by the owner-checked `/api/files/:id`.
- `red flag` knowledge: act on credible notices promptly; keep the `action_log`.
- Never expose a file listing across users or a content-hash lookup — that would
  break per-owner isolation (S5).
