CREATE TABLE products (
    id               UUID        PRIMARY KEY,
    org_id           UUID        NOT NULL REFERENCES organizations(id),
    name             TEXT        NOT NULL,
    sku              TEXT        NULL,
    unit             TEXT        NOT NULL,
    unit_price_cents INTEGER     NOT NULL,
    description      TEXT        NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at       TIMESTAMPTZ NULL,

    CONSTRAINT chk_products_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_products_unit_not_blank
        CHECK (length(btrim(unit)) > 0),
    CONSTRAINT chk_products_unit_price_cents_non_negative
        CHECK (unit_price_cents >= 0)
);

CREATE INDEX idx_products_org_id ON products(org_id);
CREATE INDEX idx_products_deleted_at ON products(deleted_at);

CREATE UNIQUE INDEX uq_products_org_name_active
    ON products(org_id, lower(name))
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_products_org_sku_active
    ON products(org_id, lower(sku))
    WHERE sku IS NOT NULL AND deleted_at IS NULL;
