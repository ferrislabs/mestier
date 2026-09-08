ALTER TABLE products ADD COLUMN photo_keys TEXT[] NOT NULL DEFAULT '{}';

ALTER TABLE products ADD CONSTRAINT chk_products_photo_keys_not_blank
    CHECK (array_position(photo_keys, '') IS NULL);
