CREATE TABLE cursus.host_company (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name           TEXT        NOT NULL,
    siret          TEXT,
    address        TEXT,
    legal_form     TEXT,
    naf_code       TEXT,
    employee_count INTEGER,
    director_name  TEXT,
    archived_at    TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT chk_cursus_host_company_employee_count CHECK (employee_count IS NULL OR employee_count >= 0)
);

CREATE INDEX idx_cursus_host_company_org ON cursus.host_company(org_id);
