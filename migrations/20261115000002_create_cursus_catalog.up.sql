CREATE SCHEMA IF NOT EXISTS cursus;

CREATE TABLE cursus.establishment (
    id           UUID        PRIMARY KEY,
    org_id       UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name         TEXT        NOT NULL,
    uai          TEXT,
    siret        TEXT,
    address      TEXT,
    is_main_site BOOLEAN     NOT NULL DEFAULT FALSE,
    archived_at  TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id)
);

CREATE INDEX idx_cursus_establishment_org ON cursus.establishment(org_id);

CREATE TABLE cursus.training (
    id               UUID        PRIMARY KEY,
    org_id           UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    establishment_id UUID        NOT NULL,
    name             TEXT        NOT NULL,
    rncp_code        TEXT,
    rncp_label       TEXT,
    level            TEXT,
    degree_type      TEXT,
    archived_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    FOREIGN KEY (org_id, establishment_id) REFERENCES cursus.establishment(org_id, id)
);

CREATE INDEX idx_cursus_training_org ON cursus.training(org_id);
CREATE INDEX idx_cursus_training_establishment ON cursus.training(org_id, establishment_id);

CREATE TABLE cursus.promotion (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    training_id UUID        NOT NULL,
    label       TEXT        NOT NULL,
    start_year  INTEGER     NOT NULL,
    end_year    INTEGER     NOT NULL,
    archived_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    FOREIGN KEY (org_id, training_id) REFERENCES cursus.training(org_id, id),
    CONSTRAINT chk_cursus_promotion_years CHECK (end_year >= start_year)
);

CREATE INDEX idx_cursus_promotion_org ON cursus.promotion(org_id);
CREATE INDEX idx_cursus_promotion_training ON cursus.promotion(org_id, training_id);

CREATE TABLE cursus.closure_reason (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    label       TEXT        NOT NULL,
    archived_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cursus_closure_reason_org ON cursus.closure_reason(org_id);
