DROP INDEX IF EXISTS uq_customer_contacts_customer_email_active;
DROP INDEX IF EXISTS idx_customer_contacts_deleted_at;
DROP INDEX IF EXISTS idx_customer_contacts_customer_id;
DROP TABLE IF EXISTS customer_contacts;

DROP INDEX IF EXISTS idx_customers_status;
ALTER TABLE customers DROP CONSTRAINT IF EXISTS chk_customers_status;
ALTER TABLE customers DROP COLUMN IF EXISTS status;
