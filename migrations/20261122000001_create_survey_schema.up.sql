CREATE SCHEMA survey;

CREATE TABLE survey.campaign (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    starts_at  TIMESTAMPTZ NULL,
    ends_at    TIMESTAMPTZ NULL,
    status     TEXT        NOT NULL,
    metadata   JSONB       NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_campaign_org_id UNIQUE (org_id, id),
    CONSTRAINT chk_campaign_name CHECK (btrim(name) <> ''),
    CONSTRAINT chk_campaign_dates CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at < ends_at),
    CONSTRAINT chk_campaign_status CHECK (status IN ('SCHEDULED', 'RUNNING', 'ENDED'))
);

CREATE TABLE survey.survey (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                   UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    campaign_id              UUID        NULL,
    title                    TEXT        NOT NULL,
    description              TEXT        NULL,
    visibility               TEXT        NOT NULL,
    graph                    JSONB       NULL,
    allow_multiple_responses BOOLEAN     NOT NULL DEFAULT false,
    response_cap             INTEGER     NULL,
    metadata                 JSONB       NOT NULL DEFAULT '{}',
    status                   TEXT        NOT NULL,
    public_slug              TEXT        NULL UNIQUE,
    opened_at                TIMESTAMPTZ NULL,
    closed_at                TIMESTAMPTZ NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_survey_org_id UNIQUE (org_id, id),
    CONSTRAINT fk_survey_campaign
        FOREIGN KEY (org_id, campaign_id) REFERENCES survey.campaign (org_id, id)
        ON DELETE RESTRICT,
    CONSTRAINT chk_survey_title CHECK (btrim(title) <> ''),
    CONSTRAINT chk_survey_visibility CHECK (visibility IN ('PUBLIC', 'MEMBERS')),
    CONSTRAINT chk_survey_status CHECK (status IN ('DRAFT', 'OPENED', 'CLOSED')),
    CONSTRAINT chk_survey_response_cap CHECK (response_cap IS NULL OR response_cap > 0),
    CONSTRAINT chk_survey_graph CHECK (status = 'DRAFT' OR graph IS NOT NULL),
    CONSTRAINT chk_survey_state CHECK (
        (status = 'DRAFT' AND opened_at IS NULL AND closed_at IS NULL)
        OR (status = 'OPENED' AND opened_at IS NOT NULL AND closed_at IS NULL)
        OR (status = 'CLOSED' AND opened_at IS NOT NULL AND closed_at IS NOT NULL
            AND opened_at <= closed_at)
    ),
    CONSTRAINT chk_survey_public_slug CHECK (
        (public_slug IS NOT NULL) = (status <> 'DRAFT' AND visibility = 'PUBLIC')
    ),
    CONSTRAINT chk_survey_public_slug_shape CHECK (
        public_slug IS NULL OR public_slug ~ '^[A-Za-z0-9_-]{8,64}$'
    )
);

CREATE INDEX idx_survey_org_campaign ON survey.survey (org_id, campaign_id);

CREATE TABLE survey.response (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id            UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    survey_id         UUID        NOT NULL,
    member_id         UUID        NULL,
    answers           JSONB       NOT NULL DEFAULT '{}',
    status            TEXT        NOT NULL,
    resume_token      TEXT        NULL UNIQUE,
    identifier_value  TEXT        NULL,
    unique_respondent BOOLEAN     NOT NULL DEFAULT false,
    completed_at      TIMESTAMPTZ NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_response_survey
        FOREIGN KEY (org_id, survey_id) REFERENCES survey.survey (org_id, id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_response_member
        FOREIGN KEY (member_id, org_id) REFERENCES organization_members (id, organization_id)
        ON DELETE SET NULL (member_id),
    CONSTRAINT chk_response_status CHECK (status IN ('IN_PROGRESS', 'COMPLETED')),
    CONSTRAINT chk_response_completed_at CHECK ((status = 'COMPLETED') = (completed_at IS NOT NULL)),
    CONSTRAINT chk_response_in_progress CHECK (
        status = 'COMPLETED' OR (identifier_value IS NULL AND NOT unique_respondent)
    ),
    CONSTRAINT chk_response_resume_token CHECK (resume_token IS NULL OR resume_token <> '')
);

CREATE INDEX idx_response_org_survey ON survey.response (org_id, survey_id);

CREATE UNIQUE INDEX uq_response_unique_member
    ON survey.response (survey_id, member_id)
    WHERE status = 'COMPLETED' AND unique_respondent AND member_id IS NOT NULL;

CREATE UNIQUE INDEX uq_response_unique_identifier
    ON survey.response (survey_id, identifier_value)
    WHERE status = 'COMPLETED' AND unique_respondent AND identifier_value IS NOT NULL;
