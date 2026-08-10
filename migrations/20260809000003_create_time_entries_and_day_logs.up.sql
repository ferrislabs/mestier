-- Field clocking: actual time spent on a task, plus declared end-of-day.
-- Distinct from employee_work_time (planned capacity / rhythms).

CREATE TABLE time_entries (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id),
    task_id        UUID        NOT NULL REFERENCES tasks(id),
    employee_id    UUID        NOT NULL REFERENCES employees(id),
    started_at     TIMESTAMPTZ NOT NULL,
    ended_at       TIMESTAMPTZ NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_time_entries_ended_at_after_started_at
        CHECK (ended_at IS NULL OR ended_at > started_at)
);

CREATE INDEX idx_time_entries_org_id ON time_entries(org_id);
CREATE INDEX idx_time_entries_task_id ON time_entries(task_id);
CREATE INDEX idx_time_entries_employee_id_started_at ON time_entries(employee_id, started_at);

-- An employee may have only one open (active) time entry at a time.
CREATE UNIQUE INDEX uq_time_entries_one_active_per_employee
    ON time_entries(employee_id)
    WHERE ended_at IS NULL;

CREATE TABLE day_logs (
    id           UUID        PRIMARY KEY,
    org_id       UUID        NOT NULL REFERENCES organizations(id),
    employee_id  UUID        NOT NULL REFERENCES employees(id),
    work_date    DATE        NOT NULL,
    ended_at     TIMESTAMPTZ NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_day_logs_employee_work_date UNIQUE (employee_id, work_date)
);

CREATE INDEX idx_day_logs_org_id ON day_logs(org_id);
CREATE INDEX idx_day_logs_employee_id ON day_logs(employee_id);
