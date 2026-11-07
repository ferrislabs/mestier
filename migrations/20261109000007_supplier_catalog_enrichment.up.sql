ALTER TABLE suppliers
    ADD COLUMN iban                TEXT        NULL,
    ADD COLUMN payment_terms_days  INTEGER     NULL,
    ADD COLUMN default_category_id UUID        NULL,
    ADD COLUMN notes               TEXT        NULL,
    ADD COLUMN archived_at         TIMESTAMPTZ NULL,
    ADD CONSTRAINT chk_suppliers_payment_terms_days_range
        CHECK (payment_terms_days IS NULL OR (payment_terms_days >= 0 AND payment_terms_days <= 999)),
    ADD CONSTRAINT chk_suppliers_iban_not_blank
        CHECK (iban IS NULL OR length(btrim(iban)) > 0),
    ADD CONSTRAINT chk_suppliers_notes_not_blank
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    ADD CONSTRAINT fk_suppliers_default_category
        FOREIGN KEY (default_category_id, org_id) REFERENCES expense_categories(id, org_id);

CREATE TABLE supplier_contacts (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    supplier_id UUID        NOT NULL,
    position    INTEGER     NOT NULL DEFAULT 0,
    first_name  TEXT        NOT NULL DEFAULT '',
    last_name   TEXT        NOT NULL DEFAULT '',
    role        TEXT        NOT NULL DEFAULT '',
    email       TEXT        NOT NULL DEFAULT '',
    phone       TEXT        NOT NULL DEFAULT '',
    mobile      TEXT        NOT NULL DEFAULT '',
    CONSTRAINT fk_supplier_contacts_supplier
        FOREIGN KEY (supplier_id, org_id) REFERENCES suppliers(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_supplier_contacts_supplier_id ON supplier_contacts(supplier_id, position);

CREATE TABLE supplier_addresses (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    supplier_id UUID        NOT NULL,
    position    INTEGER     NOT NULL DEFAULT 0,
    kind        TEXT        NOT NULL DEFAULT 'BILLING',
    label       TEXT        NOT NULL DEFAULT '',
    street      TEXT        NOT NULL DEFAULT '',
    line2       TEXT        NOT NULL DEFAULT '',
    postal_code TEXT        NOT NULL DEFAULT '',
    city        TEXT        NOT NULL DEFAULT '',
    country     TEXT        NOT NULL DEFAULT 'France',
    CONSTRAINT chk_supplier_addresses_kind_known CHECK (kind IN ('BILLING', 'SITE', 'OTHER')),
    CONSTRAINT fk_supplier_addresses_supplier
        FOREIGN KEY (supplier_id, org_id) REFERENCES suppliers(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_supplier_addresses_supplier_id ON supplier_addresses(supplier_id, position);

ALTER TABLE products
    ADD COLUMN category          TEXT        NOT NULL DEFAULT '',
    ADD COLUMN billing_frequency TEXT        NOT NULL DEFAULT 'ONE_SHOT',
    ADD COLUMN cost_cents        INTEGER     NULL,
    ADD COLUMN archived_at       TIMESTAMPTZ NULL,
    ADD CONSTRAINT chk_products_billing_frequency_known
        CHECK (billing_frequency IN ('ONE_SHOT', 'MONTHLY', 'QUARTERLY', 'YEARLY')),
    ADD CONSTRAINT chk_products_cost_cents_non_negative
        CHECK (cost_cents IS NULL OR cost_cents >= 0);

ALTER TABLE service_rates
    ADD COLUMN category          TEXT        NOT NULL DEFAULT '',
    ADD COLUMN billing_frequency TEXT        NOT NULL DEFAULT 'ONE_SHOT',
    ADD COLUMN cost_cents        INTEGER     NULL,
    ADD COLUMN archived_at       TIMESTAMPTZ NULL,
    ADD CONSTRAINT chk_service_rates_billing_frequency_known
        CHECK (billing_frequency IN ('ONE_SHOT', 'MONTHLY', 'QUARTERLY', 'YEARLY')),
    ADD CONSTRAINT chk_service_rates_cost_cents_non_negative
        CHECK (cost_cents IS NULL OR cost_cents >= 0);
