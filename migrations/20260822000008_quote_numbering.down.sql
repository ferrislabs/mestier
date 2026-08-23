ALTER TABLE quotes RENAME CONSTRAINT uq_quotes_org_number TO uq_quotes_org_reference;

-- Restoring NOT NULL is only safe if every row still has a reference. Any
-- draft numbered under this migration's rule (i.e. left NULL) blocks the
-- revert, which is the correct failure: silently inventing a reference to
-- satisfy the constraint would be worse than refusing to go back.
ALTER TABLE quotes ALTER COLUMN reference SET NOT NULL;

ALTER TABLE organizations
    DROP CONSTRAINT IF EXISTS chk_organizations_quote_number_prefix_not_blank,
    DROP COLUMN IF EXISTS quote_number_prefix;

DROP TABLE IF EXISTS quote_number_counters;
