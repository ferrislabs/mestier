-- A salaried employee has no meaningful hourly figure: their cost is zero by
-- design, not a rate nobody entered yet. Profitability must be able to tell
-- the two apart, which is what this column is for.
ALTER TABLE employees
    ADD COLUMN is_salaried BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN employees.is_salaried IS
    'True when this employee is not costed by the hour: their clocked time counts as zero labour cost and never blocks a margin for a missing rate.';
