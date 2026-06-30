ALTER TABLE quotes DROP COLUMN total_ttc_cents, DROP COLUMN total_vat_cents, DROP COLUMN total_ht_cents;
ALTER TABLE quote_lines   DROP COLUMN vat_rate;
ALTER TABLE service_rates DROP COLUMN vat_rate;
ALTER TABLE products      DROP COLUMN vat_rate;
