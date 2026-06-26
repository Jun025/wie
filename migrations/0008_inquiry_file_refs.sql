-- wie web service — 6번: let a member attach files from their OWN private vault to
-- an inquiry BY REFERENCE (file id), never by re-uploading bytes.
--
-- ★ADDITIVE ONLY (가산적): a single CREATE TABLE IF NOT EXISTS + indexes. No
-- DROP/rename/redefine. Existing data untouched.
--
-- ★격리: a reference is only the user's own file id. The bytes stay in the PRIVATE
-- R2 vault; nothing is copied into the inquiry, and there is no public/presigned
-- URL. The server validates on write that each referenced file belongs to the
-- requesting user (every row carries user_id and is FK'd to user_files). An operator
-- reviews the referenced file through the existing owner-scoped infrastructure — the
-- inquiry channel never becomes a byte-distribution path.

CREATE TABLE IF NOT EXISTS inquiry_file_refs (
    inquiry_id  TEXT    NOT NULL,                  -- the inquiry it is attached to
    file_id     TEXT    NOT NULL,                  -- a row in user_files (owned by user_id)
    user_id     TEXT    NOT NULL,                  -- owner (validated server-side on insert)
    created_at  INTEGER NOT NULL,
    PRIMARY KEY (inquiry_id, file_id),
    FOREIGN KEY (inquiry_id) REFERENCES inquiries (id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES user_files (id) ON DELETE CASCADE
);
-- Owner-leading index: a cross-user read can never be cheap or accidental (S5).
CREATE INDEX IF NOT EXISTS idx_inquiry_file_refs_owner ON inquiry_file_refs (user_id, inquiry_id);
