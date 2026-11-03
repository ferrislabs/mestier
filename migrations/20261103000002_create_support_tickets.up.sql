CREATE TABLE support_ticket_counters (
    org_id      UUID PRIMARY KEY REFERENCES organizations(id),
    last_number INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE support_tickets (
    id                 UUID PRIMARY KEY,
    org_id             UUID NOT NULL REFERENCES organizations(id),
    number             INTEGER NOT NULL,
    subject            TEXT NOT NULL,
    body               TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'new',
    assignee_member_id UUID NULL,
    assigned_at        TIMESTAMPTZ NULL,
    custom_fields      JSONB NOT NULL DEFAULT '{}'::jsonb,
    badges             JSONB NOT NULL DEFAULT '[]'::jsonb,
    external_id        TEXT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_support_tickets_assignee
        FOREIGN KEY (assignee_member_id, org_id)
        REFERENCES organization_members(id, organization_id)
        ON DELETE SET NULL (assignee_member_id),
    CONSTRAINT uq_support_tickets_number UNIQUE (org_id, number),
    CONSTRAINT uq_support_tickets_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_support_tickets_number CHECK (number > 0),
    CONSTRAINT chk_support_tickets_subject_not_blank CHECK (btrim(subject) <> ''),
    CONSTRAINT chk_support_tickets_status
        CHECK (status IN ('new', 'in_progress', 'waiting_customer', 'resolved')),
    CONSTRAINT chk_support_tickets_custom_fields CHECK (jsonb_typeof(custom_fields) = 'object'),
    CONSTRAINT chk_support_tickets_badges CHECK (jsonb_typeof(badges) = 'array'),
    CONSTRAINT chk_support_tickets_external_id_not_blank
        CHECK (external_id IS NULL OR btrim(external_id) <> '')
);

CREATE UNIQUE INDEX uq_support_tickets_external_id
    ON support_tickets(org_id, external_id)
    WHERE external_id IS NOT NULL;
CREATE INDEX idx_support_tickets_org_created ON support_tickets(org_id, created_at DESC);
CREATE INDEX idx_support_tickets_org_assignee ON support_tickets(org_id, assignee_member_id);
