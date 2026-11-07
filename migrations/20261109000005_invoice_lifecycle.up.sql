ALTER TABLE invoices
    ADD COLUMN title               TEXT        NOT NULL DEFAULT '',
    ADD COLUMN payment_terms_days  INTEGER     NOT NULL DEFAULT 30,
    ADD COLUMN legal_notice        TEXT        NOT NULL DEFAULT '',
    ADD COLUMN internal_note       TEXT        NULL,
    ADD COLUMN sent_at             TIMESTAMPTZ NULL,
    ADD COLUMN quote_id            UUID        NULL,
    ADD COLUMN issued_on           DATE        NULL,
    ADD COLUMN reason              TEXT        NULL,
    ADD COLUMN full_cancellation   BOOLEAN     NOT NULL DEFAULT FALSE;

UPDATE invoices
SET issued_on = (issued_at AT TIME ZONE 'UTC')::date
WHERE issued_at IS NOT NULL;

ALTER TABLE invoices
    ADD CONSTRAINT chk_invoices_payment_terms_days_range
        CHECK (payment_terms_days >= 0 AND payment_terms_days <= 60),
    ADD CONSTRAINT chk_invoices_internal_note_not_blank_when_present
        CHECK (internal_note IS NULL OR length(btrim(internal_note)) > 0),
    ADD CONSTRAINT chk_invoices_reason_not_blank_when_present
        CHECK (reason IS NULL OR length(btrim(reason)) > 0),
    ADD CONSTRAINT chk_invoices_issued_on_set_when_issued
        CHECK ((status = 'DRAFT') = (issued_on IS NULL)),
    ADD CONSTRAINT chk_invoices_credit_note_has_reason
        CHECK (kind <> 'CREDIT_NOTE' OR reason IS NOT NULL) NOT VALID,
    ADD CONSTRAINT chk_invoices_full_cancellation_is_a_credit_note
        CHECK (NOT full_cancellation OR kind = 'CREDIT_NOTE'),
    ADD CONSTRAINT fk_invoices_quote
        FOREIGN KEY (quote_id, org_id) REFERENCES quotes(id, org_id);

CREATE INDEX idx_invoices_quote_id ON invoices(quote_id) WHERE quote_id IS NOT NULL;

ALTER TABLE invoice_lines
    DROP CONSTRAINT chk_invoice_lines_unit_price_cents_non_negative,
    ADD COLUMN discount_percent NUMERIC(5, 2) NOT NULL DEFAULT 0,
    ADD COLUMN product_id       UUID          NULL REFERENCES products(id) ON DELETE SET NULL,
    ADD COLUMN service_rate_id  UUID          NULL REFERENCES service_rates(id) ON DELETE SET NULL,
    ADD COLUMN source_line_id   UUID          NULL REFERENCES invoice_lines(id),
    ADD CONSTRAINT chk_invoice_lines_discount_percent_range
        CHECK (discount_percent >= 0 AND discount_percent <= 100),
    ADD CONSTRAINT chk_invoice_lines_one_catalogue_reference
        CHECK (product_id IS NULL OR service_rate_id IS NULL);

CREATE INDEX idx_invoice_lines_product_id ON invoice_lines(product_id) WHERE product_id IS NOT NULL;
CREATE INDEX idx_invoice_lines_service_rate_id ON invoice_lines(service_rate_id) WHERE service_rate_id IS NOT NULL;
CREATE INDEX idx_invoice_lines_source_line_id ON invoice_lines(source_line_id) WHERE source_line_id IS NOT NULL;

UPDATE invoice_payments
SET note = CASE
        WHEN note IS NULL THEN '(mode saisi : ' || btrim(method) || ')'
        ELSE note || ' (mode saisi : ' || btrim(method) || ')'
    END,
    method = 'TRANSFER'
WHERE upper(btrim(method)) NOT IN ('CARD', 'TRANSFER', 'CHECK', 'CASH', 'DIRECT_DEBIT');

UPDATE invoice_payments
SET method = upper(btrim(method))
WHERE method <> upper(btrim(method));

ALTER TABLE invoice_payments
    DROP CONSTRAINT chk_invoice_payments_amount_cents_positive,
    ADD CONSTRAINT chk_invoice_payments_amount_cents_non_zero
        CHECK (amount_cents <> 0),
    ADD CONSTRAINT chk_invoice_payments_method_known
        CHECK (method IN ('CARD', 'TRANSFER', 'CHECK', 'CASH', 'DIRECT_DEBIT'));

CREATE TABLE invoice_events (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    invoice_id  UUID        NOT NULL,
    kind        TEXT        NOT NULL,
    detail      TEXT        NOT NULL DEFAULT '',
    at          TIMESTAMPTZ NOT NULL,
    actor       UUID        NULL REFERENCES users(id),
    CONSTRAINT chk_invoice_events_kind CHECK (kind IN ('SENT', 'REMINDER', 'NOTE')),
    CONSTRAINT fk_invoice_events_invoice
        FOREIGN KEY (invoice_id, org_id) REFERENCES invoices(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_invoice_events_invoice_id ON invoice_events(invoice_id, at);
