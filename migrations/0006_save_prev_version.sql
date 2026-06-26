-- wie web service — keep ONE previous save version per ROM (recovery safety net).
--
-- The save model is last-write-wins (latest always wins, per the directive). To
-- make that safe, each upsert moves the CURRENT payload into prev_* before writing
-- the new one — so exactly ONE prior version is retained and the user can revert
-- one step. This does NOT change the "latest wins" behavior; it only delays the
-- permanent loss of the immediately-overwritten version. Bounded at 1 version.
--
-- Non-destructive (ADD COLUMN only). Still per-owner: prev_* live on the same
-- owner-scoped saves row; there is no cross-user exposure.

ALTER TABLE saves ADD COLUMN prev_payload       TEXT    NOT NULL DEFAULT '';
ALTER TABLE saves ADD COLUMN prev_payload_bytes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE saves ADD COLUMN prev_updated_at    INTEGER NOT NULL DEFAULT 0;
