CREATE TABLE day_overrides (
    id           UUID         PRIMARY KEY,
    org_id       UUID         NOT NULL REFERENCES organizations(id),
    member_id    UUID         NOT NULL,
    date         DATE         NOT NULL,
    kind         TEXT         NOT NULL,
    start_time   TIME         NULL,
    end_time     TIME         NULL,
    lunch_start  TIME         NULL,
    lunch_end    TIME         NULL,
    note         TEXT         NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_day_overrides_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT uq_day_overrides_member_date UNIQUE (member_id, date),
    CONSTRAINT chk_day_overrides_kind CHECK (kind IN ('WORK', 'OFF')),
    CONSTRAINT chk_day_overrides_work_hours CHECK (
        (kind = 'WORK' AND start_time IS NOT NULL AND end_time IS NOT NULL AND end_time > start_time)
        OR (kind = 'OFF' AND start_time IS NULL AND end_time IS NULL
            AND lunch_start IS NULL AND lunch_end IS NULL)
    ),
    CONSTRAINT chk_day_overrides_lunch CHECK (
        (lunch_start IS NULL AND lunch_end IS NULL)
        OR (lunch_start IS NOT NULL AND lunch_end IS NOT NULL
            AND lunch_end > lunch_start
            AND lunch_start > start_time AND lunch_end < end_time)
    )
);

CREATE INDEX idx_day_overrides_org_date ON day_overrides(org_id, date, member_id);
