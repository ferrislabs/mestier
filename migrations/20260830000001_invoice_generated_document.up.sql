-- The file actually generated for an issued invoice (#342) -- Factur-X
-- today, another `DocumentFormat` adapter possibly tomorrow -- and the fact
-- that it is stored, referenced from the invoice it was generated for.
--
-- "The generated file is stored and referenced from the invoice. It is the
-- artefact that was sent, and regenerating it later from current data would
-- produce a different file, which defeats the point." That is enforced here,
-- not only in application code: `chk_invoices_document_generated_once`
-- forbids an `UPDATE` from ever moving `document_file_key` off a value it
-- already holds, so the repository's own `record_generated_document` can
-- rely on its `WHERE document_file_key IS NULL` guard being the only way in,
-- independent of whatever the service checked first.
--
-- Nullable, same reasoning as `supplier_invoices.source_file_key` (see
-- `20260829000001_supplier_invoice_source_file`): every invoice issued
-- before this migration, and every draft, has nothing to point at yet.
ALTER TABLE invoices
    ADD COLUMN document_format       TEXT        NULL,
    ADD COLUMN document_file_key     TEXT        NULL,
    ADD COLUMN document_mime_type    TEXT        NULL,
    ADD COLUMN document_generated_at TIMESTAMPTZ NULL,
    ADD CONSTRAINT chk_invoices_document_format_not_blank
        CHECK (document_format IS NULL OR length(btrim(document_format)) > 0),
    ADD CONSTRAINT chk_invoices_document_file_key_not_blank
        CHECK (document_file_key IS NULL OR length(btrim(document_file_key)) > 0),
    ADD CONSTRAINT chk_invoices_document_mime_type_not_blank
        CHECK (document_mime_type IS NULL OR length(btrim(document_mime_type)) > 0),
    -- All four together, or none: never a key with no declared format, mime
    -- type or timestamp, or any of the reverse.
    ADD CONSTRAINT chk_invoices_document_all_or_nothing
        CHECK (
            (document_format IS NULL AND document_file_key IS NULL
                AND document_mime_type IS NULL AND document_generated_at IS NULL)
            OR (document_format IS NOT NULL AND document_file_key IS NOT NULL
                AND document_mime_type IS NOT NULL AND document_generated_at IS NOT NULL)
        );

-- A trigger, not only the repository's `WHERE` clause: the invariant this
-- migration exists for ("never silently replace what this points at") must
-- hold against any writer, present or future, not only the one call site
-- that happens to remember the rule today.
CREATE FUNCTION invoices_forbid_document_reassignment() RETURNS trigger AS $$
BEGIN
    IF OLD.document_file_key IS NOT NULL
        AND NEW.document_file_key IS DISTINCT FROM OLD.document_file_key THEN
        RAISE EXCEPTION 'invoice % already has a generated document; it cannot be replaced', OLD.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_invoices_forbid_document_reassignment
    BEFORE UPDATE ON invoices
    FOR EACH ROW
    EXECUTE FUNCTION invoices_forbid_document_reassignment();
