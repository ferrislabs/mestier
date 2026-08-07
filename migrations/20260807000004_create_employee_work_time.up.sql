-- Employee working time: a recurring weekly rhythm, versioned by
-- effective_from/effective_to, plus concrete work slots pinned to a real
-- date that take priority over the rhythm for that date.
--
-- A regularly-worked employee fills employee_rhythms and never touches
-- employee_work_slots. An irregular employee defines no rhythm and only
-- fills employee_work_slots — the table is not named "exceptions" because
-- for that employee it would be a lie (see the planning module design doc).
-- Declaring that an employee does not work a given day goes through an
-- UNAVAILABLE absence, never through an empty work slot.
--
-- Minutes are counted from midnight, in the organization's local time. No
-- TIME, no float: an integer compares and adds without surprise.

CREATE TABLE employee_rhythms (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id),
    employee_id     UUID        NOT NULL REFERENCES employees(id),
    effective_from  DATE        NOT NULL,
    effective_to    DATE        NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_employee_rhythms_effective_to_after_from
        CHECK (effective_to IS NULL OR effective_to > effective_from)
);

CREATE INDEX idx_employee_rhythms_employee_id_effective_from ON employee_rhythms(employee_id, effective_from);
CREATE INDEX idx_employee_rhythms_org_id ON employee_rhythms(org_id);

CREATE TABLE employee_rhythm_slots (
    id             UUID     PRIMARY KEY,
    rhythm_id      UUID     NOT NULL REFERENCES employee_rhythms(id) ON DELETE CASCADE,
    weekday        SMALLINT NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    starts_minute  SMALLINT NOT NULL CHECK (starts_minute BETWEEN 0 AND 1439),
    ends_minute    SMALLINT NOT NULL CHECK (ends_minute BETWEEN 1 AND 1440),

    CONSTRAINT chk_employee_rhythm_slots_ends_after_starts
        CHECK (ends_minute > starts_minute)
);

CREATE INDEX idx_employee_rhythm_slots_rhythm_id ON employee_rhythm_slots(rhythm_id);

-- Plages de travail: work slots pinned to a real date. They win over the
-- rhythm for that date.
CREATE TABLE employee_work_slots (
    id             UUID     PRIMARY KEY,
    org_id         UUID     NOT NULL REFERENCES organizations(id),
    employee_id    UUID     NOT NULL REFERENCES employees(id),
    work_date      DATE     NOT NULL,
    starts_minute  SMALLINT NOT NULL CHECK (starts_minute BETWEEN 0 AND 1439),
    ends_minute    SMALLINT NOT NULL CHECK (ends_minute BETWEEN 1 AND 1440),

    CONSTRAINT chk_employee_work_slots_ends_after_starts
        CHECK (ends_minute > starts_minute)
);

CREATE INDEX idx_employee_work_slots_employee_id_work_date ON employee_work_slots(employee_id, work_date);
CREATE INDEX idx_employee_work_slots_org_id ON employee_work_slots(org_id);
