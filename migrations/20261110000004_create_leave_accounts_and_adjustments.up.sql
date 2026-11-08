CREATE TABLE leave_accounts (
    id                         UUID         PRIMARY KEY,
    org_id                     UUID         NOT NULL REFERENCES organizations(id),
    name                       TEXT         NOT NULL,
    acquisition_kind           TEXT         NOT NULL,
    acquisition_hundredths     INTEGER      NOT NULL,
    period_start_month         SMALLINT     NOT NULL,
    carry_over_max_hundredths  INTEGER      NOT NULL,
    archived_at                TIMESTAMPTZ  NULL,
    created_at                 TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT uq_leave_accounts_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_leave_accounts_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_leave_accounts_acquisition_kind
        CHECK (acquisition_kind IN ('MONTHLY', 'ANNUAL')),
    CONSTRAINT chk_leave_accounts_acquisition_positive CHECK (acquisition_hundredths > 0),
    CONSTRAINT chk_leave_accounts_period_start_month
        CHECK (period_start_month BETWEEN 1 AND 12),
    CONSTRAINT chk_leave_accounts_carry_over_max CHECK (carry_over_max_hundredths >= 0)
);

CREATE INDEX idx_leave_accounts_org_id ON leave_accounts(org_id, created_at, id);

CREATE TABLE leave_adjustments (
    id                UUID         PRIMARY KEY,
    org_id            UUID         NOT NULL REFERENCES organizations(id),
    member_id         UUID         NOT NULL,
    account_id        UUID         NOT NULL,
    date              DATE         NOT NULL,
    days_hundredths   INTEGER      NOT NULL,
    kind              TEXT         NOT NULL,
    reason            TEXT         NOT NULL,
    author_member_id  UUID         NULL,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT fk_leave_adjustments_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_leave_adjustments_account
        FOREIGN KEY (account_id, org_id) REFERENCES leave_accounts(id, org_id),
    CONSTRAINT fk_leave_adjustments_author
        FOREIGN KEY (author_member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_leave_adjustments_days_non_zero CHECK (days_hundredths <> 0),
    CONSTRAINT chk_leave_adjustments_kind CHECK (kind IN ('ADJUSTMENT', 'CARRY_OVER')),
    CONSTRAINT chk_leave_adjustments_reason_not_blank CHECK (length(btrim(reason)) > 0)
);

CREATE INDEX idx_leave_adjustments_org_date ON leave_adjustments(org_id, date, id);
CREATE INDEX idx_leave_adjustments_member ON leave_adjustments(org_id, member_id);
CREATE INDEX idx_leave_adjustments_account ON leave_adjustments(org_id, account_id);

UPDATE absence_types SET account_id = NULL WHERE account_id IS NOT NULL;

INSERT INTO leave_accounts (
    id, org_id, name, acquisition_kind, acquisition_hundredths,
    period_start_month, carry_over_max_hundredths, created_at
)
SELECT gen_random_uuid(), o.org_id, d.name, d.acquisition_kind, d.hundredths,
       d.period_start_month, d.carry_over_max, now() + d.position * interval '1 millisecond'
FROM (SELECT DISTINCT org_id FROM absence_types) o
CROSS JOIN (VALUES
    (0, 'Congés payés', 'MONTHLY', 208, 6::smallint, 500),
    (1, 'RTT',          'ANNUAL',  1000, 1::smallint, 0)
) AS d(position, name, acquisition_kind, hundredths, period_start_month, carry_over_max);

UPDATE absence_types t
SET account_id = a.id
FROM leave_accounts a
WHERE a.org_id = t.org_id
  AND a.name = t.name
  AND t.name IN ('Congés payés', 'RTT')
  AND t.account_id IS NULL;

ALTER TABLE absence_types
    ADD CONSTRAINT fk_absence_types_account
        FOREIGN KEY (account_id, org_id) REFERENCES leave_accounts(id, org_id);
