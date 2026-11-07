ALTER TABLE automation.run
    DROP CONSTRAINT IF EXISTS uq_automation_run_number;

ALTER TABLE automation.run
    DROP COLUMN IF EXISTS run_number;

DROP TABLE IF EXISTS automation.run_counter;
