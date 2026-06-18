ALTER INDEX idx_quotes_customer_context_id RENAME TO idx_quotes_property_id;
ALTER TABLE quotes RENAME CONSTRAINT quotes_customer_context_id_fkey TO quotes_property_id_fkey;
ALTER TABLE quotes RENAME COLUMN customer_context_id TO property_id;

ALTER TABLE customer_contexts DROP CONSTRAINT chk_customer_contexts_city_not_blank_when_present;
ALTER TABLE customer_contexts DROP CONSTRAINT chk_customer_contexts_postal_code_not_blank_when_present;
ALTER TABLE customer_contexts DROP CONSTRAINT chk_customer_contexts_address_line_not_blank_when_present;

ALTER TABLE customer_contexts ALTER COLUMN city SET NOT NULL;
ALTER TABLE customer_contexts ALTER COLUMN postal_code SET NOT NULL;
ALTER TABLE customer_contexts ALTER COLUMN address_line SET NOT NULL;
ALTER TABLE customer_contexts RENAME COLUMN postal_code TO zip;
ALTER TABLE customer_contexts RENAME COLUMN address_line TO street;

ALTER INDEX uq_customer_contexts_customer_label_active RENAME TO uq_properties_customer_label_active;
ALTER INDEX idx_customer_contexts_deleted_at RENAME TO idx_properties_deleted_at;
ALTER INDEX idx_customer_contexts_customer_id RENAME TO idx_properties_customer_id;

ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_properties_city_not_blank
    CHECK (length(btrim(city)) > 0);
ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_properties_zip_not_blank
    CHECK (length(btrim(zip)) > 0);
ALTER TABLE customer_contexts
    ADD CONSTRAINT chk_properties_street_not_blank
    CHECK (length(btrim(street)) > 0);

ALTER TABLE customer_contexts
    RENAME CONSTRAINT chk_customer_contexts_photo_key_not_blank_when_present TO chk_properties_photo_key_not_blank_when_present;
ALTER TABLE customer_contexts
    RENAME CONSTRAINT chk_customer_contexts_label_not_blank TO chk_properties_label_not_blank;
ALTER TABLE customer_contexts
    RENAME CONSTRAINT customer_contexts_customer_id_fkey TO properties_customer_id_fkey;

ALTER TABLE customer_contexts RENAME TO properties;
