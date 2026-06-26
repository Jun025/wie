-- wie web service — 7번: strengthen the rights-infringement takedown notice so a
-- single report carries enough to judge it (reporter identity/standing, contact,
-- right basis, sworn statements), the reporter's environment (same as 문의), and
-- private evidence the operator can review.
--
-- ★ADDITIVE ONLY (가산적): every statement below is `ALTER TABLE ... ADD COLUMN`
-- with a constant default. No DROP/rename/redefine. Existing `file_reports` rows
-- keep their values; the new columns default to empty/0. Safe to apply on a live DB.
--
-- ★격리/개인정보: the reporter's identity + contact + evidence are stored for the
-- OPERATOR ONLY (compliance handling). They are never returned by any public/list
-- endpoint, never exposed via a URL, and contain no other user's PII. Evidence is
-- bounded base64 (images/pdf/text only; game/exec/archive magics rejected in code).

-- reporter standing/identity
ALTER TABLE file_reports ADD COLUMN reporter_type TEXT NOT NULL DEFAULT '';   -- 'owner' | 'agent'
ALTER TABLE file_reports ADD COLUMN right_basis  TEXT NOT NULL DEFAULT '';    -- basis/title for the claimed right
ALTER TABLE file_reports ADD COLUMN good_faith   INTEGER NOT NULL DEFAULT 0;  -- 1 = swore the good-faith/accuracy statement

-- reporter environment (auto-attached, same as 문의) + private evidence attachment
ALTER TABLE file_reports ADD COLUMN env_info        TEXT NOT NULL DEFAULT '';
ALTER TABLE file_reports ADD COLUMN attachment_name TEXT NOT NULL DEFAULT '';
ALTER TABLE file_reports ADD COLUMN attachment_mime TEXT NOT NULL DEFAULT '';
ALTER TABLE file_reports ADD COLUMN attachment_data TEXT NOT NULL DEFAULT '';  -- base64 (operator-only, no public URL)
