ALTER TABLE organizations DROP CONSTRAINT IF EXISTS chk_organizations_invoice_number_prefix_not_blank;
ALTER TABLE organizations DROP COLUMN IF EXISTS invoice_number_prefix;

DROP TABLE IF EXISTS invoice_number_counters;

DROP INDEX IF EXISTS idx_invoice_lines_deleted_at;
DROP INDEX IF EXISTS idx_invoice_lines_invoice_id;
DROP INDEX IF EXISTS idx_invoice_lines_org_id;
DROP TABLE IF EXISTS invoice_lines;

DROP INDEX IF EXISTS idx_invoices_deleted_at;
DROP INDEX IF EXISTS idx_invoices_due_at;
DROP INDEX IF EXISTS idx_invoices_status;
DROP INDEX IF EXISTS idx_invoices_project_id;
DROP INDEX IF EXISTS idx_invoices_customer_id;
DROP INDEX IF EXISTS idx_invoices_org_id;
DROP TABLE IF EXISTS invoices;

DROP TYPE IF EXISTS invoice_kind;
DROP TYPE IF EXISTS invoice_status;
