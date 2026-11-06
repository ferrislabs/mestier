ALTER TABLE automation.run_step
    ADD COLUMN resume_at TIMESTAMPTZ NULL;

ALTER TABLE automation.run_step
    DROP CONSTRAINT chk_automation_run_step_status;

ALTER TABLE automation.run_step
    ADD CONSTRAINT chk_automation_run_step_status
        CHECK (status IN ('pending', 'in_flight', 'succeeded', 'failed', 'skipped', 'dead', 'waiting')),
    ADD CONSTRAINT chk_automation_run_step_resume_at
        CHECK ((status = 'waiting') = (resume_at IS NOT NULL));

COMMENT ON COLUMN automation.run_step.resume_at IS
    'Set while the step is waiting: the instant the worker resumes it. NULL in every other status.';

CREATE INDEX idx_automation_run_parked
    ON automation.run (next_attempt_at)
    WHERE status = 'running' AND locked_by IS NULL;
