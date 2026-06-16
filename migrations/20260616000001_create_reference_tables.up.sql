CREATE TABLE employees (
    id                UUID        PRIMARY KEY,
    org_id            UUID        NOT NULL REFERENCES organizations(id),
    user_id           UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    name              TEXT        NOT NULL,
    hourly_rate_cents INTEGER     NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at        TIMESTAMPTZ NULL,

    CONSTRAINT chk_employees_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_employees_hourly_rate_cents_non_negative
        CHECK (hourly_rate_cents >= 0)
);

CREATE INDEX idx_employees_org_id ON employees(org_id);
CREATE INDEX idx_employees_user_id ON employees(user_id);
CREATE INDEX idx_employees_deleted_at ON employees(deleted_at);

CREATE UNIQUE INDEX uq_employees_org_user_active
    ON employees(org_id, user_id)
    WHERE user_id IS NOT NULL AND deleted_at IS NULL;

CREATE TABLE equipment (
    id                UUID        PRIMARY KEY,
    org_id            UUID        NOT NULL REFERENCES organizations(id),
    name              TEXT        NOT NULL,
    hourly_rate_cents INTEGER     NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at        TIMESTAMPTZ NULL,

    CONSTRAINT chk_equipment_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_equipment_hourly_rate_cents_non_negative
        CHECK (hourly_rate_cents >= 0)
);

CREATE INDEX idx_equipment_org_id ON equipment(org_id);
CREATE INDEX idx_equipment_deleted_at ON equipment(deleted_at);

CREATE UNIQUE INDEX uq_equipment_org_name_active
    ON equipment(org_id, lower(name))
    WHERE deleted_at IS NULL;

CREATE TABLE service_rates (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id),
    label      TEXT        NOT NULL,
    unit       TEXT        NOT NULL,
    rate_cents INTEGER     NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ NULL,

    CONSTRAINT chk_service_rates_label_not_blank
        CHECK (length(btrim(label)) > 0),
    CONSTRAINT chk_service_rates_unit_not_blank
        CHECK (length(btrim(unit)) > 0),
    CONSTRAINT chk_service_rates_rate_cents_non_negative
        CHECK (rate_cents >= 0)
);

CREATE INDEX idx_service_rates_org_id ON service_rates(org_id);
CREATE INDEX idx_service_rates_deleted_at ON service_rates(deleted_at);

CREATE UNIQUE INDEX uq_service_rates_org_label_active
    ON service_rates(org_id, lower(label))
    WHERE deleted_at IS NULL;
