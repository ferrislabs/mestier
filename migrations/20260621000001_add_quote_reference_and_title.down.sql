DROP INDEX IF EXISTS idx_quotes_reference;

ALTER TABLE quotes
    DROP CONSTRAINT IF EXISTS uq_quotes_org_reference,
    DROP CONSTRAINT IF EXISTS chk_quotes_title_not_blank,
    DROP CONSTRAINT IF EXISTS chk_quotes_reference_not_blank,
    DROP COLUMN IF EXISTS title,
    DROP COLUMN IF EXISTS reference;
