ALTER TABLE cursus.closure_reason
    ADD CONSTRAINT uq_cursus_closure_reason_org_id UNIQUE (org_id, id);

CREATE TABLE cursus.triad (
    id                        UUID        PRIMARY KEY,
    org_id                    UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    promotion_id              UUID        NOT NULL,
    apprentice_id             UUID        NOT NULL,
    pedagogic_tutor_member_id UUID        REFERENCES organization_members(id) ON DELETE SET NULL,
    closed_at                 DATE,
    closure_reason_id         UUID,
    closure_note              TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT uq_cursus_triad_promotion_apprentice UNIQUE (promotion_id, apprentice_id),
    FOREIGN KEY (org_id, promotion_id) REFERENCES cursus.promotion(org_id, id),
    FOREIGN KEY (org_id, apprentice_id) REFERENCES cursus.apprentice(org_id, id),
    FOREIGN KEY (org_id, closure_reason_id) REFERENCES cursus.closure_reason(org_id, id)
);

CREATE INDEX idx_cursus_triad_org ON cursus.triad(org_id);
CREATE INDEX idx_cursus_triad_promotion ON cursus.triad(org_id, promotion_id);
CREATE INDEX idx_cursus_triad_apprentice ON cursus.triad(org_id, apprentice_id);

CREATE TABLE cursus.contract (
    id                      UUID        PRIMARY KEY,
    org_id                  UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    triad_id                UUID        NOT NULL,
    host_company_id         UUID        NOT NULL,
    kind                    TEXT        NOT NULL,
    number                  TEXT,
    start_date              DATE        NOT NULL,
    end_date                DATE        NOT NULL,
    closed_at               DATE,
    closure_reason_id       UUID,
    closure_note            TEXT,
    cerfa_file_key          TEXT,
    mission_sheet_file_key  TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT chk_cursus_contract_kind CHECK (kind IN ('APPRENTICESHIP', 'PROFESSIONALIZATION')),
    CONSTRAINT chk_cursus_contract_period CHECK (end_date >= start_date),
    FOREIGN KEY (org_id, triad_id) REFERENCES cursus.triad(org_id, id),
    FOREIGN KEY (org_id, host_company_id) REFERENCES cursus.host_company(org_id, id),
    FOREIGN KEY (org_id, closure_reason_id) REFERENCES cursus.closure_reason(org_id, id)
);

CREATE INDEX idx_cursus_contract_org ON cursus.contract(org_id);
CREATE INDEX idx_cursus_contract_triad ON cursus.contract(org_id, triad_id);
CREATE INDEX idx_cursus_contract_host_company ON cursus.contract(org_id, host_company_id);

CREATE TABLE cursus.contract_tutor (
    contract_id      UUID        NOT NULL,
    company_tutor_id UUID        NOT NULL,
    org_id           UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    rank             TEXT        NOT NULL,
    archived_at      TIMESTAMPTZ,
    PRIMARY KEY (contract_id, rank),
    CONSTRAINT chk_cursus_contract_tutor_rank CHECK (rank IN ('MAIN', 'SECONDARY')),
    FOREIGN KEY (org_id, contract_id) REFERENCES cursus.contract(org_id, id),
    FOREIGN KEY (org_id, company_tutor_id) REFERENCES cursus.company_tutor(org_id, id)
);

CREATE INDEX idx_cursus_contract_tutor_company_tutor ON cursus.contract_tutor(org_id, company_tutor_id);
