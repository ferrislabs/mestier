ALTER TABLE quotes DROP COLUMN IF EXISTS vat_breakdown;

ALTER TABLE quotes DROP CONSTRAINT IF EXISTS chk_quotes_gross_cents_non_negative;
ALTER TABLE quotes DROP COLUMN IF EXISTS gross_cents;

ALTER TABLE quotes RENAME CONSTRAINT chk_quotes_net_cents_non_negative TO chk_quotes_total_cents_non_negative;
ALTER TABLE quotes RENAME COLUMN net_cents TO total_cents;

ALTER TABLE products
    DROP CONSTRAINT IF EXISTS chk_products_default_vat_rate_bp_range,
    DROP COLUMN IF EXISTS default_vat_rate_bp;

ALTER TABLE service_rates
    DROP CONSTRAINT IF EXISTS chk_service_rates_default_vat_rate_bp_range,
    DROP COLUMN IF EXISTS default_vat_rate_bp;

ALTER TABLE quote_lines
    DROP CONSTRAINT IF EXISTS chk_quote_lines_vat_rate_bp_range,
    DROP COLUMN IF EXISTS vat_rate_bp;
