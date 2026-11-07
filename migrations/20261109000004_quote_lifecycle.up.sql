ALTER TYPE quote_status ADD VALUE IF NOT EXISTS 'SUPERSEDED';

ALTER TABLE quotes
    ADD CONSTRAINT uq_quotes_id_org UNIQUE (id, org_id),
    ADD COLUMN valid_until         DATE          NULL,
    ADD COLUMN deposit_percent     NUMERIC(4, 1) NOT NULL DEFAULT 0,
    ADD COLUMN owner_member_id     UUID          NULL,
    ADD COLUMN notes               TEXT          NULL,
    ADD COLUMN billing_contact_id  UUID          NULL,
    ADD COLUMN billing_address_id  UUID          NULL,
    ADD COLUMN responded_at        TIMESTAMPTZ   NULL,
    ADD COLUMN responded_by        TEXT          NULL,
    ADD COLUMN decline_reason      TEXT          NULL,
    ADD COLUMN revision_of_id      UUID          NULL,
    ADD COLUMN issuer_snapshot     JSONB         NULL,
    ADD COLUMN recipient_snapshot  JSONB         NULL,
    ADD CONSTRAINT chk_quotes_deposit_percent_range
        CHECK (deposit_percent >= 0 AND deposit_percent <= 100),
    ADD CONSTRAINT chk_quotes_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    ADD CONSTRAINT chk_quotes_responded_by_not_blank_when_present
        CHECK (responded_by IS NULL OR length(btrim(responded_by)) > 0),
    ADD CONSTRAINT chk_quotes_decline_reason_not_blank_when_present
        CHECK (decline_reason IS NULL OR length(btrim(decline_reason)) > 0),
    ADD CONSTRAINT chk_quotes_response_needs_responded_at
        CHECK ((responded_by IS NULL AND decline_reason IS NULL) OR responded_at IS NOT NULL),
    ADD CONSTRAINT chk_quotes_snapshots_set_together
        CHECK ((issuer_snapshot IS NULL) = (recipient_snapshot IS NULL)),
    ADD CONSTRAINT chk_quotes_revision_of_not_self
        CHECK (revision_of_id IS NULL OR revision_of_id <> id),
    ADD CONSTRAINT fk_quotes_owner_member
        FOREIGN KEY (owner_member_id, org_id) REFERENCES organization_members(id, organization_id),
    ADD CONSTRAINT fk_quotes_billing_contact
        FOREIGN KEY (billing_contact_id, customer_id) REFERENCES customer_contacts(id, customer_id),
    ADD CONSTRAINT fk_quotes_billing_address
        FOREIGN KEY (billing_address_id, customer_id) REFERENCES customer_contexts(id, customer_id),
    ADD CONSTRAINT fk_quotes_revision_of
        FOREIGN KEY (revision_of_id, org_id) REFERENCES quotes(id, org_id);

CREATE INDEX idx_quotes_revision_of_id ON quotes(revision_of_id) WHERE revision_of_id IS NOT NULL;
CREATE INDEX idx_quotes_owner_member_id ON quotes(owner_member_id) WHERE owner_member_id IS NOT NULL;

ALTER TABLE quote_lines
    ADD COLUMN discount_percent   NUMERIC(5, 2) NOT NULL DEFAULT 0,
    ADD COLUMN billing_frequency  TEXT          NOT NULL DEFAULT 'ONE_SHOT',
    ADD COLUMN product_id         UUID          NULL REFERENCES products(id) ON DELETE SET NULL,
    ADD CONSTRAINT chk_quote_lines_discount_percent_range
        CHECK (discount_percent >= 0 AND discount_percent <= 100),
    ADD CONSTRAINT chk_quote_lines_billing_frequency
        CHECK (billing_frequency IN ('ONE_SHOT', 'MONTHLY', 'QUARTERLY', 'YEARLY')),
    ADD CONSTRAINT chk_quote_lines_one_catalogue_reference
        CHECK (product_id IS NULL OR service_rate_id IS NULL);

CREATE INDEX idx_quote_lines_product_id ON quote_lines(product_id) WHERE product_id IS NOT NULL;
