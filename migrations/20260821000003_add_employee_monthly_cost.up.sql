-- What a salaried employee actually costs.
--
-- `is_salaried` was added on the premise that "their cost is zero by design,
-- not a rate nobody entered yet" (see 20260820000002). That premise is wrong.
-- A salaried person costs their employer a salary every month, and costing them
-- at zero understates every project they touch — silently, because the
-- profitability report deliberately does not flag them as missing a rate. An
-- hour of their time was reading as 0,00 €.
--
-- So `is_salaried` keeps its purpose but changes its meaning: not "free", but
-- "costed from a monthly amount rather than an hourly one".
--
-- The amount is the *employer* cost, loaded with contributions, because that is
-- the number a margin has to be computed against. Storing a gross salary would
-- need a charge coefficient somewhere, which is a second setting to keep right
-- and a second thing to explain.
--
-- The hourly equivalent is derived, never stored:
--   monthly_cost_cents / (weekly_contract_minutes × 52 / 12 / 60)
-- so a 35 h contract divides by 151,67 h and a half-time by 75,83 h. Deriving it
-- means a contract change cannot leave a stale rate behind, and
-- `weekly_contract_minutes` is already there to divide by.

ALTER TABLE employees
    ADD COLUMN monthly_cost_cents INTEGER NULL;

ALTER TABLE employees
    ADD CONSTRAINT chk_employees_monthly_cost_not_negative
        CHECK (monthly_cost_cents IS NULL OR monthly_cost_cents >= 0),
    -- The two figures are exclusive: whichever one `is_salaried` does not use is
    -- cleared rather than left to linger, so nobody reads a stale number as a
    -- value somebody forgot to update.
    ADD CONSTRAINT chk_employees_one_cost_basis
        CHECK (
            (is_salaried AND hourly_rate_cents IS NULL)
            OR (NOT is_salaried AND monthly_cost_cents IS NULL)
        );

COMMENT ON COLUMN employees.monthly_cost_cents IS
    'Monthly employer cost (loaded with contributions) for a salaried employee. NULL means not set: profitability refuses to cost it rather than treating it as free.';

COMMENT ON COLUMN employees.is_salaried IS
    'True when this employee is costed from monthly_cost_cents rather than from an hourly rate. Never means their time is free.';
