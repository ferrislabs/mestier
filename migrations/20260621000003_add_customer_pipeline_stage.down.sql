DROP INDEX IF EXISTS idx_customers_pipeline_stage;

ALTER TABLE customers
    DROP CONSTRAINT IF EXISTS chk_customers_pipeline_stage;

ALTER TABLE customers
    DROP COLUMN IF EXISTS pipeline_stage;
