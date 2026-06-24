-- wie web service — email verification / password reset + device registry
--
-- GUARDRAIL (1번 기준선 / S5): still NO game-file identity anywhere. The new
-- `devices` table stores only a client-generated device id, a user alias, login/
-- seen timestamps, and ANONYMOUS storage aggregates (a count and a byte total) —
-- never a filename, hash, or title. "How many games / how many MB" is a count,
-- explicitly permitted; "which games" is not stored.

PRAGMA foreign_keys = ON;

-- ── One-time, hashed, expiring tokens for email verification + password reset ──
-- The raw token is emailed to the user; only its SHA-256 hash is stored, so a DB
-- read cannot reconstruct a usable link. `used_at` enforces single use.
CREATE TABLE IF NOT EXISTS email_tokens (
    id          TEXT    PRIMARY KEY,                 -- uuid v4
    user_id     TEXT    NOT NULL,
    purpose     TEXT    NOT NULL,                     -- 'verify' | 'reset'
    token_hash  TEXT    NOT NULL,                     -- sha-256 hex of the raw token
    expires_at  INTEGER NOT NULL,
    used_at     INTEGER,                              -- NULL until consumed
    created_at  INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_email_tokens_hash ON email_tokens (token_hash);
CREATE INDEX IF NOT EXISTS idx_email_tokens_user ON email_tokens (user_id, purpose);

-- ── Device registry (per account) ────────────────────────────────────────────
-- device_id is a random value the CLIENT generates and stores in localStorage —
-- NOT a hardware fingerprint. The aggregate columns are counts/sizes only; the
-- server still cannot answer "which games does this device have".
CREATE TABLE IF NOT EXISTS devices (
    user_id        TEXT    NOT NULL,
    device_id      TEXT    NOT NULL,                  -- client-generated, not hardware
    label          TEXT    NOT NULL DEFAULT '',       -- user alias for the device
    last_login_at  INTEGER NOT NULL DEFAULT 0,
    last_seen_at   INTEGER NOT NULL DEFAULT 0,
    item_count     INTEGER NOT NULL DEFAULT 0,        -- anonymous: number of library items
    total_bytes    INTEGER NOT NULL DEFAULT 0,        -- anonymous: total library size
    last_run_at    INTEGER NOT NULL DEFAULT 0,        -- anonymous: most recent play
    last_save_at   INTEGER NOT NULL DEFAULT 0,        -- anonymous: most recent local save
    created_at     INTEGER NOT NULL,
    PRIMARY KEY (user_id, device_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_devices_user ON devices (user_id, last_seen_at);

-- ── Inquiry simplification: auto-collected environment + optional attachment ───
-- env_info is diagnostic text the CLIENT auto-collects (browser/OS/platform/
-- screen) with game identity EXCLUDED. The attachment is owner-scoped support
-- material (image/log/text only, game/exec files rejected) and is never exposed
-- on a public URL — the inquiry channel must not become a file-distribution path.
ALTER TABLE inquiries ADD COLUMN env_info        TEXT NOT NULL DEFAULT '';
ALTER TABLE inquiries ADD COLUMN attachment_name TEXT NOT NULL DEFAULT '';
ALTER TABLE inquiries ADD COLUMN attachment_mime TEXT NOT NULL DEFAULT '';
ALTER TABLE inquiries ADD COLUMN attachment_data TEXT NOT NULL DEFAULT '';   -- base64, small, owner-only
