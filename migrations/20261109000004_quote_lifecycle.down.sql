DROP INDEX IF EXISTS idx_quote_lines_product_id;

ALTER TABLE quote_lines
    DROP CONSTRAINT IF EXISTS chk_quote_lines_one_catalogue_reference,
    DROP CONSTRAINT IF EXISTS chk_quote_lines_billing_frequency,
    DROP CONSTRAINT IF EXISTS chk_quote_lines_discount_percent_range,
    DROP COLUMN IF EXISTS product_id,
    DROP COLUMN IF EXISTS billing_frequency,
    DROP COLUMN IF EXISTS discount_percent;

DROP INDEX IF EXISTS idx_quotes_owner_member_id;
DROP INDEX IF EXISTS idx_quotes_revision_of_id;

ALTER TABLE quotes
    DROP CONSTRAINT IF EXISTS fk_quotes_revision_of,
    DROP CONSTRAINT IF EXISTS fk_quotes_billing_address,
    DROP CONSTRAINT IF EXISTS fk_quotes_billing_contact,
    DROP CONSTRAINT IF EXISTS fk_quotes_owner_member,
    DROP CONSTRAINT IF EXISTS chk_quotes_revision_of_not_self,
    DROP CONSTRAINT IF EXISTS chk_quotes_snapshots_set_together,
    DROP CONSTRAINT IF EXISTS chk_quotes_response_needs_responded_at,
    DROP CONSTRAINT IF EXISTS chk_quotes_decline_reason_not_blank_when_present,
    DROP CONSTRAINT IF EXISTS chk_quotes_responded_by_not_blank_when_present,
    DROP CONSTRAINT IF EXISTS chk_quotes_notes_not_blank_when_present,
    DROP CONSTRAINT IF EXISTS chk_quotes_deposit_percent_range,
    DROP COLUMN IF EXISTS recipient_snapshot,
    DROP COLUMN IF EXISTS issuer_snapshot,
    DROP COLUMN IF EXISTS revision_of_id,
    DROP COLUMN IF EXISTS decline_reason,
    DROP COLUMN IF EXISTS responded_by,
    DROP COLUMN IF EXISTS responded_at,
    DROP COLUMN IF EXISTS billing_address_id,
    DROP COLUMN IF EXISTS billing_contact_id,
    DROP COLUMN IF EXISTS notes,
    DROP COLUMN IF EXISTS owner_member_id,
    DROP COLUMN IF EXISTS deposit_percent,
    DROP COLUMN IF EXISTS valid_until,
    DROP CONSTRAINT IF EXISTS uq_quotes_id_org;

UPDATE quotes SET status = 'SENT' WHERE status = 'SUPERSEDED';

ALTER TYPE quote_status RENAME TO quote_status_with_superseded;

CREATE TYPE quote_status AS ENUM ('DRAFT', 'SENT', 'ACCEPTED', 'DECLINED', 'CANCELLED');

ALTER TABLE quotes
    ALTER COLUMN status TYPE quote_status USING status::text::quote_status;

DROP TYPE quote_status_with_superseded;
