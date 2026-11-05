CREATE TABLE automation.run_counter (
    org_id     UUID   PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    last_value BIGINT NOT NULL,

    CONSTRAINT chk_automation_run_counter_value CHECK (last_value > 0)
);

ALTER TABLE automation.run
    ADD COLUMN run_number BIGINT NULL;

UPDATE automation.run
SET run_number = numbered.position
FROM (
    SELECT id, row_number() OVER (PARTITION BY org_id ORDER BY created_at, id) AS position
    FROM automation.run
) AS numbered
WHERE automation.run.id = numbered.id;

INSERT INTO automation.run_counter (org_id, last_value)
SELECT org_id, max(run_number)
FROM automation.run
GROUP BY org_id;

ALTER TABLE automation.run
    ALTER COLUMN run_number SET NOT NULL;

ALTER TABLE automation.run
    ADD CONSTRAINT uq_automation_run_number UNIQUE (org_id, run_number);
