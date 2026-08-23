-- A quote number, allocated when the quote leaves Draft, not when the
-- draft is created — allocating at creation lets a deleted draft burn a
-- number and leave a hole. Gapless and unique per organization; once
-- allocated it never changes. See #313.
--
-- The counter lives in its own table, one row per (organization, year),
-- locked by the upsert that reads it: `INSERT ... ON CONFLICT DO UPDATE`
-- takes a row lock for the statement's duration, so a second concurrent
-- allocation for the same organization and year blocks until the first
-- transaction commits or rolls back. That is what makes two quotes sent at
-- the same instant unable to receive the same number.
CREATE TABLE quote_number_counters (
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    year            INTEGER     NOT NULL,
    next_number     INTEGER     NOT NULL DEFAULT 1,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (organization_id, year),
    CONSTRAINT chk_quote_number_counters_next_number_positive
        CHECK (next_number > 0)
);

-- The format an organization controls: `{prefix}-{year}-{counter:04}`,
-- prefix defaulting to what quotes already used.
ALTER TABLE organizations
    ADD COLUMN quote_number_prefix TEXT NOT NULL DEFAULT 'DEV',
    ADD CONSTRAINT chk_organizations_quote_number_prefix_not_blank
        CHECK (length(btrim(quote_number_prefix)) > 0);

-- A draft has no number yet. Existing quotes keep whatever reference they
-- already carry — this is a rule for allocation going forward, not a
-- retroactive rewrite of history.
ALTER TABLE quotes ALTER COLUMN reference DROP NOT NULL;
ALTER TABLE quotes RENAME CONSTRAINT uq_quotes_org_reference TO uq_quotes_org_number;

-- Seed every organization's counter from the highest number its existing
-- quotes already used, so the next one allocated going forward cannot
-- collide with history. Only references shaped like the default format are
-- parsed; anything else was never produced by the allocator and is left
-- alone.
INSERT INTO quote_number_counters (organization_id, year, next_number)
SELECT
    org_id,
    split_part(reference, '-', 2)::INTEGER AS year,
    MAX(split_part(reference, '-', 3)::INTEGER) + 1 AS next_number
FROM quotes
WHERE reference ~ '^[^-]+-[0-9]{4}-[0-9]+$'
GROUP BY org_id, split_part(reference, '-', 2)::INTEGER;
