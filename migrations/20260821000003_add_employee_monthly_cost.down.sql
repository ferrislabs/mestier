ALTER TABLE employees
    DROP CONSTRAINT chk_employees_one_cost_basis,
    DROP CONSTRAINT chk_employees_monthly_cost_not_negative;

ALTER TABLE employees
    DROP COLUMN monthly_cost_cents;

COMMENT ON COLUMN employees.is_salaried IS
    'True when this employee is not costed by the hour: their clocked time counts as zero labour cost and never blocks a margin for a missing rate.';
