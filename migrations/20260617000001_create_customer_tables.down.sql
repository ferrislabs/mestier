DROP INDEX IF EXISTS uq_properties_customer_label_active;
DROP INDEX IF EXISTS idx_properties_deleted_at;
DROP INDEX IF EXISTS idx_properties_customer_id;
DROP TABLE IF EXISTS properties;

DROP INDEX IF EXISTS idx_customers_deleted_at;
DROP INDEX IF EXISTS idx_customers_org_id;
DROP TABLE IF EXISTS customers;
