-- wie web service — B안: per-owner server-side game-file storage + compliance.
--
-- ★기준선 변경 (이번 마이그레이션부터): 로그인 사용자의 게임 파일은 서버(R2)에
-- 저장되되 "1인 전용 보관함"으로 엄격히 격리된다. 이 테이블은 메타데이터만 담고
-- (바이트는 R2 binding `GAMES`), 모든 접근은 세션 인증 + 서버측 소유자 검증을 거친다.
--
-- ★S5(격리): 어떤 컬럼/인덱스/엔드포인트도 "다른 사용자가 무엇을 가졌는지"를
-- 드러내선 안 된다. 그래서 content_hash 단독 전역 인덱스는 의도적으로 만들지 않고,
-- 중복(dedup) 비교는 오직 (user_id, content_hash) 회원 단위로만 가능하게 한다.
-- 미로그인 사용자의 게임 파일/메타데이터는 여전히 서버로 전송되지 않는다(기기 로컬 only).

PRAGMA foreign_keys = ON;

-- ── Per-owner game-file registry (bytes in R2, metadata here) ──────────────────
CREATE TABLE IF NOT EXISTS user_files (
    id              TEXT    PRIMARY KEY,                 -- uuid v4
    user_id         TEXT    NOT NULL,                    -- owner (the ONLY one who can access)
    file_name       TEXT    NOT NULL,                    -- owner's own filename (their eyes only)
    kind            TEXT    NOT NULL DEFAULT '',          -- jar | jad | zip | kdf | skm
    content_hash    TEXT    NOT NULL,                    -- sha-256 hex; dedup is PER-USER only
    size            INTEGER NOT NULL DEFAULT 0,          -- bytes stored in R2
    r2_key          TEXT    NOT NULL,                    -- owner-namespaced + random; unguessable
    disabled        INTEGER NOT NULL DEFAULT 0,          -- 1 = taken down (inaccessible to owner too)
    disabled_reason TEXT    NOT NULL DEFAULT '',
    disabled_at     INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    last_seen_at    INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
-- Owner is the leading column on every index, so a cross-user read can never be
-- cheap or accidental.
CREATE INDEX IF NOT EXISTS idx_user_files_owner ON user_files (user_id, created_at);
-- ★PER-USER dedup ONLY. A (owner, content) pair is unique so the same user cannot
-- store the same file twice. There is deliberately NO index on content_hash alone
-- — the server must never be able to answer "who else has this file" (S5).
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_files_owner_hash ON user_files (user_id, content_hash);

-- ── Rights-holder takedown intake + action log (compliance) ────────────────────
-- Anonymous-allowed (a rights holder need not have an account). Stores the notice
-- and an operator action log (when/target/action/handler). It exposes NO file
-- listing and grants NO file access — it is an intake channel only.
CREATE TABLE IF NOT EXISTS file_reports (
    id               TEXT    PRIMARY KEY,                -- uuid v4
    reporter_name    TEXT    NOT NULL DEFAULT '',
    reporter_contact TEXT    NOT NULL DEFAULT '',
    work_title       TEXT    NOT NULL DEFAULT '',        -- the infringed work
    statement        TEXT    NOT NULL,                   -- good-faith statement / details
    target_hint      TEXT    NOT NULL DEFAULT '',        -- any identifying hint the reporter can give
    status           TEXT    NOT NULL DEFAULT 'open',    -- open | actioned | rejected
    action_log       TEXT    NOT NULL DEFAULT '',        -- operator log: 처리일시/대상/조치/처리자
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_file_reports_status ON file_reports (status, created_at);

-- ── Repeat-infringer tracking on accounts ──────────────────────────────────────
-- `strikes` accrues on each upheld takedown affecting this account; the operator
-- escalates `users.status` to 'restricted' (upload blocked) or 'disabled'
-- (login blocked) per the repeat-infringer policy. login.js already blocks
-- 'disabled'; it now also blocks 'restricted' for uploads (enforced in code).
ALTER TABLE users ADD COLUMN strikes INTEGER NOT NULL DEFAULT 0;
