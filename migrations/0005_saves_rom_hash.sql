-- wie web service — key cloud saves by ROM CONTENT HASH (per-owner).
--
-- WHY: saves were keyed by a user-chosen slot alias, with no link to the ROM. So
-- moving a ROM local→server (or re-uploading it) detached the save. Now a save is
-- keyed by (user_id, rom_hash): the SAME ROM always maps to the SAME save, no
-- matter where the ROM lives or what it's named. This converges on one save per
-- ROM and fixes the "server ROM can't see the old local save" bug.
--
-- 기준선(B안 격리): rom_hash is the OWNER's own ROM content hash (the same value
-- already stored per-owner in user_files.content_hash). It is per-owner isolated —
-- every saves query is scoped by user_id, and there is no cross-user/global
-- rom_hash lookup. NOT-logged-in saves are never stored here (they stay local).

PRAGMA foreign_keys = ON;

-- Re-key cleanly. saves is empty post-0004 (data wiped); this guarantees no row
-- has an empty rom_hash before the unique index is created.
DELETE FROM saves;

-- The old per-(user, slot_label) uniqueness no longer applies — the key is the
-- ROM hash now; slot_label becomes a display alias (may repeat).
DROP INDEX IF EXISTS uq_saves_owner_slot;

ALTER TABLE saves ADD COLUMN rom_hash TEXT NOT NULL DEFAULT '';

-- One save per (owner, ROM). Owner is the leading column so a cross-user read can
-- never be cheap/accidental. There is deliberately NO index on rom_hash alone —
-- the server must never be able to answer "who else has a save for this ROM".
CREATE UNIQUE INDEX IF NOT EXISTS uq_saves_owner_rom ON saves (user_id, rom_hash);
