DROP INDEX IF EXISTS idx_quote_lines_deleted_at;
DROP INDEX IF EXISTS idx_quote_lines_service_rate_id;
DROP INDEX IF EXISTS idx_quote_lines_quote_id;
DROP INDEX IF EXISTS idx_quote_lines_org_id;
DROP TABLE IF EXISTS quote_lines;

DROP INDEX IF EXISTS idx_quotes_deleted_at;
DROP INDEX IF EXISTS idx_quotes_status;
DROP INDEX IF EXISTS idx_quotes_property_id;
DROP INDEX IF EXISTS idx_quotes_customer_id;
DROP INDEX IF EXISTS idx_quotes_org_id;
DROP TABLE IF EXISTS quotes;

DROP TYPE IF EXISTS quote_status;
