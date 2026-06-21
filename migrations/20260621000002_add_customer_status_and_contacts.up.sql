ALTER TABLE customers
    ADD COLUMN status TEXT NOT NULL DEFAULT 'PROSPECT';

ALTER TABLE customers
    ADD CONSTRAINT chk_customers_status
        CHECK (status IN ('PROSPECT', 'CLIENT', 'ARCHIVED'));

CREATE INDEX idx_customers_status ON customers(status);

CREATE TABLE customer_contacts (
    id          UUID        PRIMARY KEY,
    customer_id UUID        NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    first_name  TEXT        NOT NULL,
    last_name   TEXT        NOT NULL,
    role        TEXT        NULL,
    phone       TEXT        NULL,
    email       TEXT        NULL,
    is_primary  BOOLEAN     NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ NULL,

    CONSTRAINT chk_customer_contacts_first_name_not_blank
        CHECK (length(btrim(first_name)) > 0),
    CONSTRAINT chk_customer_contacts_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0),
    CONSTRAINT chk_customer_contacts_role_not_blank_when_present
        CHECK (role IS NULL OR length(btrim(role)) > 0),
    CONSTRAINT chk_customer_contacts_phone_not_blank_when_present
        CHECK (phone IS NULL OR length(btrim(phone)) > 0),
    CONSTRAINT chk_customer_contacts_email_not_blank_when_present
        CHECK (email IS NULL OR length(btrim(email)) > 0)
);

CREATE INDEX idx_customer_contacts_customer_id ON customer_contacts(customer_id);
CREATE INDEX idx_customer_contacts_deleted_at ON customer_contacts(deleted_at);

CREATE UNIQUE INDEX uq_customer_contacts_customer_email_active
    ON customer_contacts(customer_id, lower(email))
    WHERE email IS NOT NULL AND deleted_at IS NULL;
