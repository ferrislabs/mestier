-- The technical sheet a DOE copies a supply from: who sells it, who makes
-- it, what it is, and the supplier's own documents. All optional: most
-- products are never put in a DOE.
--
-- The supplier FK is composite, same device as `uq_suppliers_id_org`: a
-- product pointing at another organization's supplier is unrepresentable.

ALTER TABLE products
    ADD COLUMN supplier_id     UUID   NULL,
    ADD COLUMN manufacturer    TEXT   NULL,
    ADD COLUMN technical_specs TEXT   NULL,
    ADD COLUMN datasheet_keys  TEXT[] NOT NULL DEFAULT '{}',
    ADD CONSTRAINT fk_products_supplier
        FOREIGN KEY (supplier_id, org_id) REFERENCES suppliers(id, org_id),
    ADD CONSTRAINT chk_products_manufacturer_not_blank
        CHECK (manufacturer IS NULL OR btrim(manufacturer) <> ''),
    ADD CONSTRAINT chk_products_technical_specs_not_blank
        CHECK (technical_specs IS NULL OR btrim(technical_specs) <> ''),
    ADD CONSTRAINT chk_products_datasheet_keys_not_blank
        CHECK (array_position(datasheet_keys, '') IS NULL);

CREATE INDEX idx_products_supplier_id ON products(supplier_id);
