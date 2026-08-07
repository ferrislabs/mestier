-- Planning module harness: shared columns every planning workstream reads.

-- An all-day planning entry is stored as the [00:00, 24:00) window of the
-- organization's timezone. Without an explicit zone, "Monday, all day" has no
-- stable definition.
ALTER TABLE organizations
    ADD COLUMN timezone TEXT NOT NULL DEFAULT 'Europe/Paris';

ALTER TABLE organizations
    ADD CONSTRAINT chk_organizations_timezone_not_blank
        CHECK (length(btrim(timezone)) > 0);

-- Contractual weekly base. Deliberately independent from the sum of the
-- employee's work slots: the gap between the two is what the planning surfaces.
ALTER TABLE employees
    ADD COLUMN weekly_contract_minutes INTEGER NOT NULL DEFAULT 0;

ALTER TABLE employees
    ADD CONSTRAINT chk_employees_weekly_contract_minutes_within_a_week
        CHECK (weekly_contract_minutes BETWEEN 0 AND 7 * 24 * 60);

-- NULL means "rate not set", 0 means "genuinely free". An employee record
-- created on the fly while assigning a work order has no rate yet, and a cost
-- computation must refuse to produce a figure rather than silently sum zero.
ALTER TABLE employees
    ALTER COLUMN hourly_rate_cents DROP NOT NULL;
