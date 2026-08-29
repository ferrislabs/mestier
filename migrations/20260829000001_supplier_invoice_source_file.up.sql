-- The original file behind a parsed supplier invoice (#339). "The file is
-- stored, not only parsed. The original is the legal record and the parse
-- is a derivation of it." Nullable: a manually entered invoice (no source
-- file at all) and every invoice created before this migration both have
-- nothing to point at.
ALTER TABLE supplier_invoices
    ADD COLUMN source_file_key       TEXT NULL,
    ADD COLUMN source_file_mime_type TEXT NULL,
    ADD CONSTRAINT chk_supplier_invoices_source_file_key_not_blank
        CHECK (source_file_key IS NULL OR length(btrim(source_file_key)) > 0),
    ADD CONSTRAINT chk_supplier_invoices_source_file_mime_type_not_blank
        CHECK (source_file_mime_type IS NULL OR length(btrim(source_file_mime_type)) > 0),
    -- Either both are set (a stored file) or neither is (no file behind
    -- this invoice) — never a key with no declared mime type or the
    -- reverse.
    ADD CONSTRAINT chk_supplier_invoices_source_file_pair
        CHECK (
            (source_file_key IS NULL AND source_file_mime_type IS NULL)
            OR (source_file_key IS NOT NULL AND source_file_mime_type IS NOT NULL)
        );
