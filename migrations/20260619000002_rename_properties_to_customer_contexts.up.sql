ALTER TABLE properties RENAME TO customer_contexts;

ALTER TABLE customer_contexts
    RENAME CONSTRAINT properties_customer_id_fkey TO customer_contexts_customer_id_fkey;
ALTER TABLE customer_contexts
    RENAME CONSTRAINT chk_properties_label_not_blank TO chk_customer_contexts_label_not_blank;
ALTER TABLE customer_contexts
    RENAME CONSTRAINT chk_properties_photo_key_not_blank_when_present TO chk_customer_contexts_photo_key_not_blank_when_present;

ALTER TABLE customer_contexts DROP CONSTRAINT chk_properties_street_not_blank;
ALTER TABLE customer_contexts DROP CONSTRAINT chk_properties_zip_not_blank;
ALTER TABLE customer_contexts DROP CONSTRAINT chk_properties_city_not_blank;

ALTER INDEX idx_properties_customer_id RENAME TO idx_customer_contexts_customer_id;
ALTER INDEX idx_properties_deleted_at RENAME TO idx_customer_contexts_deleted_at;
ALTER INDEX uq_properties_customer_label_active RENAME TO uq_customer_contexts_customer_label_active;

ALTER TABLE customer_contexts RENAME COLUMN street TO address_line;
ALTER TABLE customer_contexts RENAME COLUMN zip TO postal_code;
ALTER TABLE customer_contexts ALTER COLUMN address_line DROP NOT NULL;
ALTER TABLE customer_contexts ALTER COLUMN postal_code DROP NOT NULL;
ALTER TABLE customer_contexts ALTER COLUMN city DROP NOT NULL;

ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_customer_contexts_address_line_not_blank_when_present
    CHECK (address_line IS NULL OR length(btrim(address_line)) > 0);
ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_customer_contexts_postal_code_not_blank_when_present
    CHECK (postal_code IS NULL OR length(btrim(postal_code)) > 0);
ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_customer_contexts_city_not_blank_when_present
    CHECK (city IS NULL OR length(btrim(city)) > 0);

ALTER TABLE quotes RENAME COLUMN property_id TO customer_context_id;
ALTER TABLE quotes RENAME CONSTRAINT quotes_property_id_fkey TO quotes_customer_context_id_fkey;
ALTER INDEX idx_quotes_property_id RENAME TO idx_quotes_customer_context_id;
