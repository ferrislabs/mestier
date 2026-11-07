DROP INDEX IF EXISTS idx_customers_owner_member_id;

ALTER TABLE customers
    DROP CONSTRAINT IF EXISTS fk_customers_billing_address,
    DROP CONSTRAINT IF EXISTS fk_customers_billing_contact,
    DROP CONSTRAINT IF EXISTS fk_customers_owner_member,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS converted_at,
    DROP COLUMN IF EXISTS billing_address_id,
    DROP COLUMN IF EXISTS billing_contact_id,
    DROP COLUMN IF EXISTS payment_terms_days,
    DROP COLUMN IF EXISTS notes,
    DROP COLUMN IF EXISTS lead_source,
    DROP COLUMN IF EXISTS owner_member_id,
    DROP COLUMN IF EXISTS vat_number,
    DROP COLUMN IF EXISTS website,
    DROP COLUMN IF EXISTS kind;

ALTER TABLE customer_contexts
    DROP COLUMN IF EXISTS address_line2,
    DROP COLUMN IF EXISTS country,
    DROP COLUMN IF EXISTS kind,
    DROP CONSTRAINT IF EXISTS uq_customer_contexts_id_customer;

ALTER TABLE customer_contacts
    DROP COLUMN IF EXISTS mobile,
    DROP CONSTRAINT IF EXISTS uq_customer_contacts_id_customer;
