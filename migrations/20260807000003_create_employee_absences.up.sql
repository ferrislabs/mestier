-- Employee absences: leave, sick leave, and unavailability declarations.
--
-- Leave and unavailability are the same business fact — the person is not
-- available on this slot — with a different qualification carried by `kind`.
-- Declaring that an employee does not work a given day goes through an
-- UNAVAILABLE absence, never through an empty work slot (see the planning
-- module design doc).
--
-- An all-day absence is stored as the [00:00, 24:00) window of the
-- organization's timezone, mirroring `work_orders`; `all_day` tells the
-- renderer to draw it as a banner instead of a timed segment.
--
-- Unlike `assignments`, an absence is soft-deleted: it is business data the
-- user edits, not a disposable join row.

CREATE TYPE absence_kind AS ENUM (
    'LEAVE',
    'SICK',
    'UNAVAILABLE'
);

CREATE TABLE employee_absences (
    id          UUID          PRIMARY KEY,
    org_id      UUID          NOT NULL REFERENCES organizations(id),
    employee_id UUID          NOT NULL REFERENCES employees(id),
    kind        absence_kind  NOT NULL,
    starts_at   TIMESTAMPTZ   NOT NULL,
    ends_at     TIMESTAMPTZ   NOT NULL,
    all_day     BOOLEAN       NOT NULL DEFAULT false,
    note        TEXT          NULL,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ   NULL,

    CONSTRAINT chk_employee_absences_ends_at_after_starts_at
        CHECK (ends_at > starts_at),
    CONSTRAINT chk_employee_absences_note_not_blank_when_present
        CHECK (note IS NULL OR length(btrim(note)) > 0)
);

CREATE INDEX idx_employee_absences_org_id_starts_at ON employee_absences(org_id, starts_at);
CREATE INDEX idx_employee_absences_employee_id ON employee_absences(employee_id);
CREATE INDEX idx_employee_absences_deleted_at ON employee_absences(deleted_at);
