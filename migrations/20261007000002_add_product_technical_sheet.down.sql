DROP INDEX idx_products_supplier_id;

ALTER TABLE products
    DROP CONSTRAINT fk_products_supplier,
    DROP COLUMN datasheet_keys,
    DROP COLUMN technical_specs,
    DROP COLUMN manufacturer,
    DROP COLUMN supplier_id;
