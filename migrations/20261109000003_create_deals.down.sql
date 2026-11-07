DROP INDEX IF EXISTS idx_quotes_deal_id;

ALTER TABLE quotes
    DROP CONSTRAINT IF EXISTS fk_quotes_deal,
    DROP COLUMN IF EXISTS deal_id;

DROP TABLE IF EXISTS deals;

ALTER TABLE customers
    DROP CONSTRAINT IF EXISTS uq_customers_id_org;
