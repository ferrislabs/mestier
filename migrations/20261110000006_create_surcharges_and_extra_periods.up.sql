CREATE TABLE surcharges (
    id           UUID         PRIMARY KEY,
    org_id       UUID         NOT NULL REFERENCES organizations(id),
    name         TEXT         NOT NULL,
    rate_bp      INTEGER      NOT NULL,
    kind         TEXT         NOT NULL,
    archived_at  TIMESTAMPTZ  NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT uq_surcharges_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_surcharges_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_surcharges_rate CHECK (rate_bp BETWEEN 0 AND 20000),
    CONSTRAINT chk_surcharges_kind
        CHECK (kind IN ('OVERTIME', 'SUNDAY', 'HOLIDAY', 'NIGHT', 'OTHER'))
);

CREATE INDEX idx_surcharges_org_id ON surcharges(org_id, created_at, id);

CREATE TABLE extra_periods (
    id                     UUID         PRIMARY KEY,
    org_id                 UUID         NOT NULL REFERENCES organizations(id),
    member_id              UUID         NOT NULL,
    date                   DATE         NOT NULL,
    start_time             TIME         NOT NULL,
    end_time               TIME         NOT NULL,
    lunch_start            TIME         NULL,
    lunch_end              TIME         NULL,
    reason                 TEXT         NOT NULL,
    surcharge_id           UUID         NULL,
    rate_bp                INTEGER      NOT NULL,
    status                 TEXT         NOT NULL,
    created_by_member_id   UUID         NULL,
    decided_by_member_id   UUID         NULL,
    decided_at             TIMESTAMPTZ  NULL,
    decision_comment       TEXT         NULL,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_extra_periods_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_extra_periods_surcharge
        FOREIGN KEY (surcharge_id, org_id) REFERENCES surcharges(id, org_id),
    CONSTRAINT fk_extra_periods_creator
        FOREIGN KEY (created_by_member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_extra_periods_decider
        FOREIGN KEY (decided_by_member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_extra_periods_span CHECK (end_time > start_time),
    CONSTRAINT chk_extra_periods_lunch CHECK (
        (lunch_start IS NULL AND lunch_end IS NULL)
        OR (lunch_start IS NOT NULL AND lunch_end IS NOT NULL
            AND lunch_end > lunch_start
            AND lunch_start > start_time AND lunch_end < end_time)
    ),
    CONSTRAINT chk_extra_periods_reason_not_blank CHECK (length(btrim(reason)) > 0),
    CONSTRAINT chk_extra_periods_rate CHECK (rate_bp BETWEEN 0 AND 20000),
    CONSTRAINT chk_extra_periods_status
        CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED', 'CANCELLED')),
    CONSTRAINT chk_extra_periods_rejection_comment CHECK (
        status <> 'REJECTED' OR length(btrim(coalesce(decision_comment, ''))) > 0
    )
);

CREATE INDEX idx_extra_periods_org_date ON extra_periods(org_id, date DESC, id);
CREATE INDEX idx_extra_periods_member ON extra_periods(org_id, member_id, date);
CREATE INDEX idx_extra_periods_status ON extra_periods(org_id, status);
