DROP TABLE IF EXISTS invoice_events;

ALTER TABLE invoice_payments
    DROP CONSTRAINT IF EXISTS chk_invoice_payments_method_known,
    DROP CONSTRAINT IF EXISTS chk_invoice_payments_amount_cents_non_zero,
    ADD CONSTRAINT chk_invoice_payments_amount_cents_positive
        CHECK (amount_cents > 0) NOT VALID;

DROP INDEX IF EXISTS idx_invoice_lines_source_line_id;
DROP INDEX IF EXISTS idx_invoice_lines_service_rate_id;
DROP INDEX IF EXISTS idx_invoice_lines_product_id;

ALTER TABLE invoice_lines
    DROP CONSTRAINT IF EXISTS chk_invoice_lines_one_catalogue_reference,
    DROP CONSTRAINT IF EXISTS chk_invoice_lines_discount_percent_range,
    DROP COLUMN IF EXISTS source_line_id,
    DROP COLUMN IF EXISTS service_rate_id,
    DROP COLUMN IF EXISTS product_id,
    DROP COLUMN IF EXISTS discount_percent,
    ADD CONSTRAINT chk_invoice_lines_unit_price_cents_non_negative
        CHECK (unit_price_cents >= 0) NOT VALID;

DROP INDEX IF EXISTS idx_invoices_quote_id;

ALTER TABLE invoices
    DROP CONSTRAINT IF EXISTS fk_invoices_quote,
    DROP CONSTRAINT IF EXISTS chk_invoices_full_cancellation_is_a_credit_note,
    DROP CONSTRAINT IF EXISTS chk_invoices_credit_note_has_reason,
    DROP CONSTRAINT IF EXISTS chk_invoices_issued_on_set_when_issued,
    DROP CONSTRAINT IF EXISTS chk_invoices_reason_not_blank_when_present,
    DROP CONSTRAINT IF EXISTS chk_invoices_internal_note_not_blank_when_present,
    DROP CONSTRAINT IF EXISTS chk_invoices_payment_terms_days_range,
    DROP COLUMN IF EXISTS full_cancellation,
    DROP COLUMN IF EXISTS reason,
    DROP COLUMN IF EXISTS issued_on,
    DROP COLUMN IF EXISTS quote_id,
    DROP COLUMN IF EXISTS sent_at,
    DROP COLUMN IF EXISTS internal_note,
    DROP COLUMN IF EXISTS legal_notice,
    DROP COLUMN IF EXISTS payment_terms_days,
    DROP COLUMN IF EXISTS title;
