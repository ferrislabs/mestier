-- Field photos, taken by the employee while clocked on a task.
--
-- A dedicated table rather than a `photo_keys` array like `quote_lines` has:
-- the phase is the point of these photos. A before/after pair is what proves
-- the work to a customer, and an array cannot say which is which.
--
-- Attached to the time entry rather than the task: a task spanning three days
-- has three sessions, and "before" means before *this* session's work. The
-- task-wide gallery is a read across its entries, which loses nothing.

CREATE TYPE time_entry_photo_phase AS ENUM (
    'BEFORE',
    'DURING',
    'AFTER'
);

CREATE TABLE time_entry_photos (
    id            UUID                   PRIMARY KEY,
    org_id        UUID                   NOT NULL REFERENCES organizations(id),
    time_entry_id UUID                   NOT NULL REFERENCES time_entries(id) ON DELETE CASCADE,
    phase         time_entry_photo_phase NOT NULL,
    storage_key   TEXT                   NOT NULL,
    created_at    TIMESTAMPTZ            NOT NULL DEFAULT now(),

    CONSTRAINT chk_time_entry_photos_storage_key_not_blank
        CHECK (length(btrim(storage_key)) > 0),
    -- The same object attached twice to one entry is a double submit, never
    -- an intent.
    CONSTRAINT uq_time_entry_photos_entry_key
        UNIQUE (time_entry_id, storage_key)
);

CREATE INDEX idx_time_entry_photos_org_id ON time_entry_photos(org_id);
CREATE INDEX idx_time_entry_photos_time_entry_id_phase
    ON time_entry_photos(time_entry_id, phase);
