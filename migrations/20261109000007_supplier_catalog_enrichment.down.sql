ALTER TABLE service_rates
    DROP CONSTRAINT IF EXISTS chk_service_rates_cost_cents_non_negative,
    DROP CONSTRAINT IF EXISTS chk_service_rates_billing_frequency_known,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS cost_cents,
    DROP COLUMN IF EXISTS billing_frequency,
    DROP COLUMN IF EXISTS category;

ALTER TABLE products
    DROP CONSTRAINT IF EXISTS chk_products_cost_cents_non_negative,
    DROP CONSTRAINT IF EXISTS chk_products_billing_frequency_known,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS cost_cents,
    DROP COLUMN IF EXISTS billing_frequency,
    DROP COLUMN IF EXISTS category;

DROP TABLE IF EXISTS supplier_addresses;
DROP TABLE IF EXISTS supplier_contacts;

ALTER TABLE suppliers
    DROP CONSTRAINT IF EXISTS fk_suppliers_default_category,
    DROP CONSTRAINT IF EXISTS chk_suppliers_notes_not_blank,
    DROP CONSTRAINT IF EXISTS chk_suppliers_iban_not_blank,
    DROP CONSTRAINT IF EXISTS chk_suppliers_payment_terms_days_range,
    DROP COLUMN IF EXISTS archived_at,
    DROP COLUMN IF EXISTS notes,
    DROP COLUMN IF EXISTS default_category_id,
    DROP COLUMN IF EXISTS payment_terms_days,
    DROP COLUMN IF EXISTS iban;
