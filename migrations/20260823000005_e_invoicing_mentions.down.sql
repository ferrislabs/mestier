ALTER TABLE invoices
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_all_or_nothing,
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_country_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_city_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_postal_code_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_line2_not_blank,
    DROP CONSTRAINT IF EXISTS chk_invoices_delivery_address_line1_not_blank,
    DROP COLUMN IF EXISTS delivery_address_country,
    DROP COLUMN IF EXISTS delivery_address_city,
    DROP COLUMN IF EXISTS delivery_address_postal_code,
    DROP COLUMN IF EXISTS delivery_address_line2,
    DROP COLUMN IF EXISTS delivery_address_line1,
    DROP COLUMN IF EXISTS operation_nature;

DROP TYPE IF EXISTS invoice_operation_nature;

ALTER TABLE customers
    DROP CONSTRAINT IF EXISTS chk_customers_registration_number_not_blank,
    DROP COLUMN IF EXISTS registration_number;

ALTER TABLE organizations
    DROP COLUMN IF EXISTS vat_on_debits;
