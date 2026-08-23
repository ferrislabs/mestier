-- VAT per quote line, and the totals it feeds. The rate travels with the
-- line rather than being looked up from a referential at render time: a
-- document already sent must not change because somebody edited a rate
-- afterwards (same argument as the cost history epic, one table over).
-- Basis points (2000 = 20%, 550 = 5.5%), matching the "everything monetary
-- is an integer" house rule. See #312.

ALTER TABLE quote_lines
    ADD COLUMN vat_rate_bp INTEGER NULL,
    ADD CONSTRAINT chk_quote_lines_vat_rate_bp_range
        CHECK (vat_rate_bp IS NULL OR (vat_rate_bp >= 0 AND vat_rate_bp <= 10000));

-- Prefills, not derives: a default rate on the referential that a new line
-- copies once, never re-reads.
ALTER TABLE service_rates
    ADD COLUMN default_vat_rate_bp INTEGER NULL,
    ADD CONSTRAINT chk_service_rates_default_vat_rate_bp_range
        CHECK (default_vat_rate_bp IS NULL OR (default_vat_rate_bp >= 0 AND default_vat_rate_bp <= 10000));

ALTER TABLE products
    ADD COLUMN default_vat_rate_bp INTEGER NULL,
    ADD CONSTRAINT chk_products_default_vat_rate_bp_range
        CHECK (default_vat_rate_bp IS NULL OR (default_vat_rate_bp >= 0 AND default_vat_rate_bp <= 10000));

-- `total_cents` was always "the pre-VAT sum of the lines"; VAT gives that
-- fact a name. Existing rows carry no VAT data, so `gross_cents` starts
-- equal to `net_cents` and the breakdown starts empty — exactly the shape
-- an organization not subject to VAT produces going forward.
ALTER TABLE quotes RENAME COLUMN total_cents TO net_cents;
ALTER TABLE quotes RENAME CONSTRAINT chk_quotes_total_cents_non_negative TO chk_quotes_net_cents_non_negative;

ALTER TABLE quotes ADD COLUMN gross_cents INTEGER NULL;
UPDATE quotes SET gross_cents = net_cents;
ALTER TABLE quotes ALTER COLUMN gross_cents SET NOT NULL;
ALTER TABLE quotes ADD CONSTRAINT chk_quotes_gross_cents_non_negative CHECK (gross_cents >= 0);

ALTER TABLE quotes ADD COLUMN vat_breakdown JSONB NOT NULL DEFAULT '[]'::jsonb;
