CREATE TYPE quote_status AS ENUM (
    'DRAFT',
    'SENT',
    'ACCEPTED',
    'DECLINED',
    'CANCELLED'
);

CREATE TABLE quotes (
    id          UUID         PRIMARY KEY,
    org_id      UUID         NOT NULL REFERENCES organizations(id),
    customer_id UUID         NOT NULL REFERENCES customers(id),
    property_id UUID         NOT NULL REFERENCES properties(id),
    status      quote_status NOT NULL,
    total_cents INTEGER      NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ  NULL,

    CONSTRAINT chk_quotes_total_cents_non_negative
        CHECK (total_cents >= 0)
);

CREATE INDEX idx_quotes_org_id ON quotes(org_id);
CREATE INDEX idx_quotes_customer_id ON quotes(customer_id);
CREATE INDEX idx_quotes_property_id ON quotes(property_id);
CREATE INDEX idx_quotes_status ON quotes(status);
CREATE INDEX idx_quotes_deleted_at ON quotes(deleted_at);

CREATE TABLE quote_lines (
    id               UUID        PRIMARY KEY,
    org_id           UUID        NOT NULL REFERENCES organizations(id),
    quote_id         UUID        NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
    service_rate_id  UUID        NULL REFERENCES service_rates(id) ON DELETE SET NULL,
    label            TEXT        NOT NULL,
    quantity         NUMERIC     NOT NULL,
    unit             TEXT        NOT NULL,
    unit_price_cents INTEGER     NOT NULL,
    notes            TEXT        NULL,
    photo_keys       TEXT[]      NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at       TIMESTAMPTZ NULL,

    CONSTRAINT chk_quote_lines_label_not_blank
        CHECK (length(btrim(label)) > 0),
    CONSTRAINT chk_quote_lines_quantity_positive
        CHECK (quantity > 0),
    CONSTRAINT chk_quote_lines_unit_not_blank
        CHECK (length(btrim(unit)) > 0),
    CONSTRAINT chk_quote_lines_unit_price_cents_non_negative
        CHECK (unit_price_cents >= 0),
    CONSTRAINT chk_quote_lines_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    CONSTRAINT chk_quote_lines_photo_keys_not_blank
        CHECK (array_position(photo_keys, '') IS NULL)
);

CREATE INDEX idx_quote_lines_org_id ON quote_lines(org_id);
CREATE INDEX idx_quote_lines_quote_id ON quote_lines(quote_id);
CREATE INDEX idx_quote_lines_service_rate_id ON quote_lines(service_rate_id);
CREATE INDEX idx_quote_lines_deleted_at ON quote_lines(deleted_at);
