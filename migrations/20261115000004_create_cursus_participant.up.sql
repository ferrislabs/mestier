CREATE TABLE cursus.participant (
    id                     UUID        PRIMARY KEY,
    org_id                 UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id                UUID        REFERENCES users(id) ON DELETE SET NULL,
    last_name              TEXT        NOT NULL,
    first_name             TEXT,
    email                  TEXT        NOT NULL,
    phone                  TEXT,
    invitation_token_hash  BYTEA,
    invitation_expires_at  TIMESTAMPTZ,
    deactivated_at         TIMESTAMPTZ,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT uq_cursus_participant_user UNIQUE (org_id, user_id),
    CONSTRAINT uq_cursus_participant_invitation UNIQUE (invitation_token_hash)
);

CREATE UNIQUE INDEX uq_cursus_participant_email ON cursus.participant (org_id, lower(email));
CREATE INDEX idx_cursus_participant_org ON cursus.participant(org_id);

CREATE TABLE cursus.apprentice (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    participant_id UUID        NOT NULL,
    birth_date     DATE,
    ine_code       TEXT,
    address        TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT uq_cursus_apprentice_participant UNIQUE (participant_id),
    FOREIGN KEY (org_id, participant_id) REFERENCES cursus.participant(org_id, id)
);

CREATE TABLE cursus.company_tutor (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    participant_id  UUID        NOT NULL,
    host_company_id UUID        NOT NULL,
    job_title       TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT uq_cursus_company_tutor_company UNIQUE (participant_id, host_company_id),
    FOREIGN KEY (org_id, participant_id) REFERENCES cursus.participant(org_id, id),
    FOREIGN KEY (org_id, host_company_id) REFERENCES cursus.host_company(org_id, id)
);

CREATE INDEX idx_cursus_company_tutor_participant ON cursus.company_tutor(org_id, participant_id);
CREATE INDEX idx_cursus_company_tutor_host_company ON cursus.company_tutor(org_id, host_company_id);
