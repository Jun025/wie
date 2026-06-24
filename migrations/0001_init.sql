-- wie web service — initial D1 schema
--
-- GUARDRAIL (1번 기준선 / S5): NO table in this schema may ever store game-file
-- bytes, game filenames, game hashes, or any "which games this device owns"
-- inventory. The server only ever persists ⓐ account info and ⓑ save data.
-- `saves.slot_label` is a USER-CHOSEN alias, never a game identifier.
-- `inquiries.game_*` columns hold free text the user voluntarily typed into a
-- support ticket — they are never auto-filled from the device game library.

PRAGMA foreign_keys = ON;

-- ── Accounts ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id              TEXT    PRIMARY KEY,            -- uuid v4
    login_id        TEXT    NOT NULL UNIQUE,        -- id or email-like string
    email           TEXT,                           -- nullable; reserved for future verification
    email_verified  INTEGER NOT NULL DEFAULT 0,     -- placeholder, not implemented yet
    password_algo   TEXT    NOT NULL,               -- e.g. 'pbkdf2-sha256'
    password_iter   INTEGER NOT NULL,               -- KDF iteration count
    password_salt   TEXT    NOT NULL,               -- base64
    password_hash   TEXT    NOT NULL,               -- base64
    status          TEXT    NOT NULL DEFAULT 'active',
    created_at      INTEGER NOT NULL,               -- epoch ms
    updated_at      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_users_login_id ON users (login_id);

-- ── Sessions (signed token id is the cookie; row enables revocation) ──────────
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT    PRIMARY KEY,                -- random opaque session id
    user_id     TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,
    revoked     INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions (expires_at);

-- ── Saves (the ONLY game-related bytes allowed server-side: opaque save blobs) ─
-- `slot_label` and `device_label` are user-chosen aliases. There is deliberately
-- NO column for game title / filename / hash. The client maps a local game to a
-- slot inside the browser (IndexedDB); the server never learns the mapping.
CREATE TABLE IF NOT EXISTS saves (
    id            TEXT    PRIMARY KEY,              -- uuid v4
    user_id       TEXT    NOT NULL,
    slot_label    TEXT    NOT NULL,                 -- user alias, NOT a game id
    device_label  TEXT    NOT NULL DEFAULT '',      -- user alias for source device
    payload       TEXT    NOT NULL,                 -- base64 opaque save snapshot
    payload_bytes INTEGER NOT NULL DEFAULT 0,
    checksum      TEXT    NOT NULL DEFAULT '',      -- sha-256 hex of payload, integrity only
    updated_at    INTEGER NOT NULL,
    created_at    INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
-- Owner-scoped access: every query filters by user_id. Composite index makes the
-- owner check the leading column so cross-user reads can never be cheap/accidental.
CREATE INDEX IF NOT EXISTS idx_saves_owner ON saves (user_id, updated_at);
CREATE UNIQUE INDEX IF NOT EXISTS uq_saves_owner_slot ON saves (user_id, slot_label);

-- ── Inquiries (text only; no attachments, no binaries) ───────────────────────
CREATE TABLE IF NOT EXISTS inquiries (
    id           TEXT    PRIMARY KEY,
    user_id      TEXT    NOT NULL,
    category     TEXT    NOT NULL,                  -- 'question' | 'suggestion' | 'proposal' | 'rights_report'
    title        TEXT    NOT NULL,
    body         TEXT    NOT NULL,
    game_title   TEXT    NOT NULL DEFAULT '',       -- voluntary free text in the ticket
    game_vendor  TEXT    NOT NULL DEFAULT '',
    device_model TEXT    NOT NULL DEFAULT '',
    symptom      TEXT    NOT NULL DEFAULT '',
    status       TEXT    NOT NULL DEFAULT 'open',
    created_at   INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_inquiries_owner ON inquiries (user_id, created_at);

-- ── Best-effort rate limiting (no raw IPs stored, only a salted hash) ─────────
CREATE TABLE IF NOT EXISTS rate_limits (
    bucket_key   TEXT    PRIMARY KEY,               -- '<route>:<ip_hash>'
    window_start INTEGER NOT NULL,
    count        INTEGER NOT NULL
);
