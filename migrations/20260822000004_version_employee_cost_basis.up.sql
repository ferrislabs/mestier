-- What an employee costs, versioned by effective date.
--
-- Today `employees.hourly_rate_cents`/`is_salaried`/`monthly_cost_cents`/
-- `weekly_contract_minutes` are a single current value: raising somebody's
-- rate overwrites it, and every past profitability report computed from
-- that column silently recomputes at the new rate. A report about March
-- must keep costing March at what was true in March, even after a June
-- raise.
--
-- Shaped exactly like `employee_rhythms` (see 20260807000004), because that
-- table already solved this for hours: one row per version, effective_from
-- inclusive, effective_to exclusive and nullable (null = still open), and
-- the database — not a service check a second write path can bypass —
-- enforces that versions never overlap and that at most one stays open.
--
-- `employees` keeps its four columns as a projection of the open version,
-- maintained by the repository. Dropping them once nothing reads them
-- directly is a follow-up, the same treatment #260 gave `tasks.customer_id`.

-- The exclusion constraint below rejects two versions of the same employee
-- whose ranges overlap. Postgres can only do that through a GiST index, and
-- a GiST index needs an equality operator class for `employee_id` (UUID) to
-- combine with the range overlap operator on `effective_from`/`effective_to`
-- — that operator class is what btree_gist provides.
CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE employee_cost_bases (
    id                        UUID        PRIMARY KEY,
    org_id                    UUID        NOT NULL REFERENCES organizations(id),
    employee_id               UUID        NOT NULL REFERENCES employees(id),
    effective_from            DATE        NOT NULL,
    effective_to              DATE        NULL,
    is_salaried               BOOLEAN     NOT NULL DEFAULT false,
    hourly_rate_cents         INTEGER     NULL,
    monthly_cost_cents        INTEGER     NULL,
    weekly_contract_minutes   INTEGER     NOT NULL DEFAULT 0,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_employee_cost_bases_effective_to_after_from
        CHECK (effective_to IS NULL OR effective_to > effective_from),
    -- Per-version form of `chk_employees_one_cost_basis`: whichever figure a
    -- version does not use is absent, never a stale value nobody cleared.
    CONSTRAINT chk_employee_cost_bases_one_basis
        CHECK (
            (is_salaried AND hourly_rate_cents IS NULL)
            OR (NOT is_salaried AND monthly_cost_cents IS NULL)
        ),
    CONSTRAINT chk_employee_cost_bases_hourly_rate_non_negative
        CHECK (hourly_rate_cents IS NULL OR hourly_rate_cents >= 0),
    CONSTRAINT chk_employee_cost_bases_monthly_cost_non_negative
        CHECK (monthly_cost_cents IS NULL OR monthly_cost_cents >= 0),
    CONSTRAINT chk_employee_cost_bases_weekly_contract_minutes_range
        CHECK (weekly_contract_minutes BETWEEN 0 AND 10080)
);

CREATE INDEX idx_employee_cost_bases_employee_id_effective_from ON employee_cost_bases(employee_id, effective_from);
CREATE INDEX idx_employee_cost_bases_org_id ON employee_cost_bases(org_id);

-- At most one open version per employee.
CREATE UNIQUE INDEX uq_employee_cost_bases_open_version
    ON employee_cost_bases(employee_id)
    WHERE effective_to IS NULL;

-- No two versions of the same employee may cover the same day. `effective_to
-- IS NULL` reads as "unbounded" here (`daterange`'s upper bound defaults to
-- infinity when omitted), so the open version still participates correctly.
ALTER TABLE employee_cost_bases
    ADD CONSTRAINT excl_employee_cost_bases_no_overlap
        EXCLUDE USING gist (
            employee_id WITH =,
            daterange(effective_from, effective_to) WITH &&
        );

COMMENT ON TABLE employee_cost_bases IS
    'What an employee costs, versioned by effective date. A raise closes the open version and opens a new one; past versions are never rewritten, so a profitability report keeps the numbers it had.';

-- Backfill: one open version per existing employee, carrying today's column
-- values. `effective_from` is the profile's own creation date so no past
-- report's numbers move — a migration that shifted a published margin would
-- be its own bug.
INSERT INTO employee_cost_bases (
    id, org_id, employee_id, effective_from, effective_to,
    is_salaried, hourly_rate_cents, monthly_cost_cents, weekly_contract_minutes,
    created_at, updated_at
)
SELECT
    gen_random_uuid(), e.org_id, e.id, e.created_at::date, NULL,
    e.is_salaried, e.hourly_rate_cents, e.monthly_cost_cents, e.weekly_contract_minutes,
    e.created_at, e.updated_at
FROM employees e;
