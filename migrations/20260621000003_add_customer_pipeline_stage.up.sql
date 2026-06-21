ALTER TABLE customers
    ADD COLUMN pipeline_stage TEXT NOT NULL DEFAULT 'NEW';

ALTER TABLE customers
    ADD CONSTRAINT chk_customers_pipeline_stage
        CHECK (pipeline_stage IN ('NEW', 'CONTACTED', 'QUALIFIED', 'QUOTE_SENT', 'WON', 'LOST'));

UPDATE customers
SET pipeline_stage = CASE
    WHEN status = 'CLIENT' THEN 'WON'
    WHEN status = 'ARCHIVED' THEN 'LOST'
    ELSE 'NEW'
END;

CREATE INDEX idx_customers_pipeline_stage ON customers(pipeline_stage);
