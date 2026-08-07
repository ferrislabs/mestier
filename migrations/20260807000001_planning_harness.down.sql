-- Reverting is lossy: employees whose rate was never set are collapsed to 0,
-- which the up migration exists precisely to distinguish from "not set".
UPDATE employees SET hourly_rate_cents = 0 WHERE hourly_rate_cents IS NULL;

ALTER TABLE employees
    ALTER COLUMN hourly_rate_cents SET NOT NULL;

ALTER TABLE employees
    DROP CONSTRAINT chk_employees_weekly_contract_minutes_within_a_week;

ALTER TABLE employees
    DROP COLUMN weekly_contract_minutes;

ALTER TABLE organizations
    DROP CONSTRAINT chk_organizations_timezone_not_blank;

ALTER TABLE organizations
    DROP COLUMN timezone;
