CREATE TABLE custom_units (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id),
    code       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_custom_units_code_not_blank
        CHECK (length(btrim(code)) > 0)
);

-- Case-insensitive: "Sac" and "sac" would be indistinguishable in the unit
-- picker and ambiguous once stored on a product/service/quote line.
CREATE UNIQUE INDEX custom_units_org_id_lower_code_key
    ON custom_units (org_id, lower(code));

CREATE INDEX custom_units_org_id_idx ON custom_units (org_id);
