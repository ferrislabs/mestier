CREATE TABLE customers (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id),
    last_name  TEXT        NOT NULL,
    first_name TEXT        NOT NULL,
    phone      TEXT        NULL,
    email      TEXT        NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ NULL,

    CONSTRAINT chk_customers_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0),
    CONSTRAINT chk_customers_first_name_not_blank
        CHECK (length(btrim(first_name)) > 0),
    CONSTRAINT chk_customers_phone_not_blank_when_present
        CHECK (phone IS NULL OR length(btrim(phone)) > 0),
    CONSTRAINT chk_customers_email_not_blank_when_present
        CHECK (email IS NULL OR length(btrim(email)) > 0)
);

CREATE INDEX idx_customers_org_id ON customers(org_id);
CREATE INDEX idx_customers_deleted_at ON customers(deleted_at);

CREATE TABLE properties (
    id          UUID        PRIMARY KEY,
    customer_id UUID        NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    label       TEXT        NOT NULL,
    street      TEXT        NOT NULL,
    zip         TEXT        NOT NULL,
    city        TEXT        NOT NULL,
    photo_key   TEXT        NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ NULL,

    CONSTRAINT chk_properties_label_not_blank
        CHECK (length(btrim(label)) > 0),
    CONSTRAINT chk_properties_street_not_blank
        CHECK (length(btrim(street)) > 0),
    CONSTRAINT chk_properties_zip_not_blank
        CHECK (length(btrim(zip)) > 0),
    CONSTRAINT chk_properties_city_not_blank
        CHECK (length(btrim(city)) > 0),
    CONSTRAINT chk_properties_photo_key_not_blank_when_present
        CHECK (photo_key IS NULL OR length(btrim(photo_key)) > 0)
);

CREATE INDEX idx_properties_customer_id ON properties(customer_id);
CREATE INDEX idx_properties_deleted_at ON properties(deleted_at);

CREATE UNIQUE INDEX uq_properties_customer_label_active
    ON properties(customer_id, lower(label))
    WHERE deleted_at IS NULL;
