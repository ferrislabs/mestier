ALTER TABLE customers
    ADD CONSTRAINT uq_customers_id_org UNIQUE (id, org_id);

CREATE TABLE deals (
    id                  UUID        PRIMARY KEY,
    org_id              UUID        NOT NULL REFERENCES organizations(id),
    customer_id         UUID        NOT NULL,
    title               TEXT        NOT NULL,
    amount_cents        BIGINT      NOT NULL DEFAULT 0,
    stage               TEXT        NOT NULL DEFAULT 'NEW',
    expected_close_date DATE        NULL,
    owner_member_id     UUID        NULL,
    source              TEXT        NULL,
    lost_reason         TEXT        NULL,
    notes               TEXT        NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    stage_changed_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at           TIMESTAMPTZ NULL,

    CONSTRAINT uq_deals_id_org UNIQUE (id, org_id),
    CONSTRAINT fk_deals_customer
        FOREIGN KEY (customer_id, org_id) REFERENCES customers(id, org_id),
    CONSTRAINT fk_deals_owner_member
        FOREIGN KEY (owner_member_id, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_deals_title_not_blank CHECK (length(btrim(title)) > 0),
    CONSTRAINT chk_deals_amount_cents_non_negative CHECK (amount_cents >= 0),
    CONSTRAINT chk_deals_stage
        CHECK (stage IN ('NEW', 'VISIT', 'QUOTE_SENT', 'NEGOTIATION', 'WON', 'LOST')),
    CONSTRAINT chk_deals_source
        CHECK (source IS NULL OR source IN (
            'bouche_a_oreille', 'site_web', 'recommandation', 'salon', 'architecte', 'autre'
        )),
    CONSTRAINT chk_deals_lost_reason_when_lost
        CHECK (stage <> 'LOST' OR (lost_reason IS NOT NULL AND length(btrim(lost_reason)) > 0)),
    CONSTRAINT chk_deals_lost_reason_only_when_lost
        CHECK (stage = 'LOST' OR lost_reason IS NULL),
    CONSTRAINT chk_deals_closed_at_only_when_closed
        CHECK ((stage IN ('WON', 'LOST')) = (closed_at IS NOT NULL)),
    CONSTRAINT chk_deals_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0)
);

CREATE INDEX idx_deals_org_stage ON deals(org_id, stage);
CREATE INDEX idx_deals_customer_id ON deals(customer_id);
CREATE INDEX idx_deals_owner_member_id ON deals(owner_member_id) WHERE owner_member_id IS NOT NULL;

INSERT INTO deals (
    id, org_id, customer_id, title, amount_cents, stage, owner_member_id, lost_reason,
    created_at, stage_changed_at, closed_at
)
SELECT
    gen_random_uuid(),
    c.org_id,
    c.id,
    'Opportunité ' || c.name,
    0,
    CASE c.pipeline_stage
        WHEN 'CONTACTED' THEN 'VISIT'
        WHEN 'QUALIFIED' THEN 'VISIT'
        ELSE c.pipeline_stage
    END,
    c.owner_member_id,
    CASE WHEN c.pipeline_stage = 'LOST' THEN 'Non renseigné' END,
    c.created_at,
    c.updated_at,
    CASE WHEN c.pipeline_stage IN ('WON', 'LOST') THEN c.updated_at END
FROM customers c
WHERE c.status <> 'ARCHIVED'
  AND c.deleted_at IS NULL;

ALTER TABLE quotes
    ADD COLUMN deal_id UUID NULL,
    ADD CONSTRAINT fk_quotes_deal
        FOREIGN KEY (deal_id, org_id) REFERENCES deals(id, org_id);

CREATE INDEX idx_quotes_deal_id ON quotes(deal_id) WHERE deal_id IS NOT NULL;
