DO $$
DECLARE
    parked BIGINT;
BEGIN
    SELECT count(*) INTO parked
    FROM automation.run_step
    WHERE status = 'waiting';

    IF parked > 0 THEN
        RAISE EXCEPTION
            'cannot revert 20261107000003: % step(s) are waiting for their resume time, and the previous version cannot resume them',
            parked;
    END IF;
END
$$;

DROP INDEX IF EXISTS automation.idx_automation_run_parked;

ALTER TABLE automation.run_step
    DROP CONSTRAINT IF EXISTS chk_automation_run_step_resume_at,
    DROP CONSTRAINT IF EXISTS chk_automation_run_step_status;

ALTER TABLE automation.run_step
    DROP COLUMN IF EXISTS resume_at;

ALTER TABLE automation.run_step
    ADD CONSTRAINT chk_automation_run_step_status
        CHECK (status IN ('pending', 'in_flight', 'succeeded', 'failed', 'skipped', 'dead'));
