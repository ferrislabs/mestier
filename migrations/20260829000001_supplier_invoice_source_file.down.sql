ALTER TABLE supplier_invoices
    DROP CONSTRAINT IF EXISTS chk_supplier_invoices_source_file_pair,
    DROP CONSTRAINT IF EXISTS chk_supplier_invoices_source_file_mime_type_not_blank,
    DROP CONSTRAINT IF EXISTS chk_supplier_invoices_source_file_key_not_blank,
    DROP COLUMN IF EXISTS source_file_mime_type,
    DROP COLUMN IF EXISTS source_file_key;
