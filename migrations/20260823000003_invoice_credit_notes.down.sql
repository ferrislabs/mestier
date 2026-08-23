DROP INDEX IF EXISTS idx_invoices_source_invoice_id;

ALTER TABLE invoices
    DROP CONSTRAINT IF EXISTS fk_invoices_source_invoice,
    DROP COLUMN IF EXISTS source_invoice_id;
