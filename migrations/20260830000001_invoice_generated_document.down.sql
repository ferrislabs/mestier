DROP TRIGGER IF EXISTS trg_invoices_forbid_document_reassignment ON invoices;
DROP FUNCTION IF EXISTS invoices_forbid_document_reassignment();

ALTER TABLE invoices
    DROP CONSTRAINT IF EXISTS chk_invoices_document_all_or_nothing,
    DROP CONSTRAINT IF EXISTS chk_invoices_document_mime_type_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_document_file_key_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_document_format_not_blank,
    DROP COLUMN IF EXISTS document_generated_at,
    DROP COLUMN IF EXISTS document_mime_type,
    DROP COLUMN IF EXISTS document_file_key,
    DROP COLUMN IF EXISTS document_format;
